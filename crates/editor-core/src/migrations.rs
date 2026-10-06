//! Deterministic persisted-schema migration owner.

use crate::{CoreError, ErrorCode, History, PROJECT_SCHEMA_VERSION, Project};

pub(crate) fn migrate_project_documents(
    project: &mut Project,
    history: &mut History,
) -> Result<bool, CoreError> {
    let mut migrated_project = project.clone();
    let mut migrated_history = history.clone();
    let mut changed = migrate_project(&mut migrated_project)?;
    for snapshot in migrated_history
        .undo
        .iter_mut()
        .chain(&mut migrated_history.redo)
    {
        changed |= migrate_project(snapshot)?;
    }
    if changed {
        *project = migrated_project;
        *history = migrated_history;
    }
    Ok(changed)
}

fn migrate_project(project: &mut Project) -> Result<bool, CoreError> {
    if project.schema_version < 34
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| i.visual_properties().matte.is_some() || i.visual_properties().matte_only)
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "track mattes require schema 34",
        ));
    }
    if project.schema_version < 33
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| {
                i.visual_properties().animation_channels.iter().any(|c| {
                    c.property.mask()
                        || c.target
                            .as_ref()
                            .is_some_and(|t| t.kind == crate::AnimationTargetKind::Mask)
                })
            })
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "mask animation requires schema 33",
        ));
    }
    if project.schema_version < 32
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| !i.visual_properties().masks.is_empty())
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "mask models require schema 32",
        ));
    }
    if project.schema_version < 31
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|item| {
                let visual = item.visual_properties();
                visual.legacy_animation_clock.is_some()
                    || visual.animation_channels.iter().any(|c| c.clock.is_some())
            })
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "retained animation clocks require schema 31",
        ));
    }
    if project.schema_version < 30
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| {
                i.visual_properties()
                    .animation_preset_provenance
                    .values()
                    .any(|source| {
                        matches!(source.parameters, crate::AnimationPresetParameters::Pack(_))
                    })
            })
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "tagged preset provenance requires schema 30",
        ));
    }
    if project.schema_version < 29
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| !i.visual_properties().animation_preset_provenance.is_empty())
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "preset provenance requires schema 29",
        ));
    }
    if project.schema_version < 28
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| i.visual_properties().motion_blur.is_some())
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "motion blur requires schema 28",
        ));
    }
    if project.schema_version < 27
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| {
                i.visual_properties().crop.is_some()
                    || !i.visual_properties().effects.is_empty()
                    || i.visual_properties()
                        .animation_channels
                        .iter()
                        .any(|c| c.property.extended())
            })
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "extended visuals require schema 27",
        ));
    }
    if project.schema_version < 26
        && project
            .tracks
            .iter()
            .chain(
                project
                    .components
                    .iter()
                    .flat_map(|component| &component.tracks),
            )
            .flat_map(|track| &track.items)
            .any(|item| match item {
                crate::TimelineItem::Group(group) => group.stagger_ms != 0,
                crate::TimelineItem::ComponentInstance(instance) => instance.stagger_ms != 0,
                crate::TimelineItem::Repeater(repeater) => repeater.repeater.time_offset_ms != 0,
                _ => false,
            })
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "inherited animation timing requires schema 26",
        ));
    }
    if project.schema_version < 25
        && project
            .tracks
            .iter()
            .chain(
                project
                    .components
                    .iter()
                    .flat_map(|component| &component.tracks),
            )
            .flat_map(|track| &track.items)
            .flat_map(|item| &item.visual_properties().animation_channels)
            .any(|channel| channel.r#loop.is_some())
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "animation loops require schema 25",
        ));
    }
    if project.schema_version < 24
        && (!project.markers.is_empty()
            || project
                .components
                .iter()
                .any(|component| !component.markers.is_empty())
            || project
                .tracks
                .iter()
                .chain(
                    project
                        .components
                        .iter()
                        .flat_map(|component| &component.tracks),
                )
                .flat_map(|track| &track.items)
                .any(|item| item.visual_properties().start_time.is_some()))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "markers and relative timing require schema 24",
        ));
    }
    if project.schema_version < 23
        && project
            .tracks
            .iter()
            .chain(
                project
                    .components
                    .iter()
                    .flat_map(|component| &component.tracks),
            )
            .flat_map(|track| &track.items)
            .flat_map(|item| &item.visual_properties().animation_channels)
            .flat_map(|channel| &channel.keyframes)
            .any(|keyframe| matches!(keyframe.curve, crate::AnimationCurve::Parameterized(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "parameterized animation curves require schema 23",
        ));
    }
    if project.schema_version < 17
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| matches!(i, crate::TimelineItem::Repeater(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "repeater items require schema 17",
        ));
    }
    if project.schema_version < 16
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| matches!(i, crate::TimelineItem::Grid(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "grid items require schema 16",
        ));
    }
    if project.schema_version < 15
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| matches!(i, crate::TimelineItem::Svg(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "SVG items require schema 15",
        ));
    }
    if project.schema_version < 14
        && project
            .tracks
            .iter()
            .chain(project.components.iter().flat_map(|c| &c.tracks))
            .flat_map(|t| &t.items)
            .any(|i| matches!(i, crate::TimelineItem::Shape(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "shape items require schema 14",
        ));
    }
    if project.schema_version < 13
        && project
            .tracks
            .iter()
            .flat_map(|t| &t.items)
            .any(|item| matches!(item, crate::TimelineItem::ComponentInstance(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "root component instances require schema 13",
        ));
    }
    match project.schema_version {
        1..=8 => {
            for track in &mut project.tracks {
                for (index, item) in track.items.iter_mut().enumerate() {
                    let visual = item.visual_properties_mut();
                    visual.z_index = 0;
                    visual.stack_order = u32::try_from(index).map_err(|_| {
                        CoreError::new(ErrorCode::InvalidArgument, "too many items for stack order")
                    })?;
                }
            }
            project.schema_version = PROJECT_SCHEMA_VERSION;
            Ok(true)
        }
        9..=33 => {
            validate_source_component_transforms(project)?;
            project.schema_version = PROJECT_SCHEMA_VERSION;
            Ok(true)
        }
        PROJECT_SCHEMA_VERSION => Ok(false),
        version => Err(CoreError::new(
            ErrorCode::InternalError,
            format!(
                "unsupported project schema version {version}; this build supports up to {PROJECT_SCHEMA_VERSION}"
            ),
        )),
    }
}

