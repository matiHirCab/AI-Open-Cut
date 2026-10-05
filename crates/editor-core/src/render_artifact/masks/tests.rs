use super::*;
use crate::evaluated_scene::masks::{EvaluatedMask, MaskGrid, MaskOwnerBasis};
use crate::evaluated_scene::shapes::Contour;
use crate::{FillRule, MaskChannel, MaskOperation, Paint, VectorColor, VectorPoint};

fn solid(alpha: f64) -> Paint {
    Paint::Solid {
        color: VectorColor {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: alpha,
        },
    }
}
fn rectangle() -> MaskGrid {
    MaskGrid {
        contours: vec![Contour {
            points: vec![
                VectorPoint { x: 0.0, y: 0.0 },
                VectorPoint { x: 2.0, y: 0.0 },
                VectorPoint { x: 2.0, y: 2.0 },
                VectorPoint { x: 0.0, y: 2.0 },
            ],
            closed: true,
        }],
        fill_rule: FillRule::Nonzero,
        analytic_bounds: [0.0, 0.0, 2.0, 2.0],
        origin: (-2.0, -2.0),
        size: (6, 6),
        density: 1.0,
        segments: 4,
    }
}
fn required_scratch(grid: &MaskGrid, feather: f64) -> u64 {
    let p = u64::from(grid.size.0) * u64::from(grid.size.1);
    let kernel = if feather == 0. {
        0
    } else {
        4 * (2 * (3. * feather).ceil() as u64 + 1)
    };
    64 * p
        + 16 * (u64::from(grid.size.0) + u64::from(grid.size.1))
        + kernel
        + crate::evaluated_scene::masks::fill_transient_bytes(
            grid.segments,
            grid.contours.len() as u64,
            grid.size,
        )
        .unwrap()
}
fn mask(alpha: f64) -> EvaluatedMask {
    EvaluatedMask {
        id: "mask".into(),
        grid: Some(rectangle()),
        paint: solid(alpha),
        channel: MaskChannel::Alpha,
        operation: MaskOperation::Add,
        inverted: false,
        inverse: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        opacity: 1.0,
        expansion_grid: 0.0,
        feather_grid: 0.0,
        certified_scratch_bytes: required_scratch(&rectangle(), 0.),
    }
}
fn stack(masks: Vec<EvaluatedMask>) -> EvaluatedMaskStack {
    let peak = masks
        .iter()
        .map(|m| m.certified_scratch_bytes)
        .max()
        .unwrap_or(0);
    EvaluatedMaskStack {
        owner: MaskOwnerBasis {
            size: (2, 2),
            density: 1.0,
        },
        masks,
        work_units: 4096,
        accumulated_bytes: 16,
        retained_fact_bytes: 4096,
        peak_scratch_bytes: peak,
    }
}
fn close(actual: f32, expected: f64) {
    assert!(
        (f64::from(actual) - expected).abs() <= 1e-6,
        "{actual} != {expected}"
    );
}
#[test]
fn alpha_quarter_multiplies_all_source_components_once() {
    let mut source = [[0.4, 0.2, 0.1, 0.8]; 4];
    apply_stack(&mut source, &stack(vec![mask(0.25)])).unwrap();
    for pixel in source {
        for (actual, expected) in pixel.into_iter().zip([0.1, 0.05, 0.025, 0.2]) {
            close(actual, expected);
        }
    }
}
#[test]
fn dark_red_luma_is_not_alpha_or_encoded_red() {
    let mut m = mask(0.25);
    m.paint = Paint::Solid {
        color: VectorColor {
            r: 0.2,
            g: 0.0,
            b: 0.0,
            a: 0.25,
        },
    };
    m.channel = MaskChannel::Luma;
    let mut source = [[1.0; 4]; 4];
    apply_stack(&mut source, &stack(vec![m])).unwrap();
    for p in source {
        close(p[3], 0.0017595183432425408);
    }
}
#[test]
fn empty_geometry_inversion_is_owner_domain_one() {
    let mut m = mask(1.0);
    m.grid = None;
    m.inverted = true;
    m.opacity = 0.2;
    let mut source = [[0.4, 0.2, 0.1, 0.8]; 4];
    let before = source;
    apply_stack(&mut source, &stack(vec![m])).unwrap();
    assert_eq!(source, before);
}

