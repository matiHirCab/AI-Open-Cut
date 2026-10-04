use super::*;
use crate as core_facade;
#[path = "../../tests/support/temporal_fixture.rs"]
mod fixture;
use fixture::recipe;
fn project(b: bool) -> Project {
    let root = tempfile::tempdir().unwrap();
    fixture::seed(root.path(), b).project()
}
#[test]
fn temporal_independent_curves_and_loop_boundaries() {
    assert_eq!(recipe::family_a()[0]["items"].as_array().unwrap().len(), 6);
    for lane in 0..6 {
        let c: crate::AnimationChannel = serde_json::from_value(recipe::channel(lane)).unwrap();
        for time in recipe::TIMES {
            let actual = crate::animation::sample_scalar_channel_at(&c, time).unwrap();
            let expected = recipe::expected_a(lane, time);
            assert!(
                (actual - expected).abs() < 1e-8,
                "lane{lane} at{time}: {actual} vs{expected}"
            );
        }
    }
    assert!(recipe::expected_a(0, 300.0) > 20.0);
    let evaluated = evaluate_project(&project(false), 64, 64, 10).unwrap();
    assert_eq!(evaluated.scene.visual_layers.len(), 6);
    for layer in evaluated.scene.visual_layers {
        assert!(layer.visible_at(1299));
        assert!(!layer.visible_at(1300));
    }
}
#[test]
fn temporal_authored_stagger_repeater_retained_fractional_clocks() {
    let evaluated = evaluate_project(&project(true), 64, 64, 10).unwrap();
    assert_eq!(evaluated.scene.visual_layers.len(), 3);
    for (j, layer) in evaluated.scene.visual_layers.iter().enumerate() {
        let clock = layer.instance.unwrap();
        assert_eq!(clock.rate, 0.75);
        let start = 140.0 / 0.75 + 100.0 * j as f64;
        assert!((clock.start_ms - start).abs() < 1e-9);
        assert_eq!(clock.end_ms, 1300.0);
        assert!(!layer.visible_at(start.floor() as u64));
        assert!(layer.visible_at(start.ceil() as u64));
        for delta in [-0.5, 0.0, 0.5] {
            let root = start + delta;
            assert_eq!(root >= clock.start_ms && root < clock.end_ms, delta >= 0.0);
            let local = clock.rate * root + clock.offset;
            if delta >= 0.0 {
                let c: crate::AnimationChannel = serde_json::from_value(
                    recipe::family_b("leaf", "outer").0[0]["items"][1]["animationChannels"][0]
                        .clone(),
                )
                .unwrap();
                let actual =
                    crate::animation::sample_scalar_channel_at(&c, local.max(0.0)).unwrap();
                assert!((actual - recipe::triangle(250.0 + 0.75 * delta)).abs() < 1e-8);
            }
        }
        assert_eq!(clock.offset, -140.0 - 75.0 * j as f64);
        let c: crate::AnimationChannel = serde_json::from_value(
            recipe::family_b("leaf", "outer").0[0]["items"][1]["animationChannels"][0].clone(),
        )
        .unwrap();
        for t in recipe::TIMES {
            let local = clock.rate * t + clock.offset;
            let actual = crate::animation::sample_scalar_channel_at(&c, local.max(0.0)).unwrap();
            if local >= 0.0 {
                assert!(
                    (actual - recipe::expected_b(j, t)).abs() < 1e-8,
                    "copy{j} at{t}"
                );
            }
        }
        assert!(layer.visible_at(713));
        assert!(!layer.visible_at(1300));
        assert!(!layer.visible_at(0));
        assert_eq!(
            clock.rate * 713.0 + clock.offset + 250.0,
            [644.75, 569.75, 494.75][j]
        );
    }
}