/// Check historical constraints before migration discards the source version.
fn validate_source_component_transforms(project: &Project) -> Result<(), CoreError> {
    if (11..=12).contains(&project.schema_version)
        && project
            .components
            .iter()
            .flat_map(|component| &component.tracks)
            .flat_map(|track| &track.items)
            .any(|item| {
                matches!(item, crate::TimelineItem::ComponentInstance(instance)
                    if instance.visual_properties.transform != crate::Transform::default())
            })
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "non-default legacy component instance transforms require schema 13",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectSettings;

    fn project(schema_version: u32) -> Project {
        Project {
            markers: Vec::new(),
            fonts: Default::default(),
            components: vec![],
            schema_version,
            id: "project".into(),
            revision: 0,
            name: "Project".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        }
    }

    #[test]
    fn migrates_current_and_retained_history_together() {
        let mut current = project(1);
        let mut history = History {
            undo: vec![project(2)],
            redo: vec![project(6)],
        };
        assert!(migrate_project_documents(&mut current, &mut history).unwrap());
        assert_eq!(current.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(
            history
                .undo
                .iter()
                .chain(&history.redo)
                .all(|value| value.schema_version == PROJECT_SCHEMA_VERSION)
        );
        assert!(!migrate_project_documents(&mut current, &mut history).unwrap());
    }

    #[test]
    fn schema_24_loop_migration_rejects_retained_injection_atomically() {
        let mut current = project(24);
        let mut history = History {
            undo: vec![project(24)],
            redo: vec![project(24)],
        };
        assert!(migrate_project_documents(&mut current, &mut history).unwrap());
        assert_eq!(current.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(
            history
                .undo
                .iter()
                .chain(&history.redo)
                .all(|snapshot| snapshot.schema_version == PROJECT_SCHEMA_VERSION)
        );
        assert!(!migrate_project_documents(&mut current, &mut history).unwrap());

        let mut invalid = project(24);
        invalid.tracks = serde_json::from_value(serde_json::json!([{
            "id":"track","name":"Overlay","trackType":"overlay","items":[{
                "type":"rectangle","id":"box","startMs":0,"durationMs":1000,
                "width":10,"height":10,"color":"#ff0000",
                "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},
                "keyframes":[],
                "animationChannels":[{"property":"transform.position_x","keyframes":[
                    {"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
                    {"timeMs":500,"value":{"type":"scalar","value":0},"curve":"hold"}],
                    "loop":{"mode":"repeat","iterations":2}}]
            }]
        }]))
        .unwrap();
        let mut current = project(24);
        let mut history = History {
            undo: vec![project(24)],
            redo: vec![invalid],
        };
        let before = serde_json::to_value((&current, &history)).unwrap();
        assert_eq!(
            migrate_project_documents(&mut current, &mut history)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(serde_json::to_value((&current, &history)).unwrap(), before);
    }

    #[test]
    fn schema_26_timing_migrates_history_and_rejects_old_field_injection() {
        let mut current = project(25);
        let mut history = History {
            undo: vec![project(25)],
            redo: vec![project(25)],
        };
        assert!(migrate_project_documents(&mut current, &mut history).unwrap());
        assert_eq!(current.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(
            history
                .undo
                .iter()
                .chain(&history.redo)
                .all(|p| p.schema_version == PROJECT_SCHEMA_VERSION)
        );
        assert!(!migrate_project_documents(&mut current, &mut history).unwrap());

        let mut raw = serde_json::to_value(project(25)).unwrap();
        raw["tracks"] = serde_json::json!([{
            "id": "overlay", "name": "Overlay", "trackType": "overlay", "items": [{
                "type": "group", "id": "group", "startMs": 0, "durationMs": 1000,
                "staggerMs": 0
            }]
        }]);
        assert!(serde_json::from_value::<Project>(raw).is_err());

        let mut invalid = project(25);
        invalid.tracks = serde_json::from_value(serde_json::json!([{
            "id": "overlay", "name": "Overlay", "trackType": "overlay", "items": [{
                "type": "group", "id": "group", "startMs": 0, "durationMs": 1000,
                "staggerMs": 1
            }]
        }]))
        .unwrap();
        let mut current = project(25);
        let mut history = History {
            undo: vec![],
            redo: vec![invalid],
        };
        let before = serde_json::to_value((&current, &history)).unwrap();
        assert_eq!(
            migrate_project_documents(&mut current, &mut history)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(serde_json::to_value((&current, &history)).unwrap(), before);
    }

    #[test]
    fn rejects_unknown_future_schema_without_rewrite() {
        let future = PROJECT_SCHEMA_VERSION + 1;
        let mut current = project(future);
        let mut history = History::default();
        assert_eq!(
            migrate_project_documents(&mut current, &mut history)
                .unwrap_err()
                .code,
            ErrorCode::InternalError
        );
        assert_eq!(current.schema_version, future);
    }

    #[test]
    fn rejects_future_retained_schema_without_mutating_any_document() {
        let mut current = project(6);
        let mut history = History {
            undo: vec![project(1)],
            redo: vec![project(PROJECT_SCHEMA_VERSION + 1)],
        };
        let before = serde_json::to_value((&current, &history)).unwrap();

        assert_eq!(
            migrate_project_documents(&mut current, &mut history)
                .unwrap_err()
                .code,
            ErrorCode::InternalError
        );
        assert_eq!(serde_json::to_value((&current, &history)).unwrap(), before);
    }

    #[test]
    fn source_schema_transform_failure_preserves_all_input_documents() {
        for version in [11, 12] {
            for location in 0..3 {
                let mut invalid = project(version);
                invalid.components = serde_json::from_value(serde_json::json!([
                    {"id":"unused","name":"Unused","width":320,"height":240,"durationMs":1000,"slots":[],"tracks":[
                        {"id":"local","name":"Local","trackType":"overlay","hidden":true,"items":[
                            {"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"transform":{"positionX":0,"positionY":0,"scale":2,"opacity":1}}
                        ]}
                    ]}
                ])).unwrap();
                let mut current = project(12);
                let mut history = History {
                    undo: vec![project(11)],
                    redo: vec![project(12)],
                };
                match location {
                    0 => current = invalid,
                    1 => history.undo[0] = invalid,
                    _ => history.redo[0] = invalid,
                }
                let before = serde_json::to_value((&current, &history)).unwrap();
                assert_eq!(
                    migrate_project_documents(&mut current, &mut history)
                        .unwrap_err()
                        .code,
                    ErrorCode::InvalidArgument
                );
                assert_eq!(serde_json::to_value((&current, &history)).unwrap(), before);
            }
        }
    }
}
