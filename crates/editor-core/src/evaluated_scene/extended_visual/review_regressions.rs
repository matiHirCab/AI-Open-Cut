use super::*;
use serde_json::json;

fn layer(channels: Vec<AnimationChannel>) -> EvaluatedVisualLayer {
    EvaluatedVisualLayer {
        sampled_input: None,
        extended: Some(ExtendedVisual {
            crop: None,
            effects: vec![],
            channels,
        }),
        instance: None,
        item_id: "review".into(),
        order: EvaluatedLayerOrder {
            track_index: 0,
            item_index: 0,
        },
        span: EvaluatedTimeSpan {
            start_ms: 0,
            end_ms: u64::MAX,
        },
        transform: EvaluatedTransform {
            position_x: 0.0,
            position_y: 0.0,
            scale: 1.0,
            opacity: 1.0,
        },
        transform2d: None,
        affine: None,
        sampling_tiles: None,
        ancestors: None,
        ancestor_stages: vec![],
        source_size: Some((24, 8)),
        keyframes: vec![],
        transitions: vec![],
        source: EvaluatedVisualSource::Rectangle {
            color: "#ff0000".into(),
            width: 24,
            height: 8,
        },
    }
}

fn ramp(origin: u64, property: &str) -> AnimationChannel {
    serde_json::from_value(json!({"property":property,"keyframes":[
        {"timeMs":origin,"value":{"type":"scalar","value":0},"curve":"linear"},
        {"timeMs":origin+4,"value":{"type":"scalar","value":100},"curve":"hold"}
    ]}))
    .unwrap()
}

#[test]
fn review_integer_rotation_uses_actual_extended_consumers() {
    for origin in [9_007_199_254_740_993, u64::MAX - 8] {
        let original = layer(vec![ramp(origin, "transform.rotation_deg")]);
        for (offset, expected) in [(0, 0.0), (1, 25.0), (2, 50.0), (3, 75.0), (4, 100.0)] {
            let (mut sampled, _, _) = sample(&original, origin + offset).unwrap();
            sample_transform(&mut sampled, origin + offset, (24, 8), (64, 64)).unwrap();
            assert_eq!(
                sampled.transform2d.unwrap().rotation_deg,
                expected,
                "origin={origin} offset={offset}"
            );
        }
    }
}

#[test]
fn review_root_relative_clock_and_legacy_scalars_keep_integer_progress() {
    let origin = 9_007_199_254_740_993;
    let mut relative = layer(vec![ramp(0, "transform.rotation_deg")]);
    relative.span.start_ms = origin;
    sample_transform(&mut relative, origin + 1, (24, 8), (64, 64)).unwrap();
    assert_eq!(relative.transform2d.unwrap().rotation_deg, 25.0);
    let mut legacy = layer(vec![]);
    legacy.keyframes = vec![
        EvaluatedKeyframe {
            property: EvaluatedProperty::PositionX,
            time_ms: origin,
            value: EvaluatedKeyframeValue::Scalar { value: 0.0 },
            easing: EvaluatedEasing::Linear,
            r#loop: None,
        },
        EvaluatedKeyframe {
            property: EvaluatedProperty::PositionX,
            time_ms: origin + 4,
            value: EvaluatedKeyframeValue::Scalar { value: 100.0 },
            easing: EvaluatedEasing::Hold,
            r#loop: None,
        },
    ];
    sample_transform(&mut legacy, origin + 1, (24, 8), (64, 64)).unwrap();
    assert_eq!(legacy.transform2d.unwrap().position.x, 25.0);
}

