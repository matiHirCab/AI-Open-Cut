use super::*;
use crate::MatteChannel;
use crate::evaluated_scene::mattes::{
    DirectDraw, MATTE_CACHE_RESERVATION, MatteFrameCertificate, MatteTask, MatteTaskId,
};
use std::mem::size_of;
fn id(n: usize) -> MatteTaskId {
    MatteTaskId(n)
}
fn leaf(n: usize, time: u64, provider: Option<(MatteTaskId, MatteChannel)>) -> MatteTask {
    MatteTask::LeafSample {
        relative_owner: None,
        layer_index: n,
        at_ms: time,
        provider,
        source_live_bytes: 512,
    }
}
fn plane(
    left: usize,
    top: usize,
    width: usize,
    height: usize,
    pixel: [f32; 4],
    gain: f32,
) -> LeafSamplePlane {
    LeafSamplePlane {
        plane: LinearPlane {
            left,
            top,
            width,
            height,
            pixels: vec![pixel; width * height],
        },
        gain,
    }
}
fn fixture(
    canvas: (u32, u32),
    tasks: Vec<MatteTask>,
    direct_draw: Vec<MatteTaskId>,
) -> MatteFrameSchedule {
    let mut last_uses: Vec<_> = (0..tasks.len()).collect();
    for (i, task) in tasks.iter().enumerate() {
        match task {
            MatteTask::LeafSample {
                provider: Some((p, _)),
                ..
            } => last_uses[p.0] = i,
            MatteTask::AverageCopy { samples, .. }
            | MatteTask::AggregateProvider { copies: samples } => {
                for p in samples {
                    last_uses[p.0] = i
                }
            }
            _ => {}
        }
    }
    for (i, p) in direct_draw.iter().enumerate() {
        last_uses[p.0] = tasks.len() + i;
    }
    let owner_count = tasks
        .iter()
        .filter_map(|task| match task {
            MatteTask::LeafSample { layer_index, .. }
            | MatteTask::AverageCopy { layer_index, .. } => Some(*layer_index + 1),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let owner_modes = vec![crate::BlendMode::Normal; owner_count];
    let direct_draw = direct_draw
        .into_iter()
        .map(|task| {
            let layer_index = match tasks[task.0] {
                MatteTask::LeafSample { layer_index, .. }
                | MatteTask::AverageCopy { layer_index, .. } => layer_index,
                _ => 0,
            };
            DirectDraw {
                task,
                layer_index,
                blend_mode: crate::BlendMode::Normal,
                destination_visits: u64::from(canvas.0) * u64::from(canvas.1),
            }
        })
        .collect::<Vec<_>>();
    let descriptor = size_of::<MatteFrameSchedule>()
        + tasks.capacity() * size_of::<MatteTask>()
        + direct_draw.capacity() * size_of::<DirectDraw>()
        + owner_modes.capacity() * size_of::<crate::BlendMode>()
        + last_uses.capacity() * size_of::<usize>()
        + size_of::<Vec<Option<LinearPlane>>>()
        + 128 * tasks.len()
        + tasks
            .iter()
            .map(|t| match t {
                MatteTask::AverageCopy { samples, .. }
                | MatteTask::AggregateProvider { copies: samples } => {
                    samples.capacity() * size_of::<MatteTaskId>()
                }
                _ => 0,
            })
            .sum::<usize>();
    let pixels = u64::from(canvas.0) * u64::from(canvas.1);
    let requests = tasks
        .iter()
        .filter(|t| matches!(t, MatteTask::AggregateProvider { .. }))
        .count() as u64;
    let work = tasks
        .iter()
        .map(|t| match t {
            MatteTask::AggregateProvider { copies } => 4 * pixels * copies.len() as u64,
            MatteTask::LeafSample {
                provider: Some(_), ..
            } => 5 * pixels,
            _ => 0,
        })
        .sum();
    let fixed = MATTE_CACHE_RESERVATION + 16 * pixels + descriptor as u64;
    let peak = fixed + 64 * pixels * (tasks.len() as u64 + 1) + 512;
    MatteFrameSchedule {
        query: None,
        canvas,
        tasks,
        direct_draw,
        owner_modes,
        last_uses,
        certificate: MatteFrameCertificate {
            provider_requests: requests,
            matte_work_units: work,
            blend_work_units: 0,
            fixed_live_bytes: fixed,
            descriptor_bytes: descriptor as u64,
            peak_live_bytes: peak,
        },
    }
}
fn close(a: f32, b: f64) {
    assert!((f64::from(a) - b).abs() <= 1e-6, "{a} != {b}");
}
#[test]
fn alpha_and_linear_luma_have_independent_color_oracles() {
    for (channel, source, expected) in [
        (MatteChannel::Alpha, [0.5, 0., 0., 0.5], 0.5),
        (MatteChannel::Luma, [0.5, 0., 0., 0.5], 0.1063),
        (MatteChannel::Luma, [0., 0.5, 0., 0.5], 0.3576),
        (MatteChannel::Luma, [0., 0., 0.5, 0.5], 0.0361),
        (
            MatteChannel::Luma,
            [0.21404114, 0., 0., 1.],
            0.04550514646652264,
        ),
    ] {
        let s = fixture(
            (1, 1),
            vec![
                leaf(0, 0, None),
                MatteTask::AggregateProvider {
                    copies: vec![id(0)],
                },
                leaf(1, 0, Some((id(1), channel))),
                MatteTask::AverageCopy {
                    layer_index: 1,
                    samples: vec![id(2)],
                },
            ],
            vec![id(3)],
        );
        let mut dst = [[0., 0., 0., 1.]];
        compose_frame(&s, &mut dst, &mut |n, _| {
            Ok(plane(
                0,
                0,
                1,
                1,
                if n == 0 { source } else { [0.2, 0.4, 0.6, 0.8] },
                1.,
            ))
        })
        .unwrap();
        for (value, recipient) in dst[0][..3].iter().zip([0.2, 0.4, 0.6]) {
            close(*value, recipient * expected);
        }
        assert_eq!(dst[0][3], 1.);
    }
}
#[test]
fn overlapping_copies_average_before_aggregation_and_draw_individuals_once() {
    let tasks = vec![
        leaf(0, 0, None),
        leaf(0, 1, None),
        leaf(1, 0, None),
        leaf(1, 1, None),
        MatteTask::AverageCopy {
            layer_index: 0,
            samples: vec![id(0), id(1)],
        },
        MatteTask::AverageCopy {
            layer_index: 1,
            samples: vec![id(2), id(3)],
        },
        MatteTask::AggregateProvider {
            copies: vec![id(4), id(5)],
        },
        leaf(2, 0, Some((id(6), MatteChannel::Alpha))),
        MatteTask::AverageCopy {
            layer_index: 2,
            samples: vec![id(7)],
        },
    ];
    for direct in [vec![id(8)], vec![id(4), id(5)]] {
        let s = fixture((1, 1), tasks.clone(), direct);
        let mut calls = 0;
        let mut dst = [[0., 0., 0., 1.]];
        compose_frame(&s, &mut dst, &mut |n, t| {
            calls += 1;
            let a = if n == 2 {
                1.
            } else if (n == 0 && t == 0) || (n == 1 && t == 1) {
                0.5
            } else {
                0.
            };
            Ok(plane(0, 0, 1, 1, [a; 4], 1.))
        })
        .unwrap();
        assert_eq!(calls, 5); // immutable result reuse, not a second provider shutter
        close(dst[0][0], 0.4375);
        assert!((dst[0][0] - 0.5).abs() > 0.01);
        assert!((dst[0][0] - 0.68359375).abs() > 0.1);
    }
}
#[test]
fn provider_gain_precedes_luma_and_recipient_gain_follows_coverage() {
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Luma))),
        ],
        vec![id(2)],
    );
    let mut dst = [[0., 0., 0., 1.]];
    compose_frame(&s, &mut dst, &mut |n, _| {
        Ok(plane(
            0,
            0,
            1,
            1,
            if n == 0 { [0.5, 0., 0., 0.5] } else { [1.; 4] },
            0.5,
        ))
    })
    .unwrap();
    for color in &dst[0][..3] {
        close(*color, 0.026575);
    }
}
#[test]
fn empty_hidden_provider_is_zero_and_support_never_grows() {
    let s = fixture(
        (3, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Alpha))),
        ],
        vec![id(2)],
    );
    let mut dst = [[0.25, 0.125, 0., 1.]; 3];
    compose_frame(&s, &mut dst, &mut |n, _| {
        Ok(if n == 0 {
            plane(0, 0, 0, 0, [0.; 4], 1.)
        } else {
            plane(1, 0, 1, 1, [1.; 4], 1.)
        })
    })
    .unwrap();
    assert_eq!(dst, [[0.25, 0.125, 0., 1.]; 3]);
    let mut dst = [[0., 0., 0., 1.]; 3];
    compose_frame(&s, &mut dst, &mut |n, _| {
        Ok(if n == 0 {
            plane(0, 0, 3, 1, [1.; 4], 1.)
        } else {
            plane(1, 0, 1, 1, [0.5, 0., 0., 0.5], 1.)
        })
    })
    .unwrap();
    assert_eq!(dst[0], [0., 0., 0., 1.]);
    assert_eq!(dst[2], [0., 0., 0., 1.]);
    close(dst[1][0], 0.5);
}

