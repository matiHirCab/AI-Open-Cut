use super::*;
use crate::animation::SampleTime;
use crate::{
    AnimationChannel, AnimationChannelKeyframe, AnimationChannelProperty as P,
    AnimationChannelValue as V, AnimationCurve, AnimationTarget, AnimationTargetKind, Mask,
    MaskChannel, MaskOperation, MaskSource, PathCommand, SimpleAnimationCurve, Transform2D,
    VectorColor, VectorPath, VectorPoint,
};

fn triangle() -> Mask {
    Mask {
        id: "mask".into(),
        source: MaskSource::Path {
            path: VectorPath {
                fill_rule: FillRule::Nonzero,
                commands: vec![
                    PathCommand::MoveTo {
                        to: VectorPoint { x: 0., y: 0. },
                    },
                    PathCommand::LineTo {
                        to: VectorPoint { x: 2., y: 0. },
                    },
                    PathCommand::LineTo {
                        to: VectorPoint { x: 0., y: 2. },
                    },
                    PathCommand::Close {},
                ],
            },
            paint: Paint::Solid {
                color: VectorColor {
                    r: 1.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                },
            },
        },
        channel: MaskChannel::Alpha,
        operation: MaskOperation::Add,
        inverted: false,
        transform: Transform2D::default(),
        feather_px: 0.,
        expansion_px: 0.,
    }
}
fn owner() -> MaskOwnerBasis {
    MaskOwnerBasis {
        size: (8, 8),
        density: 1.,
    }
}
fn channel(property: P, first: V, last: V, end: u64) -> AnimationChannel {
    AnimationChannel {
        property,
        target: Some(AnimationTarget {
            kind: AnimationTargetKind::Mask,
            scope: "root".into(),
            id: "mask".into(),
        }),
        clock: None,
        r#loop: None,
        keyframes: vec![
            AnimationChannelKeyframe {
                time_ms: 0,
                value: first,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            },
            AnimationChannelKeyframe {
                time_ms: end,
                value: last,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            },
        ],
    }
}
fn scalar(property: P, a: f64, b: f64) -> AnimationChannel {
    channel(
        property,
        V::Scalar { value: a },
        V::Scalar { value: b },
        1000,
    )
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9, "{a} != {b}");
}
fn rgba(r: f64, b: f64, a: f64) -> V {
    V::Rgba { r, g: 0., b, a }
}
fn points(size: f64) -> V {
    V::PathPoints {
        points: vec![
            crate::AnimationPoint { x: 0., y: 0. },
            crate::AnimationPoint { x: size, y: 0. },
            crate::AnimationPoint { x: 0., y: size },
        ],
    }
}
fn stops(reverse: bool) -> V {
    let colors = if reverse {
        [[0., 0., 1., 0.75], [1., 0., 0., 0.25]]
    } else {
        [[1., 0., 0., 0.25], [0., 0., 1., 0.75]]
    };
    V::GradientStops {
        stops: vec![
            crate::AnimationGradientStop {
                offset: 0.,
                color: colors[0],
            },
            crate::AnimationGradientStop {
                offset: 1.,
                color: colors[1],
            },
        ],
    }
}
#[test]
fn all_fifteen_sampled_mask_properties_have_independent_fractional_values() {
    let channels = vec![
        channel(P::MaskPathPoints, points(2.), points(4.), 1000),
        channel(
            P::MaskPaintColor,
            rgba(1., 0., 0.25),
            rgba(0., 1., 0.75),
            1000,
        ),
        scalar(P::MaskFeatherPx, 0., 80.),
        scalar(P::MaskExpansionPx, -8., 8.),
        scalar(P::MaskTransformPositionX, 0., 4.),
        scalar(P::MaskTransformPositionY, 0., 8.),
        scalar(P::MaskTransformScaleX, 2., 6.),
        scalar(P::MaskTransformScaleY, 1., 5.),
        scalar(P::MaskTransformAnchorX, 0.1, 0.9),
        scalar(P::MaskTransformAnchorY, 0.2, 0.6),
        scalar(P::MaskTransformRotationDeg, 0., 720.),
        scalar(P::MaskTransformSkewXDeg, 0., 20.),
        scalar(P::MaskTransformSkewYDeg, -20., 20.),
        scalar(P::MaskTransformOpacity, 0.2, 1.),
    ];
    let sampled = sample_masks(&[triangle()], &channels, SampleTime::Fractional(250.)).unwrap();
    let m = &sampled[0];
    close(m.feather_px, 20.);
    close(m.expansion_px, -4.);
    close(m.transform.position.x, 1.);
    close(m.transform.position.y, 2.);
    close(m.transform.scale_x, 3.);
    close(m.transform.scale_y, 2.);
    close(m.transform.anchor.x, 0.3);
    close(m.transform.anchor.y, 0.3);
    close(m.transform.rotation_deg, 180.);
    close(m.transform.skew_x_deg, 5.);
    close(m.transform.skew_y_deg, -10.);
    close(m.transform.opacity, 0.4);
    let MaskSource::Path { path, paint } = &m.source;
    assert_eq!(path.commands.len(), 4);
    assert_eq!(
        path.commands[1],
        PathCommand::LineTo {
            to: VectorPoint { x: 2.5, y: 0. }
        }
    );
    assert_eq!(
        path.commands[2],
        PathCommand::LineTo {
            to: VectorPoint { x: 0., y: 2.5 }
        }
    );
    let Paint::Solid { color } = paint else {
        panic!("solid topology changed")
    };
    close(color.r, 0.7353569830524495);
    close(color.b, 0.7353569830524495);
    close(color.a, 0.375);
    let mut gradient = triangle();
    let MaskSource::Path { paint, .. } = &mut gradient.source;
    *paint=serde_json::from_value(serde_json::json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":2,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0.25}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":0.75}}]})).unwrap();
    let sample = sample_masks(
        &[gradient],
        &[channel(
            P::MaskGradientStops,
            stops(false),
            stops(true),
            1000,
        )],
        SampleTime::Split {
            whole: 250,
            fraction: 0.,
        },
    )
    .unwrap();
    let MaskSource::Path {
        paint: Paint::LinearGradient { start, end, stops },
        ..
    } = &sample[0].source
    else {
        panic!("gradient topology changed")
    };
    assert_eq!(*start, VectorPoint { x: 0., y: 0. });
    assert_eq!(*end, VectorPoint { x: 2., y: 0. });
    assert_eq!(stops.len(), 2);
    close(stops[0].offset, 0.);
    close(stops[1].offset, 1.);
    close(stops[0].color.r, 0.7353569830524495);
    close(stops[0].color.b, 0.7353569830524495);
    close(stops[0].color.a, 0.375);
    close(stops[1].color.r, 0.3491902126282938);
    close(stops[1].color.b, 0.9546871718858662);
    close(stops[1].color.a, 0.625);
}
#[test]
fn split_beyond_two_to_fifty_three_keeps_exact_endpoints_and_fraction() {
    let base = (1u64 << 53) + 1;
    let mut c = scalar(P::MaskExpansionPx, -8., 8.);
    c.keyframes[0].time_ms = base;
    c.keyframes[1].time_ms = base + 4;
    for (time, e) in [
        (SampleTime::Integer(base), -8.),
        (
            SampleTime::Split {
                whole: base,
                fraction: 0.5,
            },
            -6.,
        ),
        (SampleTime::Integer(base + 4), 8.),
    ] {
        close(
            sample_masks(&[triangle()], std::slice::from_ref(&c), time).unwrap()[0].expansion_px,
            e,
        );
    }
}
#[test]
fn retained_clock_ping_pong_hold_and_static_fallback_are_applied_once() {
    let mut c = scalar(P::MaskFeatherPx, 0., 80.);
    c.clock = Some(crate::AnimationClock {
        offset_ms: 250,
        source_duration_ms: 1000,
    });
    c.r#loop = Some(crate::AnimationLoop {
        mode: crate::AnimationLoopMode::PingPong,
        iterations: crate::AnimationLoopIterations::Finite(3),
    });
    for (time, e) in [(0, 20.), (250, 40.), (750, 80.), (1000, 60.)] {
        close(
            sample_masks(
                &[triangle()],
                std::slice::from_ref(&c),
                SampleTime::Integer(time),
            )
            .unwrap()[0]
                .feather_px,
            e,
        );
    }
    c.keyframes[0].curve = AnimationCurve::Simple(SimpleAnimationCurve::Hold);
    close(
        sample_masks(
            &[triangle()],
            std::slice::from_ref(&c),
            SampleTime::Integer(0),
        )
        .unwrap()[0]
            .feather_px,
        0.,
    );
    let mut mask = triangle();
    mask.feather_px = 7.;
    c.keyframes.clear();
    assert_eq!(
        sample_masks(std::slice::from_ref(&mask), &[c], SampleTime::Integer(900)).unwrap(),
        vec![mask]
    );
}
#[test]
fn tiny_scale_exact_keys_preserved_while_interiors_use_documented_floor() {
    let c = scalar(P::MaskTransformScaleX, 1e-12, 1e-12);
    for (time, e) in [(0, 1e-12), (500, 1e-6), (1000, 1e-12)] {
        assert_eq!(
            sample_masks(
                &[triangle()],
                std::slice::from_ref(&c),
                SampleTime::Integer(time),
            )
            .unwrap()[0]
                .transform
                .scale_x,
            e,
        );
    }
}
#[test]
fn malformed_sample_topology_and_paint_kind_return_typed_failures() {
    let c = channel(
        P::MaskPathPoints,
        points(2.),
        V::PathPoints { points: vec![] },
        1000,
    );
    let e = sample_masks(&[triangle()], &[c], SampleTime::Integer(1000)).unwrap_err();
    assert_eq!(e.code, crate::ErrorCode::InvalidArgument);
    let e = sample_masks(
        &[triangle()],
        &[channel(
            P::MaskGradientStops,
            stops(false),
            stops(true),
            1000,
        )],
        SampleTime::Integer(0),
    )
    .unwrap_err();
    assert_eq!(e.code, crate::ErrorCode::InvalidArgument);
}
#[test]
fn shared_work_budget_accepts_exact_limit_and_rejects_excess_and_overflow() {
    let mut b = MaskFrameBudget::default();
    b.charge(MAX_MASK_WORK - 1).unwrap();
    b.charge(1).unwrap();
    assert_eq!(b.work, 268_435_456);
    let e = b.charge(1).unwrap_err();
    assert_eq!(e.code, crate::ErrorCode::InvalidArgument);
    assert!(!e.retryable);
    let mut b = MaskFrameBudget { work: u64::MAX };
    assert!(b.charge(1).is_err());
    assert!(mul(u64::MAX, 2).is_err());
    assert!(add(u64::MAX, 1).is_err());
}
#[test]
fn node_budget_is_shared_and_exact_last_node_is_accepted() {
    let mask = triangle();
    let mut nodes = 65_535;
    let e = continuous_mask_envelope(&mask, &[], owner(), &mut nodes).unwrap();
    assert!(e.work > 0);
    assert_eq!(nodes, 65_536);
    let error = continuous_mask_envelope(&mask, &[], owner(), &mut nodes).unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
    assert!(error.message.contains("maxCandidateAnalysisNodes"));
    assert_eq!(nodes, 65_536);
}
#[test]
fn multiple_open_triangles_actual_closure_work_is_bounded_by_envelope() {
    let mut mask = triangle();
    let MaskSource::Path { path, .. } = &mut mask.source;
    path.commands
        .retain(|c| !matches!(c, PathCommand::Close {}));
    let more = vec![
        PathCommand::MoveTo {
            to: VectorPoint { x: 3., y: 0. },
        },
        PathCommand::LineTo {
            to: VectorPoint { x: 5., y: 0. },
        },
        PathCommand::LineTo {
            to: VectorPoint { x: 3., y: 2. },
        },
    ];
    path.commands.extend(more);
    let envelope = continuous_mask_envelope(&mask, &[], owner(), &mut 0).unwrap();
    let mut segments = 0;
    let actual = certify_sampled_masks(
        &[mask],
        owner(),
        &mut MaskFrameBudget::default(),
        &mut segments,
    )
    .unwrap();
    assert_eq!(segments, 6);
    assert!(segments <= envelope.segments);
    assert!(actual.work_units <= envelope.work + 4 * 64);
    assert!(actual.peak_scratch_bytes <= envelope.scratch);
    assert!(actual.retained_fact_bytes > 0);
}
#[test]
fn opaque_fill_bound_is_additive_for_high_segments_tiny_grid_and_checked() {
    let size = (8, 8);
    let s = 4096;
    let c = 1;
    let expected = 2048 * (4096 + 1 + 64) + 32 * (8 + 8);
    assert_eq!(fill_transient_bytes(s, c, size).unwrap(), expected);
    assert!(expected > 64 * 64 + 16 * 16);
    assert!(fill_transient_bytes(u64::MAX, 1, size).is_err());
    assert!(fill_transient_bytes(0, u64::MAX, size).is_err());
}

