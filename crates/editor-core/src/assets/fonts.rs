//! Font ingestion and ownership through the shared storage boundary.
use crate::{
    CoreError, ErrorCode, FontBinding, FontRecord, MAX_FONT_BYTES, MAX_FONT_CANDIDATES,
    MAX_FONT_DIRECTORY_DEPTH, Project, TEXT_LAYOUT_PROFILE, TimelineItem,
    fonts::{DEFAULT_FACES, record, validate_binding, validate_catalog, verify_bytes},
    persistence::{Storage, StorageEntryKind},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Default)]
pub struct FontConfig {
    pub roots: Vec<PathBuf>,
    pub default_path: Option<PathBuf>,
}

pub(crate) type FontBytes = BTreeMap<String, Vec<u8>>;

fn selector(text: &crate::TextItem) -> String {
    serde_json::to_string(&(&text.font_path, &text.font_family)).expect("string tuple serializes")
}

pub(crate) fn apply_draft_bindings(
    project: &mut Project,
    catalog: &BTreeMap<String, FontRecord>,
    bindings: &BTreeMap<String, FontBinding>,
) -> Result<(), CoreError> {
    validate_catalog(catalog)?;
    for binding in bindings.values() {
        validate_binding(binding, catalog)?;
    }
    project.fonts.extend(catalog.clone());
    for item in project
        .tracks
        .iter_mut()
        .chain(project.components.iter_mut().flat_map(|c| &mut c.tracks))
        .flat_map(|t| &mut t.items)
    {
        if let TimelineItem::Text(text) = item
            && text.font_binding.is_none()
        {
            text.font_binding = bindings.get(&selector(text)).cloned();
        }
    }
    Ok(())
}

pub(crate) fn draft_binding_step(
    before: &Project,
    after: &Project,
) -> Result<BTreeMap<String, FontBinding>, CoreError> {
    let unresolved: BTreeSet<_> = text_items(before)
        .filter(|(_, text)| text.font_binding.is_none())
        .map(|(scope, text)| (scope, text.id.as_str()))
        .collect();
    let mut step = BTreeMap::new();
    for (_, text) in
        text_items(after).filter(|(scope, text)| unresolved.contains(&(*scope, text.id.as_str())))
    {
        if let Some(binding) = &text.font_binding
            && step
                .insert(selector(text), binding.clone())
                .is_some_and(|old| old != *binding)
        {
            return Err(crate::fonts::invalid(
                "draft selector requires different font bindings; draft version 2 cannot represent this edit",
            ));
        }
    }
    Ok(step)
}

fn text_items(project: &Project) -> impl Iterator<Item = (&str, &crate::TextItem)> {
    project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| ("", item))
        .chain(project.components.iter().flat_map(|component| {
            component
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .map(move |item| (component.id.as_str(), item))
        }))
        .filter_map(|(scope, item)| match item {
            TimelineItem::Text(text) => Some((scope, text)),
            _ => None,
        })
}

/// Preparation-only retention. Local IDs are scoped by the matched component
/// operation, never persisted as selector keys or applied to sibling components.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DraftFontRetention {
    selectors: BTreeMap<String, FontBinding>,
    local: BTreeMap<String, FontBinding>,
    resolve: BTreeSet<String>,
}

pub(crate) fn component_font_bindings(
    project: &Project,
    operation: &crate::EditOperation,
) -> BTreeMap<String, FontBinding> {
    let component = match operation {
        crate::EditOperation::ComponentCreate { .. } => project.components.last(),
        crate::EditOperation::ComponentUpdate { component_id, .. } => {
            project.components.iter().find(|c| c.id == *component_id)
        }
        _ => None,
    };
    component
        .into_iter()
        .flat_map(|c| &c.tracks)
        .flat_map(|t| &t.items)
        .filter_map(|item| match item {
            TimelineItem::Text(text) => text
                .font_binding
                .clone()
                .map(|binding| (text.id.clone(), binding)),
            _ => None,
        })
        .collect()
}

