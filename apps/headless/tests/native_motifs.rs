use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

fn request(root: &std::path::Path, input: Value) -> Value {
    let media = root.join("media");
    std::fs::create_dir_all(&media).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_opencut-headless"))
        .env("OPENCUT_PROJECTS_DIR", root.join("projects"))
        .env("OPENCUT_ALLOWED_MEDIA_DIRS", media)
        .env("OPENCUT_EXPORTS_DIR", root.join("exports"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&input).unwrap().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}
fn result(root: &std::path::Path, input: Value) -> Value {
    let envelope = request(root, input);
    assert_eq!(envelope["type"], "result", "{envelope}");
    envelope["result"].clone()
}
fn resolve(value: &mut Value, replacements: &std::collections::BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(id) = replacements.get(text) {
                *text = id.clone();
            }
        }
        Value::Array(values) => {
            for v in values {
                resolve(v, replacements);
            }
        }
        Value::Object(values) => {
            for v in values.values_mut() {
                resolve(v, replacements);
            }
        }
        _ => {}
    }
}
#[test]
fn standalone_typed_motif_phases_atomic_failure_and_persisted_reopen() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    let c: Value =
        serde_json::from_str(include_str!("../../../contracts/native-motif-pack-v1.json")).unwrap();
    let id = result(
        path,
        json!({"operation":"create_project","name":"Native motifs","settings":c["canvas"]}),
    )["projectId"]
        .clone();
    let initial = result(path, json!({"operation":"get_state","projectId":id}));
    let track = initial["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["trackType"] == "overlay")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let motifs = c["motifs"].as_array().unwrap();
    let seeds: Vec<Value> = motifs.iter().map(|m| m["seed"].clone()).collect();
    let a = result(
        path,
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":seeds}),
    );
    let mut substitutions = std::collections::BTreeMap::from([("$overlay".to_owned(), track)]);
    for m in motifs {
        substitutions.insert(
            m["componentIdPlaceholder"].as_str().unwrap().to_owned(),
            a["aliases"][m["phaseAAlias"].as_str().unwrap()]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    let mut operations: Vec<Value> = motifs.iter().map(|m| m["populate"].clone()).collect();
    operations.extend(c["galleryInstances"].as_array().unwrap().iter().cloned());
    let mut operations = json!(operations);
    resolve(&mut operations, &substitutions);
    let before = result(path, json!({"operation":"get_state","projectId":id}));
    let project_dir = path.join("projects").join(id.as_str().unwrap());
    let saved = [
        std::fs::read(project_dir.join("project.json")).unwrap(),
        std::fs::read(project_dir.join("history.json")).unwrap(),
    ];
    let mut invalid = operations.clone();
    invalid
        .as_array_mut()
        .unwrap()
        .push(json!({"operation":"delete_item","itemId":"missing"}));
    let error = request(
        path,
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":invalid}),
    );
    assert_eq!(error["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(path, json!({"operation":"get_state","projectId":id})),
        before
    );
    assert_eq!(
        [
            std::fs::read(project_dir.join("project.json")).unwrap(),
            std::fs::read(project_dir.join("history.json")).unwrap()
        ],
        saved
    );
    let b = result(
        path,
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":operations}),
    );
    assert_eq!(b["aliases"].as_object().unwrap().len(), 10);
    let state = result(path, json!({"operation":"get_state","projectId":id}));
    assert_eq!(state["project"]["components"].as_array().unwrap().len(), 5);
    assert_eq!(state["project"]["assets"], json!([]));
    assert_eq!(
        result(path, json!({"operation":"open_project","projectId":id})),
        state
    );
}