#[test]
fn temporal_production_edits_undo_redo_reopen_have_independent_numeric_states() {
    for b in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let f = fixture::seed(root.path(), b);
        assert_eq!(f.family_b, b);
        let check = |p: &Project, edited: bool| {
            let scene = evaluate_project(p, 64, 64, 10).unwrap().scene;
            assert_eq!(scene.visual_layers.len(), if b { 3 } else { 6 });
            let ids: std::collections::HashSet<_> = scene
                .visual_layers
                .iter()
                .map(|l| l.item_id.clone())
                .collect();
            assert_eq!(ids.len(), scene.visual_layers.len());
            if b {
                assert_eq!(
                    scene
                        .visual_layers
                        .iter()
                        .filter(|l| l.item_id.contains("repeater:"))
                        .count(),
                    2
                );
            } else {
                for id in &f.item_ids {
                    assert!(ids.contains(id));
                }
            }
            for (j, layer) in scene.visual_layers.iter().enumerate() {
                let mut layer = layer.clone();
                let actual = extended_visual::sample_transform(&mut layer, 713, (5, 5), (64, 64))
                    .unwrap()
                    .matrix[4];
                let expected = if b {
                    recipe::triangle(
                        0.75 * 713.0 + 110.0 - if edited { 90.0 } else { 75.0 } * j as f64,
                    )
                } else if edited && j == 0 {
                    let p = 0.713_f64;
                    let w = 96.0_f64.sqrt();
                    8.0 + 12.0
                        * (1.0 - (-2.0 * p).exp() * ((w * p).cos() + 2.0 / w * (w * p).sin()))
                } else {
                    recipe::expected_a(j, 713.0)
                };
                assert!(
                    (actual - expected).abs() < 1e-8,
                    "family{b} edited{edited} lane{j}: {actual} vs{expected}"
                );
            }
        };
        check(&f.project(), false);
        let change = if b {
            let p = f.project();
            let TimelineItem::Repeater(item) = p.find_item(&f.item_ids[1]).unwrap() else {
                panic!("fixture controller")
            };
            let mut descriptor = item.repeater.clone();
            descriptor.time_offset_ms = 120;
            serde_json::json!({"operation":"update_item","itemId":f.item_ids[1],"repeater":descriptor})
        } else {
            let mut c = recipe::channel(0);
            c["keyframes"][0]["curve"]["damping"] = serde_json::json!(4);
            serde_json::json!({"operation":"set_animation_channels","itemId":f.item_ids[0],"animationChannels":[c]})
        };
        f.edit(change);
        check(&f.project(), true);
        f.core.undo(&f.id, f.project().revision).unwrap();
        check(&f.project(), false);
        f.core.redo(&f.id, f.project().revision).unwrap();
        check(&f.project(), true);
        let reopened = crate::EditorCore::new(f.core.paths().clone());
        check(&reopened.get_project(&f.id).unwrap(), true);
    }
}

#[test]
fn temporal_local_envelopes_include_independent_spring_extrema_and_retained_clocks() {
    let mut layer = evaluate_project(&project(false), 64, 64, 10)
        .unwrap()
        .scene
        .visual_layers
        .remove(0);
    layer.ancestors = Some(EvaluatedAncestors {
        matrix: IDENTITY_MATRIX,
        inverse: IDENTITY_MATRIX,
        opacity: 1.0,
        clip: layer.span,
    });
    let spring = layer.keyframes[0].easing;
    let peak = 1.0 + (-std::f64::consts::PI / 99.0_f64.sqrt()).exp();
    for clock in [
        None,
        Some(crate::AnimationClock {
            offset_ms: 250,
            source_duration_ms: 2000,
        }),
    ] {
        for property in [
            EvaluatedProperty::Position,
            EvaluatedProperty::PositionX,
            EvaluatedProperty::PositionY,
            EvaluatedProperty::Scale,
            EvaluatedProperty::ScaleX,
            EvaluatedProperty::ScaleY,
        ] {
            let scale = matches!(
                property,
                EvaluatedProperty::Scale | EvaluatedProperty::ScaleX | EvaluatedProperty::ScaleY
            );
            let (first, last) = if scale { (1.0, 3.0) } else { (8.0, 20.0) };
            let value = |v| {
                if property == EvaluatedProperty::Position {
                    EvaluatedKeyframeValue::Position { x: v, y: v }
                } else {
                    EvaluatedKeyframeValue::Scalar { value: v }
                }
            };
            layer.keyframes = vec![
                EvaluatedKeyframe {
                    property,
                    time_ms: 0,
                    value: value(first),
                    easing: spring,
                    r#loop: None,
                    clock,
                },
                EvaluatedKeyframe {
                    property,
                    time_ms: 1000,
                    value: value(last),
                    easing: EvaluatedEasing::Hold,
                    r#loop: None,
                    clock,
                },
            ];
            let measured = evaluate_layer_affine(&layer, (5, 5), (64, 64)).unwrap();
            let maximum = first + (last - first) * peak;
            let right = if scale {
                layer.transform.position_x + 5.0 * maximum
            } else {
                maximum + 5.0
            };
            let bottom = if scale {
                layer.transform.position_y + 5.0 * maximum
            } else {
                maximum + 5.0
            };
            if property != EvaluatedProperty::PositionY && property != EvaluatedProperty::ScaleY {
                assert!(
                    measured.left + f64::from(measured.width) >= right,
                    "{property:?} clock{clock:?}"
                );
            }
            if property != EvaluatedProperty::PositionX && property != EvaluatedProperty::ScaleX {
                assert!(
                    measured.top + f64::from(measured.height) >= bottom,
                    "{property:?} clock{clock:?}"
                );
            }
            // Full-source envelopes cover retained time windows without resampling.
            for easing in [
                EvaluatedEasing::Hold,
                EvaluatedEasing::Linear,
                EvaluatedEasing::EaseIn,
                EvaluatedEasing::EaseOut,
                EvaluatedEasing::EaseInOut,
            ] {
                layer.keyframes[0].easing = easing;
                let monotonic = evaluate_layer_affine(&layer, (5, 5), (64, 64)).unwrap();
                assert!(monotonic.width <= measured.width && monotonic.height <= measured.height);
            }
        }
    }
    // Descending spring travel also extends below its authored endpoints.
    layer.keyframes = vec![
        EvaluatedKeyframe {
            property: EvaluatedProperty::PositionX,
            time_ms: 0,
            value: EvaluatedKeyframeValue::Scalar { value: 40.0 },
            easing: spring,
            r#loop: None,
            clock: None,
        },
        EvaluatedKeyframe {
            property: EvaluatedProperty::PositionX,
            time_ms: 1000,
            value: EvaluatedKeyframeValue::Scalar { value: 28.0 },
            easing: EvaluatedEasing::Hold,
            r#loop: None,
            clock: None,
        },
    ];
    let measured = evaluate_layer_affine(&layer, (5, 5), (64, 64)).unwrap();
    assert!(measured.left <= 40.0 - 12.0 * peak);
}

