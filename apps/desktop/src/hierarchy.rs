//! Presentation of validated snapshots. This is not an evaluator or validator.
use std::collections::HashSet;

use opencut_editor_core::{MediaType, Project, TimelineItem, Track};

pub(crate) const ROW_LIMIT: usize = 4096;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Selection {
    pub instance_path: Vec<String>,
    pub scope: String,
    pub item_id: String,
}

impl Selection {
    pub fn root(id: &str) -> Self {
        Self {
            instance_path: vec![],
            scope: "root".into(),
            item_id: id.into(),
        }
    }

    pub fn resolve<'a>(&self, project: &'a Project) -> Option<(&'a Track, &'a TimelineItem)> {
        let mut tracks = &project.tracks;
        let mut scope = "root".to_owned();
        for id in &self.instance_path {
            let (_, TimelineItem::ComponentInstance(instance)) = find(tracks, id)? else {
                return None;
            };
            let component = project
                .components
                .iter()
                .find(|c| c.id == instance.component_id)?;
            scope = format!("component:{}", component.id);
            tracks = &component.tracks;
        }
        (scope == self.scope)
            .then(|| find(tracks, &self.item_id))
            .flatten()
    }
}

fn find<'a>(tracks: &'a [Track], id: &str) -> Option<(&'a Track, &'a TimelineItem)> {
    tracks.iter().find_map(|track| {
        track
            .items
            .iter()
            .find(|item| item.id() == id)
            .map(|item| (track, item))
    })
}

pub(crate) fn editable(project: &Project, selection: &Selection) -> bool {
    if !selection.instance_path.is_empty() || selection.scope != "root" {
        return false;
    }
    match selection.resolve(project).map(|(_, item)| item) {
        Some(TimelineItem::Transition(_)) | None => false,
        Some(TimelineItem::Media(media)) => project
            .assets
            .iter()
            .any(|asset| asset.id == media.asset_id && asset.media_type != MediaType::Audio),
        Some(_) => true,
    }
}

pub(crate) fn kind(item: &TimelineItem) -> &'static str {
    match item {
        TimelineItem::ComponentInstance(_) => "Instance",
        TimelineItem::Group(_) => "Group",
        TimelineItem::Media(_) => "Media",
        TimelineItem::Text(_) => "Text",
        TimelineItem::SolidColor(_) => "Solid",
        TimelineItem::Rectangle(_) => "Rectangle",
        TimelineItem::Caption(_) => "Caption",
        TimelineItem::Transition(_) => "Transition",
    }
}

pub(crate) struct Row {
    pub selection: Selection,
    pub depth: usize,
    pub expandable: bool,
    pub label: String,
}

#[derive(Default)]
pub(crate) struct Hierarchy {
    pub rows: Vec<Row>,
    pub truncated: bool,
}

impl Hierarchy {
    pub fn build(project: &Project, expanded: &HashSet<Selection>) -> Self {
        let mut tree = Self::default();
        tree.append(
            project,
            expanded,
            &project.tracks,
            &Selection::root(""),
            None,
            0,
        );
        tree
    }

    fn append(
        &mut self,
        project: &Project,
        expanded: &HashSet<Selection>,
        tracks: &[Track],
        context: &Selection,
        parent: Option<&str>,
        depth: usize,
    ) {
        for track in tracks {
            for item in &track.items {
                if item
                    .visual_properties()
                    .parent
                    .as_ref()
                    .map(|p| p.id.as_str())
                    != parent
                {
                    continue;
                }
                if self.rows.len() == ROW_LIMIT {
                    self.truncated = true;
                    return;
                }
                let selection = Selection {
                    item_id: item.id().into(),
                    ..context.clone()
                };
                let component = if let TimelineItem::ComponentInstance(instance) = item {
                    project
                        .components
                        .iter()
                        .find(|c| c.id == instance.component_id)
                } else {
                    None
                };
                let has_children = tracks.iter().flat_map(|t| &t.items).any(|i| {
                    i.visual_properties()
                        .parent
                        .as_ref()
                        .is_some_and(|p| p.id == item.id())
                });
                self.rows.push(Row {
                    selection: selection.clone(),
                    depth,
                    expandable: has_children
                        || component.is_some_and(|c| c.tracks.iter().any(|t| !t.items.is_empty())),
                    label: format!(
                        "{} {} · {}{}",
                        kind(item),
                        item.id(),
                        track.name,
                        if item.hidden() { " · hidden" } else { "" }
                    ),
                });
                if expanded.contains(&selection) {
                    if has_children {
                        self.append(
                            project,
                            expanded,
                            tracks,
                            context,
                            Some(item.id()),
                            depth + 1,
                        );
                    }
                    if let Some(component) = component {
                        let mut path = context.instance_path.clone();
                        path.push(item.id().into());
                        self.append(
                            project,
                            expanded,
                            &component.tracks,
                            &Selection {
                                instance_path: path,
                                scope: format!("component:{}", component.id),
                                item_id: String::new(),
                            },
                            None,
                            depth + 1,
                        );
                    }
                }
                if self.truncated {
                    return;
                }
            }
        }
    }
}