#[test]
fn bad_premultiplied_pixels_gain_and_late_callback_preserve_destination_bits() {
    let source = [[0.25, -0.0, 0., 1.]];
    let s = fixture(
        (1, 1),
        vec![leaf(0, 0, None), leaf(1, 0, None)],
        vec![id(0), id(1)],
    );
    for (bad, gain) in [
        ([f32::NAN, 0., 0., 0.5], 1.),
        ([0., 0., 0., f32::INFINITY], 1.),
        ([0., 0., 0., 1.01], 1.),
        ([-0.01, 0., 0., 0.5], 1.),
        ([0.6, 0., 0., 0.5], 1.),
        ([0.5, 0., 0., 0.5], f32::NAN),
        ([0.5, 0., 0., 0.5], 1.01),
        ([0.5, 0., 0., 0.5], -0.01),
    ] {
        let mut dst = source;
        let error = compose_frame(&s, &mut dst, &mut |n, _| {
            Ok(plane(
                0,
                0,
                1,
                1,
                if n == 0 { [0.5, 0., 0., 0.5] } else { bad },
                if n == 0 { 1. } else { gain },
            ))
        })
        .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert_eq!(
            dst.map(|p| p.map(f32::to_bits)),
            source.map(|p| p.map(f32::to_bits))
        );
    }
    let mut dst = source;
    let error = compose_frame(&s, &mut dst, &mut |n, _| {
        if n == 1 {
            Err(CoreError::new(
                crate::ErrorCode::DependencyUnavailable,
                "independent late failure",
            ))
        } else {
            Ok(plane(0, 0, 1, 1, [0.5, 0., 0., 0.5], 1.))
        }
    })
    .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::DependencyUnavailable);
    assert_eq!(
        dst.map(|p| p.map(f32::to_bits)),
        source.map(|p| p.map(f32::to_bits))
    );
}
#[test]
fn canonical_f32_tolerance_clamps_before_luma_without_tiny_alpha_cutoff() {
    let alpha = 1e-9f32;
    let boundary = alpha + 1e-6f32;
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Luma))),
        ],
        vec![id(2)],
    );
    let mut dst = [[0., 0., 0., 1.]];
    compose_frame(&s, &mut dst, &mut |n, _| {
        Ok(plane(
            0,
            0,
            1,
            1,
            if n == 0 {
                [boundary, 0., 0., alpha]
            } else {
                [1.; 4]
            },
            1.,
        ))
    })
    .unwrap();
    let expected = (f64::from(alpha) * 0.2126) as f32;
    assert!(dst[0][0] > 0.);
    assert!((dst[0][0] - expected).abs() < 1e-15);
    let mut dst = [[0., 0., 0., 1.]];
    let next = f32::from_bits(boundary.to_bits() + 1);
    assert!(
        compose_frame(&s, &mut dst, &mut |n, _| Ok(plane(
            0,
            0,
            1,
            1,
            if n == 0 {
                [next, 0., 0., alpha]
            } else {
                [1.; 4]
            },
            1.
        )))
        .is_err()
    );
    assert_eq!(dst, [[0., 0., 0., 1.]]);
}
#[test]
fn weighted_duplicate_samples_and_exact_live_task_reuse_do_not_resample() {
    let large = (1u64 << 53) + 1;
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, large, None),
            leaf(0, large + 1, None),
            MatteTask::AverageCopy {
                layer_index: 0,
                samples: vec![id(0), id(0), id(1)],
            },
            MatteTask::AggregateProvider {
                copies: vec![id(2)],
            },
            leaf(1, large, Some((id(3), MatteChannel::Alpha))),
            leaf(2, large, Some((id(3), MatteChannel::Alpha))),
        ],
        vec![id(4), id(5)],
    );
    let mut calls = Vec::new();
    let mut dst = [[0., 0., 0., 1.]];
    compose_frame(&s, &mut dst, &mut |n, t| {
        calls.push((n, t));
        Ok(plane(
            0,
            0,
            1,
            1,
            if n == 0 {
                if t == large { [0.25; 4] } else { [1.; 4] }
            } else {
                [1.; 4]
            },
            1.,
        ))
    })
    .unwrap();
    assert_eq!(
        calls,
        vec![(0, large), (0, large + 1), (1, large), (2, large)]
    );
    // Weighted copy alpha=(.25+.25+1)/3=.5; two draws yield .75.
    for c in &dst[0][..3] {
        close(*c, 0.75);
    }
}
#[test]
fn forged_descriptor_last_use_work_request_and_peak_fail_before_callback() {
    let original = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Alpha))),
        ],
        vec![id(2)],
    );
    for kind in 0..7 {
        let mut s = original.clone();
        match kind {
            0 => s.certificate.descriptor_bytes = 0,
            1 => s.certificate.fixed_live_bytes = MATTE_CACHE_RESERVATION,
            2 => s.certificate.provider_requests = 0,
            3 => s.certificate.matte_work_units = 0,
            4 => s.certificate.peak_live_bytes = s.certificate.fixed_live_bytes - 1,
            5 => s.last_uses[0] = 0,
            _ => s.last_uses.pop().map(|_| ()).unwrap(),
        }
        let mut calls = 0;
        let mut dst = [[0.125, 0., 0., 1.]];
        let error = compose_frame(&s, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 1, 1, [1.; 4], 1.))
        })
        .unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
        assert_eq!(calls, 0);
        assert_eq!(dst, [[0.125, 0., 0., 1.]]);
    }
}
#[test]
fn callback_capacity_is_charged_and_undersized_source_reserve_preserves_pixels() {
    let mut s = fixture((1, 1), vec![leaf(0, 0, None)], vec![id(0)]);
    s.certificate.peak_live_bytes = s.certificate.fixed_live_bytes + 511;
    let mut calls = 0;
    let mut dst = [[0.125, 0., 0., 1.]];
    assert!(
        compose_frame(&s, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 1, 1, [1.; 4], 1.))
        })
        .is_err()
    );
    assert_eq!(calls, 0);
    s.certificate.peak_live_bytes = s.certificate.fixed_live_bytes + 4096;
    let mut pixels = Vec::with_capacity(65);
    pixels.push([1.; 4]);
    let mut returned = Some(LeafSamplePlane {
        plane: LinearPlane {
            left: 0,
            top: 0,
            width: 1,
            height: 1,
            pixels,
        },
        gain: 1.,
    });
    assert!(compose_frame(&s, &mut dst, &mut |_, _| Ok(returned.take().unwrap())).is_err());
    assert_eq!(dst, [[0.125, 0., 0., 1.]]);
}
#[test]
fn out_of_canvas_size_mismatch_empty_shape_and_forward_task_references_fail_closed() {
    let s = fixture((1, 1), vec![leaf(0, 0, None)], vec![id(0)]);
    for kind in 0..4 {
        let mut dst = [[0., 0., 0., 1.]];
        assert!(
            compose_frame(&s, &mut dst, &mut |_, _| {
                let mut p = plane(0, 0, 1, 1, [1.; 4], 1.);
                match kind {
                    0 => p.plane.left = 1,
                    1 => p.plane.width = usize::MAX,
                    2 => p.plane.pixels.clear(),
                    _ => {
                        p.plane.width = 0;
                        p.plane.pixels.clear();
                    }
                }
                Ok(p)
            })
            .is_err()
        );
        assert_eq!(dst, [[0., 0., 0., 1.]]);
    }
    let mut s = fixture((1, 1), vec![leaf(0, 0, None)], vec![id(0)]);
    s.tasks[0] = MatteTask::AverageCopy {
        layer_index: 0,
        samples: vec![id(0)],
    };
    let mut calls = 0;
    let mut dst = [[0., 0., 0., 1.]];
    assert!(
        compose_frame(&s, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 1, 1, [1.; 4], 1.))
        })
        .is_err()
    );
    assert_eq!(calls, 0);
}