#[test]
fn review_integer_holds_and_reflected_loops_preserve_seams() {
    for origin in [9_007_199_254_740_993, u64::MAX - 20] {
        let mut held = ramp(origin, "transform.rotation_deg");
        held.keyframes[0].curve = crate::AnimationCurve::Simple(crate::SimpleAnimationCurve::Hold);
        let original = layer(vec![held]);
        for (offset, expected) in [(0, 0.0), (1, 0.0), (3, 0.0), (4, 100.0), (5, 100.0)] {
            let mut sampled = original.clone();
            sample_transform(&mut sampled, origin + offset, (24, 8), (64, 64)).unwrap();
            assert_eq!(sampled.transform2d.unwrap().rotation_deg, expected);
        }
        let mut ping = ramp(origin, "transform.rotation_deg");
        ping.r#loop =
            Some(serde_json::from_value(json!({"mode":"ping_pong","iterations":2})).unwrap());
        let original = layer(vec![ping]);
        for (offset, expected) in [
            (0, 0.0),
            (1, 25.0),
            (4, 100.0),
            (5, 75.0),
            (8, 0.0),
            (9, 25.0),
            (16, 0.0),
            (17, 0.0),
        ] {
            let mut sampled = original.clone();
            sample_transform(&mut sampled, origin + offset, (24, 8), (64, 64)).unwrap();
            assert_eq!(sampled.transform2d.unwrap().rotation_deg, expected);
        }
        let repeat:AnimationChannel=serde_json::from_value(json!({"property":"transform.rotation_deg","loop":{"mode":"repeat","iterations":2},"keyframes":[
            {"timeMs":origin,"value":{"type":"scalar","value":0},"curve":"linear"},
            {"timeMs":origin+2,"value":{"type":"scalar","value":100},"curve":"linear"},
            {"timeMs":origin+4,"value":{"type":"scalar","value":0},"curve":"hold"}
        ]})).unwrap();
        let original = layer(vec![repeat]);
        for (offset, expected) in [
            (1, 50.0),
            (2, 100.0),
            (3, 50.0),
            (4, 0.0),
            (5, 50.0),
            (8, 0.0),
            (9, 0.0),
        ] {
            let mut sampled = original.clone();
            sample_transform(&mut sampled, origin + offset, (24, 8), (64, 64)).unwrap();
            assert_eq!(sampled.transform2d.unwrap().rotation_deg, expected);
        }
    }
}

#[test]
fn review_fractional_rotation_and_tiny_crop_keep_canonical_samples() {
    let mut original = layer(vec![ramp(0, "transform.rotation_deg")]);
    original.instance = Some(EvaluatedInstance {
        rate: 0.5,
        offset: 0.0,
        start_ms: 0.0,
        end_ms: 20.0,
        canvas: (64, 64),
    });
    sample_transform(&mut original, 1, (24, 8), (64, 64)).unwrap();
    assert_eq!(original.transform2d.unwrap().rotation_deg, 12.5);
    for curve in ["linear", "hold"] {
        let crop = serde_json::from_value(json!({"property":"media.crop_width","keyframes":[
            {"timeMs":0,"value":{"type":"scalar","value":0.0000001},"curve":curve},
            {"timeMs":4,"value":{"type":"scalar","value":0.0000002},"curve":"hold"}
        ]}))
        .unwrap();
        let original = layer(vec![crop]);
        assert_eq!(sample(&original, 0).unwrap().1.unwrap().width, 0.0000001);
        assert_eq!(sample(&original, 4).unwrap().1.unwrap().width, 0.0000002);
        assert_eq!(
            sample(&original, 1).unwrap().1.unwrap().width,
            if curve == "hold" { 0.0000001 } else { 0.000001 }
        );
        let mut inherited = original.clone();
        inherited.instance = Some(EvaluatedInstance {
            rate: 0.5,
            offset: 0.0,
            start_ms: 0.0,
            end_ms: 20.0,
            canvas: (64, 64),
        });
        assert_eq!(
            sample(&inherited, 1).unwrap().1.unwrap().width,
            if curve == "hold" { 0.0000001 } else { 0.000001 }
        );
    }
}

#[test]
fn review_compound_tint_uses_integer_and_fractional_consumers() {
    for (origin, clock, at) in [
        (9_007_199_254_740_993u64, None, 9_007_199_254_740_994),
        (
            0,
            Some(EvaluatedInstance {
                rate: 0.5,
                offset: 0.0,
                start_ms: 0.0,
                end_ms: 20.0,
                canvas: (64, 64),
            }),
            2,
        ),
    ] {
        let tint=serde_json::from_value(json!({"property":"effect.tint_color","target":{"kind":"effect","scope":"root","id":"tint"},"keyframes":[
            {"timeMs":origin,"value":{"type":"rgba","r":0,"g":1,"b":0,"a":0},"curve":"linear"},
            {"timeMs":origin+4,"value":{"type":"rgba","r":1,"g":0,"b":0,"a":1},"curve":"hold"}
        ]})).unwrap();
        let mut original = layer(vec![tint]);
        original.instance = clock;
        original.extended.as_mut().unwrap().effects = vec![VisualEffect::ColorTint {
            id: "tint".into(),
            color: crate::VectorColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        }];
        let (_, _, effects) = sample(&original, at).unwrap();
        let VisualEffect::ColorTint { color, .. } = &effects[0] else {
            panic!("tint missing")
        };
        assert!((color.r - 1.0).abs() < 1e-12);
        assert_eq!((color.g, color.b, color.a), (0.0, 0.0, 0.25));
    }
}