pub(crate) fn apply_retained_fonts(
    project: &mut Project,
    operation: &crate::EditOperation,
    catalog: &BTreeMap<String, FontRecord>,
    retained: &DraftFontRetention,
) -> Result<BTreeMap<String, FontBinding>, CoreError> {
    apply_draft_bindings(project, catalog, &retained.selectors)?;
    for binding in retained.local.values() {
        validate_binding(binding, catalog)?;
    }
    let component = match operation {
        crate::EditOperation::ComponentCreate { .. } => project.components.last_mut(),
        crate::EditOperation::ComponentUpdate { component_id, .. } => project
            .components
            .iter_mut()
            .find(|c| c.id == *component_id),
        _ => None,
    };
    let mut inherited = BTreeMap::new();
    if let Some(component) = component {
        for item in component.tracks.iter_mut().flat_map(|t| &mut t.items) {
            if let TimelineItem::Text(text) = item
                && let Some(binding) = retained.local.get(&text.id)
            {
                if text
                    .font_binding
                    .as_ref()
                    .is_some_and(|current| current != binding)
                {
                    return Err(crate::fonts::invalid(
                        "draft selector retention conflicts with an inherited binding; draft version 2 cannot represent this edit",
                    ));
                }
                text.font_binding = Some(binding.clone());
            }
            if let TimelineItem::Text(text) = item
                && retained.resolve.contains(&text.id)
                && let Some(binding) = text.font_binding.take()
            {
                inherited.insert(text.id.clone(), binding);
            }
        }
    }
    Ok(inherited)
}

pub(crate) fn validate_resolved_component_fonts(
    project: &Project,
    operation: &crate::EditOperation,
    inherited: &BTreeMap<String, FontBinding>,
) -> Result<(), CoreError> {
    if !inherited.is_empty() {
        let resolved = component_font_bindings(project, operation);
        if inherited
            .iter()
            .any(|(id, binding)| resolved.get(id) != Some(binding))
        {
            return Err(crate::fonts::invalid(
                "resolved component selector differs from its inherited binding; draft version 2 cannot represent this edit",
            ));
        }
    }
    Ok(())
}

mod matching;

pub(crate) struct DraftFontMatches {
    old: Vec<serde_json::Value>,
    new: Vec<serde_json::Value>,
    steps: Vec<BTreeMap<String, FontBinding>>,
    component_bindings: Vec<BTreeMap<String, FontBinding>>,
    assignment: matching::Assignment,
}