#[test]
fn isolated_luma_does_not_sample_the_destination_background() {
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Luma))),
        ],
        vec![id(2)],
    );
    for background in [[0., 0., 0., 1.], [0., 0.4, 0., 1.]] {
        let mut dst = [background];
        compose_frame(&s, &mut dst, &mut |n, _| {
            Ok(plane(
                0,
                0,
                1,
                1,
                if n == 0 {
                    [0.5, 0., 0., 0.5]
                } else {
                    [0., 0., 0.8, 0.8]
                },
                if n == 0 { 1. } else { 0.75 },
            ))
        })
        .unwrap();
        close(dst[0][2], 0.06378);
        close(dst[0][1], f64::from(background[1]) * (1. - 0.06378));
    }
}
#[test]
fn chained_provider_mattes_and_gain_match_hand_multiplied_luma() {
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Alpha))),
            MatteTask::AggregateProvider {
                copies: vec![id(2)],
            },
            leaf(2, 0, Some((id(3), MatteChannel::Luma))),
        ],
        vec![id(4)],
    );
    let mut dst = [[0., 0., 0., 1.]];
    compose_frame(&s, &mut dst, &mut |n, _| {
        Ok(match n {
            0 => plane(0, 0, 1, 1, [0., 0., 0., 0.5], 1.),
            1 => plane(0, 0, 1, 1, [0.5, 0., 0., 0.5], 0.8),
            _ => plane(0, 0, 1, 1, [0.6; 4], 0.5),
        })
    })
    .unwrap();
    for c in &dst[0][..3] {
        close(*c, 0.012756);
    }
}
#[test]
fn ordered_provider_source_over_is_color_noncommutative() {
    for (copies, expected) in [(vec![id(0), id(1)], 0.08925), (vec![id(1), id(0)], 0.12435)] {
        let s = fixture(
            (1, 1),
            vec![
                leaf(0, 0, None),
                leaf(1, 0, None),
                MatteTask::AggregateProvider { copies },
                leaf(2, 0, Some((id(2), MatteChannel::Luma))),
            ],
            vec![id(3)],
        );
        let mut dst = [[0., 0., 0., 1.]];
        compose_frame(&s, &mut dst, &mut |n, _| {
            Ok(plane(
                0,
                0,
                1,
                1,
                match n {
                    0 => [0.5, 0., 0., 0.5],
                    1 => [0., 0., 0.5, 0.5],
                    _ => [1.; 4],
                },
                1.,
            ))
        })
        .unwrap();
        for c in &dst[0][..3] {
            close(*c, expected);
        }
    }
}
#[test]
fn sample_gain_is_applied_before_own_copy_average_and_disjoint_union_stays_transparent() {
    let s = fixture(
        (3, 1),
        vec![
            leaf(0, 0, None),
            leaf(0, 1, None),
            MatteTask::AverageCopy {
                layer_index: 0,
                samples: vec![id(0), id(1)],
            },
            MatteTask::AggregateProvider {
                copies: vec![id(2)],
            },
            leaf(1, 0, Some((id(3), MatteChannel::Alpha))),
        ],
        vec![id(4)],
    );
    let mut dst = [[0., 0., 0., 1.]; 3];
    compose_frame(&s, &mut dst, &mut |n, t| {
        Ok(if n == 1 {
            plane(0, 0, 3, 1, [1.; 4], 1.)
        } else {
            plane(
                if t == 0 { 0 } else { 2 },
                0,
                1,
                1,
                [0.5; 4],
                if t == 0 { 0.5 } else { 1. },
            )
        })
    })
    .unwrap();
    close(dst[0][0], 0.125);
    assert_eq!(dst[1], [0., 0., 0., 1.]);
    close(dst[2][0], 0.25);
}
#[test]
fn zero_alpha_accepted_roundoff_and_alpha_endpoints_use_existing_f32_domain() {
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            leaf(1, 0, Some((id(1), MatteChannel::Luma))),
        ],
        vec![id(2)],
    );
    for (provider, expected) in [
        ([1e-6, 0., 0., 0.], 0.),
        ([0., 0., 0., -0.5e-6], 0.),
        ([1. + 0.5e-6, 0., 0., 1. + 0.5e-6], 0.2126),
    ] {
        let mut dst = [[0., 0., 0., 1.]];
        compose_frame(&s, &mut dst, &mut |n, _| {
            Ok(plane(
                0,
                0,
                1,
                1,
                if n == 0 { provider } else { [1.; 4] },
                1.,
            ))
        })
        .unwrap();
        close(dst[0][0], expected);
    }
}
#[test]
fn actual_outer_and_nested_schedule_capacities_cannot_be_hidden_by_len() {
    let mut tasks = Vec::with_capacity(129);
    tasks.push(leaf(0, 0, None));
    tasks.push(leaf(0, 1, None));
    let mut samples = Vec::with_capacity(65);
    samples.extend([id(0), id(1)]);
    tasks.push(MatteTask::AverageCopy {
        layer_index: 0,
        samples,
    });
    let mut direct = Vec::with_capacity(67);
    direct.push(id(2));
    let mut original = fixture((1, 1), tasks, direct);
    let mut last = Vec::with_capacity(71);
    last.extend(original.last_uses.iter().copied());
    original.last_uses = last;
    let last_spare =
        (original.last_uses.capacity() - original.last_uses.len()) * size_of::<usize>();
    original.certificate.descriptor_bytes += last_spare as u64;
    original.certificate.fixed_live_bytes += last_spare as u64;
    original.certificate.peak_live_bytes += last_spare as u64;
    for decrement in [
        (original.tasks.capacity() - original.tasks.len()) * size_of::<MatteTask>(),
        (65 - 2) * size_of::<MatteTaskId>(),
        (67 - 1) * size_of::<MatteTaskId>(),
        last_spare,
    ] {
        // Clone would erase source spare capacities, so temporarily change only
        // the certificate while retaining the actual original payload.
        original.certificate.descriptor_bytes -= decrement as u64;
        let mut calls = 0;
        let mut dst = [[0., 0., 0., 1.]];
        let error = compose_frame(&original, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 1, 1, [1.; 4], 1.))
        })
        .unwrap_err();
        assert!(error.message.contains("descriptor"));
        assert_eq!(calls, 0);
        original.certificate.descriptor_bytes += decrement as u64;
    }
}
#[test]
fn last_use_releases_unused_planes_and_keeps_dependencies_through_peak() {
    assert!(
        size_of::<Slot>() <= 128,
        "owning descriptor reservation must dominate actual slot layout"
    );
    let mut unused = fixture((1, 1), vec![leaf(0, 0, None), leaf(1, 0, None)], vec![]);
    unused.certificate.peak_live_bytes = unused.certificate.fixed_live_bytes + 512;
    let mut calls = 0;
    let mut dst = [[0., 0., 0., 1.]];
    let observed = execute(&unused, &mut dst, &mut |_, _| {
        calls += 1;
        Ok(plane(0, 0, 1, 1, [0.5; 4], 1.))
    })
    .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(observed.peak, unused.certificate.fixed_live_bytes + 512);
    assert_eq!(observed.bytes, unused.certificate.fixed_live_bytes);
    let mut retained = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            leaf(0, 1, None),
            MatteTask::AverageCopy {
                layer_index: 0,
                samples: vec![id(0), id(1)],
            },
        ],
        vec![id(2)],
    );
    retained.certificate.peak_live_bytes = retained.certificate.fixed_live_bytes + 528;
    let observed = execute(&retained, &mut dst, &mut |_, _| {
        Ok(plane(0, 0, 1, 1, [0.5; 4], 1.))
    })
    .unwrap();
    assert_eq!(observed.peak, retained.certificate.fixed_live_bytes + 528);
    assert_eq!(observed.bytes, retained.certificate.fixed_live_bytes);
    retained.certificate.peak_live_bytes -= 1;
    let mut calls = 0;
    let mut dst = [[0., 0., 0., 1.]];
    assert!(
        compose_frame(&retained, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 1, 1, [0.5; 4], 1.))
        })
        .is_err()
    );
    assert_eq!(calls, 1);
    assert_eq!(dst, [[0., 0., 0., 1.]]);
}
#[test]
fn exact_request_cap_and_repeated_materialization_are_charged_without_large_planes() {
    use crate::evaluated_scene::mattes::MAX_MATTE_REQUESTS;
    let tasks = (0..MAX_MATTE_REQUESTS)
        .map(|_| MatteTask::AggregateProvider { copies: vec![] })
        .collect();
    let s = fixture((1, 1), tasks, vec![]);
    let mut dst = [[0., 0., 0., 1.]];
    let observed = execute(&s, &mut dst, &mut |_, _| {
        panic!("empty provider should not sample")
    })
    .unwrap();
    assert_eq!(observed.requests, 4096);
    assert_eq!(observed.bytes, s.certificate.fixed_live_bytes);
    let tasks = (0..=MAX_MATTE_REQUESTS)
        .map(|_| MatteTask::AggregateProvider { copies: vec![] })
        .collect();
    let s = fixture((1, 1), tasks, vec![]);
    assert!(preflight(&s, 1).is_err());
    let s = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
            MatteTask::AggregateProvider {
                copies: vec![id(0)],
            },
        ],
        vec![],
    );
    let observed = execute(&s, &mut dst, &mut |_, _| {
        Ok(plane(0, 0, 1, 1, [0.5; 4], 1.))
    })
    .unwrap();
    assert_eq!(observed.requests, 2);
    assert_eq!(observed.work, 8);
}
#[test]
fn exact_work_live_limits_and_checked_overflow_have_independent_admission_controls() {
    use crate::evaluated_scene::mattes::{MAX_MATTE_LIVE_BYTES, MAX_MATTE_WORK};
    let tasks = vec![
        leaf(0, 0, None),
        leaf(1, 0, None),
        leaf(2, 0, None),
        leaf(3, 0, None),
        MatteTask::AggregateProvider {
            copies: vec![id(0), id(1), id(2), id(3)],
        },
    ];
    let mut s = fixture((4096, 4096), tasks, vec![]);
    s.certificate.peak_live_bytes = MAX_MATTE_LIVE_BYTES;
    assert_eq!(s.certificate.matte_work_units, 268_435_456);
    preflight(&s, 16_777_216).unwrap();
    s.certificate.matte_work_units = MAX_MATTE_WORK + 1;
    assert!(preflight(&s, 16_777_216).is_err());
    let mut s = fixture((1, 1), vec![leaf(0, 0, None)], vec![]);
    s.certificate.peak_live_bytes = MAX_MATTE_LIVE_BYTES;
    let allowed = MAX_MATTE_LIVE_BYTES - s.certificate.fixed_live_bytes;
    for reserve in [allowed, allowed + 1, u64::MAX] {
        let MatteTask::LeafSample {
            source_live_bytes, ..
        } = &mut s.tasks[0]
        else {
            unreachable!()
        };
        *source_live_bytes = reserve;
        let mut calls = 0;
        let mut dst = [[0., 0., 0., 1.]];
        let result = compose_frame(&s, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 0, 0, [0.; 4], 1.))
        });
        if reserve == allowed {
            assert!(result.is_ok());
            assert_eq!(calls, 1);
        } else {
            assert!(result.is_err());
            assert_eq!(calls, 0);
        }
    }
    assert!(mul(u64::MAX, 2).is_err());
    assert!(add(u64::MAX, 1).is_err());
}
#[test]
fn duplicate_direct_draw_aggregate_draw_and_invalid_destination_fail_before_callbacks() {
    for kind in 0..3 {
        let mut s = fixture(
            (1, 1),
            vec![
                leaf(0, 0, None),
                MatteTask::AggregateProvider {
                    copies: vec![id(0)],
                },
            ],
            vec![id(0)],
        );
        match kind {
            0 => s.direct_draw.push(s.direct_draw[0]),
            1 => s.direct_draw[0].task = id(1),
            _ => {
                s.tasks[1] = MatteTask::AggregateProvider {
                    copies: vec![id(0), id(0)],
                }
            }
        }
        let mut calls = 0;
        let mut dst = [[0., 0., 0., 1.]];
        assert!(
            compose_frame(&s, &mut dst, &mut |_, _| {
                calls += 1;
                Ok(plane(0, 0, 1, 1, [1.; 4], 1.))
            })
            .is_err()
        );
        assert_eq!(calls, 0);
    }
    let s = fixture((1, 1), vec![leaf(0, 0, None)], vec![id(0)]);
    let mut dst = [[f32::NAN, 0., 0., 1.]];
    let before = dst.map(|p| p.map(f32::to_bits));
    let mut calls = 0;
    assert!(
        compose_frame(&s, &mut dst, &mut |_, _| {
            calls += 1;
            Ok(plane(0, 0, 1, 1, [1.; 4], 1.))
        })
        .is_err()
    );
    assert_eq!(calls, 0);
    assert_eq!(dst.map(|p| p.map(f32::to_bits)), before);
}