#[test]
fn temporal_spring_scale_envelope_retains_existing_resource_rejection() {
    let mut layer = evaluate_project(&project(false), 64, 64, 10)
        .unwrap()
        .scene
        .visual_layers
        .remove(0);
    let spring = layer.keyframes[0].easing;
    for clock in [
        None,
        Some(crate::AnimationClock {
            offset_ms: 250,
            source_duration_ms: 2000,
        }),
    ] {
        layer.keyframes = vec![
            EvaluatedKeyframe {
                property: EvaluatedProperty::ScaleX,
                time_ms: 0,
                value: EvaluatedKeyframeValue::Scalar { value: 1.0 },
                easing: spring,
                r#loop: None,
                clock,
            },
            EvaluatedKeyframe {
                property: EvaluatedProperty::ScaleX,
                time_ms: 1000,
                value: EvaluatedKeyframeValue::Scalar { value: 80.0 },
                easing: EvaluatedEasing::Hold,
                r#loop: None,
                clock,
            },
        ];
        // Peak raw scale≈137.61 is sampled at the existing clamp100: width20000
        // fails the existing16384 bound before the64px canvas clips it away.
        assert_eq!(
            evaluate_layer_affine(&layer, (200, 5), (64, 64))
                .unwrap_err()
                .code,
            crate::ErrorCode::InvalidArgument
        );
        layer.keyframes[0].easing = EvaluatedEasing::Linear;
        assert!(evaluate_layer_affine(&layer, (200, 5), (64, 64)).is_ok());
        // A valid negative initial velocity undershoots below zero; sampled
        // scale retains its positive floor instead of rejecting raw extrema.
        layer.keyframes[0].easing = EvaluatedEasing::Spring {
            mass: 1.0,
            stiffness: 100.0,
            damping: 2.0,
            initial_velocity: -100.0,
        };
        layer.keyframes[1].value = EvaluatedKeyframeValue::Scalar { value: 3.0 };
        assert!(evaluate_layer_affine(&layer, (5, 5), (64, 64)).is_ok());
    }
}

type TemporalDecoder<'a> =
    dyn Fn(&std::path::Path, u64, (u32, u32)) -> Result<Vec<u8>, CoreError> + 'a;