fn brute_distances(c: &[f32], size: (u32, u32), inside: bool) -> Vec<Option<u64>> {
    let w = size.0 as usize;
    let seeds: Vec<_> = c
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (if inside { *c > 0.0 } else { *c < 1.0 }).then_some(i))
        .collect();
    (0..c.len())
        .map(|i| {
            seeds
                .iter()
                .map(|j| {
                    let dx = (i % w).abs_diff(j % w) as u64;
                    let dy = (i / w).abs_diff(j / w) as u64;
                    dx * dx + dy * dy
                })
                .min()
        })
        .collect()
}
fn brute_expansion(c: &[f32], size: (u32, u32), e: f64) -> Vec<f64> {
    if e == 0.0 {
        return c.iter().copied().map(f64::from).collect();
    }
    let inside = brute_distances(c, size, true);
    let outside = brute_distances(c, size, false);
    c.iter()
        .enumerate()
        .map(|(i, c)| {
            if inside[i].is_none() {
                return 0.0;
            }
            let d = if *c == 0.0 {
                0.5 - (inside[i].unwrap() as f64).sqrt()
            } else if *c == 1.0 {
                (outside[i].unwrap() as f64).sqrt() - 0.5
            } else {
                f64::from(*c) - 0.5
            };
            (0.5 + d + e).clamp(0.0, 1.0)
        })
        .collect()
}
#[test]
fn exact_edt_matches_all_512_binary_three_by_three_seed_sets() {
    assert_eq!(size_of::<Distance>(), 8);
    assert_eq!(size_of::<Site>(), 16);
    for bits in 0..512u32 {
        let coverage: Vec<_> = (0..9)
            .map(|i| if bits & (1 << i) != 0 { 1.0 } else { 0.0 })
            .collect();
        for inside in [true, false] {
            let scratch = Scratch::new(4096);
            let actual = distances(
                &coverage,
                (3, 3),
                inside,
                &scratch,
                &mut DistanceWork::default(),
            )
            .unwrap();
            let expected = brute_distances(&coverage, (3, 3), inside);
            for (a, e) in actual.iter().zip(expected) {
                assert_eq!(a.map(|v| v.get() - 1), e, "bits={bits} inside={inside}");
            }
        }
    }
}
#[test]
fn weighted_envelope_negative_crossovers_and_lower_index_ties() {
    assert_eq!(first_winning_sample(0, 4, 1, 0).unwrap(), -1);
    assert_eq!(first_winning_sample(0, 0, 2, 0).unwrap(), 2); // tie atx1 stays atp0
    let inputs = [
        Some(finite_distance(4).unwrap()),
        Some(finite_distance(0).unwrap()),
        None,
    ];
    let mut output = [None; 3];
    let mut sites = [Site::default(); 3];
    distance_line(
        3,
        |i| inputs[i],
        |i, d| output[i] = d,
        &mut sites,
        &mut DistanceWork::default(),
    )
    .unwrap();
    assert_eq!(output.map(|v| v.unwrap().get() - 1), [1, 0, 1]);
    let inputs = [NonZeroU64::new(1), None, NonZeroU64::new(1)];
    distance_line(
        3,
        |i| inputs[i],
        |_, _| {},
        &mut sites,
        &mut DistanceWork::default(),
    )
    .unwrap();
    assert_eq!(sites[1].start, 2);
    assert_eq!(sites[0].index, 0);
}
#[test]
fn partial_thin_and_hole_expansion_match_independent_distance_reconstruction() {
    let cases = [
        vec![0., 0., 0., 0., 0.25, 0., 0., 0., 0.],
        vec![0., 0., 0., 0., 1.0 / 255.0, 0., 0., 0., 0.],
        vec![0., 0., 0., 0., 1., 0., 0., 0., 0.],
        vec![
            0., 0., 0., 0., 0., 0., 0.2, 0.8, 0., 0., 0., 0., 0.2, 0.8, 0., 0., 0., 0., 0.2, 0.,
            0., 0., 0., 0., 0.,
        ],
        (0..25)
            .map(|i| {
                if (1..=3).contains(&(i % 5)) && (1..=3).contains(&(i / 5)) && i != 12 {
                    1.0
                } else {
                    0.0
                }
            })
            .collect(),
    ];
    for original in cases {
        let side = if original.len() == 9 { 3 } else { 5 };
        for e in [-0.25, -0.125, 0.25, 0.75] {
            let expected = brute_expansion(&original, (side, side), e);
            let mut actual = original.clone();
            let scratch = Scratch::new(65536);
            expand(
                &mut actual,
                (side, side),
                e,
                &scratch,
                &mut DistanceWork::default(),
            )
            .unwrap();
            for (a, e) in actual.into_iter().zip(expected) {
                close(a, e);
            }
        }
    }
}
#[test]
fn expansion_zero_empty_and_analytic_no_grid_have_exact_fast_paths() {
    let mut c = [0.0, 1.0 / 255.0, 0.25, 1.0];
    let before = c.map(f32::to_bits);
    let scratch = Scratch::new(0);
    let mut work = DistanceWork::default();
    expand(&mut c, (2, 2), 0.0, &scratch, &mut work).unwrap();
    assert_eq!(c.map(f32::to_bits), before);
    assert_eq!(scratch.allocations.get(), 0);
    assert_eq!(work.outputs, 0);
    let mut empty = [0.0; 4];
    expand(&mut empty, (2, 2), 128.0, &scratch, &mut work).unwrap();
    assert_eq!(empty, [0.0; 4]);
    assert_eq!(scratch.allocations.get(), 0);
    let mut m = mask(1.0);
    m.grid = None;
    m.certified_scratch_bytes = 0;
    let mut source = [[0.3, 0.2, 0.1, 0.5]; 4];
    apply_stack(&mut source, &stack(vec![m])).unwrap();
    assert_eq!(source, [[0.0; 4]; 4]);
}
#[test]
fn distance_work_is_linear_for_long_rectangular_dense_and_sparse_grids() {
    for size in [(1, 4096), (4096, 1), (64, 64), (127, 31)] {
        let n = count(size).unwrap();
        let w = size.0 as usize;
        let c: Vec<_> = (0..n)
            .map(|i| {
                if (i % w + i / w).is_multiple_of(2) {
                    1.0
                } else {
                    0.0
                }
            })
            .collect();
        let scratch = Scratch::new(64 * n as u64 + 16 * (u64::from(size.0) + u64::from(size.1)));
        let mut work = DistanceWork::default();
        let inside = distances(&c, size, true, &scratch, &mut work).unwrap();
        let outside = distances(&c, size, false, &scratch, &mut work).unwrap();
        assert_eq!(inside.len(), n);
        assert_eq!(outside.len(), n);
        assert!(work.crossovers <= 8 * n);
        assert!(work.insertions <= 4 * n);
        assert!(work.pops <= 4 * n);
        assert!(work.advances <= 4 * n);
        assert!(work.outputs <= 4 * n);
        assert!(scratch.peak.get() <= 24 * n as u64 + 16 * u64::from(size.0.max(size.1)));
    }
}
fn direct_gaussian(c: &[f32], size: (u32, u32), sigma: f64) -> Vec<f64> {
    let r = (3.0 * sigma).ceil() as i64;
    let w = i64::from(size.0);
    let h = i64::from(size.1);
    let raw: Vec<_> = (-r..=r)
        .map(|k| (-0.5 * (k as f64 / sigma).powi(2)).exp())
        .collect();
    let normalization: f64 = raw.iter().sum();
    (0..c.len())
        .map(|i| {
            let x = i as i64 % w;
            let y = i as i64 / w;
            let mut result = 0.0;
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = x + dx;
                    let sy = y + dy;
                    if sx >= 0 && sx < w && sy >= 0 && sy < h {
                        result += f64::from(c[(sy * w + sx) as usize])
                            * raw[(dx + r) as usize]
                            * raw[(dy + r) as usize]
                            / (normalization * normalization);
                    }
                }
            }
            result
        })
        .collect()
}
#[test]
fn normalized_gaussian_matches_independent_two_dimensional_impulse_step_and_boundary() {
    for (size, original, sigma) in [
        (
            (7, 7),
            (0..49)
                .map(|i| if i == 24 { 1.0 } else { 0.0 })
                .collect::<Vec<_>>(),
            0.5,
        ),
        (
            (5, 5),
            (0..25).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect(),
            0.5,
        ),
        ((3, 3), vec![1.0; 9], 0.5),
        (
            (7, 7),
            (0..49)
                .map(|i| if i % 7 >= 3 { 1.0 } else { 0.0 })
                .collect(),
            1.0,
        ),
    ] {
        let expected = direct_gaussian(&original, size, sigma);
        let mut actual = original;
        feather(&mut actual, size, sigma, &Scratch::new(65536)).unwrap();
        for (a, e) in actual.into_iter().zip(expected) {
            close(a, e);
        }
    }
    let mut c = [0.1, 0.3, 0.7, 0.9];
    let before = c.map(f32::to_bits);
    feather(&mut c, (2, 2), 0.0, &Scratch::new(0)).unwrap();
    assert_eq!(c.map(f32::to_bits), before);
}
#[test]
fn wide_gaussian_full_one_domain_stays_in_tolerance_and_is_deterministic() {
    // Radius150 creates301taps per axis; retained f32 taps are required, but
    // accumulation may use f64 to avoid a false coverage-domain rejection.
    let size = (305, 305);
    let mut a = vec![1.0; count(size).unwrap()];
    let mut b = a.clone();
    feather(&mut a, size, 50.0, &Scratch::new(1_000_000)).unwrap();
    feather(&mut b, size, 50.0, &Scratch::new(1_000_000)).unwrap();
    close(a[152 * 305 + 152], 1.0);
    assert_eq!(a, b);
    let raw: f64 = (-150..=150)
        .map(|k| (-0.5 * (f64::from(k) / 50.0).powi(2)).exp())
        .sum();
    let one_side: f64 = (0..=150)
        .map(|k| (-0.5 * (f64::from(k) / 50.0).powi(2)).exp() / raw)
        .sum();
    close(a[0], one_side * one_side);
}
#[test]
fn bilinear_transparent_extension_uses_pixel_centers_without_border_clamping() {
    let c = [0.1, 0.7, 0.9, 0.3];
    for (x, y, e) in [
        (0.5, 0.5, 0.1),
        (1.0, 1.0, 0.5),
        (0.25, 0.5, 0.075),
        (0.0, 0.0, 0.025),
        (-0.5, 0.5, 0.0),
        (1.5, 1.5, 0.3),
    ] {
        close(bilinear(&c, (2, 2), x, y).unwrap(), e);
    }
    assert!(bilinear(&c, (2, 2), f64::NAN, 0.5).is_err());
}
#[test]
fn every_operation_seed_and_noncommutative_order_matches_hand_math() {
    for (op, e) in [
        (MaskOperation::Add, 0.7),
        (MaskOperation::Subtract, 0.1),
        (MaskOperation::Intersect, 0.15),
        (MaskOperation::Exclude, 0.55),
    ] {
        close(combine(0.25, 0.6, op).unwrap(), e);
    }
    for (op, e) in [
        (MaskOperation::Add, 0.6),
        (MaskOperation::Subtract, 0.4),
        (MaskOperation::Intersect, 0.6),
        (MaskOperation::Exclude, 0.6),
    ] {
        let mut m = mask(0.6);
        m.operation = op;
        let mut p = [[1.0; 4]; 4];
        apply_stack(&mut p, &stack(vec![m])).unwrap();
        close(p[0][3], e);
    }
    let a = mask(0.4);
    let mut b = mask(0.25);
    b.operation = MaskOperation::Subtract;
    let mut p = [[1.0; 4]; 4];
    apply_stack(&mut p, &stack(vec![a.clone(), b.clone()])).unwrap();
    close(p[0][3], 0.3);
    let mut p = [[1.0; 4]; 4];
    apply_stack(&mut p, &stack(vec![b, a])).unwrap();
    close(p[0][3], 0.85);
}
#[test]
fn original_coordinate_gradient_and_transparent_paint_remain_independent_of_expansion() {
    let mut m = mask(1.0);
    m.paint=serde_json::from_value(serde_json::json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":2,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0.25}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":0.75}}]})).unwrap();
    let mut g = rectangle();
    g.size = (4, 1);
    g.origin = (-1.0, 0.0);
    let mut c = [0.5; 4];
    apply_paint(&mut c, &g, &m).unwrap();
    for (a, e) in c.into_iter().zip([0.125, 0.1875, 0.3125, 0.375]) {
        close(a, e);
    }
    m.channel = MaskChannel::Luma;
    let mut c = [0.5; 4];
    apply_paint(&mut c, &g, &m).unwrap();
    for (a, e) in c.into_iter().zip([0.026575, 0.0267, 0.02695, 0.027075]) {
        close(a, e);
    }
    m.paint = solid(0.0);
    let mut c = [1.0; 4];
    apply_paint(&mut c, &g, &m).unwrap();
    assert_eq!(c, [0.0; 4]);
}
#[test]
fn subpixel_geometry_aa_and_expansion_precede_painted_alpha() {
    let mut m = mask(0.25);
    let g = m.grid.as_mut().unwrap();
    g.contours[0].points[1].x = 0.25;
    g.contours[0].points[2].x = 0.25;
    g.analytic_bounds[2] = 0.25;
    let scratch = Scratch::new(m.certified_scratch_bytes);
    let c = local_coverage(m.grid.as_ref().unwrap(), &m, &scratch).unwrap();
    close(c[2 * 6 + 2], 64.0 / 255.0 * 0.25);
    drop(c);
    m.expansion_grid = 0.25;
    let c = local_coverage(m.grid.as_ref().unwrap(), &m, &scratch).unwrap();
    close(c[2 * 6 + 2], (64.0 / 255.0 + 0.25) * 0.25);
    close(c[2 * 6 + 3], 0.25 * 0.25);
}
#[test]
fn source_premultiplied_domain_rejects_invalid_and_clamps_only_after_success() {
    let bad = [
        [f32::NAN, 0., 0., 0.5],
        [0.6, 0., 0., 0.5],
        [0., 0., 0., 1.1],
        [-0.01, 0., 0., 0.5],
    ];
    for pixel in bad {
        let mut p = [pixel; 4];
        let before = p.map(|v| v.map(f32::to_bits));
        assert!(apply_stack(&mut p, &stack(vec![mask(0.5)])).is_err());
        assert_eq!(p.map(|v| v.map(f32::to_bits)), before);
    }
    let alpha = 1e-9f32;
    let boundary = alpha + 1e-6f32;
    let mut p = [[boundary, -1e-6, 0., alpha]; 4];
    apply_stack(&mut p, &stack(vec![mask(0.5)])).unwrap();
    assert_eq!(p[0], [alpha * 0.5, 0., 0., alpha * 0.5]);
    let mut p = [[f32::from_bits(boundary.to_bits() + 1), 0., 0., alpha]; 4];
    assert!(apply_stack(&mut p, &stack(vec![mask(0.5)])).is_err());
}
#[test]
fn late_paint_failure_and_invalid_certified_facts_leave_source_bits_unchanged() {
    let source = [[0.4, 0.2, 0.1, 0.8]; 4];
    let mut late = mask(1.0);
    late.paint = solid(f64::NAN);
    let mut p = source;
    let error = apply_stack(&mut p, &stack(vec![mask(0.25), late])).unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(p, source);
    for mutate in 0..7 {
        let mut m = mask(1.0);
        match mutate {
            0 => m.inverse[0] = f64::NAN,
            1 => m.opacity = 1.1,
            2 => m.expansion_grid = f64::INFINITY,
            3 => m.feather_grid = -1.0,
            4 => m.certified_scratch_bytes = 1,
            5 => m.grid.as_mut().unwrap().density = 0.0,
            _ => m.inverse = [0.0; 6],
        }
        let mut p = source;
        assert!(apply_stack(&mut p, &stack(vec![mask(0.25), m])).is_err());
        assert_eq!(p, source);
    }
    let mut p = source;
    let mut s = stack(vec![mask(1.0)]);
    s.accumulated_bytes = 15;
    assert!(apply_stack(&mut p, &s).is_err());
    assert_eq!(p, source);
}
#[test]
fn scratch_allocation_ledger_measures_live_capacity_and_releases_all_paths() {
    let scratch = Scratch::new(64);
    let a = scratch.buffer(4, 0u64).unwrap();
    assert_eq!(scratch.live.get(), 32);
    let b = scratch.buffer(8, 0u32).unwrap();
    assert_eq!(scratch.peak.get(), 64);
    assert!(scratch.buffer(1, 0u8).is_err());
    assert_eq!(scratch.live.get(), 64);
    drop(a);
    assert_eq!(scratch.live.get(), 32);
    drop(b);
    assert_eq!(scratch.live.get(), 0);
    assert!(scratch.buffer(usize::MAX, 0u64).is_err());
    assert_eq!(scratch.live.get(), 0);
    let m = mask(0.25);
    let scratch = Scratch::new(m.certified_scratch_bytes);
    let c = local_coverage(m.grid.as_ref().unwrap(), &m, &scratch).unwrap();
    assert!(scratch.peak.get() <= m.certified_scratch_bytes);
    drop(c);
    assert_eq!(scratch.live.get(), 0);
}
#[test]
fn empty_stack_preserves_original_source_bits_and_validation_path() {
    let mut s = stack(vec![]);
    s.owner.size = (0, 0);
    s.owner.density = f64::NAN;
    s.accumulated_bytes = 0;
    let mut p = [[f32::NAN, -0.0, 2.0, f32::INFINITY]];
    let before = p.map(|v| v.map(f32::to_bits));
    apply_stack(&mut p, &s).unwrap();
    assert_eq!(p.map(|v| v.map(f32::to_bits)), before);
}