pub(crate) fn align_draft_font_steps(
    draft: &crate::EditDraft,
    operations: &[crate::EditOperation],
    component_bindings: Vec<BTreeMap<String, FontBinding>>,
) -> Result<DraftFontMatches, CoreError> {
    let steps = draft
        .font_steps
        .as_ref()
        .filter(|steps| steps.len() == draft.operations.len())
        .ok_or_else(|| crate::fonts::invalid("draft font steps are incomplete"))?
        .clone();
    let serialize = |operation: &crate::EditOperation| {
        serde_json::to_value(operation)
            .map_err(|_| crate::fonts::invalid("cannot compare draft operations"))
    };
    let old = draft
        .operations
        .iter()
        .map(serialize)
        .collect::<Result<Vec<_>, _>>()?;
    let new = operations
        .iter()
        .map(serialize)
        .collect::<Result<Vec<_>, _>>()?;
    let old_intents: Vec<_> = old.iter().map(font_intent).collect();
    let new_intents: Vec<_> = new.iter().map(font_intent).collect();
    // One structural match outweighs every possible additional intent match.
    let exact_weight = (new.len() + 2) as i64;
    let weights: Vec<Vec<_>> = new
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            old.iter()
                .enumerate()
                .map(|(previous, old_operation)| {
                    if operation == old_operation {
                        Some(exact_weight)
                    } else if intent_matches(
                        old_operation,
                        operation,
                        &old_intents[previous],
                        &new_intents[index],
                    ) {
                        Some(1)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    Ok(DraftFontMatches {
        old,
        new,
        steps,
        component_bindings,
        assignment: matching::optimal_assignments(&weights),
    })
}

impl DraftFontMatches {
    pub(crate) fn for_step(
        &self,
        index: usize,
        project: &Project,
        operation: &crate::EditOperation,
    ) -> Result<DraftFontRetention, CoreError> {
        let inherited = component_font_bindings(project, operation);
        let outcome = |previous: Option<usize>| {
            if let Some(previous) = previous {
                retention(
                    &self.old[previous],
                    &self.new[index],
                    &self.steps[previous],
                    &self.component_bindings[previous],
                    &inherited,
                )
            } else {
                DraftFontRetention {
                    resolve: component_texts(&self.new[index])
                        .keys()
                        .filter(|id| !inherited.contains_key(**id))
                        .map(|id| (*id).to_owned())
                        .collect(),
                    ..Default::default()
                }
            }
        };
        let expected = outcome(self.assignment.canonical[index]);
        if self.assignment.alternatives[index]
            .iter()
            .any(|&previous| outcome(previous) != expected)
        {
            return Err(crate::fonts::invalid(
                "ambiguous draft font intent; preserve distinct operations or change selectors",
            ));
        }
        Ok(expected)
    }
}

#[cfg(test)]
fn match_font_actions(
    edges: &[Vec<usize>],
    old_count: usize,
    outcome: impl Fn(usize, usize) -> DraftFontRetention,
    unmatched: impl Fn(usize) -> DraftFontRetention,
) -> Result<Vec<Option<usize>>, CoreError> {
    let weights: Vec<Vec<_>> = edges
        .iter()
        .map(|edges| {
            (0..old_count)
                .map(|old| edges.contains(&old).then_some(1))
                .collect()
        })
        .collect();
    let assignment = matching::optimal_assignments(&weights);
    for (index, alternatives) in assignment.alternatives.iter().enumerate() {
        let outcome = |previous: Option<usize>| {
            previous
                .map(|old| outcome(index, old))
                .unwrap_or_else(|| unmatched(index))
        };
        let expected = outcome(assignment.canonical[index]);
        if alternatives
            .iter()
            .any(|&previous| outcome(previous) != expected)
        {
            return Err(crate::fonts::invalid("ambiguous draft font intent"));
        }
    }
    Ok(assignment.canonical)
}
fn component_texts(operation: &serde_json::Value) -> BTreeMap<&str, &serde_json::Value> {
    operation
        .get("tracks")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|track| {
            track
                .get("items")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|item| item["type"] == "text")
        .filter_map(|item| item["id"].as_str().map(|id| (id, item)))
        .collect()
}

fn intent_matches(
    old: &serde_json::Value,
    new: &serde_json::Value,
    old_intent: &Option<serde_json::Value>,
    new_intent: &Option<serde_json::Value>,
) -> bool {
    if old_intent.is_none() || old_intent != new_intent {
        return false;
    }
    if new["operation"] == "component_create" {
        let old_texts = component_texts(old);
        component_texts(new)
            .keys()
            .any(|id| old_texts.contains_key(id))
    } else {
        true
    }
}

fn retention(
    old: &serde_json::Value,
    new: &serde_json::Value,
    step: &BTreeMap<String, FontBinding>,
    component_bindings: &BTreeMap<String, FontBinding>,
    inherited_bindings: &BTreeMap<String, FontBinding>,
) -> DraftFontRetention {
    if !matches!(
        new["operation"].as_str(),
        Some("component_create" | "component_update")
    ) {
        return DraftFontRetention {
            selectors: step.clone(),
            ..Default::default()
        };
    }
    let old_texts = component_texts(old);
    let mut resolve = BTreeSet::new();
    let local = component_texts(new)
        .into_iter()
        .filter_map(|(id, text)| {
            if old_texts.get(id).is_none_or(|previous| {
                ["fontPath", "fontFamily"]
                    .iter()
                    .any(|key| previous.get(*key) != text.get(*key))
            }) {
                resolve.insert(id.to_owned());
                return None;
            }
            component_bindings
                .get(id)
                // Known inherited bindings already produce the same outcome
                // without matching. Unresolved new bindings remain distinct.
                .filter(|binding| inherited_bindings.get(id) != Some(*binding))
                .map(|binding| (id.to_owned(), binding.clone()))
        })
        .collect();
    DraftFontRetention {
        selectors: BTreeMap::new(),
        local,
        resolve,
    }
}

fn font_intent(operation: &serde_json::Value) -> Option<serde_json::Value> {
    use serde_json::{Map, Value};
    let kind = operation.get("operation")?.as_str()?;
    let keys: &[&str] = match kind {
        "add_text" => &["operation", "trackId", "fontFamily", "fontPath"],
        "update_item" => &["operation", "itemId", "fontFamily", "fontPath"],
        "component_create" | "component_update" => &["operation", "componentId"],
        _ => return None,
    };
    let intent: Map<String, Value> = keys
        .iter()
        .filter_map(|key| operation.get(*key).map(|v| ((*key).into(), v.clone())))
        .collect();
    Some(Value::Object(intent))
}

fn unavailable(message: &str) -> CoreError {
    CoreError::new(ErrorCode::DependencyUnavailable, message)
}

#[cfg(test)]
mod matching_tests {
    use super::*;

    fn outcome(index: usize, previous: usize, mode: usize) -> DraftFontRetention {
        if mode >= 4 && (mode == 4 || previous.is_multiple_of(2)) {
            return DraftFontRetention {
                resolve: BTreeSet::from([index.to_string()]),
                ..Default::default()
            };
        }
        if mode == 0 || (mode == 3 && index == previous) {
            return DraftFontRetention::default();
        }
        let binding = FontBinding {
            profile: TEXT_LAYOUT_PROFILE.into(),
            regular: "same".into(),
            bold: if mode == 1 {
                "same".into()
            } else {
                previous.to_string()
            },
            italic: "same".into(),
            bold_italic: "same".into(),
            warnings: vec![],
        };
        DraftFontRetention {
            selectors: BTreeMap::from([("selector".into(), binding)]),
            ..Default::default()
        }
    }

    // Independent exhaustive oracle for every graph with three destinations and
    // three prior actions. Production uses alternating paths, not enumeration.
    fn assignments(
        edges: &[Vec<usize>],
        chosen: &mut Vec<Option<usize>>,
        all: &mut Vec<Vec<Option<usize>>>,
    ) {
        if chosen.len() == edges.len() {
            all.push(chosen.clone());
            return;
        }
        let index = chosen.len();
        chosen.push(None);
        assignments(edges, chosen, all);
        chosen.pop();
        for &previous in &edges[index] {
            if chosen.contains(&Some(previous)) {
                continue;
            }
            chosen.push(Some(previous));
            assignments(edges, chosen, all);
            chosen.pop();
        }
    }

    #[test]
    fn alternating_paths_agree_with_exhaustive_retention_outcomes() {
        for mask in 0..512 {
            let edges: Vec<Vec<usize>> = (0..3)
                .map(|index| {
                    (0..3)
                        .filter(|previous| mask & (1 << (index * 3 + previous)) != 0)
                        .collect()
                })
                .collect();
            let mut all = vec![];
            assignments(&edges, &mut vec![], &mut all);
            let size = all
                .iter()
                .map(|a| a.iter().flatten().count())
                .max()
                .unwrap();
            all.retain(|a| a.iter().flatten().count() == size);
            for mode in 0..6 {
                let unmatched = |index: usize| {
                    if mode >= 4 && index.is_multiple_of(2) {
                        outcome(index, 0, 4)
                    } else {
                        DraftFontRetention::default()
                    }
                };
                let outcomes = |assignment: &[Option<usize>]| -> Vec<_> {
                    assignment
                        .iter()
                        .enumerate()
                        .map(|(index, previous)| {
                            previous
                                .map(|p| outcome(index, p, mode))
                                .unwrap_or_else(|| unmatched(index))
                        })
                        .collect()
                };
                let expected = outcomes(&all[0]);
                let ambiguous = all.iter().any(|a| outcomes(a) != expected);
                let result = match_font_actions(&edges, 3, |i, p| outcome(i, p, mode), unmatched);
                assert_eq!(result.is_err(), ambiguous, "graph {mask}, mode {mode}");
                if let Ok(assignment) = result {
                    assert_eq!(assignment.iter().flatten().count(), size);
                    assert_eq!(outcomes(&assignment), expected);
                    let ordered = all
                        .iter()
                        .min_by_key(|a| a.iter().map(|p| p.unwrap_or(3)).collect::<Vec<_>>())
                        .unwrap();
                    assert_eq!(&assignment, ordered, "original order for graph {mask}");
                }
            }
        }
    }

    #[test]
    fn equivalent_actions_pair_in_original_order_at_edit_limit() {
        let edges = vec![(0..100).collect(); 100];
        assert_eq!(
            match_font_actions(
                &edges,
                100,
                |i, p| outcome(i, p, 1),
                |_| DraftFontRetention::default()
            )
            .unwrap(),
            (0..100).map(Some).collect::<Vec<_>>()
        );
    }
}

fn read_bounded(storage: &dyn Storage, path: &Path) -> Result<Vec<u8>, CoreError> {
    let mut bytes = Vec::new();
    storage
        .open_read(path)
        .map_err(|_| unavailable("font source cannot be opened"))?
        .take(MAX_FONT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable("font source cannot be read"))?;
    Ok(bytes)
}

fn reject_network_path(path: &Path) -> Result<(), CoreError> {
    let raw = path.to_string_lossy();
    let local_verbatim = matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), std::path::Prefix::VerbatimDisk(_)));
    if raw.contains("://") || (raw.starts_with("\\\\") && !local_verbatim) || raw.starts_with("//")
    {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "network fonts are not permitted",
        ));
    }
    Ok(())
}