fn temporal_prepare_samples(
    mut scene: EvaluatedScene,
    intent: crate::render_plan::RenderIntent,
    decode: &TemporalDecoder<'_>,
) -> (EvaluatedScene, Vec<Vec<u8>>) {
    use crate::render_artifact::{FileSystemArtifactIo, PreparedRenderResources};
    use crate::render_plan::MediaInputRequest;
    let (start, end, frame) = match intent {
        crate::render_plan::RenderIntent::Frame { at_ms } => (at_ms, at_ms, true),
        crate::render_plan::RenderIntent::Range {
            start_ms, end_ms, ..
        } => (start_ms, end_ms, false),
        crate::render_plan::RenderIntent::Export => (0, scene.duration_ms, false),
    };
    extended_visual::finalize_intrinsic_sources(&mut scene, start);
    extended_visual::preflight_samples(&scene, start, end, frame).unwrap();
    let workspace = tempfile::tempdir().unwrap();
    let mut resources = PreparedRenderResources {
        media_inputs: vec![],
        media_paths: vec![],
        text_layers: Default::default(),
    };
    for layer in &mut scene.visual_layers {
        if matches!(
            layer.source,
            EvaluatedVisualSource::Media { .. } | EvaluatedVisualSource::Text(_)
        ) {
            layer.source_size = Some((5, 5));
            let path = workspace
                .path()
                .join(format!("input-{}.pam", layer.item_id));
            let mut bytes =
                b"P7\nWIDTH 5\nHEIGHT 5\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
                    .to_vec();
            bytes.extend([255, 0, 0, 255].repeat(25));
            std::fs::write(&path, bytes).unwrap();
            resources.media_inputs.push(MediaInputRequest {
                item_id: layer.item_id.clone(),
                asset_id: "synthetic".into(),
                project_relative_path: path.clone(),
                media_type: crate::MediaType::Video,
                source_in_ms: 100,
                duration_ms: 1300,
                input_index: 2,
            });
            resources.media_paths.push(path);
        }
    }
    let captured = std::cell::RefCell::new(vec![]);
    crate::render_artifact::extended_visual::prepare(
        &FileSystemArtifactIo,
        &mut scene,
        workspace.path(),
        &mut resources,
        intent,
        &crate::render_artifact::extended_visual::VisualPreparation {
            decode,
            encode: &|_, fps, count, produce| {
                assert_eq!(fps, 10);
                for n in 0..count {
                    captured.borrow_mut().push(produce(n)?);
                }
                Ok(())
            },
            caption: &|_, _, _| panic!("this fixture has no caption"),
        },
    )
    .unwrap();
    if frame {
        for layer in &scene.visual_layers {
            if layer.sampled_input.is_some() {
                let index = resources
                    .media_inputs
                    .iter()
                    .position(|i| i.item_id == layer.sampled_input.as_ref().unwrap().0)
                    .unwrap();
                captured
                    .borrow_mut()
                    .push(std::fs::read(&resources.media_paths[index]).unwrap());
            }
        }
    }
    (scene, captured.into_inner())
}
fn temporal_pam_alpha(bytes: &[u8]) -> (f64, f64, f64) {
    let marker = b"ENDHDR\n";
    let offset = bytes
        .windows(marker.len())
        .position(|b| b == marker)
        .unwrap()
        + marker.len();
    let pixels = &bytes[offset..];
    assert_eq!(pixels.len(), 64 * 64 * 4);
    let (mut mass, mut x, mut y) = (0.0, 0.0, 0.0);
    for (i, p) in pixels.as_chunks::<4>().0.iter().enumerate() {
        let a = f64::from(p[3]);
        mass += a;
        x += (i % 64) as f64 * a;
        y += (i / 64) as f64 * a;
    }
    (mass, x / mass, y / mass)
}
#[test]
fn temporal_requested_origin_sampling_preserves_clocks_opacity_transitions_and_activity() {
    let mut scene = evaluate_project(&project(false), 64, 64, 10).unwrap().scene;
    scene.visual_layers.truncate(1);
    scene.visual_layers[0].source_size = Some((5, 5));
    for key in &mut scene.visual_layers[0].keyframes {
        key.clock = Some(crate::AnimationClock {
            offset_ms: 250,
            source_duration_ms: 2000,
        });
    }
    for (time, value) in [(0, 0.0), (1000, 1.0)] {
        scene.visual_layers[0].keyframes.push(EvaluatedKeyframe {
            property: EvaluatedProperty::Opacity,
            time_ms: time,
            value: EvaluatedKeyframeValue::Scalar { value },
            easing: EvaluatedEasing::Linear,
            r#loop: None,
            clock: None,
        });
    }
    scene.visual_layers[0].transitions = vec![EvaluatedTransition {
        role: EvaluatedTransitionRole::In,
        kind: EvaluatedTransitionKind::Fade,
        span: EvaluatedTimeSpan {
            start_ms: 0,
            end_ms: 1000,
        },
    }];
    let (_, images) = temporal_prepare_samples(
        scene.clone(),
        crate::render_plan::RenderIntent::Range {
            start_ms: 713,
            end_ms: 913,
            include_audio: true,
        },
        &|_, _, _| panic!("rectangle cannot decode media"),
    );
    assert_eq!(images.len(), 2);
    for (n, image) in images.iter().enumerate() {
        let time = 713.0 + 100.0 * n as f64;
        let (mass, x, y) = temporal_pam_alpha(image);
        assert!((x - (recipe::expected_a(0, time + 250.0) + 2.0)).abs() < 0.04);
        assert!((y - 7.0).abs() < 0.04);
        assert!((mass / (25.0 * 255.0) - (time / 1000.0).powi(2)).abs() < 0.003);
    }
    let (_, frame) = temporal_prepare_samples(
        scene.clone(),
        crate::render_plan::RenderIntent::Frame { at_ms: 713 },
        &|_, _, _| panic!(),
    );
    assert_eq!(frame[0], images[0]);
    scene.visual_layers[0].span.end_ms = 713;
    if let Some(parent) = &mut scene.visual_layers[0].ancestors {
        parent.clip.end_ms = 713;
    }
    if let Some(clock) = &mut scene.visual_layers[0].instance {
        clock.end_ms = 713.0;
    }
    let (_, inactive) = temporal_prepare_samples(
        scene,
        crate::render_plan::RenderIntent::Frame { at_ms: 713 },
        &|_, _, _| panic!(),
    );
    assert_eq!(temporal_pam_alpha(&inactive[0]).0, 0.0);
}
#[test]
fn temporal_requested_origin_eligibility_and_media_source_mapping_are_scoped() {
    let mut scene = evaluate_project(&project(false), 64, 64, 10).unwrap().scene;
    scene.visual_layers.truncate(1);
    let layer = &mut scene.visual_layers[0];
    layer.source_size = Some((5, 5));
    assert!(!extended_visual::required(layer));
    assert!(extended_visual::sampling_required(layer, 713, 10));
    for start in [0, 300, 500, 1000] {
        assert!(!extended_visual::sampling_required(layer, start, 10));
    }
    assert!(extended_visual::sampling_required(layer, u64::MAX, 10));
    layer.keyframes.clear();
    assert!(extended_visual::sampling_required(layer, 713, 10));
    layer.transform2d = Some(Default::default());
    assert!(extended_visual::sampling_required(layer, 713, 10));
    layer.source = EvaluatedVisualSource::Caption(EvaluatedCaption {
        text: "caption".into(),
        font_size: 12,
        color: "#ffffff".into(),
        background_color: "#000000".into(),
        bottom_margin_px: 0,
    });
    assert!(extended_visual::sampling_required(layer, 713, 10));
    layer.source = EvaluatedVisualSource::Rectangle {
        color: "#ff0000".into(),
        width: 5,
        height: 5,
    };
    layer.transform2d = None;
    layer.keyframes = vec![
        EvaluatedKeyframe {
            property: EvaluatedProperty::Position,
            time_ms: 0,
            value: EvaluatedKeyframeValue::Position { x: 8.0, y: 5.0 },
            easing: EvaluatedEasing::EaseIn,
            r#loop: None,
            clock: None,
        },
        EvaluatedKeyframe {
            property: EvaluatedProperty::Position,
            time_ms: 1000,
            value: EvaluatedKeyframeValue::Position { x: 20.0, y: 5.0 },
            easing: EvaluatedEasing::Hold,
            r#loop: None,
            clock: None,
        },
    ];
    let (_, legacy) = temporal_prepare_samples(
        scene.clone(),
        crate::render_plan::RenderIntent::Frame { at_ms: 713 },
        &|_, _, _| panic!(),
    );
    assert!(
        (temporal_pam_alpha(&legacy[0]).1 - (8.0 + 12.0 * 0.713_f64.powi(2) + 2.0)).abs() < 0.04
    );
    scene.visual_layers[0].source = EvaluatedVisualSource::Media {
        asset_id: "synthetic".into(),
        source_in_ms: 100,
    };
    let calls = std::cell::RefCell::new(vec![]);
    scene.audio_layers.push(EvaluatedAudioLayer {
        instance: None,
        item_id: "unchanged-audio".into(),
        order: EvaluatedLayerOrder {
            track_index: 2,
            item_index: 0,
        },
        asset_id: "audio".into(),
        span: EvaluatedTimeSpan {
            start_ms: 100,
            end_ms: 1300,
        },
        source_in_ms: 50,
        volume: 0.5,
        fade_in_ms: 30,
        fade_out_ms: 40,
        volume_keyframes: vec![],
        retained_timeline_delay: true,
        role: EvaluatedAudioRole::Music,
        ducking: None,
    });
    let audio = scene.audio_layers.clone();
    let (after, _) = temporal_prepare_samples(
        scene,
        crate::render_plan::RenderIntent::Range {
            start_ms: 713,
            end_ms: 913,
            include_audio: true,
        },
        &|_, time, size| {
            assert_eq!(size, (5, 5));
            calls.borrow_mut().push(time);
            Ok([255, 0, 0, 255].repeat(25))
        },
    );
    assert_eq!(*calls.borrow(), vec![813, 913]);
    assert_eq!(after.audio_layers, audio);
}
#[test]
fn temporal_requested_origin_unsafe_canvas_and_dependency_fail_before_workspace() {
    let mut p = project(false);
    p.settings.width = 8192;
    p.settings.height = 8192;
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("uncreated-project");
    let renderer = crate::Renderer::new("/unavailable-ffmpeg", "/unavailable-ffprobe", None);
    let scene = evaluate_project(&p, 8192, 8192, 10).unwrap().scene;
    let error = extended_visual::preflight_samples(&scene, 713, 713, true).unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
    assert!(error.message.contains("sampled output surface"));
    // Preserve the existing facade's earlier read-only backend readiness error.
    assert_eq!(
        renderer
            .render_preview(&p, &destination, 713)
            .unwrap_err()
            .code,
        crate::ErrorCode::DependencyUnavailable
    );
    assert!(!destination.exists());
}

