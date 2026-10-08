use super::audit_core::{EditOperation, EditorCore, PathPolicy, ProjectSettings};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
pub(crate) const NAMES: [&str; 7] = [
    "animationChannels",
    "startTime",
    "staggerMs",
    "timeOffsetMs",
    "crop",
    "effects",
    "motionBlur",
];
pub(crate) fn op(v: Value) -> EditOperation {
    serde_json::from_value(v).unwrap()
}
pub(crate) fn seed() -> (tempfile::TempDir, EditorCore, String, Value) {
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
            "Historical dictionary collisions",
            ProjectSettings::default(),
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let layers:Vec<Value>=NAMES.iter().enumerate().map(|(i,_)|json!({"type":"rectangle","id":format!("layer{i}"),"stackOrder":i,"startMs":0,"durationMs":1000,"width":5,"height":5,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"keyframes":[]})).collect();
    let slots:Vec<Value>=NAMES.iter().enumerate().map(|(i,name)|json!({"id":name,"name":name,"kind":"number","required":false,"binding":{"targetLayerId":format!("layer{i}"),"property":"visual.opacity"},"constraints":{},"defaultValue":{"type":"number","value":1.0}})).collect();
    let leaf=core.edit(&id,0,op(json!({"operation":"component_create","name":"Leaf","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":layers}],"slots":slots}))).unwrap().changed_ids[0].clone();
    let values: serde_json::Map<String, Value> = NAMES
        .iter()
        .map(|name| ((*name).to_owned(), json!({"type":"number","value":0.25})))
        .collect();
    let outer=core.edit(&id,1,op(json!({"operation":"component_create","name":"Outer","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"outer","name":"Outer","trackType":"overlay","items":[{"type":"component_instance","id":"nested","componentId":leaf,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":values}]}]}))).unwrap().changed_ids[0].clone();
    let values: serde_json::Map<String, Value> = NAMES
        .iter()
        .map(|name| ((*name).to_owned(), json!({"type":"number","value":0.75})))
        .collect();
    core.edit(&id,2,op(json!({"operation":"add_component_instance","componentId":leaf,"trackId":track,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":values}))).unwrap();
    core.edit(&id,3,op(json!({"operation":"add_component_instance","componentId":outer,"trackId":track,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1}))).unwrap();
    let project = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    (root, core, id, project)
}
pub(crate) fn inventory(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(inventory(&path));
        } else {
            result.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    result
}
pub(crate) fn write(dir: &Path, current: &Value, undo: &Value, redo: &Value) {
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(current).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[undo],"redo":[redo]})).unwrap(),
    )
    .unwrap();
}
pub(crate) fn historical(mut value: Value, version: u32) -> Value {
    value["schemaVersion"] = json!(version);
    if version < 39 {
        value.as_object_mut().unwrap().remove("audioBuses");
    }
    if version < 40 {
        value.as_object_mut().unwrap().remove("soundDefinitions");
    }
    value
}
pub(crate) fn slot_generation(mut p: Value, value: f64) -> Value {
    for name in NAMES {
        p["tracks"][1]["items"][0]["slotValues"][name]["value"] = json!(value);
        p["components"][1]["tracks"][0]["items"][0]["slotValues"][name]["value"] =
            json!(value / 2.0);
    }
    p
}
