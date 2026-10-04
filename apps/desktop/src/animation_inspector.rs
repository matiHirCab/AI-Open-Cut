//! Bounded presentation of authoritative authored animation, never a sampler.
use crate::{
    hierarchy::Selection,
    inspector_edit::{Field, FieldKind, add},
};
use opencut_editor_core::{AnimationChannelValue, MediaType, Project, TimelineItem};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub channel: usize,
    pub key: usize,
    pub legacy: usize,
}
impl Cursor {
    pub fn channel_index(self, item: &TimelineItem) -> usize {
        self.channel.min(
            item.visual_properties()
                .animation_channels
                .len()
                .saturating_sub(1),
        )
    }
    pub fn key_index(self, item: &TimelineItem) -> usize {
        self.key.min(
            item.visual_properties()
                .animation_channels
                .get(self.channel_index(item))
                .map_or(0, |c| c.keyframes.len().saturating_sub(1)),
        )
    }
}
#[derive(Clone, Debug)]
pub(crate) struct DraftIdentity {
    pub selection: Selection,
    pub revision: u64,
    pub cursor: Cursor,
}
impl DraftIdentity {
    pub fn matches(
        &self,
        project: &Project,
        selection: Option<&Selection>,
        cursor: Cursor,
    ) -> bool {
        self.revision == project.revision
            && selection == Some(&self.selection)
            && self.cursor == cursor
            && self.selection.resolve(project).is_some()
    }
}
pub(crate) fn audio_only(project: &Project, item: &TimelineItem) -> bool {
    matches!(item, TimelineItem::Media(media) if project.assets.iter().any(|asset| asset.id == media.asset_id && asset.media_type == MediaType::Audio))
}
pub(crate) fn editable(selection: &Selection) -> bool {
    selection.scope == "root" && selection.instance_path.is_empty()
}
pub(crate) fn fields(item: &TimelineItem, cursor: Cursor, audio_only: bool) -> Vec<Field> {
    let channels = &item.visual_properties().animation_channels;
    let index = cursor.channel_index(item);
    let Some(channel) = channels.get(index) else {
        return vec![];
    };
    let key_index = cursor.key_index(item);
    let Some(key) = channel.keyframes.get(key_index) else {
        return vec![];
    };
    if !matches!(key.value, AnimationChannelValue::Scalar { .. })
        || (audio_only && channel.property != opencut_editor_core::AnimationChannelProperty::GainDb)
    {
        return vec![];
    }
    let source = serde_json::to_value(item).unwrap();
    let mut fields = vec![];
    for (label, path, kind) in [
        (
            "Source key time · ms",
            format!("/animationChannels/{index}/keyframes/{key_index}/timeMs"),
            FieldKind::Milliseconds,
        ),
        (
            "Scalar key value",
            format!("/animationChannels/{index}/keyframes/{key_index}/value/value"),
            FieldKind::Number,
        ),
    ] {
        add(&mut fields, &source, label, path, "animationChannels", kind);
    }
    if let Some(curve) = source
        .pointer(&format!(
            "/animationChannels/{index}/keyframes/{key_index}/curve"
        ))
        .and_then(Value::as_object)
    {
        for name in [
            "x1",
            "y1",
            "x2",
            "y2",
            "mass",
            "stiffness",
            "damping",
            "initialVelocity",
        ] {
            if curve.contains_key(name) {
                add(
                    &mut fields,
                    &source,
                    format!("Curve {name}"),
                    format!("/animationChannels/{index}/keyframes/{key_index}/curve/{name}"),
                    "animationChannels",
                    FieldKind::Number,
                );
            }
        }
    }
    add(
        &mut fields,
        &source,
        "Loop mode · none/repeat/ping_pong",
        format!("/animationChannels/{index}/loop/mode"),
        "animationChannels",
        FieldKind::LoopMode,
    );
    let last = fields.last_mut().unwrap();
    if channel.r#loop.is_none() {
        last.value = "none".into();
    }
    if channel.r#loop.is_some() {
        add(
            &mut fields,
            &source,
            "Loop iterations · count/infinite",
            format!("/animationChannels/{index}/loop/iterations"),
            "animationChannels",
            FieldKind::LoopIterations,
        );
    }
    fields
}
pub(crate) fn descriptions(item: &TimelineItem, cursor: Cursor) -> Vec<String> {
    let visual = item.visual_properties();
    let mut result = vec![format!(
        "Animation channels: {} · source times are independent of visible local time",
        visual.animation_channels.len()
    )];
    if let Some(channel) = visual.animation_channels.get(cursor.channel_index(item)) {
        let property = serde_json::to_value(channel.property).unwrap();
        result.push(format!(
            "Channel {} / {} · {}",
            cursor.channel_index(item) + 1,
            visual.animation_channels.len(),
            property.as_str().unwrap()
        ));
        result.push(format!(
            "Target: {}",
            channel
                .target
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap())
                .unwrap_or_else(|| "item".into())
        ));
        result.push(format!(
            "Source clock: {}",
            channel
                .clock
                .map(|c| format!(
                    "offset {} ms · source duration {} ms (read-only)",
                    c.offset_ms, c.source_duration_ms
                ))
                .unwrap_or_else(|| "implicit local clock".into())
        ));
        result.push(format!(
            "Loop: {}",
            channel
                .r#loop
                .map(|v| serde_json::to_string(&v).unwrap())
                .unwrap_or_else(|| "none".into())
        ));
        if let Some(key) = channel.keyframes.get(cursor.key_index(item)) {
            result.push(format!(
                "Source key {} / {} · {} ms · value {} · curve {}",
                cursor.key_index(item) + 1,
                channel.keyframes.len(),
                key.time_ms,
                serde_json::to_string(&key.value).unwrap(),
                serde_json::to_string(&key.curve).unwrap()
            ));
        }
        if let Some(source) = visual.animation_preset_provenance.get(&channel.property) {
            result.push(format!(
                "Preset attribution: {} v{} · compiler {} · {}",
                source.preset_id,
                source.preset_version,
                source.compiler_version,
                serde_json::to_string(&source.parameters).unwrap()
            ));
        }
        if !matches!(
            channel.keyframes.first().map(|k| &k.value),
            Some(AnimationChannelValue::Scalar { .. })
        ) {
            result.push("Compound source values · read-only".into());
        }
    }
    result.push(format!(
        "Legacy animation: {} keys · read-only",
        item.keyframes().len()
    ));
    if let Some(key) = item
        .keyframes()
        .get(cursor.legacy.min(item.keyframes().len().saturating_sub(1)))
    {
        result.push(format!(
            "Legacy source key {} / {}: {}",
            cursor.legacy.min(item.keyframes().len() - 1) + 1,
            item.keyframes().len(),
            serde_json::to_string(key).unwrap()
        ));
    }
    if let Some(c) = visual.legacy_animation_clock {
        result.push(format!(
            "Legacy source clock: offset {} ms · source duration {} ms (read-only)",
            c.offset_ms, c.source_duration_ms
        ));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inspector_edit,
        session::{Command, Session, Startup},
    };
    use opencut_editor_core::{EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings};
    use serde_json::json;
    fn op(value: Value) -> EditOperation {
        serde_json::from_value(value).unwrap()
    }
    fn setup() -> (tempfile::TempDir, EditorCore, Startup, String) {
        let root = tempfile::tempdir().unwrap();
        let store = root.path().join("projects");
        std::fs::create_dir(root.path().join("media")).unwrap();
        let core = EditorCore::new(
            PathPolicy::new(
                &store,
                [root.path().join("media")],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let project = core
            .create_project("Animation inspector", ProjectSettings::default())
            .unwrap()
            .project_id;
        let track = core.get_project(&project).unwrap().tracks[1].id.clone();
        let item=core.edit(&project,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":10,"height":10,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
        core.edit(&project,1,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"soft","type":"gaussian_blur","radiusPx":1.0}]}))).unwrap();
        core.edit(&project,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"effect.blur_radius","target":{"kind":"effect","scope":"root","id":"soft"},"clock":{"offsetMs":-50,"sourceDurationMs":1000},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1.0},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":2.0},"curve":"hold"}]}]}))).unwrap();
        core.edit(&project,3,op(json!({"operation":"apply_animation_preset","itemId":item,"presetId":"scalar_tween","presetVersion":1,"parameters":{"property":"transform.position_x","startMs":0,"durationMs":900,"from":0.0,"to":10.0,"curve":{"type":"cubic_bezier","x1":0.2,"y1":0.8,"x2":0.7,"y2":0.9}}}))).unwrap();
        core.edit(
            &project,
            4,
            op(json!({"operation":"trim_item","itemId":item,"startMs":100,"durationMs":800})),
        )
        .unwrap();
        let startup = Startup {
            store,
            project_id: project,
        };
        (root, core, startup, item)
    }
    fn chosen(fields: &[Field], label: &str) -> Field {
        fields.iter().find(|f| f.label == label).unwrap().clone()
    }
    fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
        let dir = core.paths().project_dir(id).unwrap();
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
        )
    }
    #[test]
    fn scalar_edits_preserve_scoped_channels_clocks_and_core_history() {
        let (_root, core, startup, item) = setup();
        let original = core.get_project(&startup.project_id).unwrap();
        let selected = original.find_item(&item).unwrap();
        let cursor = Cursor {
            channel: 1,
            key: 0,
            legacy: 0,
        };
        let fs = fields(selected, cursor, false);
        let edit =
            inspector_edit::build(selected, &chosen(&fs, "Scalar key value"), "4.5").unwrap();
        let request = serde_json::to_value(&edit).unwrap();
        let channels =
            serde_json::to_value(&selected.visual_properties().animation_channels).unwrap();
        assert_eq!(request["animationChannels"][0], channels[0]);
        let mut expected = channels.clone();
        expected[1]["keyframes"][0]["value"]["value"] = json!(4.5);
        assert_eq!(request["animationChannels"], expected);
        let after = startup
            .execute(Command::Edit(original.revision, Box::new(edit)))
            .unwrap();
        let visual = after.find_item(&item).unwrap().visual_properties();
        assert!(visual.animation_preset_provenance.is_empty());
        assert_eq!(
            serde_json::to_value(&visual.animation_channels).unwrap(),
            expected
        );
        let undo = startup.execute(Command::Undo(after.revision)).unwrap();
        assert_eq!(
            serde_json::to_value(undo.find_item(&item).unwrap()).unwrap(),
            serde_json::to_value(selected).unwrap()
        );
        let redo = startup.execute(Command::Redo(undo.revision)).unwrap();
        assert_eq!(
            serde_json::to_value(redo.find_item(&item).unwrap()).unwrap(),
            serde_json::to_value(after.find_item(&item).unwrap()).unwrap()
        );
        let reopened = startup.execute(Command::Refresh).unwrap();
        assert_eq!(
            serde_json::to_value(reopened).unwrap(),
            serde_json::to_value(redo).unwrap()
        );
        let time =
            inspector_edit::build(selected, &chosen(&fs, "Source key time · ms"), "25").unwrap();
        let mut expected = channels.clone();
        expected[1]["keyframes"][0]["timeMs"] = json!(25);
        assert_eq!(
            serde_json::to_value(time).unwrap()["animationChannels"],
            expected
        );
        let curve = inspector_edit::build(selected, &chosen(&fs, "Curve x1"), "0.3").unwrap();
        let mut expected = channels;
        expected[1]["keyframes"][0]["curve"]["x1"] = json!(0.3);
        assert_eq!(
            serde_json::to_value(curve).unwrap()["animationChannels"],
            expected
        );
    }
    #[test]
    fn loop_controls_preserve_complete_collection_and_default_only_new_loop() {
        let (_root, core, startup, item) = setup();
        let project = core.get_project(&startup.project_id).unwrap();
        let selected = project.find_item(&item).unwrap();
        let fs = fields(
            selected,
            Cursor {
                channel: 1,
                ..Cursor::default()
            },
            false,
        );
        let edit = inspector_edit::build(
            selected,
            &chosen(&fs, "Loop mode · none/repeat/ping_pong"),
            "ping_pong",
        )
        .unwrap();
        let after = startup
            .execute(Command::Edit(project.revision, Box::new(edit)))
            .unwrap();
        let selected = after.find_item(&item).unwrap();
        let fs = fields(
            selected,
            Cursor {
                channel: 1,
                ..Cursor::default()
            },
            false,
        );
        let channels =
            serde_json::to_value(&selected.visual_properties().animation_channels).unwrap();
        assert_eq!(
            channels[1]["loop"],
            json!({"mode":"ping_pong","iterations":1})
        );
        let iterations = inspector_edit::build(
            selected,
            &chosen(&fs, "Loop iterations · count/infinite"),
            "infinite",
        )
        .unwrap();
        let mut expected = channels.clone();
        expected[1]["loop"]["iterations"] = json!("infinite");
        assert_eq!(
            serde_json::to_value(iterations).unwrap()["animationChannels"],
            expected
        );
        let removed = inspector_edit::build(
            selected,
            &chosen(&fs, "Loop mode · none/repeat/ping_pong"),
            "none",
        )
        .unwrap();
        let mut expected = channels;
        expected[1].as_object_mut().unwrap().remove("loop");
        assert_eq!(
            serde_json::to_value(removed).unwrap()["animationChannels"],
            expected
        );
    }
    #[test]
    fn errors_remain_authoritative_and_conflicts_require_refresh() {
        let (_root, core, startup, item) = setup();
        let original = core.get_project(&startup.project_id).unwrap();
        let selected = original.find_item(&item).unwrap();
        let fs = fields(
            selected,
            Cursor {
                channel: 1,
                ..Cursor::default()
            },
            false,
        );
        let before = files(&core, &startup.project_id);
        assert!(
            inspector_edit::build(selected, &chosen(&fs, "Source key time · ms"), "0.5").is_err()
        );
        assert!(inspector_edit::build(selected, &chosen(&fs, "Scalar key value"), "NaN").is_err());
        assert_eq!(files(&core, &startup.project_id), before);
        for (label, input) in [("Curve x1", "-1"), ("Source key time · ms", "1001")] {
            let edit = inspector_edit::build(selected, &chosen(&fs, label), input).unwrap();
            let error = startup
                .execute(Command::Edit(original.revision, Box::new(edit)))
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(!error.retryable);
            assert_eq!(files(&core, &startup.project_id), before);
        }
        let edit = inspector_edit::build(selected, &chosen(&fs, "Scalar key value"), "4").unwrap();
        let error = startup
            .execute(Command::Edit(original.revision - 1, Box::new(edit)))
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::RevisionConflict);
        let mut session = Session::default();
        session.project = Some(original.clone());
        session.selected = Some(Selection::root(&item));
        let generation = session.begin().unwrap();
        session.finish(generation, Err(error));
        assert!(session.needs_refresh);
        assert!(
            session
                .error
                .as_ref()
                .unwrap()
                .contains("REVISION_CONFLICT")
        );
        assert!(session.error.as_ref().unwrap().contains("retryable: true"));
        let generation = session.begin().unwrap();
        session.finish(generation, startup.execute(Command::Refresh));
        assert!(!session.needs_refresh);
        let mut missing = serde_json::to_value(
            inspector_edit::build(selected, &chosen(&fs, "Scalar key value"), "4").unwrap(),
        )
        .unwrap();
        missing["itemId"] = json!("missing");
        assert_eq!(
            startup
                .execute(Command::Edit(original.revision, Box::new(op(missing))))
                .unwrap_err()
                .code,
            ErrorCode::ItemNotFound
        );
        assert_eq!(files(&core, &startup.project_id), before);
        let track = original.tracks[1].id.clone();
        core.edit(
            &startup.project_id,
            original.revision,
            op(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
        let locked = core.get_project(&startup.project_id).unwrap();
        let before = files(&core, &startup.project_id);
        let edit = inspector_edit::build(
            locked.find_item(&item).unwrap(),
            &chosen(&fs, "Scalar key value"),
            "4",
        )
        .unwrap();
        assert_eq!(
            startup
                .execute(Command::Edit(locked.revision, Box::new(edit)))
                .unwrap_err()
                .code,
            ErrorCode::TrackLocked
        );
        assert_eq!(files(&core, &startup.project_id), before);
    }
    #[test]
    fn bounded_inspection_and_draft_identity_keep_occurrences_and_audio_separate() {
        let (_root, core, startup, item) = setup();
        let project = core.get_project(&startup.project_id).unwrap();
        let selected = project.find_item(&item).unwrap();
        let cursor = Cursor {
            channel: usize::MAX,
            key: usize::MAX,
            legacy: usize::MAX,
        };
        assert_eq!(cursor.channel_index(selected), 1);
        assert_eq!(cursor.key_index(selected), 1);
        assert!(descriptions(selected, cursor).len() < 12);
        assert!(fields(selected, cursor, false).len() < 10);
        let selection = Selection::root(&item);
        let binding = DraftIdentity {
            selection: selection.clone(),
            revision: project.revision,
            cursor: Cursor::default(),
        };
        assert!(binding.matches(&project, Some(&selection), Cursor::default()));
        assert!(!binding.matches(&project, Some(&selection), cursor));
        assert!(!binding.matches(&project, Some(&Selection::root("other")), Cursor::default()));
        let mut next = project.clone();
        next.revision += 1;
        assert!(!binding.matches(&next, Some(&selection), Cursor::default()));
        let local = Selection {
            instance_path: vec!["instance".into()],
            scope: "component:source".into(),
            item_id: item,
        };
        assert!(!editable(&local));
        assert!(fields(selected, Cursor::default(), true).is_empty());
        let compound:TimelineItem=serde_json::from_value(json!({"type":"rectangle","id":"compound","startMs":0,"durationMs":1000,"width":1,"height":1,"color":"#ffffff","keyframes":[],"animationChannels":[{"property":"graphic.fill_color","keyframes":[{"timeMs":0,"value":{"type":"rgba","r":1.0,"g":0.0,"b":0.0,"a":1.0},"curve":"linear"},{"timeMs":900,"value":{"type":"rgba","r":0.0,"g":1.0,"b":0.0,"a":1.0},"curve":"hold"}]}]})).unwrap();
        assert!(fields(&compound, Cursor::default(), false).is_empty());
        assert!(
            descriptions(&compound, Cursor::default())
                .iter()
                .any(|v| v.contains("Compound source values"))
        );
    }
    #[test]
    fn root_audio_gain_edits_are_independent_of_visual_eligibility() {
        let root = tempfile::tempdir().unwrap();
        let fixture = crate::tests::fixture::seed(root.path());
        let project = fixture.project();
        let media = project
            .tracks
            .iter()
            .flat_map(|t| &t.items)
            .find(|i| matches!(i, TimelineItem::Media(_)))
            .unwrap();
        let id = media.id().to_owned();
        let selection = Selection::root(&id);
        assert!(audio_only(&project, media));
        assert!(!crate::hierarchy::editable(&project, &selection));
        assert!(editable(&selection));
        fixture.core.edit(&fixture.id,project.revision,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[{"property":"audio.gain_db","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":-12.0},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":0.0},"curve":"hold"}]}]}))).unwrap();
        let project = fixture.project();
        let media = project.find_item(&id).unwrap();
        let source = serde_json::to_value(media).unwrap();
        let fs = fields(media, Cursor::default(), audio_only(&project, media));
        let startup = Startup {
            store: fixture.core.paths().projects_root().to_owned(),
            project_id: fixture.id.clone(),
        };
        let after = startup
            .execute(Command::Edit(
                project.revision,
                Box::new(
                    inspector_edit::build(media, &chosen(&fs, "Scalar key value"), "-6.0").unwrap(),
                ),
            ))
            .unwrap();
        let mut expected = source;
        expected["animationChannels"][0]["keyframes"][0]["value"]["value"] = json!(-6.0);
        assert_eq!(
            serde_json::to_value(after.find_item(&id).unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::to_value(startup.execute(Command::Refresh).unwrap()).unwrap(),
            serde_json::to_value(after).unwrap()
        );
    }
    #[test]
    fn controller_fields_patch_only_stagger_or_repeater_time_and_core_checks_bounds() {
        let (_root, core, startup, _) = setup();
        let p = core.get_project(&startup.project_id).unwrap();
        let track = p.tracks[1].id.clone();
        let group = core
            .edit(
                &startup.project_id,
                p.revision,
                op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let p = core.get_project(&startup.project_id).unwrap();
        let selected = p.find_item(&group).unwrap();
        let fs = inspector_edit::fields(selected);
        let field = chosen(&fs, "Stagger · ms");
        assert_eq!(field.value, "0");
        let edit = inspector_edit::build(selected, &field, "30").unwrap();
        let serialized = serde_json::to_value(&edit).unwrap();
        assert_eq!(serialized["staggerMs"], 30);
        assert!(serialized.get("animationChannels").is_none());
        let after = startup
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap();
        assert_eq!(
            serde_json::to_value(after.find_item(&group).unwrap()).unwrap()["staggerMs"],
            30
        );
        let before = files(&core, &startup.project_id);
        let invalid =
            inspector_edit::build(after.find_item(&group).unwrap(), &field, "60001").unwrap();
        assert_eq!(
            startup
                .execute(Command::Edit(after.revision, Box::new(invalid)))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &startup.project_id), before);
        let source=core.edit(&startup.project_id,after.revision,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"rectangle","width":40,"height":20},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}))).unwrap().changed_ids[0].clone();
        let p = core.get_project(&startup.project_id).unwrap();
        let descriptor = json!({"source":{"scope":"root","id":source},"copies":2,"transformOffset":{"position":{"unit":"pixels","x":0.0,"y":20.0},"scaleX":1.0,"scaleY":1.0,"rotationDeg":0.0,"skewXDeg":0.0,"skewYDeg":0.0},"opacityOffset":0.0});
        let repeater=core.edit(&startup.project_id,p.revision,op(json!({"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor}))).unwrap().changed_ids[0].clone();
        let p = core.get_project(&startup.project_id).unwrap();
        let selected = p.find_item(&repeater).unwrap();
        let fs = inspector_edit::fields(selected);
        let field = chosen(&fs, "Repeater time offset · signed ms");
        assert_eq!(field.value, "0");
        let mut expected = serde_json::to_value(selected).unwrap();
        expected["repeater"]["timeOffsetMs"] = json!(-25);
        let after = startup
            .execute(Command::Edit(
                p.revision,
                Box::new(inspector_edit::build(selected, &field, "-25").unwrap()),
            ))
            .unwrap();
        assert_eq!(
            serde_json::to_value(after.find_item(&repeater).unwrap()).unwrap(),
            expected
        );
        let before = files(&core, &startup.project_id);
        let invalid = inspector_edit::build(
            after.find_item(&repeater).unwrap(),
            &field,
            "9223372036854775807",
        )
        .unwrap();
        assert_eq!(
            startup
                .execute(Command::Edit(after.revision, Box::new(invalid)))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &startup.project_id), before);
    }
    #[test]
    fn instance_stagger_preserves_trim_scale_slots_and_animation() {
        let (_root, core, startup, _) = setup();
        let p = core.get_project(&startup.project_id).unwrap();
        let component = core.edit(&startup.project_id, p.revision, op(json!({"operation":"component_create","name":"Temporal child","width":320,"height":240,"durationMs":1000,"tracks":[]}))).unwrap().changed_ids[0].clone();
        let p = core.get_project(&startup.project_id).unwrap();
        let instance = core.edit(&startup.project_id, p.revision, op(json!({"operation":"add_component_instance","trackId":p.tracks[1].id,"componentId":component,"startMs":0,"trimStartMs":25,"durationMs":500,"timeScale":1.5}))).unwrap().changed_ids[0].clone();
        let p = core.get_project(&startup.project_id).unwrap();
        let item = p.find_item(&instance).unwrap();
        let field = chosen(&inspector_edit::fields(item), "Stagger · ms");
        let mut expected = serde_json::to_value(item).unwrap();
        expected["staggerMs"] = json!(15);
        let after = startup
            .execute(Command::Edit(
                p.revision,
                Box::new(inspector_edit::build(item, &field, "15").unwrap()),
            ))
            .unwrap();
        assert_eq!(
            serde_json::to_value(after.find_item(&instance).unwrap()).unwrap(),
            expected
        );
        let undone = startup.execute(Command::Undo(after.revision)).unwrap();
        assert_eq!(
            serde_json::to_value(undone.find_item(&instance)).unwrap(),
            serde_json::to_value(p.find_item(&instance)).unwrap()
        );
        let redone = startup.execute(Command::Redo(undone.revision)).unwrap();
        assert_eq!(
            serde_json::to_value(redone.find_item(&instance)).unwrap(),
            serde_json::to_value(after.find_item(&instance)).unwrap()
        );
        assert_eq!(
            serde_json::to_value(startup.execute(Command::Refresh).unwrap()).unwrap(),
            serde_json::to_value(redone).unwrap()
        );
    }
    #[test]
    fn maximum_collection_presentation_only_materializes_selected_key_fields() {
        // Structural worst-case sizing tests presentation separately from the
        // core's identity/target validation; core owns document acceptance.
        let keys:Vec<Value>=(0..1000).map(|time|json!({"timeMs":time,"value":{"type":"scalar","value":1.0},"curve":"linear"})).collect();
        let channels: Vec<Value> = (0..64)
            .map(|_| json!({"property":"transform.position_x","keyframes":keys}))
            .collect();
        let item:TimelineItem=serde_json::from_value(json!({"type":"rectangle","id":"bounded","startMs":0,"durationMs":1001,"width":1,"height":1,"color":"#ffffff","keyframes":[],"animationChannels":channels})).unwrap();
        let cursor = Cursor {
            channel: 63,
            key: 999,
            legacy: 0,
        };
        assert!(fields(&item, cursor, false).len() <= 8);
        assert!(descriptions(&item, cursor).len() < 12);
        assert!(
            descriptions(&item, cursor)
                .iter()
                .any(|s| s.contains("Source key 1000 / 1000"))
        );
    }
}
