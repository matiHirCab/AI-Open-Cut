//! Every request executes the default actual headless binary in a fresh process.
#[path = "../../../crates/editor-core/tests/support/masked_hero_authoring.rs"]
mod authoring;
#[path = "../../../crates/editor-core/tests/support/masked_hero_media.rs"]
mod media;
use media::{
    Native, aligned_stereo, audio_reference, close_bytes, decoded_movie, independent_film,
    media_reference, raw_pam, ssim,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/masked-hero-reveal-v1.json"
    ))
    .unwrap()
}
fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(p: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in std::fs::read_dir(p).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.insert(p.clone(), b"directory".to_vec());
                visit(&p, out);
            } else if p.file_name().unwrap() != ".lock" {
                out.insert(p.clone(), std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, &mut out);
    out
}
struct Wire {
    root: tempfile::TempDir,
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}
impl Wire {
    fn new(native: &Native) -> (Self, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        for name in ["projects", "media", "exports"] {
            std::fs::create_dir(root.path().join(name)).unwrap();
        }
        std::fs::copy(
            media_reference("source.wav"),
            root.path().join("media/source.wav"),
        )
        .unwrap();
        let (_, captured) = native.capturing_renderer(root.path());
        let ffmpeg = root
            .path()
            .join(format!("ffmpeg-capture{}", std::env::consts::EXE_SUFFIX));
        (
            Self {
                root,
                ffmpeg,
                ffprobe: native.ffprobe.clone(),
            },
            captured,
        )
    }
    fn request(&self, input: Value) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_opencut-headless"))
            .env("OPENCUT_PROJECTS_DIR", self.root.path().join("projects"))
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", self.root.path().join("media"))
            .env("OPENCUT_EXPORTS_DIR", self.root.path().join("exports"))
            .env("OPENCUT_FFMPEG_PATH", &self.ffmpeg)
            .env("OPENCUT_FFPROBE_PATH", &self.ffprobe)
            .env_remove("OPENCUT_DEFAULT_FONT_PATH")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{}", input).unwrap();
        let output = child.wait_with_output().unwrap();
        let event: Value = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|e| matches!(e["type"].as_str(), Some("result" | "error")))
            .expect("actual headless terminal event");
        if event["type"] == "result" {
            assert!(output.status.success(), "{event}");
        } else {
            assert!(!output.status.success(), "{event}");
        }
        event
    }
    fn ok(&self, input: Value) -> Value {
        let event = self.request(input);
        assert_eq!(event["type"], "result", "{event}");
        event["result"].clone()
    }
    fn state(&self, id: &Value) -> Value {
        self.ok(json!({"operation":"get_state","projectId":id}))
    }
    fn rev(&self, id: &Value) -> u64 {
        self.state(id)["project"]["revision"].as_u64().unwrap()
    }
    fn edit(&self, id: &Value, operation: Value) -> Value {
        self.ok(json!({"operation":"edit","projectId":id,"expectedRevision":self.rev(id),"edit":operation}))
    }
}
fn assert_movie(native: &Native, root: &Path, path: &Path, generation: &str, start: u64) {
    let probe = native.probe(path);
    let video = probe["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["codec_type"] == "video")
        .unwrap();
    let audio = probe["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["codec_type"] == "audio")
        .unwrap();
    assert_eq!(video["nb_frames"], ((800 - start) / 100).to_string());
    assert_eq!(video["width"], 64);
    assert_eq!(video["height"], 64);
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
    let context = independent_film(native, root, generation, start);
    for time in (start..800).step_by(100) {
        let rgba = std::fs::read(media_reference(&format!("{generation}-{time:04}.rgba"))).unwrap();
        let expected = native.authored_plate(root, rgba.as_chunks::<4>().0);
        let actual = native.decode(
            path,
            false,
            Some(&format!("{}", (time - start) as f64 / 1000.0)),
        );
        assert!(
            ssim(
                &actual,
                &native.decode(
                    &context,
                    false,
                    Some(&format!("{}", (time - start) as f64 / 1000.0))
                )
            ) >= 0.99
        );
        assert!(
            ssim(&actual, &expected) >= 0.99,
            "{generation}/{start}/{time}"
        );
    }
    let actual = native.decode(path, true, None);
    let expected = native.decode(&audio_reference(native, root, start, false), true, None);
    assert!(
        aligned_stereo(&actual, &expected)
            .iter()
            .all(|v| *v <= 0.0001)
    );
}
#[test]
fn fresh_default_headless_exact_standalone_alias_hero_native_lifecycle_and_rollback() {
    let Some(native) = Native::configured() else {
        eprintln!("SKIP: optional native hero dependencies not configured");
        return;
    };
    let c = catalog();
    for batch in [false, true] {
        let (wire, captured) = Wire::new(&native);
        let id=wire.ok(json!({"operation":"create_project","name":"Masked hero reveal v1","settings":c["settings"]}))["projectId"].clone();
        let initial = wire.state(&id);
        let tracks = &initial["project"]["tracks"];
        assert_eq!(tracks[1]["name"], "Overlay");
        assert_eq!(tracks[2]["name"], "Audio");
        let imported=wire.ok(json!({"operation":"import_asset","projectId":id,"expectedRevision":0,"path":wire.root.path().join("media/source.wav"),"mediaType":"audio"}));
        let operations: Vec<Value> = serde_json::from_str(
            &serde_json::to_string(&c["operationTranscript"]["aliasBatch"])
                .unwrap()
                .replace("{{overlayTrackId}}", tracks[1]["id"].as_str().unwrap())
                .replace("{{audioTrackId}}", tracks[2]["id"].as_str().unwrap())
                .replace(
                    "{{audioAssetId}}",
                    imported["changedIds"][0].as_str().unwrap(),
                ),
        )
        .unwrap();
        let mut aliases = BTreeMap::<String, String>::new();
        if batch {
            aliases=serde_json::from_value(wire.ok(json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":operations}))["aliases"].clone()).unwrap();
        } else {
            for mut op in operations {
                let alias = op.as_object_mut().unwrap().remove("resultAlias");
                let mut text = serde_json::to_string(&op).unwrap();
                for (role, id) in &aliases {
                    text = text.replace(&format!("\"@{role}\""), &format!("\"{id}\""));
                }
                let write = wire.edit(&id, serde_json::from_str(&text).unwrap());
                if let Some(alias) = alias {
                    aliases.insert(
                        alias.as_str().unwrap().into(),
                        write["changedIds"][0].as_str().unwrap().into(),
                    );
                }
            }
        }
        let dir = wire.root.path().join("projects").join(id.as_str().unwrap());
        let baseline = wire.state(&id);
        assert_eq!(baseline["project"]["schemaVersion"], 37);
        assert_eq!(baseline["durationMs"], 800);
        assert_eq!(aliases.len(), 5);
        authoring::assert_recipe(
            &serde_json::from_value(baseline["project"].clone()).unwrap(),
            &aliases,
            &c,
            false,
        );
        let frame = |generation: &str, time: u64, draft: Option<&Value>| {
            let input = if let Some(draft) = draft {
                json!({"operation":"render_draft_preview","projectId":id,"draftId":draft,"timeMs":time})
            } else {
                json!({"operation":"render_preview","projectId":id,"expectedRevision":wire.rev(&id),"timeMs":time})
            };
            let artifact = wire.ok(input);
            let rgba =
                std::fs::read(media_reference(&format!("{generation}-{time:04}.rgba"))).unwrap();
            close_bytes(&raw_pam(&captured), &rgba, "fresh headless complete PAM");
            let pixels = native.decode(
                &dir.join(artifact["relativePath"].as_str().unwrap()),
                false,
                None,
            );
            let expected = native.authored_plate(wire.root.path(), rgba.as_chunks::<4>().0);
            close_bytes(&pixels, &expected, "fresh headless independent PNG");
            pixels
        };
        let movies = |generation: &str, label: &str| {
            let mut signatures = BTreeMap::new();
            let mut full = None;
            let mut partial = None;
            for start in [0, 200] {
                let artifact=wire.ok(json!({"operation":"render_preview_range","projectId":id,"expectedRevision":wire.rev(&id),"startMs":start,"endMs":800,"width":64,"height":64,"fps":10,"includeAudio":true}));
                let path = dir.join(artifact["relativePath"].as_str().unwrap());
                assert_movie(&native, wire.root.path(), &path, generation, start);
                signatures.insert(format!("range-{start}"), decoded_movie(&native, &path));
                if start == 0 {
                    full = Some(path);
                } else {
                    partial = Some(path);
                }
            }
            let name = format!("hero-{generation}-{batch}-{label}.mp4");
            wire.ok(json!({"operation":"export_video","projectId":id,"expectedRevision":wire.rev(&id),"relativePath":name,"width":64,"height":64,"overwrite":false}));
            let output = wire.root.path().join("exports").join(name);
            assert_movie(&native, wire.root.path(), &output, generation, 0);
            signatures.insert("export".into(), decoded_movie(&native, &output));
            for time in (0..800).step_by(100) {
                let seek = format!("{}", time as f64 / 1000.0);
                assert!(
                    ssim(
                        &native.decode(full.as_ref().unwrap(), false, Some(&seek)),
                        &native.decode(&output, false, Some(&seek))
                    ) >= 0.99
                );
            }
            for time in (200..800).step_by(100) {
                let original = format!("{}", time as f64 / 1000.0);
                let local = format!("{}", (time - 200) as f64 / 1000.0);
                let frame = native.decode(partial.as_ref().unwrap(), false, Some(&local));
                assert!(
                    ssim(
                        &frame,
                        &native.decode(full.as_ref().unwrap(), false, Some(&original))
                    ) >= 0.99
                );
                assert!(ssim(&frame, &native.decode(&output, false, Some(&original))) >= 0.99);
            }
            signatures
        };
        for time in (0..800).step_by(100) {
            assert_eq!(frame("baseline", time, None), frame("baseline", time, None));
        }
        let cold = movies("baseline", "cold");
        assert_eq!(cold, movies("baseline", "warm"));
        let hero = &aliases["hero"];
        let mut effects = c["roles"]["hero"]["effects"].clone();
        effects.as_array_mut().unwrap().swap(1, 2);
        let reorder = json!({"operation":"update_item","itemId":hero,"effects":effects});
        let stable = inventory(&dir);
        let draft=wire.ok(json!({"operation":"create_draft","projectId":id,"expectedRevision":wire.rev(&id),"operations":[reorder],"label":"Reverse hero"}));
        assert_eq!(wire.state(&id), baseline);
        for (path, bytes) in stable {
            if path.is_dir() {
                assert_eq!(bytes, b"directory");
            } else {
                assert_eq!(std::fs::read(path).unwrap(), bytes);
            }
        }
        for time in (0..800).step_by(100) {
            frame("reverse", time, Some(&draft["id"]));
        }
        let stable = inventory(&dir);
        for (input, code, retryable) in [
            (
                json!({"operation":"edit","projectId":id,"expectedRevision":wire.rev(&id),"edit":{"operation":"update_item","itemId":hero,"geometry":{"type":"rectangle","width":0,"height":40}}}),
                "INVALID_ARGUMENT",
                false,
            ),
            (
                json!({"operation":"edit","projectId":id,"expectedRevision":wire.rev(&id)-1,"edit":reorder}),
                "REVISION_CONFLICT",
                true,
            ),
            (
                json!({"operation":"edit_batch","projectId":id,"expectedRevision":wire.rev(&id),"operations":[reorder,{"operation":"update_item","itemId":hero,"matte":{"sourceId":"missing","channel":"alpha"}}]}),
                "ITEM_NOT_FOUND",
                false,
            ),
            (
                json!({"operation":"update_draft","projectId":id,"draftId":draft["id"],"expectedRevision":wire.rev(&id),"operations":[{"operation":"update_item","itemId":hero,"geometry":{"type":"rectangle","width":0,"height":40}}]}),
                "INVALID_ARGUMENT",
                false,
            ),
        ] {
            let event = wire.request(input);
            assert_eq!(event["error"]["code"], code, "{event}");
            assert_eq!(event["error"]["retryable"], retryable);
            assert_eq!(inventory(&dir), stable);
        }
        wire.ok(json!({"operation":"commit_draft","projectId":id,"draftId":draft["id"],"expectedRevision":wire.rev(&id)}));
        let candidate_media = movies("reverse", "committed");
        let reversed = frame("reverse", 400, None);
        wire.ok(json!({"operation":"undo","projectId":id,"expectedRevision":wire.rev(&id)}));
        frame("baseline", 400, None);
        wire.ok(json!({"operation":"redo","projectId":id,"expectedRevision":wire.rev(&id)}));
        assert_eq!(frame("reverse", 400, None), reversed);
        let state = wire.state(&id);
        authoring::assert_recipe(
            &serde_json::from_value(state["project"].clone()).unwrap(),
            &aliases,
            &c,
            true,
        );
        assert_eq!(wire.state(&id), state);
        assert_eq!(frame("reverse", 400, None), reversed);
        assert_eq!(candidate_media, movies("reverse", "reopened"));
    }
}
