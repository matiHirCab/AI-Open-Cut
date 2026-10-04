use super::core_facade::{
    EditOperation, EditorCore, PathPolicy, Project, ProjectSettings, TrackType,
};
use serde_json::{Value, json};
use std::path::Path;
#[path = "temporal_recipe.rs"]
pub mod recipe;
pub struct Fixture {
    pub core: EditorCore,
    pub id: String,
    pub item_ids: Vec<String>,
    pub track_id: String,
    pub family_b: bool,
}
impl Fixture {
    pub fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
    pub fn edit(&self, value: Value) {
        self.core
            .edit(&self.id, self.project().revision, operation(value))
            .unwrap();
    }
}
pub fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
pub fn seed(root: &Path, family_b: bool) -> Fixture {
    assert_eq!(recipe::family_a()[0]["items"].as_array().unwrap().len(), 6);
    for time in recipe::TIMES {
        for lane in 0..6 {
            assert!(recipe::expected_a(lane, time).is_finite());
        }
        for copy in 0..3 {
            assert!(recipe::expected_b(copy, time).is_finite());
        }
    }
    let media = root.join("media");
    std::fs::create_dir_all(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(root.join("projects"), [&media], root.join("exports")).unwrap(),
    );
    let id = core
        .create_project(
            if family_b {
                "Temporal B: inherited clocks"
            } else {
                "Temporal A: curves and loops"
            },
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track_id = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Overlay)
        .unwrap()
        .id
        .clone();
    let mut f = Fixture {
        core,
        id,
        item_ids: vec![],
        track_id,
        family_b,
    };
    if !family_b {
        for lane in 0..6 {
            let item = recipe::rectangle("unused", lane, 5.0 + 9.0 * lane as f64);
            let result=f.core.edit(&f.id,f.project().revision,operation(json!({"operation":"add_rectangle","trackId":f.track_id,"startMs":0,"durationMs":1300,"width":5,"height":5,"color":item["color"],"transform":item["transform"]}))).unwrap();
            let item_id = result.changed_ids[0].clone();
            f.edit(json!({"operation":"set_animation_channels","itemId":item_id,"animationChannels":[recipe::channel(lane)]}));
            f.item_ids.push(item_id);
        }
    } else {
        let create = |name, duration| json!({"operation":"component_create","name":name,"width":64,"height":64,"durationMs":duration,"tracks":[],"slots":[]});
        let leaf = f
            .core
            .edit(
                &f.id,
                f.project().revision,
                operation(create("Temporal leaf", 1500)),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let outer = f
            .core
            .edit(
                &f.id,
                f.project().revision,
                operation(create("Temporal outer", 2000)),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let (leaf_tracks, outer_tracks, _) = recipe::family_b(&leaf, &outer);
        f.edit(json!({"operation":"component_update","componentId":leaf,"name":"Temporal leaf","width":64,"height":64,"durationMs":1500,"tracks":leaf_tracks,"slots":[]}));
        f.edit(json!({"operation":"component_update","componentId":outer,"name":"Temporal outer","width":64,"height":64,"durationMs":2000,"tracks":outer_tracks,"slots":[]}));
        let source=f.core.edit(&f.id,f.project().revision,operation(json!({"operation":"add_component_instance","trackId":f.track_id,"componentId":outer,"startMs":100,"trimStartMs":50,"durationMs":1200,"timeScale":1.5,"slotValues":{}}))).unwrap().changed_ids[0].clone();
        let (_, _, roots) = recipe::family_b(&leaf, &outer);
        let mut repeater = roots[1]["repeater"].clone();
        repeater["source"]["id"] = json!(source);
        let copies=f.core.edit(&f.id,f.project().revision,operation(json!({"operation":"add_repeater","trackId":f.track_id,"startMs":0,"durationMs":1300,"repeater":repeater}))).unwrap().changed_ids[0].clone();
        f.item_ids = vec![source, copies];
    }
    f
}