fn safe_candidate(
    storage: &dyn Storage,
    roots: &[PathBuf],
    path: &Path,
) -> Result<Option<PathBuf>, CoreError> {
    reject_network_path(path)?;
    let raw = path.to_string_lossy();
    if path.components().any(|c| matches!(c, Component::ParentDir))
        || raw.split(['/', '\\']).any(|s| s == "..")
    {
        return Err(CoreError::new(
            ErrorCode::PathTraversal,
            "font selector contains traversal",
        ));
    }
    let candidates: Vec<_> = if path.is_absolute() {
        vec![path.to_owned()]
    } else {
        roots.iter().map(|r| r.join(path)).collect()
    };
    if path.is_absolute()
        && !roots.iter().any(|r| {
            path.starts_with(r)
                || storage
                    .canonicalize_storage_path(r)
                    .is_ok_and(|r| path.starts_with(r))
        })
    {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "font path is outside configured roots",
        ));
    }
    for candidate in candidates {
        let resolved = match storage.canonicalize_storage_path(&candidate) {
            Ok(path) => path,
            Err(_) => continue,
        };
        reject_network_path(&resolved)?;
        if !roots.iter().any(|root| {
            storage
                .canonicalize_storage_path(root)
                .is_ok_and(|r| resolved.starts_with(r))
        }) {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "font path escapes configured roots",
            ));
        }
        if storage.entry_kind(&resolved).ok() == Some(StorageEntryKind::File) {
            return Ok(Some(resolved));
        }
    }
    Ok(None)
}

