//! Thread-local allocator observation of the pinned opaque scalar-fill call.
use super::*;
use crate::evaluated_scene::masks::fill_transient_bytes;
use crate::evaluated_scene::shapes::Contour;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

#[derive(Clone, Copy)]
struct AllocationStats {
    live: usize,
    peak: usize,
    allocations: usize,
    reallocations: usize,
    invalid: bool,
    requests: [usize; 128],
    recorded: usize,
    record_requests: bool,
}
impl Default for AllocationStats {
    fn default() -> Self {
        Self {
            live: 0,
            peak: 0,
            allocations: 0,
            reallocations: 0,
            invalid: false,
            requests: [0; 128],
            recorded: 0,
            record_requests: true,
        }
    }
}
thread_local! { static OBSERVATION: Cell<Option<AllocationStats>> = const { Cell::new(None) }; }
struct ObservedSystem;
#[global_allocator]
static ALLOCATOR: ObservedSystem = ObservedSystem;
fn update(action: impl FnOnce(&mut AllocationStats)) {
    let _ = OBSERVATION.try_with(|cell| {
        if let Some(mut stats) = cell.get() {
            action(&mut stats);
            cell.set(Some(stats));
        }
    });
}
fn request(stats: &mut AllocationStats, size: usize) {
    if !stats.record_requests {
        return;
    }
    if stats.recorded < stats.requests.len() {
        stats.requests[stats.recorded] = size;
        stats.recorded += 1;
    } else {
        stats.invalid = true;
    }
}
// Runtime production remains safe. Unsafe forwarding exists only in this
// independently observed System allocator used by the editor-core test binary.
unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc(layout) };
        if !result.is_null() {
            update(|s| {
                s.allocations += 1;
                request(s, layout.size());
                match s.live.checked_add(layout.size()) {
                    Some(live) => {
                        s.live = live;
                        s.peak = s.peak.max(live)
                    }
                    None => s.invalid = true,
                }
            });
        }
        result
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc_zeroed(layout) };
        if !result.is_null() {
            update(|s| {
                s.allocations += 1;
                request(s, layout.size());
                match s.live.checked_add(layout.size()) {
                    Some(live) => {
                        s.live = live;
                        s.peak = s.peak.max(live)
                    }
                    None => s.invalid = true,
                }
            });
        }
        result
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        update(|s| {
            if let Some(live) = s.live.checked_sub(layout.size()) {
                s.live = live
            } else {
                s.invalid = true;
            }
        });
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Conservatively count the new block beside the old one even when
        // System actually grows in place. Failure leaves the old live block.
        update(|s| {
            s.reallocations += 1;
            request(s, new_size);
            if let Some(peak) = s.live.checked_add(new_size) {
                s.peak = s.peak.max(peak)
            } else {
                s.invalid = true;
            }
        });
        let result = unsafe { System.realloc(ptr, layout, new_size) };
        if !result.is_null() {
            update(|s| {
                match s
                    .live
                    .checked_sub(layout.size())
                    .and_then(|n| n.checked_add(new_size))
                {
                    Some(live) => s.live = live,
                    None => s.invalid = true,
                }
            });
        }
        result
    }
}
fn zigzag(segments: usize) -> Vec<Contour> {
    let mut points = Vec::with_capacity(segments + 1);
    for i in 0..segments {
        let mut y = 2.0 + (i % 3) as f64;
        if i + 1 == segments && y == 2.0 {
            y = 3.5;
        }
        points.push(VectorPoint {
            x: if i % 2 == 0 { 2.0 } else { 4.0 },
            y,
        });
    }
    points.push(points[0]);
    vec![Contour {
        points,
        closed: true,
    }]
}
fn observed_fill(contours: &[Contour], size: (u32, u32)) -> AllocationStats {
    // All input objects and TLS initialization precede tracking. The returned
    // scalar is dropped before disabling, so live0 also proves cleanup.
    OBSERVATION.with(|cell| {
        assert!(cell.get().is_none());
        cell.set(Some(AllocationStats::default()));
    });
    let result = fill_contours(contours, FillRule::Nonzero, (0., 0.), 1., size);
    let success = result.is_ok();
    drop(result);
    let stats = OBSERVATION.with(|cell| cell.replace(None).unwrap());
    assert!(success);
    assert!(
        !stats.invalid,
        "allocator tracking capacity/accounting failure"
    );
    assert_eq!(stats.live, 0);
    stats
}
#[test]
fn opaque_fill_actual_peak_fits_additive_bound_for_high_s_tiny_p() {
    let contours = zigzag(4096);
    let size = (8, 8);
    let stats = observed_fill(&contours, size);
    let pixel_bytes = 8 * u64::from(size.0) * u64::from(size.1);
    let certificate = fill_transient_bytes(4096, 1, size).unwrap() + pixel_bytes;
    eprintln!(
        "opaque high-S fill: peak={} certificate={} allocations={} reallocations={} final-live={}",
        stats.peak, certificate, stats.allocations, stats.reallocations, stats.live
    );
    assert!(stats.peak as u64 <= certificate);
    assert!(
        stats.peak > 64 * 64,
        "must observe geometry-scaled scratch beyond64P"
    );
    assert!(stats.allocations > 3);
    assert!(stats.reallocations > 0);
}
#[test]
fn opaque_edge_initial_capacity_layout_growth_and_sentinels_are_observed() {
    for edges in [63, 64, 65] {
        let contours = zigzag(edges);
        let stats = observed_fill(&contours, (8, 8));
        let certificate = fill_transient_bytes(edges as u64, 1, (8, 8)).unwrap() + 8 * 64;
        assert!(stats.peak as u64 <= certificate);
        assert!(stats.reallocations > 0);
        // On this small line-only fixture, earlier pixel/path requests are
        // smaller. The first64-multiple request>64*64 is BasicEdgeBuilder's
        // initial64-element Vec<Edge>, not a Vec<LineEdge>.
        let initial = stats.requests[..stats.recorded]
            .iter()
            .copied()
            .find(|n| *n % 64 == 0 && *n > 64 * 64 && *n <= 128 * 64)
            .expect("initial64 Edge allocation absent");
        eprintln!(
            "opaque edge boundary: edges={} edge-bytes={} initial-capacity=64 peak={} certificate={} final-live={}",
            edges,
            initial / 64,
            stats.peak,
            certificate,
            stats.live
        );
        assert!(
            (65..=128).contains(&(initial / 64)),
            "pinned Edge exceeds64B; allowance128B"
        );
        assert!(
            stats.requests[..stats.recorded].contains(&(initial * 2)),
            "edge capacity growth including sentinels absent"
        );
    }
}
#[test]
fn opaque_fill_budget_formula_and_overflow_do_not_hide_actual_storage() {
    assert_eq!(
        fill_transient_bytes(4096, 1, (8, 8)).unwrap(),
        2048 * (4096 + 1 + 64) + 32 * 16
    );
    assert!(fill_transient_bytes(u64::MAX, 1, (8, 8)).is_err());
    assert!(fill_transient_bytes(1, u64::MAX, (8, 8)).is_err());
}

/// Sibling observation mode retains checked live/peak and moving-realloc
/// overlap, while disabling only the fill test's128-request recording cap.
/// The closure must drop all of its allocated outputs before returning.
pub(crate) fn observe(action: impl FnOnce()) -> (usize, usize, usize) {
    OBSERVATION.with(|cell| {
        assert!(cell.get().is_none());
        cell.set(Some(AllocationStats {
            record_requests: false,
            ..Default::default()
        }));
    });
    action();
    let stats = OBSERVATION.with(|cell| cell.replace(None).unwrap());
    assert!(!stats.invalid);
    assert_eq!(stats.live, 0);
    (stats.peak, stats.allocations, stats.reallocations)
}
