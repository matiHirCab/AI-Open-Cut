use opencut_editor_core::MotionBlur;
use serde_json::Value;

// Select exactly the requested frame on backends that evaluate filters ahead.
const SINGLE_FRAME_SSIM: &str = "[0:v]trim=end_frame=1,setpts=PTS-STARTPTS[reference];[1:v]trim=end_frame=1,setpts=PTS-STARTPTS[actual];[reference][actual]ssim";

fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/motion-blur-sampling-v1.json"
    ))
    .unwrap()
}

#[test]
fn canonical_numeric_and_closed_record_cases() {
    assert_eq!(
        u64::from(MotionBlur::MAX_SAMPLES),
        contract()["limits"]["maxSampleCount"].as_u64().unwrap()
    );
    assert_eq!(
        MotionBlur::MAX_PIXEL_WORK,
        contract()["limits"]["maxPixelWorkPerOutputFrame"]
            .as_u64()
            .unwrap()
    );
    for case in contract()["cases"].as_array().unwrap() {
        let accepted = serde_json::from_value::<MotionBlur>(case["value"].clone())
            .is_ok_and(|value| value.validate().is_ok());
        assert_eq!(
            accepted,
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["name"]
        );
    }
    for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            MotionBlur {
                shutter_angle_deg: angle,
                sample_count: 8
            }
            .validate()
            .is_err()
        );
    }
}

#[test]
fn canonical_midpoints_and_boundary_clamps() {
    for case in contract()["sampleCases"].as_array().unwrap() {
        let settings: MotionBlur = serde_json::from_value(case["settings"].clone()).unwrap();
        let actual = settings
            .sample_times(
                case["atMs"].as_u64().unwrap(),
                case["fps"].as_u64().unwrap() as u32,
                case["durationMs"].as_u64().unwrap(),
            )
            .unwrap();
        let expected: Vec<u64> = serde_json::from_value(case["expected"].clone()).unwrap();
        assert_eq!(actual, expected, "{}", case["name"]);
    }
}

#[test]
fn shutter_offsets_preserve_adjacent_large_integer_times() {
    let settings = MotionBlur {
        shutter_angle_deg: 180.0,
        sample_count: 4,
    };
    let root = (1_u64 << 53) + 1;
    assert_eq!(
        settings.sample_times(root, 25, u64::MAX).unwrap(),
        [root - 8, root - 3, root + 2, root + 7]
    );
    assert!(settings.sample_times(root, 0, u64::MAX).is_err());
    assert!(settings.sample_times(0, 25, 0).is_err());
}

fn setup() -> (
    tempfile::TempDir,
    opencut_editor_core::EditorCore,
    String,
    String,
) {
    use opencut_editor_core::{EditorCore, PathPolicy, ProjectSettings};
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Shutter",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

fn edit(value: Value) -> opencut_editor_core::EditOperation {
    serde_json::from_value(value).unwrap()
}

#[test]
fn shutter_alias_batch_lifecycle_and_failure_rollback() {
    use opencut_editor_core::ErrorCode;
    use serde_json::json;
    let (_root, core, id, track) = setup();
    let batch: Vec<opencut_editor_core::BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"},
        {"operation":"update_item","itemId":"@leaf","motionBlur":{"shutterAngleDeg":180,"sampleCount":8}}
    ])).unwrap();
    core.edit_batch(&id, 0, batch).unwrap();
    let state = core.get_project(&id).unwrap();
    let item = state.tracks[1].items[0].id().to_owned();
    assert_eq!(state.revision, 1);
    assert_eq!(
        state
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .motion_blur
            .unwrap()
            .sample_count,
        8
    );
    let before = serde_json::to_value(&state).unwrap();
    for (expected, value, code) in [
        (
            1,
            json!({"operation":"update_item","itemId":item,"motionBlur":{"shutterAngleDeg":361,"sampleCount":8}}),
            ErrorCode::InvalidArgument,
        ),
        (
            0,
            json!({"operation":"update_item","itemId":item,"motionBlur":{"shutterAngleDeg":180,"sampleCount":8}}),
            ErrorCode::RevisionConflict,
        ),
        (
            1,
            json!({"operation":"update_item","itemId":"missing","motionBlur":{"shutterAngleDeg":180,"sampleCount":8}}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        assert_eq!(
            core.edit(&id, expected, edit(value)).unwrap_err().code,
            code
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    let failed: Vec<opencut_editor_core::BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"update_item","itemId":item,"motionBlur":{"shutterAngleDeg":0,"sampleCount":1}},
        {"operation":"update_item","itemId":item,"motionBlur":{"shutterAngleDeg":180,"sampleCount":17}}
    ])).unwrap();
    assert_eq!(
        core.edit_batch(&id, 1, failed).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().find_item(&item).is_none());
    core.redo(&id, 2).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .motion_blur
            .unwrap()
            .sample_count,
        8
    );
    let dir = core.paths().project_dir(&id).unwrap();
    let bytes = std::fs::read(dir.join("project.json")).unwrap();
    core.get_project(&id).unwrap();
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
}