fn candidates(
    storage: &dyn Storage,
    root: &Path,
    dir: &Path,
    depth: usize,
    count: &mut usize,
    output: &mut Vec<PathBuf>,
) -> Result<(), CoreError> {
    let mut entries = storage
        .list(dir)
        .map_err(|_| unavailable("font root cannot be enumerated"))?;
    entries.sort();
    for path in entries {
        *count += 1;
        if *count > MAX_FONT_CANDIDATES {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "font discovery exceeds 4096 candidates",
            ));
        }
        match storage
            .entry_kind(&path)
            .map_err(|_| unavailable("font candidate cannot be inspected"))?
        {
            StorageEntryKind::Directory => {
                if depth >= MAX_FONT_DIRECTORY_DEPTH {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "font discovery exceeds depth 8",
                    ));
                }
                candidates(storage, root, &path, depth + 1, count, output)?;
            }
            StorageEntryKind::File | StorageEntryKind::Symlink => {
                if let Some(path) = safe_candidate(storage, &[root.to_owned()], &path)? {
                    output.push(path);
                }
            }
            StorageEntryKind::Other => {}
        }
    }
    Ok(())
}

fn normalized(value: &str) -> String {
    value.to_lowercase().replace([' ', '-', '_'], "")
}

fn resolve(
    storage: &dyn Storage,
    config: &FontConfig,
    requested_path: Option<&str>,
    family: Option<&str>,
) -> Result<([Vec<u8>; 4], Vec<String>), CoreError> {
    for root in &config.roots {
        reject_network_path(root)?;
    }
    let mut warnings = vec![];
    let mut base = None;
    if let Some(path) = requested_path {
        base = safe_candidate(storage, &config.roots, Path::new(path))?;
        if base.is_none() {
            warnings.push("Requested font path unavailable; pinned fallback selected".into());
        }
    }
    if base.is_none()
        && let Some(family) = family
    {
        for root in &config.roots {
            if !storage.storage_path_exists(root) {
                continue;
            }
            let mut files = vec![];
            candidates(storage, root, root, 0, &mut 0, &mut files)?;
            files.sort();
            base = files.into_iter().find(|p| {
                p.file_stem().and_then(|s| s.to_str()).is_some_and(|s| {
                    normalized(s) == normalized(family)
                        || normalized(s) == format!("{}regular", normalized(family))
                })
            });
            if base.is_some() {
                break;
            }
        }
        if base.is_none() {
            warnings.push("Requested font family unavailable; pinned default selected".into());
        }
    }
    if base.is_none()
        && let Some(default) = &config.default_path
    {
        base = safe_candidate(storage, &config.roots, default)?;
        if base.is_none() {
            return Err(unavailable("configured default font unavailable"));
        }
    }
    let Some(base) = base else {
        return Ok((DEFAULT_FACES.map(|b| b.to_vec()), warnings));
    };
    let parent = base
        .parent()
        .ok_or_else(|| unavailable("font has no parent directory"))?;
    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| unavailable("invalid font filename"))?
        .trim_end_matches("-Regular")
        .trim_end_matches("Regular");
    let ext = base
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| unavailable("font filename needs extension"))?;
    let mut faces = [read_bounded(storage, &base)?, vec![], vec![], vec![]];
    for (index, suffixes) in [
        &["-Bold", "Bold", "bd", "b"][..],
        &["-Italic", "-Oblique", "Italic", "i"][..],
        &["-BoldItalic", "-BoldOblique", "BoldItalic", "bi"][..],
    ]
    .iter()
    .enumerate()
    {
        let mut selected = None;
        for suffix in *suffixes {
            if let Some(path) = safe_candidate(
                storage,
                &config.roots,
                &parent.join(format!("{stem}{suffix}.{ext}")),
            )? {
                selected = Some(path);
                break;
            }
        }
        faces[index + 1] = read_bounded(
            storage,
            &selected.ok_or_else(|| {
                unavailable("selected family requires regular, bold, italic and bold-italic faces")
            })?,
        )?;
    }
    Ok((faces, warnings))
}