#[test]
fn temporal_requested_origin_text_requires_existing_shaped_pam_binding() {
    let mut scene = evaluate_project(&project(false), 64, 64, 10).unwrap().scene;
    scene.visual_layers.truncate(1);
    scene.visual_layers[0].source_size = Some((5, 5));
    let mut text = EvaluatedText {
        spans: None,
        font_binding: None,
        shaped: None,
        rich_runs: None,
        text: "synthetic PAM".into(),
        font_size: 12,
        color: "#ff0000".into(),
        font_resource_id: None,
        style: evaluate_text_style(&crate::TextStyle::default()).unwrap(),
    };
    scene.visual_layers[0].source = EvaluatedVisualSource::Text(Box::new(text.clone()));
    assert!(!extended_visual::sampling_required(
        &scene.visual_layers[0],
        713,
        10
    ));
    let (_, excluded) = temporal_prepare_samples(
        scene.clone(),
        crate::render_plan::RenderIntent::Frame { at_ms: 713 },
        &|_, _, _| panic!(),
    );
    assert!(excluded.is_empty());
    text.shaped = Some(crate::fonts::shaping::ShapedText {
        layout: None,
        glyphs: vec![],
        line_widths: vec![5.0],
        glyph_lines: vec![],
        line_height: 5.0,
        width: 5.0,
        height: 5.0,
        font_size: 12,
    });
    scene.visual_layers[0].source = EvaluatedVisualSource::Text(Box::new(text));
    assert!(extended_visual::sampling_required(
        &scene.visual_layers[0],
        713,
        10
    ));
    let (sampled, included) = temporal_prepare_samples(
        scene,
        crate::render_plan::RenderIntent::Frame { at_ms: 713 },
        &|_, _, _| panic!("text reads its prepared PAM rather than decoding media"),
    );
    assert_eq!(included.len(), 1);
    assert!(temporal_pam_alpha(&included[0]).0 > 1000.0);
    assert!(sampled.visual_layers[0].sampled_input.is_some());
}