#[test]
fn migration_adopts_complete_history_and_rejects_premature_fields_atomically() {
    use serde_json::json;
    let (_root, core, id, track) = setup();
    core.edit(&id,0,edit(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let leaf = core.get_project(&id).unwrap().tracks[1].items[0]
        .id()
        .to_string();
    core.edit(
        &id,
        1,
        edit(json!({"operation":"update_item","itemId":leaf,"hidden":true})),
    )
    .unwrap();
    core.undo(&id, 2).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    current["schemaVersion"] = json!(27);
    for key in ["undo", "redo"] {
        for snapshot in history[key].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(27);
        }
    }
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&current).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&history).unwrap(),
    )
    .unwrap();
    assert_eq!(core.get_project(&id).unwrap().schema_version, 28);
    let adopted: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for key in ["undo", "redo"] {
        let snapshots = adopted[key].as_array().unwrap();
        assert!(!snapshots.is_empty(), "fixture must retain {key}");
        assert!(snapshots.iter().all(|p| p["schemaVersion"] == 28));
    }
    let current_adopted = std::fs::read(dir.join("project.json")).unwrap();
    let history_adopted = std::fs::read(dir.join("history.json")).unwrap();
    for stack in ["undo", "redo"] {
        let mut invalid_history = adopted.clone();
        invalid_history[stack][0]["schemaVersion"] = json!(27);
        let mut premature = current["tracks"][1]["items"][0].clone();
        premature["motionBlur"] = Value::Null;
        invalid_history[stack][0]["tracks"][1]["items"] = json!([premature]);
        let invalid_bytes = serde_json::to_vec(&invalid_history).unwrap();
        std::fs::write(dir.join("history.json"), &invalid_bytes).unwrap();
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            opencut_editor_core::ErrorCode::InternalError
        );
        assert_eq!(
            std::fs::read(dir.join("project.json")).unwrap(),
            current_adopted
        );
        assert_eq!(
            std::fs::read(dir.join("history.json")).unwrap(),
            invalid_bytes
        );
        std::fs::write(dir.join("history.json"), &history_adopted).unwrap();
    }
    current["tracks"][1]["items"][0]["motionBlur"] = json!({"shutterAngleDeg":0,"sampleCount":1});
    let bytes = serde_json::to_vec(&current).unwrap();
    std::fs::write(dir.join("project.json"), &bytes).unwrap();
    let history_before = std::fs::read(dir.join("history.json")).unwrap();
    assert!(core.get_project(&id).is_err());
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
    assert_eq!(
        std::fs::read(dir.join("history.json")).unwrap(),
        history_before
    );
}