pub(crate) fn managed_bytes(
    storage: &dyn Storage,
    dir: &Path,
    face: &FontRecord,
) -> Result<Vec<u8>, CoreError> {
    let catalog = BTreeMap::from([(face.sha256.clone(), face.clone())]);
    validate_catalog(&catalog)?;
    let root = storage
        .canonicalize_storage_path(dir)
        .map_err(|_| unavailable("project directory unavailable"))?;
    let path = storage
        .canonicalize_storage_path(&dir.join(&face.relative_path))
        .map_err(|_| {
            CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "managed font reference is missing",
            )
        })?;
    if !path.starts_with(&root) {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "managed font escapes project",
        ));
    }
    let bytes = read_bounded(storage, &path).map_err(|_| {
        CoreError::new(
            ErrorCode::AssetIntegrityFailed,
            "managed font cannot be read",
        )
    })?;
    verify_bytes(face, &bytes)?;
    Ok(bytes)
}

pub(crate) fn prepare_fonts(
    storage: &dyn Storage,
    dir: &Path,
    project: &mut Project,
    config: &FontConfig,
    staged: &mut FontBytes,
) -> Result<(), CoreError> {
    validate_catalog(&project.fonts)?;
    // Verify retained bytes before pruning: an unused catalog entry is still an
    // integrity claim made by the source snapshot.
    for (hash, face) in &project.fonts {
        if let Some(bytes) = staged.get(hash) {
            if bytes.len() as u64 != face.size_bytes {
                return Err(CoreError::new(
                    ErrorCode::AssetIntegrityFailed,
                    "font snapshots disagree on byte length",
                ));
            }
        } else {
            staged.insert(hash.clone(), managed_bytes(storage, dir, face)?);
        }
    }
    let mut used = BTreeSet::new();
    let mut resolved = BTreeMap::<String, FontBinding>::new();
    for item in project
        .tracks
        .iter_mut()
        .chain(project.components.iter_mut().flat_map(|c| &mut c.tracks))
        .flat_map(|t| &mut t.items)
    {
        let TimelineItem::Text(text) = item else {
            continue;
        };
        if text.font_binding.is_none() {
            text.font_binding = resolved.get(&selector(text)).cloned();
        }
        if text.font_binding.is_none() {
            let (faces, warnings) = resolve(
                storage,
                config,
                text.font_path.as_deref(),
                text.font_family.as_deref(),
            )?;
            let mut hashes = vec![];
            for bytes in faces {
                let face = record(&bytes)?;
                hashes.push(face.sha256.clone());
                staged.entry(face.sha256.clone()).or_insert(bytes);
                project.fonts.insert(face.sha256.clone(), face);
            }
            text.font_binding = Some(FontBinding {
                profile: TEXT_LAYOUT_PROFILE.into(),
                regular: hashes[0].clone(),
                bold: hashes[1].clone(),
                italic: hashes[2].clone(),
                bold_italic: hashes[3].clone(),
                warnings,
            });
            resolved.insert(
                selector(text),
                text.font_binding.clone().expect("resolved font"),
            );
        }
        let binding = text.font_binding.as_ref().expect("binding prepared");
        validate_binding(binding, &project.fonts)?;
        used.extend(binding.hashes().map(str::to_owned));
    }
    project.fonts.retain(|hash, _| used.contains(hash));
    validate_catalog(&project.fonts)?;
    for (hash, face) in &project.fonts {
        if let Some(bytes) = staged.get(hash) {
            if bytes.len() as u64 != face.size_bytes {
                return Err(CoreError::new(
                    ErrorCode::AssetIntegrityFailed,
                    "font snapshots disagree on byte length",
                ));
            }
        } else {
            staged.insert(hash.clone(), managed_bytes(storage, dir, face)?);
        }
    }
    Ok(())
}