#[test]
fn temporal_requested_origin_legacy_scale_uses_exact713_and813_pixels() {
    let mut scene = evaluate_project(&project(false), 64, 64, 10).unwrap().scene;
    scene.visual_layers.truncate(1);
    let layer = &mut scene.visual_layers[0];
    layer.source_size = Some((5, 5));
    layer.keyframes = vec![
        EvaluatedKeyframe {
            property: EvaluatedProperty::Scale,
            time_ms: 0,
            value: EvaluatedKeyframeValue::Scalar { value: 1.0 },
            easing: EvaluatedEasing::Linear,
            r#loop: None,
            clock: None,
        },
        EvaluatedKeyframe {
            property: EvaluatedProperty::Scale,
            time_ms: 1000,
            value: EvaluatedKeyframeValue::Scalar { value: 2.0 },
            easing: EvaluatedEasing::Hold,
            r#loop: None,
            clock: None,
        },
    ];
    let (_, images) = temporal_prepare_samples(
        scene,
        crate::render_plan::RenderIntent::Range {
            start_ms: 713,
            end_ms: 913,
            include_audio: false,
        },
        &|_, _, _| panic!(),
    );
    assert_eq!(images.len(), 2);
    for (n, image) in images.iter().enumerate() {
        let scale = 1.0 + (713.0 + 100.0 * n as f64) / 1000.0;
        let (mass, x, y) = temporal_pam_alpha(image);
        assert!((x - (8.0 + 2.5 * scale - 0.5)).abs() < 0.04);
        assert!((y - (5.0 + 2.5 * scale - 0.5)).abs() < 0.04);
        assert!((mass / (25.0 * 255.0) - scale * scale).abs() < 0.02);
    }
}

#[test]
fn temporal_requested_origin_opacity_only_rectangle_retains_intrinsic_pixels() {
    let mut p = project(false);
    p.tracks[1].items.truncate(1);
    p.tracks[1].items[0]
        .visual_properties_mut()
        .animation_channels = serde_json::from_value(serde_json::json!([{
        "property":"transform.opacity","keyframes":[
            {"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"linear"},
            {"timeMs":1000,"value":{"type":"scalar","value":0.5},"curve":"hold"}]
    }]))
    .unwrap();
    let scene = evaluate_project(&p, 64, 64, 10).unwrap().scene;
    let layer = &scene.visual_layers[0];
    assert!(!layer.requires_affine());
    assert_eq!(layer.source_size, Some((5, 5)));
    let (_, images) = temporal_prepare_samples(
        scene,
        crate::render_plan::RenderIntent::Range {
            start_ms: 713,
            end_ms: 913,
            include_audio: false,
        },
        &|_, _, _| panic!(),
    );
    assert_eq!(images.len(), 2);
    for image in images {
        let marker = b"ENDHDR\n";
        let offset = image
            .windows(marker.len())
            .position(|b| b == marker)
            .unwrap()
            + marker.len();
        assert_eq!(
            image[offset..]
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] > 0)
                .count(),
            25
        );
        let (mass, x, y) = temporal_pam_alpha(&image);
        assert!((mass - 25.0 * 127.5).abs() <= 12.5);
        assert_eq!((x, y), (10.0, 7.0));
    }
}