#[test]
fn zero_origin_density_noncentral_inverse_opacity_and_inversion_match_hand_coordinates() {
    let mut m = mask(1.0);
    let g = m.grid.as_mut().unwrap();
    g.origin = (3.0, 7.0);
    g.size = (2, 2);
    g.density = 4.0;
    g.analytic_bounds = [3., 7., 3.5, 7.5];
    g.contours[0].points = vec![
        VectorPoint { x: 3., y: 7. },
        VectorPoint { x: 3.5, y: 7. },
        VectorPoint { x: 3.5, y: 7.5 },
        VectorPoint { x: 3., y: 7.5 },
    ];
    m.paint=serde_json::from_value(serde_json::json!({"type":"linearGradient","start":{"x":3,"y":7},"end":{"x":3,"y":7.5},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0.1}},{"offset":1,"color":{"r":1,"g":0,"b":0,"a":0.9}}]})).unwrap();
    // Owner(.75,.75)→q(3.125,7.25)→grid(.5,1); scalar average(.3,.7)=.5.
    m.inverse = [0., -1., 0.5, 0., 2.75, 8.];
    m.opacity = 0.4;
    m.inverted = true;
    let mut s = stack(vec![m]);
    s.owner.size = (4, 4);
    s.owner.density = 2.;
    s.accumulated_bytes = 64;
    let mut p = [[1.0; 4]; 16];
    apply_stack(&mut p, &s).unwrap();
    close(p[5][3], 0.8);
}
#[test]
fn evenodd_holes_implicit_closure_and_source_zero_alpha_are_preserved() {
    let mut m = mask(1.0);
    let g = m.grid.as_mut().unwrap();
    g.size = (10, 10);
    g.origin = (-2., -2.);
    g.analytic_bounds = [0., 0., 6., 6.];
    g.contours = vec![
        Contour {
            points: vec![
                VectorPoint { x: 0., y: 0. },
                VectorPoint { x: 6., y: 0. },
                VectorPoint { x: 6., y: 6. },
                VectorPoint { x: 0., y: 6. },
            ],
            closed: false,
        },
        Contour {
            points: vec![
                VectorPoint { x: 2., y: 2. },
                VectorPoint { x: 4., y: 2. },
                VectorPoint { x: 4., y: 4. },
                VectorPoint { x: 2., y: 4. },
            ],
            closed: false,
        },
    ];
    g.fill_rule = FillRule::Evenodd;
    g.segments = 8;
    m.certified_scratch_bytes = required_scratch(m.grid.as_ref().unwrap(), m.feather_grid);
    let scratch = Scratch::new(m.certified_scratch_bytes);
    let c = local_coverage(m.grid.as_ref().unwrap(), &m, &scratch).unwrap();
    close(c[2 * 10 + 2], 1.);
    close(c[4 * 10 + 4], 0.);
    drop(c);
    let mut s = stack(vec![m]);
    s.owner.size = (6, 6);
    s.accumulated_bytes = 144;
    s.peak_scratch_bytes = s.masks[0].certified_scratch_bytes;
    let mut p = [[0.2, 0.1, 0.05, 0.4]; 36];
    p[0] = [0.; 4];
    apply_stack(&mut p, &s).unwrap();
    assert_eq!(p[0], [0.; 4]);
    assert_eq!(p[2 * 6 + 2], [0.; 4]);
    assert_eq!(p[1], [0.2, 0.1, 0.05, 0.4]);
}
#[test]
fn simultaneous_edt_kernel_and_scalar_capacity_fit_reserved_scratch() {
    let mut m = mask(0.25);
    m.expansion_grid = 0.75;
    m.feather_grid = 0.5;
    m.certified_scratch_bytes = required_scratch(m.grid.as_ref().unwrap(), m.feather_grid);
    let scratch = Scratch::new(m.certified_scratch_bytes);
    let c = local_coverage(m.grid.as_ref().unwrap(), &m, &scratch).unwrap();
    assert!(scratch.peak.get() <= m.certified_scratch_bytes);
    let opaque = crate::evaluated_scene::masks::fill_transient_bytes(4, 1, (6, 6)).unwrap();
    assert!(scratch.peak.get() >= opaque + 8 * 36); // conservative opaque precharge
    drop(c);
    assert_eq!(scratch.live.get(), 0);
    // Observe the explicit EDT/scalar and feather/scalar overlap separately:
    // opaque fill reservation intentionally dominates the combined peak.
    let explicit = Scratch::new(64 * 36 + 16 * 12 + 4 * 5);
    let mut scalar = explicit.buffer(36, 0.25f32).unwrap();
    expand(
        &mut scalar,
        (6, 6),
        0.75,
        &explicit,
        &mut DistanceWork::default(),
    )
    .unwrap();
    assert!(explicit.peak.get() > 8 * 36);
    feather(&mut scalar, (6, 6), 0.5, &explicit).unwrap();
    assert!(explicit.peak.get() <= explicit.limit);
    drop(scalar);
    assert_eq!(explicit.live.get(), 0);
    let mut m = mask(1.0);
    m.feather_grid = 1.0;
    let required = required_scratch(m.grid.as_ref().unwrap(), m.feather_grid);
    m.certified_scratch_bytes = required - 1;
    let mut p = [[1.; 4]; 4];
    assert!(apply_stack(&mut p, &stack(vec![m])).is_err());
    assert_eq!(p, [[1.; 4]; 4]);
}
#[test]
fn unsafe_fill_conversion_and_nonfinite_runtime_sampling_leave_pixels_unchanged() {
    let before = [[0.4, 0.2, 0.1, 0.8]; 4];
    let mut m = mask(1.0);
    m.grid.as_mut().unwrap().contours[0].points[1].x = 1e30;
    let mut p = before;
    assert!(apply_stack(&mut p, &stack(vec![m])).is_err());
    assert_eq!(p, before);
    let mut m = mask(1.0);
    m.inverse = [f64::MAX, 0., 0., 1., f64::MAX, 0.];
    let mut p = before;
    assert!(apply_stack(&mut p, &stack(vec![m])).is_err());
    assert_eq!(p, before);
    assert!(count((u32::MAX, u32::MAX)).is_err());
    assert!(unit(f32::from_bits((1.0f32 + 1e-6).to_bits() + 1)).is_err());
}