pub(crate) fn publish_fonts(
    storage: &dyn Storage,
    dir: &Path,
    staged: &FontBytes,
) -> Result<(), CoreError> {
    if staged.is_empty() {
        return Ok(());
    }
    let folder = dir.join("fonts");
    storage
        .create_dir_all(&folder)
        .map_err(|e| CoreError::io("cannot create managed fonts", e))?;
    let root = storage
        .canonicalize_storage_path(dir)
        .map_err(|e| CoreError::io("cannot resolve project", e))?;
    let canonical = storage
        .canonicalize_storage_path(&folder)
        .map_err(|e| CoreError::io("cannot resolve fonts", e))?;
    if !canonical.starts_with(root) {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "managed font folder escapes project",
        ));
    }
    for bytes in staged.values() {
        let face = record(bytes)?;
        let path = dir.join(&face.relative_path);
        if storage.storage_path_exists(&path) {
            managed_bytes(storage, dir, &face)?;
        } else {
            storage
                .atomic_replace(&path, bytes)
                .map_err(|e| CoreError::io("cannot publish managed font", e))?;
        }
    }
    Ok(())
}

pub(crate) fn verify_project_fonts(
    storage: &dyn Storage,
    dir: &Path,
    project: &Project,
) -> Result<(), CoreError> {
    validate_catalog(&project.fonts)?;
    for item in project
        .tracks
        .iter()
        .chain(project.components.iter().flat_map(|c| &c.tracks))
        .flat_map(|t| &t.items)
    {
        if let TimelineItem::Text(text) = item {
            let binding = text.font_binding.as_ref().ok_or_else(|| {
                CoreError::new(
                    ErrorCode::AssetIntegrityFailed,
                    "text has no retained font binding",
                )
            })?;
            validate_binding(binding, &project.fonts)?;
        }
    }
    for face in project.fonts.values() {
        managed_bytes(storage, dir, face)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::FileSystemStorage;

    #[test]
    fn relative_selectors_cannot_access_network_roots() {
        for root in [
            "//server/share",
            "\\\\server\\share",
            "https://example.invalid/fonts",
        ] {
            let config = FontConfig {
                roots: vec![root.into()],
                default_path: None,
            };
            assert_eq!(
                resolve(
                    &FileSystemStorage,
                    &config,
                    Some("Family.ttf"),
                    Some("Family")
                )
                .unwrap_err()
                .code,
                ErrorCode::PathNotAllowed
            );
        }
    }

    #[test]
    fn selector_precedence_root_order_and_fallback_are_deterministic() {
        let root = tempfile::tempdir().unwrap();
        let roots: Vec<_> = ["first", "second"]
            .map(|name| root.path().join(name))
            .into();
        for (index, dir) in roots.iter().enumerate() {
            std::fs::create_dir(dir).unwrap();
            for (name, bytes) in [
                "Family.ttf",
                "Family-Bold.ttf",
                "Family-Italic.ttf",
                "Family-BoldItalic.ttf",
            ]
            .into_iter()
            .zip(DEFAULT_FACES)
            {
                std::fs::write(
                    dir.join(name),
                    if name == "Family.ttf" {
                        DEFAULT_FACES[index]
                    } else {
                        bytes
                    },
                )
                .unwrap();
            }
        }
        let mut config = FontConfig {
            roots,
            default_path: None,
        };
        let (family, warnings) =
            resolve(&FileSystemStorage, &config, None, Some("Fa-mi_ly")).unwrap();
        assert_eq!(family[0], DEFAULT_FACES[0]);
        assert!(warnings.is_empty());
        let explicit = config.roots[1].join("Family.ttf");
        let (path, _) = resolve(
            &FileSystemStorage,
            &config,
            explicit.to_str(),
            Some("Family"),
        )
        .unwrap();
        assert_eq!(path[0], DEFAULT_FACES[1]);
        config.default_path = Some(explicit);
        let (default, warnings) = resolve(
            &FileSystemStorage,
            &config,
            Some("missing.ttf"),
            Some("missing family"),
        )
        .unwrap();
        assert_eq!(default[0], DEFAULT_FACES[1]);
        assert_eq!(warnings.len(), 2);
        config.default_path = None;
        assert_eq!(
            resolve(&FileSystemStorage, &config, None, None).unwrap().0,
            DEFAULT_FACES.map(|b| b.to_vec())
        );
    }

    #[test]
    fn discovery_count_and_depth_limits_are_inclusive() {
        let root = tempfile::tempdir().unwrap();
        for index in 0..MAX_FONT_CANDIDATES {
            std::fs::write(root.path().join(format!("{index:04}.ttf")), []).unwrap();
        }
        let mut files = vec![];
        candidates(
            &FileSystemStorage,
            root.path(),
            root.path(),
            0,
            &mut 0,
            &mut files,
        )
        .unwrap();
        assert_eq!(files.len(), MAX_FONT_CANDIDATES);
        assert!(files.windows(2).all(|pair| pair[0] < pair[1]));
        std::fs::write(root.path().join("overflow.ttf"), []).unwrap();
        assert_eq!(
            candidates(
                &FileSystemStorage,
                root.path(),
                root.path(),
                0,
                &mut 0,
                &mut vec![]
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        let root = tempfile::tempdir().unwrap();
        let mut dir = root.path().to_owned();
        for _ in 0..MAX_FONT_DIRECTORY_DEPTH {
            dir = dir.join("nested");
            std::fs::create_dir(&dir).unwrap();
        }
        candidates(
            &FileSystemStorage,
            root.path(),
            root.path(),
            0,
            &mut 0,
            &mut vec![],
        )
        .unwrap();
        std::fs::create_dir(dir.join("overflow")).unwrap();
        assert_eq!(
            candidates(
                &FileSystemStorage,
                root.path(),
                root.path(),
                0,
                &mut 0,
                &mut vec![]
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }
}