fn blend_fixture(mode: crate::BlendMode) -> MatteFrameSchedule {
    let mut schedule = fixture((1, 1), vec![leaf(0, 0, None)], vec![id(0)]);
    schedule.owner_modes[0] = mode;
    schedule.direct_draw[0].blend_mode = mode;
    schedule.certificate.blend_work_units = if mode.is_normal() { 0 } else { 32 };
    schedule
}
fn blend_contract() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../../../contracts/blend-modes-v1.json")).unwrap()
}
fn rgba(value: &serde_json::Value) -> [f32; 4] {
    std::array::from_fn(|i| value[i].as_f64().unwrap() as f32)
}
fn bits_rgba(value: &serde_json::Value) -> [f32; 4] {
    std::array::from_fn(|i| f32::from_bits(value[i].as_u64().unwrap() as u32))
}
#[test]
fn canonical_seven_modes_match_independent_partial_alpha_and_black_oracles() {
    let contract = blend_contract();
    for name in contract["fields"]["blendMode"]["values"]
        .as_array()
        .unwrap()
    {
        let mode = serde_json::from_value(name.clone()).unwrap();
        let oracle = &contract["numericOracles"]["translucentColored"];
        let source = bits_rgba(&oracle["source"]["f32Bits"]);
        let backdrop = bits_rgba(&oracle["backdrop"]["f32Bits"]);
        let expected = rgba(&oracle["expected"][name.as_str().unwrap()]["unroundedF64"]);
        let mut destination = [backdrop];
        compose_frame(&blend_fixture(mode), &mut destination, &mut |_, _| {
            Ok(plane(0, 0, 1, 1, source, 1.0))
        })
        .unwrap();
        for c in 0..4 {
            assert!(
                (destination[0][c] - expected[c]).abs() <= f32::EPSILON,
                "{name} channel{c}: {:?}",
                destination[0]
            );
        }
        let mut destination = [[0.0, 0.0, 0.0, 1.0]];
        compose_frame(&blend_fixture(mode), &mut destination, &mut |_, _| {
            Ok(plane(
                0,
                0,
                1,
                1,
                bits_rgba(
                    &contract["numericOracles"]["nativeAuthoredColored"]["source"]["f32Bits"],
                ),
                1.0,
            ))
        })
        .unwrap();
        let expected = rgba(
            &contract["numericOracles"]["firstLayerBlack"][name.as_str().unwrap()]["outputF32"],
        );
        assert_eq!(destination[0], expected, "first layer {name}");
    }
}
#[test]
fn nonnormal_tiny_alpha_uses_original_f32_boundary_then_clamps_before_f64() {
    let contract = blend_contract();
    let oracle = &contract["numericOracles"]["tinyAlphaCanonicalEndpoint"];
    let source = bits_rgba(&oracle["source"]["f32Bits"]);
    let backdrop = bits_rgba(&oracle["backdrop"]["f32Bits"]);
    let mut destination = [backdrop];
    compose_frame(
        &blend_fixture(crate::BlendMode::Screen),
        &mut destination,
        &mut |_, _| Ok(plane(0, 0, 1, 1, source, 1.0)),
    )
    .unwrap();
    let expected = rgba(&oracle["correctScreenF32"]);
    for c in 0..4 {
        assert!((destination[0][c] - expected[c]).abs() <= 1e-8);
    }
    assert_ne!(destination[0], rgba(&oracle["wrongNoClampF32"]));
    let mut invalid = source;
    invalid[0] = f32::from_bits(oracle["nextPositiveBits"].as_u64().unwrap() as u32);
    let before = destination;
    assert!(
        compose_frame(
            &blend_fixture(crate::BlendMode::Screen),
            &mut destination,
            &mut |_, _| Ok(plane(0, 0, 1, 1, invalid, 1.0))
        )
        .is_err()
    );
    assert_eq!(destination, before);
    let negative =
        bits_rgba(&contract["numericOracles"]["tinyAlphaNegativeRoundoff"]["source"]["f32Bits"]);
    let mut destination = [[0.0, 0.0, 0.0, 1.0]];
    compose_frame(
        &blend_fixture(crate::BlendMode::Screen),
        &mut destination,
        &mut |_, _| Ok(plane(0, 0, 1, 1, negative, 1.0)),
    )
    .unwrap();
    assert_eq!(destination, [[0.0, 0.0, 0.0, 1.0]]);
}
#[test]
fn add_blends_after_own_shutter_average_once() {
    let mut schedule = fixture(
        (1, 1),
        vec![
            leaf(0, 0, None),
            leaf(0, 1, None),
            MatteTask::AverageCopy {
                layer_index: 0,
                samples: vec![id(0), id(1)],
            },
        ],
        vec![id(2)],
    );
    schedule.owner_modes[0] = crate::BlendMode::Add;
    schedule.direct_draw[0].blend_mode = crate::BlendMode::Add;
    schedule.certificate.blend_work_units = 32;
    let mut destination = [[0.75, 0.75, 0.75, 1.0]];
    compose_frame(&schedule, &mut destination, &mut |_, time| {
        Ok(plane(
            0,
            0,
            1,
            1,
            if time == 0 {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [1.0; 4]
            },
            1.0,
        ))
    })
    .unwrap();
    assert_eq!(destination, [[1.0; 4]]);
    assert_ne!(destination, [[0.875, 0.875, 0.875, 1.0]]);
}
#[test]
fn owning_mode_owner_and_blend_work_forgeries_fail_before_callbacks() {
    for kind in 0..6 {
        let mut schedule = blend_fixture(crate::BlendMode::Multiply);
        match kind {
            0 => schedule.direct_draw[0].blend_mode = crate::BlendMode::Screen,
            1 => schedule.direct_draw[0].layer_index = 1,
            2 => schedule.tasks[0] = leaf(1, 0, None),
            3 => schedule.certificate.blend_work_units = 31,
            4 => schedule.certificate.blend_work_units = 268_435_457,
            _ => schedule.direct_draw[0].destination_visits = u64::MAX,
        }
        let mut calls = 0;
        let mut destination = [[0.25, 0.5, 0.75, 1.0]];
        let before = destination;
        assert!(
            compose_frame(&schedule, &mut destination, &mut |_, _| {
                calls += 1;
                Ok(plane(0, 0, 1, 1, [1.0; 4], 1.0))
            })
            .is_err()
        );
        assert_eq!(calls, 0);
        assert_eq!(destination, before);
    }
}