#[test]
fn skewed_inverse_samples_original_gradient_without_stretching_paint() {
    let mut m = mask(1.0);
    m.paint=serde_json::from_value(serde_json::json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":2,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0}},{"offset":1,"color":{"r":1,"g":0,"b":0,"a":1}}]})).unwrap();
    m.inverse = [1., -0.5, -0.25, 1., 0.25, 0.5];
    m.opacity = 0.4;
    let mut p = [[1.; 4]; 4];
    apply_stack(&mut p, &stack(vec![m])).unwrap();
    // Owner(.5,.5)→q(.625,.75): original gradient alpha .3125, opacity .4 once.
    close(p[0][3], 0.125);
}
#[test]
fn valid_roundoff_source_is_not_clamped_when_a_later_mask_fails() {
    let alpha = 1e-9f32;
    let source = [[alpha + 1e-6, -1e-6, 0., alpha]; 4];
    let mut p = source;
    let mut late = mask(1.);
    late.paint = solid(f64::NAN);
    assert!(apply_stack(&mut p, &stack(vec![mask(0.25), late])).is_err());
    assert_eq!(
        p.map(|v| v.map(f32::to_bits)),
        source.map(|v| v.map(f32::to_bits))
    );
}
#[test]
fn adopted_capacity_excess_fails_without_corrupting_scratch_ledger() {
    let scratch = Scratch::new(16);
    let mut vector = Vec::with_capacity(8);
    vector.push(0u32);
    assert!(scratch.adopt(vector).is_err());
    assert_eq!(scratch.live.get(), 0);
    assert_eq!(scratch.peak.get(), 0);
    let mut m = mask(1.);
    m.grid.as_mut().unwrap().size = (16384, 16384);
    let mut p = [[1.; 4]; 4];
    assert!(apply_stack(&mut p, &stack(vec![m])).is_err());
    assert_eq!(p, [[1.; 4]; 4]);
}

#[test]
fn opaque_fill_undersized_certificate_and_forged_segments_reject_before_source_mutation() {
    let source = [[0.4, 0.2, 0.1, 0.8]; 4];
    let mut mask = mask(0.25);
    // The old pixel/EDT-only reservation fits every explicit buffer, but omits
    // the independent opaque path/scanner bound and must now fail closed.
    let legacy = 64 * 36 + 16 * 12;
    mask.certified_scratch_bytes = legacy;
    let scratch = Scratch::new(legacy);
    let error = local_coverage(mask.grid.as_ref().unwrap(), &mask, &scratch)
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(scratch.live.get(), 0);
    assert_eq!(scratch.allocations.get(), 0);
    let mut pixels = source;
    let error = apply_stack(&mut pixels, &stack(vec![mask])).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(
        pixels.map(|p| p.map(f32::to_bits)),
        source.map(|p| p.map(f32::to_bits))
    );
    let mut forged = self::mask(1.);
    forged.grid.as_mut().unwrap().segments = 0;
    forged.certified_scratch_bytes = required_scratch(forged.grid.as_ref().unwrap(), 0.);
    let mut pixels = source;
    let error = apply_stack(&mut pixels, &stack(vec![forged])).unwrap_err();
    assert!(error.message.contains("underdeclares emitted lines"));
    assert_eq!(pixels, source);
}
