#[path = "support/temporal_fixture.rs"]
mod fixture;
use fixture::{operation as op, recipe};
use opencut_editor_core as core_facade;
use opencut_editor_core::{
    BatchEditOperation, ErrorCode, ExportOptions, PreviewRangeOptions, Project, Renderer,
    TimelineItem,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[test]
fn temporal_fixture_manifest_matches_reviewed_recipe() {
    let m: Value = serde_json::from_str(include_str!("fixtures/temporal/recipe.json")).unwrap();
    assert_eq!(m["identity"], "issue48-independent-temporal-v1");
    assert_eq!(
        m["canvas"],
        json!({"width":64,"height":64,"fps":10,"durationMs":1300})
    );
    assert_eq!(m["native"]["frameTimesMs"], json!([300, 500, 1000, 1200]));
    assert_eq!(m["native"]["rangeMs"], json!([300, 1300]));
    assert_eq!(m["native"]["fractionalRangeMs"], json!([713, 913]));
    assert_eq!(m["native"]["actualRenderCount"], 39);
    assert_eq!(m["native"]["maximumRenderCount"], 40);
    assert_eq!(
        m["native"]["productionRenderBreakdown"],
        json!({"pngReferences":10,"additionalLossless713Actuals":2,"encodedSequenceReferences":8,"existingActuals":19})
    );
    assert_eq!(m["native"]["rgbMseExclusiveMaximum"], 20);
    assert_eq!(m["native"]["centroidDistanceExclusiveMaximum"], 0.4);
    let a = recipe::family_a();
    for j in 0..6 {
        assert_eq!(
            a[0]["items"][j]["transform"]["positionY"].as_f64().unwrap(),
            m["familyA"]["lanesY"][j].as_f64().unwrap()
        );
        assert_eq!(a[0]["items"][j]["width"], m["familyA"]["rectangleSize"][0]);
    }
    assert_eq!(
        recipe::channel(0)["keyframes"][0]["curve"],
        json!({"type":"spring","mass":1,"stiffness":100,"damping":2,"initialVelocity":0})
    );
    assert_eq!(
        recipe::channel(2)["loop"]["iterations"],
        m["familyA"]["finiteIterations"]
    );
    let leaf = recipe::family_b("leaf", "outer").0;
    assert_eq!(
        leaf[0]["items"][1]["animationChannels"][0]["clock"],
        m["familyB"]["retainedClock"]
    );
    assert_eq!(
        leaf[0]["items"][1]["durationMs"],
        m["familyB"]["visibleLeafDurationMs"]
    );
    assert_eq!(
        m["familyB"]["sourceTimesAt713Ms"],
        json!([644.75, 569.75, 494.75])
    );
}
fn state(f: &fixture::Fixture) -> (Value, Vec<u8>, Vec<u8>) {
    let dir = f.core.paths().project_dir(&f.id).unwrap();
    (
        serde_json::to_value(f.project()).unwrap(),
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}
#[test]
fn temporal_fixture_controller_channel_history_and_atomic_failure() {
    for b in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let f = fixture::seed(root.path(), b);
        let before = state(&f);
        let id = &f.item_ids[0];
        let rev = f.project().revision;
        let change = if b {
            let p = f.project();
            let mut repeater = serde_json::to_value(p.find_item(&f.item_ids[1]).unwrap()).unwrap()
                ["repeater"]
                .clone();
            repeater["timeOffsetMs"] = json!(120);
            json!({"operation":"update_item","itemId":f.item_ids[1],"repeater":repeater})
        } else {
            let mut c = recipe::channel(0);
            c["keyframes"][0]["curve"]["damping"] = json!(4);
            json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[c]})
        };
        f.edit(change.clone());
        let edited = serde_json::to_value(f.project()).unwrap();
        assert_ne!(edited["tracks"], before.0["tracks"]);
        f.core.undo(&f.id, f.project().revision).unwrap();
        assert_eq!(
            serde_json::to_value(f.project()).unwrap()["tracks"],
            before.0["tracks"]
        );
        f.core.redo(&f.id, f.project().revision).unwrap();
        assert_eq!(
            serde_json::to_value(f.project()).unwrap()["tracks"],
            edited["tracks"]
        );
        let reopened = opencut_editor_core::EditorCore::new(f.core.paths().clone());
        assert_eq!(
            serde_json::to_value(reopened.get_project(&f.id).unwrap()).unwrap(),
            serde_json::to_value(f.project()).unwrap()
        );
        let stable = state(&f);
        assert_eq!(
            f.core
                .edit(&f.id, rev, op(change.clone()))
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        let mut missing = change.clone();
        missing["itemId"] = json!("missing");
        assert_eq!(
            f.core
                .edit(&f.id, f.project().revision, op(missing))
                .unwrap_err()
                .code,
            ErrorCode::ItemNotFound
        );
        let bad = if b {
            json!({"operation":"update_item","itemId":id,"staggerMs":60001})
        } else {
            let mut c = recipe::channel(1);
            c["keyframes"][0]["curve"]["y1"] = json!(1.5);
            json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[c]})
        };
        assert_eq!(
            f.core
                .edit(&f.id, f.project().revision, op(bad))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(state(&f), stable);
        let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([{ "operation":"add_rectangle","trackId":f.track_id,"startMs":0,"durationMs":1300,"width":5,"height":5,"color":"#ffffff","transform":{"positionX":8,"positionY":5,"scale":1,"opacity":1},"resultAlias":"temporary"},{"operation":"set_animation_channels","itemId":"@temporary","animationChannels":[{"property":"transform.position_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":8},"curve":"linear"},{"timeMs":100,"value":{"type":"scalar","value":20},"curve":"hold"}],"loop":{"mode":"repeat","iterations":2}}]}])).unwrap();
        assert_eq!(
            f.core
                .edit_batch(&f.id, f.project().revision, batch)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(state(&f), stable);
        f.edit(json!({"operation":"update_track","trackId":f.track_id,"locked":true}));
        let locked = state(&f);
        assert_eq!(
            f.core
                .edit(&f.id, f.project().revision, op(change))
                .unwrap_err()
                .code,
            ErrorCode::TrackLocked
        );
        assert_eq!(state(&f), locked);
    }
}
fn decode(ffmpeg: &Path, path: &Path, at: u64) -> Vec<u8> {
    let o = std::process::Command::new(ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-ss",
            &format!("{:.6}", at as f64 / 1000.0),
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
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(o.stdout.len(), 64 * 64 * 3, "{} at{at}", path.display());
    o.stdout
}
fn expected_x(b: bool, j: usize, at: u64, candidate: bool) -> f64 {
    if b {
        recipe::triangle(0.75 * at as f64 + 110.0 - if candidate { 90.0 } else { 75.0 } * j as f64)
    } else {
        let x = recipe::expected_a(j, at as f64);
        if candidate && j == 0 {
            8.0 + (x - 8.0) * 16.0 / 12.0
        } else {
            x
        }
    }
}
fn active(b: bool, j: usize, at: u64, candidate: bool) -> bool {
    at < 1300
        && (!b || (at as f64) >= (140.0 + if candidate { 90.0 } else { 75.0 } * j as f64) / 0.75)
}
fn static_control(mut p: Project, b: bool, at: u64, candidate: bool) -> Project {
    p.components.clear();
    let items: Vec<TimelineItem> = (0..if b { 3 } else { 6 })
        .map(|j| {
            let mut r = recipe::rectangle(
                &format!("oracle{j}"),
                if b { 3 } else { j },
                if b {
                    8.0 + 16.0 * j as f64
                } else {
                    5.0 + 9.0 * j as f64
                },
            );
            r["stackOrder"] = json!(j);
            r["transform"]["positionX"] = json!(expected_x(b, j, at, candidate));
            r["hidden"] = json!(!active(b, j, at, candidate));
            r["animationChannels"] = json!([]);
            r["effects"] = json!([{ "id":"oracle-workspace","type":"vignette","amount":0}]);
            serde_json::from_value(r).unwrap()
        })
        .collect();
    p.tracks.iter_mut().for_each(|t| t.items.clear());
    p.tracks[1].items = items;
    p
}
fn static_sequence(p: Project, b: bool, start: u64, end: u64, candidate: bool) -> Project {
    let mut result = static_control(p, b, start, candidate);
    let mut items = vec![];
    for at in (start..end).step_by(100) {
        let control = static_control(result.clone(), b, at, candidate);
        for (j, item) in control.tracks[1].items.iter().enumerate() {
            let mut value = serde_json::to_value(item).unwrap();
            value["id"] = json!(format!("oracle-{at}-{j}"));
            value["startMs"] = json!(at);
            value["durationMs"] = json!((end - at).min(100));
            value["stackOrder"] = json!(items.len());
            value["effects"] = json!([]);
            let mut item: TimelineItem = serde_json::from_value(value).unwrap();
            item.visual_properties_mut().transform2d = Some(core_facade::Transform2D {
                position: core_facade::TransformPosition {
                    x: expected_x(b, j, at, candidate),
                    y: (if b { 8 + 16 * j } else { 5 + 9 * j }) as f64,
                    unit: core_facade::PositionUnit::Pixels,
                },
                ..Default::default()
            });
            let roundtrip: TimelineItem =
                serde_json::from_value(serde_json::to_value(&item).unwrap()).unwrap();
            let visual = roundtrip.visual_properties();
            let transform = visual
                .transform2d
                .expect("encoded static reference must retain explicit affine geometry");
            assert_eq!(transform.position.x, expected_x(b, j, at, candidate));
            assert_eq!(
                transform.position.y,
                (if b { 8 + 16 * j } else { 5 + 9 * j }) as f64
            );
            assert!(
                visual.animation_channels.is_empty()
                    && roundtrip.keyframes().is_empty()
                    && visual.effects.is_empty()
            );
            assert!(visual.crop.is_none() && visual.motion_blur.is_none());
            items.push(roundtrip);
        }
    }
    // Match the source project's duration without adding visible geometry.
    let mut placeholder = recipe::rectangle("oracle-duration", 0, 0.0);
    placeholder["startMs"] = json!(1200);
    placeholder["durationMs"] = json!(100);
    placeholder["hidden"] = json!(true);
    placeholder["stackOrder"] = json!(items.len());
    placeholder["animationChannels"] = json!([]);
    items.push(serde_json::from_value(placeholder).unwrap());
    result.tracks[1].items = items;
    result
}
#[test]
fn temporal_fixture_static_references_validate_all_families_candidates_and_grids() {
    for b in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let fixture = fixture::seed(root.path(), b);
        let project = fixture.project();
        let validate = |reference: &Project| {
            // Reopen through the canonical production validator without invoking rendering.
            let dir = fixture.core.paths().project_dir(&fixture.id).unwrap();
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(reference).unwrap(),
            )
            .unwrap();
            fixture.core.get_project(&fixture.id).unwrap();
        };
        for candidate in [false, true] {
            for at in [0, 300, 500, 713, 813, 1000, 1200] {
                let reference = static_control(project.clone(), b, at, candidate);
                validate(&reference);
                assert_eq!(reference.tracks[1].items.len(), if b { 3 } else { 6 });
                for (index, item) in reference.tracks[1].items.iter().enumerate() {
                    let value = serde_json::to_value(item).unwrap();
                    assert_eq!(value["stackOrder"], json!(index));
                }
            }
            for (start, end) in [(300, 1300), (713, 913), (0, 1300), (900, 1300)] {
                let reference = static_sequence(project.clone(), b, start, end, candidate);
                validate(&reference);
                let frames = (start..end).step_by(100).count();
                assert_eq!(
                    reference.tracks[1].items.len(),
                    frames * if b { 3 } else { 6 } + 1
                );
            }
        }
    }
}
fn encoder_settings(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let prefix = b"options: ";
    let offset = bytes
        .windows(prefix.len())
        .position(|w| w == prefix)
        .expect("native libx264 output must record its encoder settings");
    let settings = &bytes[offset..];
    let end = settings.iter().position(|b| *b == 0).unwrap();
    String::from_utf8(settings[..end].to_vec()).unwrap()
}
fn matching_profile(reference: &Path, actual: &Path, range: bool) {
    let expected = encoder_settings(reference);
    let actual = encoder_settings(actual);
    assert_eq!(
        actual, expected,
        "independent reference must match actual encoder configuration"
    );
    if range {
        assert!(
            actual.contains("crf=28.0"),
            "reviewed range profile changed: {actual}"
        );
    }
}
fn inspect_geometry(
    pixels: &[u8],
    b: bool,
    at: u64,
    candidate: bool,
    analytic: bool,
    label: &str,
) -> Vec<Option<(f64, f64)>> {
    let mut centers = vec![];
    for j in 0..if b { 3 } else { 6 } {
        let y = if b { 8 + 16 * j } else { 5 + 9 * j };
        let color = if b {
            [true, true, false]
        } else {
            [
                [true, false, false],
                [false, true, false],
                [false, false, true],
                [true, true, false],
                [true, false, true],
                [false, true, true],
            ][j]
        };
        let expected = (expected_x(b, j, at, candidate) + 2.0, y as f64 + 2.0);
        let (mut w, mut x, mut yy) = (0.0, 0.0, 0.0);
        let mut mass = [0.0; 3];
        let mut squared = 0.0;
        let mut n = 0;
        for (i, p) in pixels.as_chunks::<3>().0.iter().enumerate() {
            let row = i / 64;
            if (y - 3..y + 8).contains(&row) {
                // Isolate the authored color, removing neighboring color/chroma bleed.
                // Observe authored hue in physical linear intensity. Applying
                // the output transfer first would overweigh dim chroma bleed.
                let intensity = p.map(|byte| {
                    let v = f64::from(byte) / 255.0;
                    255.0
                        * if v <= 0.04045 {
                            v / 12.92
                        } else {
                            ((v + 0.055) / 1.055).powf(2.4)
                        }
                });
                let on = (0..3)
                    .filter(|k| color[*k])
                    .map(|k| intensity[k])
                    .fold(f64::INFINITY, f64::min);
                let off = (0..3)
                    .filter(|k| !color[*k])
                    .map(|k| intensity[k])
                    .fold(0.0, f64::max);
                let weight = (on - off).max(0.0);
                w += weight;
                x += (i % 64) as f64 * weight;
                yy += row as f64 * weight;
                for k in 0..3 {
                    if weight > 0.0 {
                        mass[k] += intensity[k];
                    }
                    squared += f64::from(p[k]).powi(2);
                }
                n += 3;
            }
        }
        if !active(b, j, at, candidate) {
            assert!(
                squared / (n as f64) < 20.0,
                "{label} lane{j} must be inactive"
            );
            centers.push(None);
            continue;
        }
        assert!(w > 1000.0, "{label} expected active authored color lane{j}");
        for k in 0..3 {
            if color[k] {
                assert!(
                    mass[k] > 1000.0,
                    "{label} lane{j} missing authored color channel{k}"
                );
            } else {
                assert!(
                    mass[k] < w / 2.0,
                    "{label} lane{j} wrong authored color channel{k}"
                );
            }
        }
        let center = (x / w, yy / w);
        if analytic {
            assert!(
                (center.0 - expected.0).abs() < 0.4 && (center.1 - expected.1).abs() < 0.4,
                "{label} lane{j} analytic center{expected:?} vs{center:?}"
            );
        }
        centers.push(Some(center));
    }
    centers
}
fn compare(
    reference: &[u8],
    actual: &[u8],
    b: bool,
    at: u64,
    candidate: bool,
    label: &str,
    analytic: bool,
) {
    let mse = reference
        .iter()
        .zip(actual)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
        .sum::<f64>()
        / reference.len() as f64;
    assert!(mse < 20.0, "{label} MSE{mse}");
    let reference = inspect_geometry(
        reference,
        b,
        at,
        candidate,
        analytic,
        &format!("{label} reference"),
    );
    let actual = inspect_geometry(
        actual,
        b,
        at,
        candidate,
        analytic,
        &format!("{label} actual"),
    );
    for (j, pair) in reference.into_iter().zip(actual).enumerate() {
        if let (Some(a), Some(b)) = pair {
            assert!(
                (a.0 - b.0).abs() < 0.4 && (a.1 - b.1).abs() < 0.4,
                "{label} lane{j} centroid parity"
            );
        }
    }
}
#[test]
fn native_temporal_fixtures_match_independent_frame_range_draft_export() {
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1"),
            "temporal native tools required"
        );
        assert_ne!(
            std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(),
            Ok("1"),
            "temporal native tools required"
        );
        return;
    };
    let ffmpeg = PathBuf::from(ffmpeg);
    let mut renders = 0;
    for b in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let f = fixture::seed(root.path(), b);
        assert_eq!(f.family_b, b);
        let p = f.project();
        let dir = f.core.paths().project_dir(&f.id).unwrap();
        let renderer = Renderer::new(
            &ffmpeg,
            PathBuf::from(&ffprobe),
            std::env::var_os("OPENCUT_TEST_FONT_PATH").map(PathBuf::from),
        );
        renderer.readiness().unwrap();
        let draft = f
            .core
            .create_draft(
                &f.id,
                p.revision,
                vec![op(if b {let mut descriptor=serde_json::to_value(p.find_item(&f.item_ids[1]).unwrap()).unwrap()["repeater"].clone();descriptor["timeOffsetMs"]=json!(120);json!({"operation":"update_item","itemId":f.item_ids[1],"repeater":descriptor})}else{let mut c=recipe::channel(0);c["keyframes"][1]["value"]["value"]=json!(24);json!({"operation":"set_animation_channels","itemId":f.item_ids[0],"animationChannels":[c]})})],
                None,
            )
            .unwrap();
        let candidate = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
        let stable = state(&f);
        let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
        let draft_bytes = std::fs::read(&draft_path).unwrap();
        assert!(!draft_bytes.is_empty());
        let mut refs = BTreeMap::new();
        for at in [300, 500, 1000, 1200] {
            let control = static_control(p.clone(), b, at, false);
            let r = renderer.render_preview(&control, &dir, at).unwrap();
            renders += 1;
            refs.insert(at, decode(&ffmpeg, &dir.join(r.relative_path), 0));
        }
        let mut candidate_refs = BTreeMap::new();
        {
            let at = 1000;
            let control = static_control(p.clone(), b, at, true);
            let r = renderer.render_preview(&control, &dir, at).unwrap();
            renders += 1;
            candidate_refs.insert(at, decode(&ffmpeg, &dir.join(r.relative_path), 0));
        }
        for at in [300, 500, 1000, 1200] {
            let r = renderer.render_preview(&p, &dir, at).unwrap();
            renders += 1;
            compare(
                &refs[&at],
                &decode(&ffmpeg, &dir.join(r.relative_path), 0),
                b,
                at,
                false,
                &format!("family{b} frame{at}"),
                true,
            );
        }
        let lossless = renderer.render_preview(&p, &dir, 713).unwrap();
        renders += 1;
        inspect_geometry(
            &decode(&ffmpeg, &dir.join(lossless.relative_path), 0),
            b,
            713,
            false,
            true,
            "actual fractional713",
        );
        let range = |project: &Project, start, end| {
            renderer
                .render_preview_range(
                    project,
                    &dir,
                    PreviewRangeOptions {
                        start_ms: start,
                        end_ms: end,
                        width: 64,
                        height: 64,
                        fps: 10,
                        include_audio: true,
                    },
                    |_| {},
                )
                .unwrap()
        };
        let reference = range(&static_sequence(p.clone(), b, 300, 1300, false), 300, 1300);
        let r = range(&p, 300, 1300);
        renders += 2;
        matching_profile(
            &dir.join(&reference.relative_path),
            &dir.join(&r.relative_path),
            true,
        );
        for at in [300, 500, 1000, 1200] {
            compare(
                &decode(&ffmpeg, &dir.join(&reference.relative_path), at - 300),
                &decode(&ffmpeg, &dir.join(&r.relative_path), at - 300),
                b,
                at,
                false,
                &format!("family{b} range{at}"),
                false,
            );
        }
        let reference = range(&static_sequence(p.clone(), b, 713, 913, false), 713, 913);
        let r = range(&p, 713, 913);
        renders += 2;
        matching_profile(
            &dir.join(&reference.relative_path),
            &dir.join(&r.relative_path),
            true,
        );
        for at in [713, 813] {
            compare(
                &decode(&ffmpeg, &dir.join(&reference.relative_path), at - 713),
                &decode(&ffmpeg, &dir.join(&r.relative_path), at - 713),
                b,
                at,
                false,
                "fractional range713/813",
                false,
            );
        }
        let reference_output = root.path().join("exports/reference.mp4");
        renderer
            .export_video(
                &static_sequence(p.clone(), b, 0, 1300, false),
                &dir,
                ExportOptions {
                    output: &reference_output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        renders += 1;
        let output = root.path().join("exports/temporal.mp4");
        renderer
            .export_video(
                &p,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        renders += 1;
        matching_profile(&reference_output, &output, false);
        for at in [300, 500, 1000, 1200] {
            compare(
                &decode(&ffmpeg, &reference_output, at),
                &decode(&ffmpeg, &output, at),
                b,
                at,
                false,
                &format!("family{b} export{at}"),
                false,
            );
        }
        let r = renderer.render_preview(&candidate, &dir, 1000).unwrap();
        renders += 1;
        compare(
            &candidate_refs[&1000],
            &decode(&ffmpeg, &dir.join(r.relative_path), 0),
            b,
            1000,
            true,
            "draft frame",
            true,
        );
        let reference = range(&static_sequence(p.clone(), b, 900, 1300, true), 900, 1300);
        let r = range(&candidate, 900, 1300);
        renders += 2;
        matching_profile(
            &dir.join(&reference.relative_path),
            &dir.join(&r.relative_path),
            true,
        );
        for at in [900, 1000, 1200] {
            compare(
                &decode(&ffmpeg, &dir.join(&reference.relative_path), at - 900),
                &decode(&ffmpeg, &dir.join(&r.relative_path), at - 900),
                b,
                at,
                true,
                "draft range",
                false,
            );
        }
        if b {
            let r = renderer.render_preview(&p, &dir, 0).unwrap();
            renders += 1;
            let pixels = decode(&ffmpeg, &dir.join(r.relative_path), 0);
            let mse =
                pixels.iter().map(|p| f64::from(*p).powi(2)).sum::<f64>() / pixels.len() as f64;
            assert!(mse < 20.0, "inactive inherited root0 MSE{mse}");
        }
        assert_eq!(state(&f), stable);
        assert_eq!(std::fs::read(&draft_path).unwrap(), draft_bytes);
        f.core.discard_draft(&f.id, &draft.id).unwrap();
    }
    assert_eq!(renders, 39);
    assert!(renders <= 40);
}

#[test]
fn temporal_fixture_authored_color_mask_rejects_absent_white_and_wrong_colors() {
    let mut image = vec![0_u8; 64 * 64 * 3];
    let colors = [
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [255, 255, 0],
        [255, 0, 255],
        [0, 255, 255],
    ];
    for (j, color) in colors.iter().enumerate() {
        let x = expected_x(false, j, 1000, false) as usize;
        for y in 5 + 9 * j..10 + 9 * j {
            for x in x..x + 5 {
                image[(y * 64 + x) * 3..(y * 64 + x) * 3 + 3].copy_from_slice(color);
            }
        }
    }
    // Pure neighbor-blue evidence lies in the yellow strip but outside its
    // authored color mask; it must not turn yellow's off-channel mass into blue.
    image[(31 * 64 + 1) * 3..(31 * 64 + 1) * 3 + 3].copy_from_slice(&[0, 0, 255]);
    assert_eq!(
        inspect_geometry(
            &image,
            false,
            1000,
            false,
            true,
            "synthetic yellow plus neighbor blue"
        )
        .len(),
        6
    );
    let x = expected_x(false, 3, 1000, false) as usize;
    for wrong in [[0, 0, 0], [255, 255, 255], [255, 0, 255]] {
        let mut wrong_image = image.clone();
        for y in 32..37 {
            for x in x..x + 5 {
                wrong_image[(y * 64 + x) * 3..(y * 64 + x) * 3 + 3].copy_from_slice(&wrong);
            }
        }
        assert!(
            std::panic::catch_unwind(|| inspect_geometry(
                &wrong_image,
                false,
                1000,
                false,
                true,
                "deliberately invalid palette"
            ))
            .is_err()
        );
    }
}