#[test]
fn graph_free_4096_owning_averages_execute_all65536_callbacks_without_providers() {
    let items=(0..4096).map(|index|serde_json::json!({"type":"rectangle","id":format!("ordinary-{index}"),"color":"#ffffff","width":1,"height":1,"startMs":0,"durationMs":1000,"blendMode":"screen","motionBlur":{"shutterAngleDeg":360,"sampleCount":16},"stackOrder":index,"zIndex":0,"keyframes":[]})).collect::<Vec<_>>();
    let project:crate::Project=serde_json::from_value(serde_json::json!({"schemaVersion":crate::PROJECT_SCHEMA_VERSION, "audioBuses":crate::default_audio_buses(),"id":"p","revision":0,"name":"Callbacks","createdAtMs":1,"updatedAtMs":1,"settings":{"width":1,"height":1,"fps":10},"fonts":{},"markers":[],"assets":[],"components":[],"tracks":[{"id":"t","name":"Track","trackType":"overlay","items":items}]})).unwrap();
    let evaluated = crate::evaluated_scene::evaluate_project(&project, 1, 1, 10).unwrap();
    assert!(evaluated.scene.mattes.is_none());
    let schedule = crate::evaluated_scene::mattes::frame_schedule(&evaluated.scene, 400).unwrap();
    assert_eq!(schedule.certificate.provider_requests, 0);
    let mut calls = vec![0; 4096];
    let mut destination = vec![[0., 0., 0., 1.]];
    compose_frame(&schedule, &mut destination, &mut |owner, time| {
        calls[owner] += 1;
        assert!((350..450).contains(&time));
        Ok(LeafSamplePlane {
            plane: LinearPlane {
                left: 0,
                top: 0,
                width: 1,
                height: 1,
                pixels: vec![[0.5, 0.5, 0.5, 1.]],
            },
            gain: 1.,
        })
    })
    .unwrap();
    assert!(calls.iter().all(|count| *count == 16));
    assert_eq!(calls.iter().sum::<usize>(), 65536);
    assert_eq!(destination, vec![[1., 1., 1., 1.]]);
}