#[test]
fn native_inherited_shutter_matches_independent_pixels_and_range_export() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    use serde_json::json;
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (root, core, id, track) = setup();
    let operations:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"update_item","itemId":"@parent","transform2d":null},
        {"operation":"set_animation_channels","itemId":"@parent","animationChannels":[{"property":"transform.position_x","keyframes":[{"timeMs":0,"curve":"linear","value":{"type":"scalar","value":0}},{"timeMs":999,"curve":"hold","value":{"type":"scalar","value":39.96}}]}]},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"},
        {"operation":"update_item","itemId":"@leaf","transform2d":{"position":{"x":10,"y":10,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1,"anchor":{"x":0,"y":0}},"motionBlur":{"shutterAngleDeg":360,"sampleCount":4}},
        {"operation":"item_set_parent","itemId":"@leaf","parent":{"scope":"root","id":"@parent"}}
    ])).unwrap();
    core.edit_batch(&id, 0, operations).unwrap();
    let wav = root.path().join("media/tone.wav");
    let generated = std::process::Command::new(&ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(&wav)
        .output()
        .unwrap();
    assert!(generated.status.success());
    let asset = core
        .import_asset(
            &id,
            1,
            &wav,
            opencut_editor_core::MediaType::Audio,
            opencut_editor_core::MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let audio_track = core.get_project(&id).unwrap().tracks[2].id.clone();
    core.edit(&id,2,edit(json!({"operation":"add_media","trackId":audio_track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap();
    let project = core.get_project(&id).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let frame = renderer.render_preview(&project, &dir, 500).unwrap();
    let decode = |path: &std::path::Path, time: f64| {
        let result = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", &format!("{time:.3}"), "-i"])
            .arg(path)
            .args([
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout.len(), 64 * 64 * 3);
        result.stdout
    };
    let decoded = decode(&dir.join(&frame.relative_path), 0.0);
    let leaf_id = project.tracks[1]
        .items
        .iter()
        .find(|i| i.visual_properties().motion_blur.is_some())
        .unwrap()
        .id()
        .to_owned();
    let mut omitted = project.clone();
    omitted.tracks[1]
        .items
        .iter_mut()
        .find(|i| i.id() == leaf_id)
        .unwrap()
        .visual_properties_mut()
        .motion_blur = None;
    let legacy = renderer.render_preview(&omitted, &dir, 500).unwrap();
    let legacy_pixels = decode(&dir.join(legacy.relative_path), 0.0);
    for settings in [
        MotionBlur {
            shutter_angle_deg: 0.0,
            sample_count: 16,
        },
        MotionBlur {
            shutter_angle_deg: 360.0,
            sample_count: 1,
        },
    ] {
        let mut compatible = omitted.clone();
        compatible.tracks[1]
            .items
            .iter_mut()
            .find(|i| i.id() == leaf_id)
            .unwrap()
            .visual_properties_mut()
            .motion_blur = Some(settings);
        let frame = renderer.render_preview(&compatible, &dir, 500).unwrap();
        assert_eq!(decode(&dir.join(frame.relative_path), 0.0), legacy_pixels);
    }
    let draft=core.create_draft(&id,3,vec![edit(json!({"operation":"update_item","itemId":leaf_id,"motionBlur":{"shutterAngleDeg":360,"sampleCount":4}}))],Some("Shutter oracle".into())).unwrap();
    let draft_project = core.get_draft_state(&id, &draft.id).unwrap().project;
    let draft_frame = renderer.render_preview(&draft_project, &dir, 500).unwrap();
    assert_eq!(decode(&dir.join(draft_frame.relative_path), 0.0), decoded);
    let mut clipped = project.clone();
    if let opencut_editor_core::TimelineItem::Rectangle(leaf) = clipped.tracks[1]
        .items
        .iter_mut()
        .find(|i| i.id() == leaf_id)
        .unwrap()
    {
        leaf.duration_ms = 600;
    }
    let tail = renderer.render_preview(&clipped, &dir, 620).unwrap();
    assert!(
        decode(&dir.join(tail.relative_path), 0.0)
            .as_chunks::<3>()
            .0
            .iter()
            .any(|p| p[0] > 10),
        "a shutter sample before the leaf end must survive center-time clipping"
    );

    // Independent analytic midpoint/parent-transform oracle. The final backend
    // uses RGBA stacking and final YUV420 conversion. Apply that fixed output
    // selection to independently computed alpha before comparing decoded RGB.
    let coverage = |local: f64| {
        if (-0.5..0.5).contains(&local) {
            local + 0.5
        } else if (0.5..=3.5).contains(&local) {
            1.0
        } else if (3.5..4.5).contains(&local) {
            4.5 - local
        } else {
            0.0
        }
    };
    let mut pam =
        b"P7\nWIDTH 64\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
    for y in 0..64 {
        for x in 0..64 {
            let alpha = [462.0, 487.0, 512.0, 537.0]
                .into_iter()
                .map(|t| {
                    coverage(x as f64 + 0.5 - (10.0 + 40.0 * t / 1000.0))
                        * coverage(y as f64 + 0.5 - 10.0)
                })
                .sum::<f64>()
                / 4.0;
            pam.extend_from_slice(&[
                if alpha == 0.0 { 0 } else { 255 },
                0,
                0,
                (alpha * 255.0).round() as u8,
            ]);
        }
    }
    let oracle = root.path().join("oracle.pam");
    std::fs::write(&oracle, pam).unwrap();
    let reference=std::process::Command::new(&ffmpeg).args(["-v","error","-f","lavfi","-i","color=c=black:s=64x64:r=10:d=1","-i"]).arg(&oracle)
        .args(["-filter_complex","[0:v]format=rgba[base];[1:v]format=rgba[leaf];[base][leaf]overlay=format=auto:x=0:y=0:eof_action=pass,format=yuv420p[out]","-map","[out]","-frames:v","1","-f","rawvideo","-pix_fmt","rgb24","pipe:1"]).output().unwrap();
    assert!(
        reference.status.success(),
        "{}",
        String::from_utf8_lossy(&reference.stderr)
    );
    assert_eq!(reference.stdout.len(), decoded.len());
    let mse = decoded
        .iter()
        .zip(&reference.stdout)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
        .sum::<f64>()
        / decoded.len() as f64;
    assert!(mse <= 1.0, "independent inherited shutter MSE={mse}");
    let nonzero = (20..40).filter(|x| decoded[(11 * 64 + x) * 3] > 0).count();
    assert!(nonzero > 4, "blur must widen the four-pixel leaf");
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 400,
                end_ms: 700,
                width: 64,
                height: 64,
                fps: 20,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let exports = root.path().join("exports");
    std::fs::create_dir_all(&exports).unwrap();
    let export = exports.join("blur.mp4");
    let blurred_started = std::time::Instant::now();
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &export,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let blurred_elapsed = blurred_started.elapsed();
    let mut control = project.clone();
    for item in &mut control.tracks[1].items {
        let visual = item.visual_properties_mut();
        if visual.motion_blur.is_some() {
            visual.motion_blur = Some(MotionBlur {
                shutter_angle_deg: 360.0,
                sample_count: 1,
            });
            // An identity effect keeps the instantaneous control on the same
            // sampled raster/FFV1 pipeline for a report-only cost observation.
            visual
                .effects
                .push(opencut_editor_core::VisualEffect::GaussianBlur {
                    id: "benchmark-identity".into(),
                    radius_px: 0.0,
                });
        }
    }
    let control_export = exports.join("control.mp4");
    let control_started = std::time::Instant::now();
    renderer
        .export_video(
            &control,
            &dir,
            ExportOptions {
                output: &control_export,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    println!(
        "64x64 10-fps 1-second export: four shutter samples {:?}; one sampled-pipeline sample {:?}",
        blurred_elapsed,
        control_started.elapsed()
    );
    let control_range = renderer
        .render_preview_range(
            &control,
            &dir,
            PreviewRangeOptions {
                start_ms: 400,
                end_ms: 700,
                width: 64,
                height: 64,
                fps: 20,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let pcm = |path: &std::path::Path| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(path)
            .args([
                "-map",
                "0:a:0",
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "-ac",
                "2",
                "-ar",
                "48000",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
            .stdout
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| f32::from_le_bytes(*v))
            .collect::<Vec<_>>()
    };
    for (a, b) in [
        (&export, &control_export),
        (
            &dir.join(&range.relative_path),
            &dir.join(control_range.relative_path),
        ),
    ] {
        let actual = pcm(a);
        let expected = pcm(b);
        assert_eq!(actual.len(), expected.len());
        assert!(!actual.is_empty());
        let rms = (actual
            .iter()
            .zip(expected)
            .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
            .sum::<f64>()
            / actual.len() as f64)
            .sqrt();
        assert!(rms <= 0.0001, "unchanged shutter audio RMS={rms}");
    }
    for (path, time) in [(dir.join(range.relative_path), 0.1), (export, 0.5)] {
        let comparison = std::process::Command::new(&ffmpeg)
            .current_dir(root.path())
            .args(["-v", "info", "-i"])
            .arg(dir.join(&frame.relative_path))
            .args(["-ss", &format!("{time:.3}"), "-i"])
            .arg(path)
            .args([
                "-lavfi",
                &format!("{SINGLE_FRAME_SSIM}=stats_file=single-frame-ssim.txt"),
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .output()
            .unwrap();
        assert!(comparison.status.success());
        assert_eq!(
            std::fs::read_to_string(root.path().join("single-frame-ssim.txt"))
                .unwrap()
                .lines()
                .count(),
            1,
            "comparison evaluated later animation frames"
        );
        let log = String::from_utf8_lossy(&comparison.stderr);
        let score = log
            .split("All:")
            .last()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(score >= 0.99, "{log}");
    }
}

#[test]
fn unsupported_controllers_and_hidden_sample_work_fail_before_publication() {
    use opencut_editor_core::{EditorCore, ErrorCode, PathPolicy, ProjectSettings};
    use serde_json::json;
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Work",
            ProjectSettings {
                width: 4096,
                height: 4096,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let group = core
        .edit(
            &id,
            0,
            edit(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    assert_eq!(core.edit(&id,1,edit(json!({"operation":"update_item","itemId":group,"motionBlur":{"shutterAngleDeg":0,"sampleCount":1}}))).unwrap_err().code,ErrorCode::InvalidArgument);
    let leaf=core.edit(&id,1,edit(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":1,"height":1,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id,2,edit(json!({"operation":"update_item","itemId":leaf,"motionBlur":{"shutterAngleDeg":360,"sampleCount":16}}))).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let history = std::fs::read(dir.join("history.json")).unwrap();
    let operations:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":1,"height":1,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"excess"},
        {"operation":"update_item","itemId":"@excess","hidden":true,"motionBlur":{"shutterAngleDeg":360,"sampleCount":16}}
    ])).unwrap();
    assert_eq!(
        core.edit_batch(&id, 3, operations).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
}

#[test]
fn native_shutter_nested_repeated_staggered_loop_and_effect_samples_share_range() {
    use opencut_editor_core::{BatchEditOperation, PreviewRangeOptions, Renderer};
    use serde_json::json;
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (_root, core, id, track) = setup();
    let scalar_channel = |property: &str, first: f64, last: f64, curve: serde_json::Value| json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":first},"curve":curve},{"timeMs":500,"value":{"type":"scalar","value":last},"curve":"hold"}]});
    let component = core.edit(&id,0,edit(json!({"operation":"component_create","name":"Nested","width":64,"height":64,"durationMs":2000,"tracks":[]}))).unwrap().changed_ids[0].clone();
    let mut rotation = scalar_channel("transform.rotation_deg", 0.0, 90.0, json!("linear"));
    rotation["keyframes"][1]["timeMs"] = json!(200);
    rotation["loop"] = json!({"mode":"ping_pong","iterations":"infinite"});
    let mut tint = scalar_channel("effect.vignette_amount", 0.0, 0.8, json!("linear"));
    tint["target"] = json!({"kind":"effect","scope":format!("component:{component}"),"id":"edge"});
    core.edit(&id,1,edit(json!({"operation":"component_update","componentId":component,"name":"Nested","width":64,"height":64,"durationMs":2000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"rectangle","id":"leaf","startMs":0,"durationMs":2000,"width":16,"height":8,"color":"#dddddd","transform":{"positionX":20,"positionY":20,"scale":1,"opacity":1},"motionBlur":{"shutterAngleDeg":360,"sampleCount":4},"effects":[{"id":"edge","type":"vignette","amount":0}],"animationChannels":[rotation,tint],"keyframes":[],"zIndex":0,"stackOrder":0}]}]}))).unwrap();
    let edits: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"staggerMs":50,"resultAlias":"group"},
        {"operation":"set_animation_channels","itemId":"@group","animationChannels":[scalar_channel("transform.rotation_deg",0.0,20.0,json!("linear"))]},
        {"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":2,"resultAlias":"instance"},
        {"operation":"item_set_parent","itemId":"@instance","parent":{"scope":"root","id":"@group"}},
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":"@group"},"copies":1,"timeOffsetMs":-100,"opacityOffset":0,"transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}
    ])).unwrap();
    core.edit_batch(&id, 2, edits).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    project.settings.fps = 20;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 200,
                end_ms: 1000,
                width: 64,
                height: 64,
                fps: 20,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let mut frames = Vec::new();
    for time in [200, 250, 400, 450, 900] {
        let frame = renderer.render_preview(&project, &dir, time).unwrap();
        frames.push(std::fs::read(dir.join(&frame.relative_path)).unwrap());
        let comparison = std::process::Command::new(&ffmpeg)
            .current_dir(_root.path())
            .args(["-v", "info", "-i"])
            .arg(dir.join(frame.relative_path))
            .args(["-ss", &format!("{:.3}", (time - 200) as f64 / 1000.0), "-i"])
            .arg(dir.join(&range.relative_path))
            .args([
                "-frames:v",
                "1",
                "-lavfi",
                &format!("{SINGLE_FRAME_SSIM}=stats_file=single-frame-ssim.txt"),
                "-f",
                "null",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            ])
            .output()
            .unwrap();
        assert!(
            comparison.status.success(),
            "{}",
            String::from_utf8_lossy(&comparison.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(_root.path().join("single-frame-ssim.txt"))
                .unwrap()
                .lines()
                .count(),
            1,
            "comparison evaluated later animation frames"
        );
        let log = String::from_utf8_lossy(&comparison.stderr);
        let ssim = log
            .split("All:")
            .nth(1)
            .and_then(|v| v.split_whitespace().next())
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(ssim >= 0.99, "nested sample {time}: {ssim}");
    }
    assert!(frames.windows(2).any(|pair| pair[0] != pair[1]));
}

#[test]
fn legacy_draft_fields_fail_before_adoption_and_compatible_drafts_reopen_deterministically() {
    use serde_json::json;
    let (_root, core, id, track) = setup();
    let leaf = core.edit(&id, 0, edit(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    let draft = core.create_draft(&id, 1, vec![edit(json!({"operation":"update_item","itemId":leaf,"motionBlur":{"shutterAngleDeg":180,"sampleCount":4}}))], None).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
    let mut project: Value =
        serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    project["schemaVersion"] = json!(27);
    for stack in ["undo", "redo"] {
        for snapshot in history[stack].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(27);
        }
    }
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&history).unwrap(),
    )
    .unwrap();
    let paths = [
        dir.join("project.json"),
        dir.join("history.json"),
        draft_path.clone(),
    ];
    let before: Vec<_> = paths.iter().map(|p| std::fs::read(p).unwrap()).collect();
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        opencut_editor_core::ErrorCode::InvalidArgument
    );
    for (path, bytes) in paths.iter().zip(&before) {
        assert_eq!(std::fs::read(path).unwrap(), *bytes);
    }
    let mut compatible: Value = serde_json::from_slice(&before[2]).unwrap();
    compatible["operations"] = json!([{"operation":"update_item","itemId":leaf,"hidden":false}]);
    std::fs::write(&draft_path, serde_json::to_vec(&compatible).unwrap()).unwrap();
    assert_eq!(core.get_project(&id).unwrap().schema_version, 28);
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .schema_version,
        28
    );
    let adopted: Vec<_> = paths.iter().map(|p| std::fs::read(p).unwrap()).collect();
    core.get_project(&id).unwrap();
    core.get_draft_state(&id, &draft.id).unwrap();
    for (path, bytes) in paths.iter().zip(&adopted) {
        assert_eq!(std::fs::read(path).unwrap(), *bytes);
    }
}