#[test]
fn epic6_static_sources_use_requested_grid_and_canonical_local_dimensions() {
    let mut p = project(false);
    p.tracks[1].items.truncate(1);
    p.tracks[1].items[0]
        .visual_properties_mut()
        .animation_channels
        .clear();
    let mut scene = evaluate_project(&p, 64, 64, 10).unwrap().scene;
    let base = scene.visual_layers[0].clone();
    assert!(!extended_visual::required(&base));
    for source in [
        base.source.clone(),
        EvaluatedVisualSource::SolidColor {
            color: "#ff0000".into(),
        },
        EvaluatedVisualSource::Media {
            asset_id: "asset".into(),
            source_in_ms: 0,
        },
        EvaluatedVisualSource::Caption(EvaluatedCaption {
            text: "HH : % \\".into(),
            font_size: 12,
            color: "#ff0000".into(),
            background_color: "#000000".into(),
            bottom_margin_px: 3,
        }),
    ] {
        let mut layer = base.clone();
        layer.source = source;
        assert!(extended_visual::sampling_required(&layer, 713, 10));
        assert!(extended_visual::sampling_required(&layer, 713, 30));
        assert!(!extended_visual::sampling_required(&layer, 700, 10));
        assert!(!extended_visual::sampling_required(&layer, 0, 10));
    }
    scene.visual_layers[0].source = EvaluatedVisualSource::Caption(EvaluatedCaption {
        text: "HH".into(),
        font_size: 12,
        color: "#ff0000".into(),
        background_color: "#000000".into(),
        bottom_margin_px: 3,
    });
    scene.visual_layers[0].source_size = None;
    extended_visual::finalize_intrinsic_sources(&mut scene, 713);
    assert_eq!(scene.visual_layers[0].source_size, Some((64, 64)));
    let instance = evaluate_project(&project(true), 64, 64, 10)
        .unwrap()
        .scene
        .visual_layers[0]
        .instance
        .unwrap();
    let mut instance = instance;
    instance.canvas = (31, 23);
    scene.visual_layers[0].instance = Some(instance);
    extended_visual::finalize_intrinsic_sources(&mut scene, 713);
    assert_eq!(scene.visual_layers[0].source_size, Some((31, 23)));
    let caption = match &scene.visual_layers[0].source {
        EvaluatedVisualSource::Caption(c) => c,
        _ => unreachable!(),
    };
    assert_eq!(
        caption_legacy_position(caption, (11, 13), (31, 23)),
        (10.0, 19.0)
    );
    // Full-canvas plain paint owns its placement; do not center it a second time.
    scene.visual_layers[0].instance = None;
    let a = extended_visual::sample_transform(&mut scene.visual_layers[0], 713, (64, 64), (64, 64))
        .unwrap();
    assert_eq!(a.matrix, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    // Legacy raw facts stay unchanged; sampled preparation owns intrinsic enrichment.
    let value = serde_json::to_value(&p.tracks[1].items[0]).unwrap();
    let mut value = value;
    value["type"] = serde_json::json!("solid_color");
    value.as_object_mut().unwrap().remove("width");
    value.as_object_mut().unwrap().remove("height");
    p.tracks[1].items[0] = serde_json::from_value(value).unwrap();
    let mut solid = evaluate_project(&p, 64, 64, 10).unwrap().scene;
    assert!(solid.visual_layers[0].source_size.is_none());
    let raw = format!("{solid:?}");
    extended_visual::finalize_intrinsic_sources(&mut solid, 700);
    assert_eq!(format!("{solid:?}"), raw);
    extended_visual::finalize_intrinsic_sources(&mut solid, 713);
    assert_eq!(solid.visual_layers[0].source_size, Some((64, 64)));
    extended_visual::preflight_samples(&solid, 713, 713, true).unwrap();
    let mut local = solid.clone();
    local.visual_layers[0].source_size = None;
    local.visual_layers[0].instance = Some(instance);
    extended_visual::finalize_intrinsic_sources(&mut local, 713);
    assert_eq!(local.visual_layers[0].source_size, Some((31, 23)));
    extended_visual::preflight_samples(&local, 713, 713, true).unwrap();
    // A genuine already-measured raster is never replaced by canvas enrichment.
    local.visual_layers[0].source_size = Some((16_385, 1));
    extended_visual::finalize_intrinsic_sources(&mut local, 713);
    assert_eq!(local.visual_layers[0].source_size, Some((16_385, 1)));
    assert_eq!(
        extended_visual::preflight_samples(&local, 713, 713, true)
            .unwrap_err()
            .code,
        crate::ErrorCode::InvalidArgument
    );
}

#[test]
fn epic6_caption_stream_caches_one_source_and_samples_half_open_activity() {
    use crate::render_artifact::{FileSystemArtifactIo, PreparedRenderResources};
    let mut p = project(false);
    p.tracks[1].items.truncate(1);
    p.tracks[1].items[0]
        .visual_properties_mut()
        .animation_channels
        .clear();
    let mut scene = evaluate_project(&p, 64, 64, 10).unwrap().scene;
    let layer = &mut scene.visual_layers[0];
    layer.source = EvaluatedVisualSource::Caption(EvaluatedCaption {
        text: "HH".into(),
        font_size: 12,
        color: "#ff0000".into(),
        background_color: "#000000".into(),
        bottom_margin_px: 3,
    });
    layer.span = EvaluatedTimeSpan {
        start_ms: 713,
        end_ms: 799,
    };
    extended_visual::finalize_intrinsic_sources(&mut scene, 713);
    extended_visual::preflight_samples(&scene, 713, 913, false).unwrap();
    let root = tempfile::tempdir().unwrap();
    let mut resources = PreparedRenderResources {
        media_inputs: vec![],
        media_paths: vec![],
        text_layers: Default::default(),
    };
    let calls = std::cell::Cell::new(0);
    let frames = std::cell::RefCell::new(vec![]);
    crate::render_artifact::extended_visual::prepare(
        &FileSystemArtifactIo,
        &mut scene,
        root.path(),
        &mut resources,
        crate::render_plan::RenderIntent::Range {
            start_ms: 713,
            end_ms: 913,
            include_audio: false,
        },
        &crate::render_artifact::extended_visual::VisualPreparation {
            decode: &|_, _, _| panic!("caption must not decode media"),
            encode: &|_, fps, count, produce| {
                assert_eq!((fps, count), (10, 2));
                for n in 0..count {
                    frames.borrow_mut().push(produce(n)?);
                }
                Ok(())
            },
            caption: &|_, _, size| {
                calls.set(calls.get() + 1);
                assert_eq!(size, (64, 64));
                Ok([255, 0, 0, 255].repeat(64 * 64))
            },
        },
    )
    .unwrap();
    assert_eq!(calls.get(), 1);
    let images = frames.borrow();
    assert_eq!(images.len(), 2);
    assert_eq!(temporal_pam_alpha(&images[0]).0, 64.0 * 64.0 * 255.0);
    let bytes = &images[1];
    let header = b"ENDHDR\n";
    let offset = bytes
        .windows(header.len())
        .position(|p| p == header)
        .unwrap()
        + header.len();
    assert!(bytes[offset..].iter().all(|b| *b == 0));
}

#[test]
fn epic6_source_limits_fail_before_source_callbacks_or_output() {
    assert_eq!(
        extended_visual::validate_sampled_source_size((4096, 4096)).unwrap(),
        67_108_864
    );
    assert_eq!(
        extended_visual::validate_sampled_source_size((16384, 1)).unwrap(),
        65_536
    );
    for size in [
        (0, 1),
        (1, 0),
        (16385, 1),
        (4097, 4096),
        (u32::MAX, u32::MAX),
    ] {
        assert_eq!(
            extended_visual::validate_sampled_source_size(size)
                .unwrap_err()
                .code,
            crate::ErrorCode::InvalidArgument
        );
    }
    let mut p = project(false);
    p.tracks[1].items.truncate(1);
    p.tracks[1].items[0]
        .visual_properties_mut()
        .animation_channels
        .clear();
    let mut scene = evaluate_project(&p, 64, 64, 10).unwrap().scene;
    scene.visual_layers[0].source_size = Some((4097, 4096));
    assert_eq!(
        extended_visual::preflight_samples(&scene, 713, 913, false)
            .unwrap_err()
            .code,
        crate::ErrorCode::InvalidArgument
    );
    let executor = crate::render_process::SystemProcessExecutor;
    for size in [(0, 1), (4097, 4096), (16385, 1)] {
        assert_eq!(
            crate::render_process::ProcessExecutor::raster_source(
                &executor,
                std::path::Path::new("/must-not-execute"),
                "invalid",
                size
            )
            .unwrap_err()
            .code,
            crate::ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn epic6_component_solid_uses_authored_leaf_canvas_with_fractional_retained_clock() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path(), true);
    let p = f.project();
    let original_ids: Vec<_> = evaluate_project(&p, 64, 64, 10)
        .unwrap()
        .scene
        .visual_layers
        .into_iter()
        .map(|l| l.item_id)
        .collect();
    let leaf = p
        .components
        .iter()
        .find(|c| c.name == "Temporal leaf")
        .unwrap();
    let mut tracks = serde_json::to_value(&leaf.tracks).unwrap();
    let items = tracks[0]["items"].as_array_mut().unwrap();
    let visible_ids: Vec<_> = items
        .iter()
        .filter(|item| {
            item["type"] == "rectangle"
                && item["hidden"] == false
                && !item["animationChannels"].as_array().unwrap().is_empty()
        })
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(visible_ids.len(), 1);
    let backing = items[0].clone();
    let item = items
        .iter_mut()
        .find(|item| item["id"] == visible_ids[0])
        .unwrap();
    assert_eq!(
        item["animationChannels"][0]["clock"],
        serde_json::json!({"offsetMs":250,"sourceDurationMs":1500})
    );
    item["type"] = serde_json::json!("solid_color");
    item.as_object_mut().unwrap().remove("width");
    item.as_object_mut().unwrap().remove("height");
    let id = item["id"].as_str().unwrap().to_owned();
    f.edit(serde_json::json!({"operation":"component_update","componentId":leaf.id,"name":leaf.name,"width":31,"height":23,"durationMs":leaf.duration_ms,"tracks":tracks,"slots":leaf.slots}));
    let updated = f.project();
    assert_eq!(
        serde_json::to_value(
            &updated
                .components
                .iter()
                .find(|c| c.id == leaf.id)
                .unwrap()
                .tracks[0]
                .items[0]
        )
        .unwrap(),
        backing
    );
    let authored = updated
        .components
        .iter()
        .find(|c| c.id == leaf.id)
        .unwrap()
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|item| item.id() == id)
        .unwrap();
    assert!(matches!(authored, TimelineItem::SolidColor(_)));
    let scene = evaluate_project(&updated, 64, 64, 10).unwrap().scene;
    assert_eq!(scene.visual_layers.len(), 3);
    assert_eq!(
        scene
            .visual_layers
            .iter()
            .map(|l| l.item_id.clone())
            .collect::<Vec<_>>(),
        original_ids
    );
    for (j, layer) in scene.visual_layers.iter().enumerate() {
        assert!(matches!(
            layer.source,
            EvaluatedVisualSource::SolidColor { .. }
        ));
        assert_eq!(layer.source_size, Some((31, 23)));
        let clock = layer.instance.unwrap();
        assert_eq!(clock.canvas, (31, 23));
        assert_eq!(clock.rate, 0.75);
        assert_eq!(
            clock.rate * 713.0 + clock.offset + 250.0,
            [644.75, 569.75, 494.75][j]
        );
        assert!(layer.visible_at(713));
        assert!(!layer.visible_at(1300));
    }
}