#[test]
fn independent_zero_alpha_overlay_backdrop_branches_and_bounded_add_oracles() {
    let contract = blend_contract();
    let numeric = &contract["numericOracles"];
    for name in contract["fields"]["blendMode"]["values"]
        .as_array()
        .unwrap()
    {
        let mode = serde_json::from_value(name.clone()).unwrap();
        let source = bits_rgba(&numeric["translucentColored"]["source"]["f32Bits"]);
        let backdrop = bits_rgba(&numeric["translucentColored"]["backdrop"]["f32Bits"]);
        for (case, source, backdrop) in [
            ("sourceZero", [0.; 4], backdrop),
            ("backdropZero", source, [0.; 4]),
            ("bothZero", [0.; 4], [0.; 4]),
        ] {
            let mut destination = [backdrop];
            compose_frame(&blend_fixture(mode), &mut destination, &mut |_, _| {
                Ok(plane(0, 0, 1, 1, source, 1.))
            })
            .unwrap();
            assert_eq!(
                destination[0],
                rgba(&numeric["zeroAlphaPureOnly"][case][name.as_str().unwrap()]["outputF32"])
            );
        }
    }
    let branch = &numeric["overlayBackdropBranches"];
    let straight = branch["sourceStraight"].as_f64().unwrap() as f32;
    let alpha = branch["sourceAlpha"].as_f64().unwrap() as f32;
    for case in branch["cases"].as_array().unwrap() {
        let backdrop = case["backdropStraight"].as_f64().unwrap() as f32;
        let mut destination = [[backdrop, backdrop, backdrop, 1.]];
        compose_frame(
            &blend_fixture(crate::BlendMode::Overlay),
            &mut destination,
            &mut |_, _| {
                Ok(plane(
                    0,
                    0,
                    1,
                    1,
                    [straight * alpha, straight * alpha, straight * alpha, alpha],
                    1.,
                ))
            },
        )
        .unwrap();
        let expected = case["expected"].as_f64().unwrap() as f32;
        assert_eq!(destination[0], [expected, expected, expected, 1.]);
        if backdrop < 0.5 {
            assert_ne!(
                destination[0][0],
                case["wrongSourceBranch"].as_f64().unwrap() as f32
            );
        }
    }
    let add = &numeric["boundedAddSaturation"];
    let mut destination = [rgba(&add["backdrop"])];
    let source = rgba(&add["source"]);
    compose_frame(
        &blend_fixture(crate::BlendMode::Add),
        &mut destination,
        &mut |_, _| Ok(plane(0, 0, 1, 1, source, 1.)),
    )
    .unwrap();
    assert_eq!(destination[0], rgba(&add["expected"]));
    assert_ne!(destination[0], rgba(&add["wrongUnboundedRgb"]));
}