thread_local! {
    static INTERVALS: std::cell::RefCell<Option<Vec<(Boundary, Boundary)>>> = const { std::cell::RefCell::new(None) };
}
pub(super) fn record_interval(low: Boundary, high: Boundary) {
    INTERVALS.with(|trace| {
        if let Some(intervals) = trace.borrow_mut().as_mut() {
            intervals.push((low, high));
        }
    });
}
#[test]
fn source_program_memory_charges_actual_spare_outer_and_nested_capacity() {
    let mut masks = Vec::with_capacity(257);
    let mut mask = triangle();
    mask.id.reserve(128);
    let MaskSource::Path { path, .. } = &mut mask.source;
    path.commands.reserve(128);
    masks.push(mask);
    let MaskSource::Path { path, .. } = &masks[0].source;
    let actual = masks.capacity() * std::mem::size_of::<Mask>()
        + masks[0].id.capacity()
        + path.commands.capacity() * std::mem::size_of::<PathCommand>();
    assert_eq!(owning_mask_bytes(&masks).unwrap(), actual as u64);
    assert_eq!(
        owning_mask_bytes(&masks).unwrap() - authored_mask_bytes(&masks).unwrap(),
        ((masks.capacity() - masks.len()) * std::mem::size_of::<Mask>()) as u64
    );
    let mut channels = Vec::with_capacity(193);
    let mut c = scalar(P::MaskFeatherPx, 0., 1.);
    c.keyframes.reserve(128);
    c.target.as_mut().unwrap().id.reserve(64);
    c.target.as_mut().unwrap().scope.reserve(64);
    channels.push(c);
    let c = &channels[0];
    let target = c.target.as_ref().unwrap();
    let expected = channels.capacity() * std::mem::size_of::<AnimationChannel>()
        + c.keyframes.capacity() * std::mem::size_of::<AnimationChannelKeyframe>()
        + target.id.capacity()
        + target.scope.capacity();
    assert_eq!(channel_bytes(&channels).unwrap(), expected as u64);
}
#[test]
fn rasterless_sixty_four_stop_mask_retains_metadata_and_capacity_without_grid() {
    let mut mask = triangle();
    mask.id.reserve(256);
    let MaskSource::Path { path, paint } = &mut mask.source;
    path.commands.truncate(1);
    let mut stops = Vec::with_capacity(129);
    for i in 0..64 {
        stops.push(crate::GradientStop {
            offset: f64::from(i) / 63.,
            color: VectorColor {
                r: 1.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        });
    }
    *paint = Paint::LinearGradient {
        start: VectorPoint { x: 0., y: 0. },
        end: VectorPoint { x: 1., y: 0. },
        stops,
    };
    let stack =
        certify_sampled_masks(&[mask], owner(), &mut MaskFrameBudget::default(), &mut 0).unwrap();
    assert!(stack.masks[0].grid.is_none());
    assert_eq!(stack.peak_scratch_bytes, 0);
    let Paint::LinearGradient { stops, .. } = &stack.masks[0].paint else {
        panic!("paint changed")
    };
    assert_eq!(stops.len(), 64);
    let actual_metadata = std::mem::size_of::<EvaluatedMaskStack>()
        + stack.masks.capacity() * std::mem::size_of::<EvaluatedMask>()
        + stack.masks[0].id.capacity()
        + stops.capacity() * std::mem::size_of::<crate::GradientStop>();
    assert!(
        stack.retained_fact_bytes >= actual_metadata as u64,
        "certified {} versus actual metadata {}",
        stack.retained_fact_bytes,
        actual_metadata
    );
}
#[test]
fn continuous_subdivision_admits_left_before_right_and_preserves_shared_budget() {
    let mut mask = triangle();
    mask.feather_px = 10.;
    let channels = vec![scalar(P::MaskTransformScaleX, 1., 100.)];
    INTERVALS.with(|trace| *trace.borrow_mut() = Some(Vec::new()));
    let mut nodes = 65_533;
    let result = continuous_mask_envelope(&mask, &channels, owner(), &mut nodes);
    let trace = INTERVALS.with(|trace| trace.borrow_mut().take().unwrap());
    assert!(result.is_err());
    assert_eq!(nodes, 65_536);
    assert_eq!(trace.len(), 3);
    let boundary = |whole| Boundary {
        whole,
        fraction: 0.,
    };
    assert_eq!(trace[0], (boundary(0), boundary(1000)));
    assert_eq!(trace[1], (boundary(0), boundary(500)));
    assert_eq!(trace[2], (boundary(500), boundary(1000)));
}