#[test]
fn review_legacy_shape_affine_preserves_origin_for_identity_extensions() {
    for (x, y) in [
        (10.0, 20.0),
        (-10.0, -20.0),
        (600_000.0, 600_000.0),
        (-600_000.0, -600_000.0),
    ] {
        for rotation in [false, true] {
            let geometry = serde_json::from_value(
                json!({"type":"path","path":{"fillRule":"nonzero","commands":[
                    {"type":"moveTo","to":{"x":x,"y":y}},
                    {"type":"lineTo","to":{"x":x+20.0,"y":y}},
                    {"type":"lineTo","to":{"x":x+20.0,"y":y+20.0}},
                    {"type":"lineTo","to":{"x":x,"y":y+20.0}},
                    {"type":"close"}
                ]}}),
            )
            .unwrap();
            let fill =
                serde_json::from_value(json!({"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}}))
                    .unwrap();
            let shape = shapes::EvaluatedShape::new(geometry, Some(fill), None, 1.0).unwrap();
            let mut original = layer(vec![]);
            original.transform = EvaluatedTransform {
                position_x: 3.0,
                position_y: -4.0,
                scale: 2.0,
                opacity: 0.6,
            };
            original.source_size = Some(shape.size);
            original.source = EvaluatedVisualSource::Shape(Box::new(shape));
            if rotation {
                let mut zero = ramp(0, "transform.rotation_deg");
                zero.keyframes[1].value = V::Scalar { value: 0.0 };
                original.extended.as_mut().unwrap().channels = vec![zero];
            } else {
                original.extended.as_mut().unwrap().effects = vec![VisualEffect::Vignette {
                    id: "identity".into(),
                    amount: 0.0,
                }];
            }
            let source = original.source_size.unwrap();
            let expected = evaluate_layer_affine(&original, source, (64, 64)).unwrap();
            // The source raster includes one pixel of transparent padding.
            assert_eq!(
                expected.matrix,
                [
                    2.0,
                    0.0,
                    0.0,
                    2.0,
                    3.0 + 2.0 * (x - 1.0),
                    -4.0 + 2.0 * (y - 1.0)
                ]
            );
            let mut inherited = original.clone();
            inherited.ancestor_stages = vec![EvaluatedAncestorStage {
                scope: 0,
                item_id: "parent".into(),
                matrix: [1.5, 0.0, 0.0, 1.5, 7.0, 9.0],
                inverse: [2.0 / 3.0, 0.0, 0.0, 2.0 / 3.0, -14.0 / 3.0, -6.0],
                opacity: 0.5,
                animation: None,
            }];
            let nested = sample_transform(&mut inherited, 0, source, (64, 64)).unwrap();
            assert_eq!(
                nested.matrix,
                [
                    3.0,
                    0.0,
                    0.0,
                    3.0,
                    7.0 + 1.5 * (3.0 + 2.0 * (x - 1.0)),
                    9.0 + 1.5 * (-4.0 + 2.0 * (y - 1.0))
                ]
            );
            assert_eq!(nested.opacity, 0.3);
            let actual = sample_transform(&mut original, 0, source, (64, 64)).unwrap();
            assert_eq!(actual.matrix, expected.matrix);
            assert_eq!(actual.opacity, 0.6);
        }
    }
}

#[test]
fn review_compound_paths_and_gradients_preserve_large_and_fractional_clocks() {
    for (origin, clock, at) in [
        (9_007_199_254_740_993u64, None, 9_007_199_254_740_994),
        (
            0,
            Some(EvaluatedInstance {
                rate: 0.5,
                offset: 0.0,
                start_ms: 0.0,
                end_ms: 20.0,
                canvas: (64, 64),
            }),
            2,
        ),
    ] {
        let geometry:crate::ShapeGeometry=serde_json::from_value(json!({"type":"path","path":{"fillRule":"nonzero","commands":[
            {"type":"moveTo","to":{"x":10,"y":20}},{"type":"lineTo","to":{"x":30,"y":20}},
            {"type":"lineTo","to":{"x":30,"y":40}},{"type":"lineTo","to":{"x":10,"y":40}},{"type":"close"}
        ]}})).unwrap();
        let fill=serde_json::from_value(json!({"type":"linearGradient","start":{"x":10,"y":20},"end":{"x":30,"y":20},"stops":[
            {"offset":0,"color":{"r":1,"g":1,"b":1,"a":1}},{"offset":0.25,"color":{"r":1,"g":1,"b":1,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}
        ]})).unwrap();
        let points = |delta: i32| json!({"type":"path_points","points":[{"x":10+delta,"y":20},{"x":30+delta,"y":20},{"x":30+delta,"y":40},{"x":10+delta,"y":40}]});
        let stops = |middle: f64| json!({"type":"gradient_stops","stops":[{"offset":0,"color":[1,1,1,1]},{"offset":middle,"color":[1,1,1,1]},{"offset":1,"color":[1,1,1,1]}]});
        let channel = |property: &str, kind: &str, a: serde_json::Value, b: serde_json::Value| {
            serde_json::from_value(json!({"property":property,"target":{"kind":kind,"scope":"root","id":"review"},"keyframes":[{"timeMs":origin,"value":a,"curve":"linear"},{"timeMs":origin+4,"value":b,"curve":"hold"}]})).unwrap()
        };
        let mut original = layer(vec![
            channel(
                "graphic.path_points",
                "graphic_geometry",
                points(0),
                points(4),
            ),
            channel(
                "graphic.gradient_stops",
                "graphic_fill",
                stops(0.25),
                stops(0.75),
            ),
        ]);
        original.instance = clock;
        let shape = shapes::EvaluatedShape::new(geometry, Some(fill), None, 1.0).unwrap();
        original.source_size = Some(shape.size);
        original.source = EvaluatedVisualSource::Shape(Box::new(shape));
        let (sampled, _, _) = sample(&original, at).unwrap();
        let EvaluatedVisualSource::Shape(shape) = sampled.source else {
            panic!("shape missing")
        };
        let crate::ShapeGeometry::Path { path } = shape.geometry else {
            panic!("path missing")
        };
        let crate::PathCommand::MoveTo { to } = path.commands[0] else {
            panic!("start missing")
        };
        assert_eq!((to.x, to.y), (11.0, 20.0));
        let Some(crate::Paint::LinearGradient { stops, .. }) = shape.fill else {
            panic!("gradient missing")
        };
        assert_eq!(
            (stops[0].offset, stops[1].offset, stops[2].offset),
            (0.0, 0.375, 1.0)
        );
        for channel in &mut original.extended.as_mut().unwrap().channels {
            channel.r#loop =
                Some(serde_json::from_value(json!({"mode":"ping_pong","iterations":1})).unwrap());
        }
        let reflected_at = at + if clock.is_some() { 8 } else { 4 };
        let (reflected, _, _) = sample(&original, reflected_at).unwrap();
        let EvaluatedVisualSource::Shape(shape) = reflected.source else {
            panic!("shape missing")
        };
        let crate::ShapeGeometry::Path { path } = shape.geometry else {
            panic!("path missing")
        };
        let crate::PathCommand::MoveTo { to } = path.commands[0] else {
            panic!("start missing")
        };
        assert_eq!((to.x, to.y), (13.0, 20.0));
        let Some(crate::Paint::LinearGradient { stops, .. }) = shape.fill else {
            panic!("gradient missing")
        };
        assert_eq!(stops[1].offset, 0.625);
    }
}

#[test]
fn review_inherited_scale_envelope_does_not_require_rotation() {
    let mut original = layer(vec![]);
    original.extended.as_mut().unwrap().effects = vec![VisualEffect::Vignette {
        id: "identity".into(),
        amount: 0.0,
    }];
    let channels=["transform.scale_x","transform.scale_y"].into_iter().map(|property| serde_json::from_value(json!({"property":property,"keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":1},"curve":{"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0}},
        {"timeMs":500,"value":{"type":"scalar","value":3},"curve":"hold"}
    ]})).unwrap()).collect();
    original.ancestor_stages = vec![EvaluatedAncestorStage {
        scope: 0,
        item_id: "parent".into(),
        matrix: IDENTITY_MATRIX,
        inverse: IDENTITY_MATRIX,
        opacity: 1.0,
        animation: Some(EvaluatedAncestorAnimation {
            base_transform: crate::Transform2D::default(),
            source_canvas: (64, 64),
            channels,
            clock: EvaluatedInstance {
                rate: 0.5,
                offset: 0.25,
                start_ms: 0.0,
                end_ms: 1000.0,
                canvas: (64, 64),
            },
            start_ms: 0,
            transform: original.transform,
            keyframes: std::sync::Arc::from([]),
        }),
    }];
    let bound = super::super::extended_certification::transform_magnification(&original, (64, 64))
        .unwrap()
        .unwrap();
    // At normalized progress .3 this underdamped spring exceeds scale 4.
    assert!(bound > 4.0);
    let mut sampled = original.clone();
    let actual = sample_transform(&mut sampled, 300, (24, 8), (64, 64)).unwrap();
    assert!(actual.matrix[0] > 4.0);
    assert!(bound >= actual.matrix[0]);
}
