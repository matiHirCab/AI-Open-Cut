//! Frozen complete hero plates and actual public media/lifecycle conformance.
#[path = "support/masked_hero_reveal.rs"]
mod hero;
use hero::{Fixture, catalog, inventory, op, reference};
use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Project, Renderer};
use serde_json::{Value, json};
#[path = "support/masked_hero_media.rs"]
mod media;
use media::{
    Native, aligned_stereo, audio_reference, close_bytes, decoded_movie, independent_film, raw_pam,
    ssim,
};
use std::path::{Path, PathBuf};
fn expected(generation: &str, time: u64) -> Vec<u8> {
    std::fs::read(reference(&format!("{generation}-{time:04}.rgba"))).unwrap()
}
fn still(
    native: &Native,
    renderer: &Renderer,
    f: &Fixture,
    captured: &Path,
    p: &Project,
    generation: &str,
    time: u64,
) -> Vec<u8> {
    let rgba = expected(generation, time);
    let pixels: Vec<[u8; 4]> = rgba.as_chunks::<4>().0.to_vec();
    let converted = native.authored_plate(f.root.path(), &pixels);
    let artifact = renderer.render_preview(p, &f.dir(), time).unwrap();
    if let Some(root) = std::env::var_os("OPENCUT_HERO_DIAGNOSTICS") {
        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::copy(
            captured,
            root.join(format!("{generation}-{time:04}-actual.pam")),
        )
        .unwrap();
        std::fs::write(
            root.join(format!("{generation}-{time:04}-project.json")),
            serde_json::to_vec_pretty(p).unwrap(),
        )
        .unwrap();
    }
    close_bytes(&raw_pam(captured), &rgba, "complete canonical hero PAM");
    let decoded = native.decode(&f.dir().join(artifact.relative_path), false, None);
    close_bytes(&decoded, &converted, "independent converted hero PNG");
    decoded
}
fn movies(
    native: &Native,
    renderer: &Renderer,
    f: &Fixture,
    p: &Project,
    generation: &str,
    label: &str,
) -> std::collections::BTreeMap<String, (Vec<u8>, Vec<u8>)> {
    let mut signatures = std::collections::BTreeMap::new();
    let mut partial = None;
    let mut full = None;
    for start in [0, 200] {
        let artifact = renderer
            .render_preview_range(
                p,
                &f.dir(),
                PreviewRangeOptions {
                    start_ms: start,
                    end_ms: 800,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let path = f.dir().join(artifact.relative_path);
        verify_movie(native, f, &path, generation, start);
        signatures.insert(format!("range-{start}"), decoded_movie(native, &path));
        if start == 0 {
            full = Some(path);
        } else {
            partial = Some(path);
        }
    }
    let output = f.root.path().join(format!("exports/hero-{label}.mp4"));
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    renderer
        .export_video(
            p,
            &f.dir(),
            ExportOptions {
                output: &output,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    verify_movie(native, f, &output, generation, 0);
    signatures.insert("export".into(), decoded_movie(native, &output));
    for time in (0..800).step_by(100) {
        assert!(
            ssim(
                &native.decode(
                    full.as_ref().unwrap(),
                    false,
                    Some(&format!("{}", time as f64 / 1000.0))
                ),
                &native.decode(&output, false, Some(&format!("{}", time as f64 / 1000.0)))
            ) >= 0.99
        );
    }
    for time in (200..800).step_by(100) {
        let original = format!("{}", time as f64 / 1000.0);
        let local = format!("{}", (time - 200) as f64 / 1000.0);
        let partial = native.decode(partial.as_ref().unwrap(), false, Some(&local));
        assert!(
            ssim(
                &partial,
                &native.decode(full.as_ref().unwrap(), false, Some(&original))
            ) >= 0.99
        );
        assert!(ssim(&partial, &native.decode(&output, false, Some(&original))) >= 0.99);
    }
    signatures
}
fn verify_movie(native: &Native, f: &Fixture, path: &Path, generation: &str, start: u64) {
    let probe = native.probe(path);
    let streams = probe["streams"].as_array().unwrap();
    let video = streams.iter().find(|v| v["codec_type"] == "video").unwrap();
    let audio = streams.iter().find(|v| v["codec_type"] == "audio").unwrap();
    assert_eq!(video["width"], 64);
    assert_eq!(video["height"], 64);
    assert_eq!(video["nb_frames"], ((800 - start) / 100).to_string());
    assert_eq!(audio["sample_rate"], "48000");
    assert_eq!(audio["channels"], 2);
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(audio["codec_name"], "aac");
    assert_eq!(video["r_frame_rate"], "10/1");
    let start_time = |v: &Value| v.as_str().unwrap().parse::<f64>().unwrap();
    assert!((start_time(&video["start_time"]) - start_time(&audio["start_time"])).abs() <= 0.1);
    let duration = |v: &Value| v.as_str().unwrap().parse::<f64>().unwrap();
    assert!((duration(&probe["format"]["duration"]) - (800 - start) as f64 / 1000.0).abs() <= 0.1);
    assert!((duration(&video["duration"]) - duration(&audio["duration"])).abs() <= 0.1);
    let context = independent_film(native, f.root.path(), generation, start);
    for time in (start..800).step_by(100) {
        let rgba = expected(generation, time);
        let converted = native.authored_plate(f.root.path(), rgba.as_chunks::<4>().0);
        let decoded = native.decode(
            path,
            false,
            Some(&format!("{}", (time - start) as f64 / 1000.0)),
        );
        assert!(
            ssim(
                &decoded,
                &native.decode(
                    &context,
                    false,
                    Some(&format!("{}", (time - start) as f64 / 1000.0))
                )
            ) >= 0.99,
            "independent encoded context"
        );
        assert!(
            ssim(&decoded, &converted) >= 0.99,
            "{generation}/{start}/{time}"
        );
    }
    let actual = native.decode(path, true, None);
    let reference = native.decode(
        &audio_reference(native, f.root.path(), start, false),
        true,
        None,
    );
    let rms = aligned_stereo(&actual, &reference);
    assert!(
        rms.iter().all(|v| *v <= 0.0001),
        "stereo original-clock RMS {rms:?}"
    );
    assert!((actual.len() as f64 / 8.0 / 48000.0 - (800 - start) as f64 / 1000.0).abs() <= 0.1);
}
#[test]
fn complete_hero_stills_movies_audio_draft_commit_undo_redo_reopen_and_repeat() {
    let Some(native) = Native::configured() else {
        eprintln!("SKIP: optional native hero tools not configured");
        return;
    };
    let f = Fixture::new(true);
    let (renderer, captured) = native.capturing_renderer(f.root.path());
    let baseline = f.core.get_project(&f.id).unwrap();
    let stable = inventory(&f.dir());
    for time in (0..800).step_by(100) {
        let a = still(
            &native, &renderer, &f, &captured, &baseline, "baseline", time,
        );
        let b = still(
            &native, &renderer, &f, &captured, &baseline, "baseline", time,
        );
        assert_eq!(a, b);
    }
    let baseline_cold = movies(&native, &renderer, &f, &baseline, "baseline", "baseline");
    assert_eq!(
        baseline_cold,
        movies(
            &native,
            &renderer,
            &f,
            &baseline,
            "baseline",
            "warm-baseline"
        )
    );
    let draft = f
        .core
        .create_draft(&f.id, f.rev(), vec![f.reorder()], None)
        .unwrap();
    assert_eq!(
        serde_json::to_value(f.core.get_project(&f.id).unwrap()).unwrap(),
        serde_json::to_value(&baseline).unwrap()
    );
    for (path, bytes) in stable {
        if path.is_dir() {
            assert_eq!(bytes, b"directory");
        } else {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
    let candidate = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
    for time in (0..800).step_by(100) {
        still(
            &native, &renderer, &f, &captured, &candidate, "reverse", time,
        );
    }
    let draft_media = movies(&native, &renderer, &f, &candidate, "reverse", "draft");
    f.core.commit_draft(&f.id, &draft.id, f.rev()).unwrap();
    f.assert_recipe(true);
    let committed = f.core.get_project(&f.id).unwrap();
    let committed_media = movies(&native, &renderer, &f, &committed, "reverse", "committed");
    assert_eq!(draft_media, committed_media);
    f.core.undo(&f.id, f.rev()).unwrap();
    f.assert_recipe(false);
    still(
        &native,
        &renderer,
        &f,
        &captured,
        &f.core.get_project(&f.id).unwrap(),
        "baseline",
        400,
    );
    f.core.redo(&f.id, f.rev()).unwrap();
    f.assert_recipe(true);
    still(
        &native,
        &renderer,
        &f,
        &captured,
        &f.core.get_project(&f.id).unwrap(),
        "reverse",
        400,
    );
    let media = f.root.path().join("media");
    let fresh = opencut_editor_core::EditorCore::new(
        opencut_editor_core::PathPolicy::new(
            f.root.path().join("projects"),
            [&media],
            f.root.path().join("exports"),
        )
        .unwrap(),
    );
    let reopened = fresh.get_project(&f.id).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(f.core.get_project(&f.id).unwrap()).unwrap()
    );
    still(&native, &renderer, &f, &captured, &reopened, "reverse", 400);
    let cold_renderer = Renderer::new(
        f.root
            .path()
            .join(format!("ffmpeg-capture{}", std::env::consts::EXE_SUFFIX)),
        &native.ffprobe,
        Some(native.font.clone()),
    );
    assert_eq!(
        committed_media,
        movies(
            &native,
            &cold_renderer,
            &f,
            &reopened,
            "reverse",
            "cold-reopened"
        )
    );
    let correct = native.decode(
        &audio_reference(&native, f.root.path(), 200, false),
        true,
        None,
    );
    let restart = native.decode(
        &audio_reference(&native, f.root.path(), 200, true),
        true,
        None,
    );
    assert!(
        aligned_stereo(&restart, &correct)[2] > 0.0001,
        "every allowed offset rejects partial clock restart"
    );
}
#[test]
fn every_feature_and_noncommuting_order_has_its_frozen_native_discriminator() {
    let Some(native) = Native::configured() else {
        eprintln!("SKIP: optional native hero tools not configured");
        return;
    };
    let references = tempfile::tempdir().unwrap();
    let generated = references.path().join("independent-references");
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/create-masked-hero-reveal-references.py");
    let result = std::process::Command::new(if cfg!(windows) { "python" } else { "python3" })
        .arg(script)
        .arg("--output")
        .arg(&generated)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for w in catalog()["plates"]["witnesses"].as_array().unwrap() {
        let f = Fixture::new(true);
        let (renderer, captured) = native.capturing_renderer(f.root.path());
        let variant = w["variant"].as_str().unwrap();
        let hero = &f.ids["hero"];
        let owner = &f.ids["owner"];
        let edit = match variant {
            "no-mask" => {
                f.core.edit_batch(&f.id,f.rev(),vec![op(json!({"operation":"set_animation_channels","itemId":hero,"animationChannels":[]})),op(json!({"operation":"update_item","itemId":hero,"masks":[]}))]).unwrap();
                None
            }
            "no-matte" => Some(op(
                json!({"operation":"update_item","itemId":hero,"matte":null}),
            )),
            "no-clip" => Some(op(
                json!({"operation":"update_item","itemId":owner,"clip":null}),
            )),
            "reverse" => Some(f.reorder()),
            _ => {
                let role = if matches!(variant, "no-flash" | "no-particles") {
                    "owner"
                } else {
                    "hero"
                };
                let removed = match variant {
                    "no-glow" => "glow",
                    "no-tint" => "color_tint",
                    "no-flash" => "screen_flash",
                    "no-particles" => "particle_overlay",
                    _ => panic!("unknown witness"),
                };
                let effects: Vec<_> = catalog()["roles"][role]["effects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["type"] != removed)
                    .cloned()
                    .collect();
                Some(op(
                    json!({"operation":"update_item","itemId":f.ids[role],"effects":effects}),
                ))
            }
        };
        if let Some(edit) = edit {
            f.core.edit(&f.id, f.rev(), edit).unwrap();
        }
        renderer
            .render_preview(
                &f.core.get_project(&f.id).unwrap(),
                &f.dir(),
                w["atMs"].as_u64().unwrap(),
            )
            .unwrap();
        let actual = raw_pam(&captured);
        let independent = std::fs::read(
            generated.join(format!("{variant}-{:04}.rgba", w["atMs"].as_u64().unwrap())),
        )
        .unwrap();
        close_bytes(&actual, &independent, "complete independent counterfactual");
        if let Some(root) = std::env::var_os("OPENCUT_HERO_DIAGNOSTICS") {
            let root = PathBuf::from(root);
            std::fs::create_dir_all(&root).unwrap();
            std::fs::copy(&captured, root.join(format!("{variant}-actual.pam"))).unwrap();
            std::fs::write(
                root.join(format!("{variant}-project.json")),
                serde_json::to_vec_pretty(&f.core.get_project(&f.id).unwrap()).unwrap(),
            )
            .unwrap();
        }
        let offset =
            (w["pixel"][1].as_u64().unwrap() * 64 + w["pixel"][0].as_u64().unwrap()) as usize * 4;
        let expected: Vec<u8> = serde_json::from_value(w["counterfactual"].clone()).unwrap();
        close_bytes(&actual[offset..offset + 4], &expected, variant);
    }
}
#[test]
fn canonical_positive_hero_path_dependency_and_runtime_failures_preserve_sentinel_and_complete_inventory()
 {
    let Some(native) = Native::configured() else {
        eprintln!("SKIP: optional native hero dependencies not configured");
        return;
    };
    let f = Fixture::new(true);
    let exports = f.root.path().join("exports");
    std::fs::create_dir_all(&exports).unwrap();
    let destination = exports.join("existing.mp4");
    std::fs::write(&destination, b"existing destination sentinel").unwrap();
    let missing = f.root.path().join("unavailable-ffmpeg");
    let renderer = Renderer::new(&missing, &native.ffprobe, Some(native.font.clone()));
    let baseline = inventory(f.root.path());
    for (path, code) in [
        (
            PathBuf::from("../existing.mp4"),
            opencut_editor_core::ErrorCode::PathTraversal,
        ),
        (
            f.root.path().join("disallowed.mp4"),
            opencut_editor_core::ErrorCode::PathNotAllowed,
        ),
    ] {
        let error = f.core.paths().export_path(path).unwrap_err();
        assert_eq!(error.code, code);
        assert!(!error.retryable);
        assert_eq!(inventory(f.root.path()), baseline);
    }
    let error = renderer
        .export_video(
            &f.core.get_project(&f.id).unwrap(),
            &f.dir(),
            ExportOptions {
                output: &destination,
                width: 64,
                height: 64,
                overwrite: true,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        opencut_editor_core::ErrorCode::DependencyUnavailable
    );
    assert!(!error.retryable);
    assert_eq!(inventory(f.root.path()), baseline);
    // Forward real codec-readiness probes, then fail the actually launched admitted encoder.
    let diagnostic = tempfile::tempdir().unwrap();
    let marker = diagnostic.path().join("encoder-launched");
    let source = diagnostic.path().join("fail.rs");
    let executable = diagnostic
        .path()
        .join(format!("fail-encoder{}", std::env::consts::EXE_SUFFIX));
    let program = format!(
        r#"use std::process::{{Command,Stdio}};
fn main(){{let args:Vec<_>=std::env::args_os().skip(1).collect();if !args.iter().any(|a|a=="-progress"){{let status=Command::new({:?}).args(args).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).status().unwrap();std::process::exit(status.code().unwrap_or(1));}}std::fs::write({:?},b"admitted encoder executed").unwrap();eprintln!("injected hero runtime encoder failure");std::process::exit(42);}}"#,
        native.ffmpeg.to_str().unwrap(),
        marker.to_str().unwrap()
    );
    std::fs::write(&source, program).unwrap();
    let result =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition", "2024", "--crate-name", "fail_encoder"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let renderer = Renderer::new(&executable, &native.ffprobe, Some(native.font.clone()));
    let error = renderer
        .export_video(
            &f.core.get_project(&f.id).unwrap(),
            &f.dir(),
            ExportOptions {
                output: &destination,
                width: 64,
                height: 64,
                overwrite: true,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, opencut_editor_core::ErrorCode::FfmpegFailed);
    assert!(!error.retryable);
    assert_eq!(error.ffmpeg_exit_code, Some(42));
    assert!(
        marker.exists(),
        "the admitted encoder process actually executed"
    );
    assert_eq!(
        std::fs::read(&destination).unwrap(),
        b"existing destination sentinel"
    );
    assert_eq!(
        inventory(f.root.path()),
        baseline,
        "no temporary output/workspace or project/history/draft/resource writes"
    );
}

#[test]
fn semantic_audio_and_video_variants_preserve_original_hero_oracles_through_drafts() {
    let Some(native) = Native::configured() else {
        eprintln!("SKIP: optional native hero tools not configured");
        return;
    };
    for video in [false, true] {
        let mut f = Fixture::new(true);
        f.root.disable_cleanup(true);
        eprintln!(
            "preserved semantic native fixture: {}",
            f.root.path().display()
        );
        let (renderer, captured) = native.capturing_renderer(f.root.path());
        let baseline = f.core.get_project(&f.id).unwrap();
        let old = baseline.tracks[2].items[0].clone();
        let opencut_editor_core::TimelineItem::Media(original) = &old else {
            panic!("expected original audio")
        };
        let asset_id = if video {
            let movie = f.root.path().join("media/audio-bearing-video.mkv");
            let wav = f.root.path().join("media/source.wav");
            let result = std::process::Command::new(&native.ffmpeg)
                .args([
                    "-v",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=yellow:s=64x64:r=10:d=0.8",
                    "-i",
                ])
                .arg(&wav)
                .args(["-c:v", "ffv1", "-c:a", "pcm_s16le", "-shortest", "-y"])
                .arg(&movie)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            f.core
                .import_asset(
                    &f.id,
                    f.rev(),
                    movie,
                    opencut_editor_core::MediaType::Video,
                    opencut_editor_core::MediaProbeFacts {
                        duration_ms: Some(800),
                        has_audio: true,
                        has_video: true,
                        audio_codec: Some("pcm_s16le".into()),
                        audio_channels: Some(2),
                        audio_sample_rate_hz: Some(48000),
                        ..Default::default()
                    },
                )
                .unwrap()
                .changed_ids[0]
                .clone()
        } else {
            original.asset_id.clone()
        };
        f.core
            .edit(
                &f.id,
                f.rev(),
                op(
                    json!({"operation":"sound_event_register","event":"hero_audio",
            "variantAssetIds":[asset_id],"defaultGainDb":-6,"busId":"sfx","variantSeed":0}),
                ),
            )
            .unwrap();
        let draft = f.core.create_draft(&f.id,f.rev(),vec![
            op(json!({"operation":"delete_item","itemId":original.id})),
            op(json!({"operation":"timeline_add_audio_event","scope":"root","trackId":baseline.tracks[2].id,
                "event":"hero_audio","at":{"type":"milliseconds","valueMs":0},"durationMs":800,"gainDb":6}))
        ],None).unwrap();
        let candidate = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
        for time in (0..800).step_by(100) {
            still(
                &native, &renderer, &f, &captured, &candidate, "baseline", time,
            );
        }
        let rendered = movies(
            &native,
            &renderer,
            &f,
            &candidate,
            "baseline",
            if video {
                "event-video-draft"
            } else {
                "event-audio-draft"
            },
        );
        f.core.commit_draft(&f.id, &draft.id, f.rev()).unwrap();
        let committed = f.core.get_project(&f.id).unwrap();
        assert_eq!(
            rendered,
            movies(
                &native,
                &renderer,
                &f,
                &committed,
                "baseline",
                if video {
                    "event-video-committed"
                } else {
                    "event-audio-committed"
                }
            )
        );
        f.core.undo(&f.id, f.rev()).unwrap();
        still(
            &native,
            &renderer,
            &f,
            &captured,
            &f.core.get_project(&f.id).unwrap(),
            "baseline",
            400,
        );
        f.core.redo(&f.id, f.rev()).unwrap();
        let reopened = opencut_editor_core::EditorCore::new(f.core.paths().clone())
            .get_project(&f.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(f.core.get_project(&f.id).unwrap()).unwrap()
        );
        assert_eq!(reopened.revision, committed.revision + 2);
        assert_eq!(
            serde_json::to_value(&reopened.tracks).unwrap(),
            serde_json::to_value(&committed.tracks).unwrap()
        );
        still(
            &native, &renderer, &f, &captured, &reopened, "baseline", 400,
        );
        let existing = reopened.tracks[2].items[0].id();
        let gained_draft = f.core.create_draft(&f.id, f.rev(), vec![
            op(json!({"operation":"delete_item","itemId":existing})),
            op(json!({"operation":"timeline_add_audio_event","scope":"root","trackId":baseline.tracks[2].id,
                "event":"hero_audio","at":{"type":"milliseconds","valueMs":0},"durationMs":800,"gainDb":-3}))
        ], None).unwrap();
        let gained = f
            .core
            .get_draft_state(&f.id, &gained_draft.id)
            .unwrap()
            .project;
        for start in [0, 200] {
            let artifact = renderer
                .render_preview_range(
                    &gained,
                    &f.dir(),
                    PreviewRangeOptions {
                        start_ms: start,
                        end_ms: 800,
                        width: 64,
                        height: 64,
                        fps: 10,
                        include_audio: true,
                    },
                    |_| {},
                )
                .unwrap();
            // Independent -9 dB reference uses the unchanged six-decimal native
            // graph precision; sub-quantum input differences alter AAC coding.
            let reference_path = f.root.path().join(format!("independent-gain-{start}.m4a"));
            let result = std::process::Command::new(&native.ffmpeg)
                .args(["-v", "error", "-nostdin", "-i"])
                .arg(media::media_reference("source.wav"))
                .args([
                    "-af",
                    "volume=0.354813",
                    "-ss",
                    &format!("{}", start as f64 / 1000.0),
                    "-t",
                    &format!("{}", (800 - start) as f64 / 1000.0),
                    "-c:a",
                    "aac",
                    "-y",
                ])
                .arg(&reference_path)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let expected_audio = native.decode(&reference_path, true, None);
            let actual_audio = native.decode(&f.dir().join(artifact.relative_path), true, None);
            let rms = aligned_stereo(&actual_audio, &expected_audio);
            assert!(
                rms.iter().all(|v| *v <= 0.0001),
                "independent -9dB stereo RMS {rms:?}"
            );
            if start == 0 {
                let output = f.root.path().join("exports/hero-event-nonzero-gain.mp4");
                renderer
                    .export_video(
                        &gained,
                        &f.dir(),
                        ExportOptions {
                            output: &output,
                            width: 64,
                            height: 64,
                            overwrite: false,
                        },
                        |_| {},
                    )
                    .unwrap();
                let rms = aligned_stereo(&native.decode(&output, true, None), &expected_audio);
                assert!(
                    rms.iter().all(|v| *v <= 0.0001),
                    "export independent -9dB stereo RMS {rms:?}"
                );
            }
        }
    }
}
