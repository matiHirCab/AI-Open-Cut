//! Render workspace and artifact publication owner.

use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    CoreError, ErrorCode,
    evaluated_scene::{
        EvaluatedKeyframeValue, EvaluatedMediaKind, EvaluatedProperty, EvaluatedSceneResult,
        EvaluatedVisualSource, FontResourceBinding,
    },
    render_plan::{MediaInputRequest, PreparedText},
};

#[cfg(test)]
use crate::{KeyframeProperty, KeyframeValue};

pub(crate) const PUBLISH_STAGE: &str = "publish";
pub(crate) const GRAPH_BUILD_STAGE: &str = "graph_build";

pub(crate) trait ArtifactIo: Debug + Send + Sync {
    fn request_id(&self) -> String;
    fn create_dir(&self, path: &Path) -> std::io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>>;
    fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()>;
    fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;
    fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind>;
    fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf>;
    fn artifact_path_exists(&self, path: &Path) -> bool;
    fn remove(&self, path: &Path) -> std::io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn size(&self, path: &Path) -> std::io::Result<u64>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArtifactEntryKind {
    File,
    Directory,
    Other,
}

pub(crate) struct PreparedRenderResources {
    pub(crate) media_inputs: Vec<MediaInputRequest>,
    pub(crate) media_paths: Vec<PathBuf>,
    pub(crate) text_layers: HashMap<String, PreparedText>,
}

pub(crate) struct PreparedMediaResources {
    pub(crate) media_inputs: Vec<MediaInputRequest>,
    pub(crate) media_paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileSystemArtifactIo;

impl ArtifactIo for FileSystemArtifactIo {
    fn request_id(&self) -> String {
        env::var("OPENCUT_REQUEST_ID")
            .ok()
            .filter(|value| valid_request_id(value))
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    }
    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir(path)
    }
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(path)
    }
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }
    fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }
    fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        std::fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect()
    }
    fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind> {
        let file_type = std::fs::metadata(path)?.file_type();
        Ok(if file_type.is_file() {
            ArtifactEntryKind::File
        } else if file_type.is_dir() {
            ArtifactEntryKind::Directory
        } else {
            ArtifactEntryKind::Other
        })
    }
    fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf> {
        path.canonicalize()
    }
    fn artifact_path_exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)
    }
    fn size(&self, path: &Path) -> std::io::Result<u64> {
        Ok(path.metadata()?.len())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderArtifact {
    pub relative_path: String,
    pub mime_type: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
}

pub(crate) struct RenderWorkspace {
    path: PathBuf,
    io: Arc<dyn ArtifactIo>,
}

impl Drop for RenderWorkspace {
    fn drop(&mut self) {
        let _ = self.io.remove_dir_all(&self.path);
    }
}

fn valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

pub(crate) fn temporary_output(io: &dyn ArtifactIo, parent: &Path, extension: &str) -> PathBuf {
    let request_id = io.request_id();
    parent.join(format!(".opencut-{request_id}.{extension}"))
}

impl RenderWorkspace {
    pub(crate) fn create(io: Arc<dyn ArtifactIo>, project_dir: &Path) -> Result<Self, CoreError> {
        let request_id = io.request_id();
        let path = project_dir.join(format!(".opencut-work-{request_id}"));
        io.create_dir(&path)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        Ok(Self { path, io })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
pub(crate) fn prepare_text_layers(
    io: &dyn ArtifactIo,
    text_resources: &[&crate::TextItem],
    workspace: &Path,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Result<HashMap<String, PreparedText>, CoreError> {
    let mut result = HashMap::new();
    for text in text_resources {
        let path = workspace.join(format!("text-{}.txt", text.id));
        let font_path = resolve_text_font(io, text, default_font_path, font_roots, warnings);
        let content = wrap_text_with_io(
            io,
            &text.text,
            text.style.wrap_width_px,
            text.font_size,
            font_path.as_deref(),
        );
        io.write(&path, content.as_bytes())
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        let metrics = measure_text_block(io, &content, text.font_size, font_path.as_deref());
        let outline = text.style.outline_width_px;
        let shadow_left =
            text.style.shadow.offset_x.unsigned_abs() * u32::from(text.style.shadow.offset_x < 0);
        let shadow_right = text.style.shadow.offset_x.max(0) as u32;
        let shadow_top =
            text.style.shadow.offset_y.unsigned_abs() * u32::from(text.style.shadow.offset_y < 0);
        let shadow_bottom = text.style.shadow.offset_y.max(0) as u32;
        let text_x = text
            .style
            .padding
            .left
            .saturating_add(outline)
            .saturating_add(shadow_left);
        let text_y = text
            .style
            .padding
            .top
            .saturating_add(outline)
            .saturating_add(shadow_top);
        let layer_width = text_x
            .saturating_add(metrics.width.ceil() as u32)
            .saturating_add(text.style.padding.right)
            .saturating_add(outline)
            .saturating_add(shadow_right)
            .saturating_add(2)
            .max(1);
        let line_spacing = text
            .style
            .line_spacing_px
            .saturating_mul(metrics.line_count.saturating_sub(1) as i32);
        let text_height = (metrics.height + f64::from(line_spacing)).max(1.0);
        let layer_height = text_y
            .saturating_add(text_height.ceil() as u32)
            .saturating_add(text.style.padding.bottom)
            .saturating_add(outline)
            .saturating_add(shadow_bottom)
            .saturating_add(2)
            .max(1);
        let maximum_scale = text
            .keyframes
            .iter()
            .filter_map(|keyframe| match (keyframe.property, &keyframe.value) {
                (KeyframeProperty::Scale, KeyframeValue::Scalar { value }) => Some(*value),
                _ => None,
            })
            .fold(text.transform.scale, f64::max)
            .max(0.01);
        let canvas_width = ((f64::from(layer_width) * maximum_scale).ceil() as u32)
            .saturating_add(2)
            .max(1);
        let canvas_height = ((f64::from(layer_height) * maximum_scale).ceil() as u32)
            .saturating_add(2)
            .max(1);
        result.insert(
            text.id.clone(),
            PreparedText {
                file_path: path,
                font_path,
                layer_width,
                layer_height,
                canvas_width,
                canvas_height,
                text_x,
                text_y,
            },
        );
    }
    Ok(result)
}

pub(crate) fn prepare_render_resources(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    media: PreparedMediaResources,
    workspace: &Path,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Result<PreparedRenderResources, CoreError> {
    let text_layers = prepare_evaluated_text_layers(
        io,
        evaluated,
        workspace,
        default_font_path,
        font_roots,
        warnings,
    )?;
    Ok(PreparedRenderResources {
        media_inputs: media.media_inputs,
        media_paths: media.media_paths,
        text_layers,
    })
}

pub(crate) fn prepare_media_resources(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    project_dir: &Path,
) -> Result<PreparedMediaResources, CoreError> {
    let media_inputs = media_input_requests(evaluated)?;
    validate_font_resource_bindings(evaluated)?;
    let binding_by_asset = evaluated
        .resource_bindings
        .media
        .iter()
        .map(|binding| {
            (
                binding.asset_id.as_str(),
                binding.project_relative_path.as_str(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut media_paths = Vec::with_capacity(media_inputs.len());
    for input in &media_inputs {
        let relative = binding_by_asset
            .get(input.asset_id.as_str())
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::InternalError,
                    "evaluated media resource binding is missing",
                )
            })?;
        media_paths.push(resolve_project_asset(io, project_dir, Path::new(relative))?);
    }
    Ok(PreparedMediaResources {
        media_inputs,
        media_paths,
    })
}

fn validate_font_resource_bindings(evaluated: &EvaluatedSceneResult) -> Result<(), CoreError> {
    let font_bindings = evaluated
        .resource_bindings
        .fonts
        .iter()
        .map(|binding| binding.font_resource_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    for layer in &evaluated.scene.visual_layers {
        let EvaluatedVisualSource::Text(text) = &layer.source else {
            continue;
        };
        if text
            .font_resource_id
            .as_deref()
            .is_some_and(|id| !font_bindings.contains(id))
        {
            return Err(CoreError::new(
                ErrorCode::InternalError,
                "evaluated font resource binding is missing",
            ));
        }
    }
    Ok(())
}

pub(crate) fn media_input_requests(
    evaluated: &EvaluatedSceneResult,
) -> Result<Vec<MediaInputRequest>, CoreError> {
    let kind_by_asset = evaluated
        .scene
        .resources
        .iter()
        .map(|resource| (resource.asset_id.as_str(), resource.kind))
        .collect::<HashMap<_, _>>();
    let mut instances = evaluated
        .scene
        .visual_layers
        .iter()
        .filter_map(|layer| match &layer.source {
            EvaluatedVisualSource::Media {
                asset_id,
                source_in_ms,
            } => Some((
                layer.order,
                layer.item_id.as_str(),
                asset_id.as_str(),
                *source_in_ms,
                layer.span.end_ms - layer.span.start_ms,
            )),
            _ => None,
        })
        .chain(evaluated.scene.audio_layers.iter().map(|layer| {
            (
                layer.order,
                layer.item_id.as_str(),
                layer.asset_id.as_str(),
                layer.source_in_ms,
                layer.span.end_ms - layer.span.start_ms,
            )
        }))
        .collect::<Vec<_>>();
    instances.sort_by_key(|instance| instance.0);
    instances.dedup_by(|left, right| left.1 == right.1);

    let binding_by_asset = evaluated
        .resource_bindings
        .media
        .iter()
        .map(|binding| {
            (
                binding.asset_id.as_str(),
                binding.project_relative_path.as_str(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut media_inputs = Vec::with_capacity(instances.len());
    for (_, item_id, asset_id, source_in_ms, duration_ms) in instances {
        let relative = binding_by_asset.get(asset_id).ok_or_else(|| {
            CoreError::new(
                ErrorCode::InternalError,
                "evaluated media resource binding is missing",
            )
        })?;
        let kind = kind_by_asset.get(asset_id).ok_or_else(|| {
            CoreError::new(
                ErrorCode::InternalError,
                "evaluated media resource metadata is missing",
            )
        })?;
        let project_relative_path = PathBuf::from(relative);
        validate_project_relative_path(&project_relative_path)?;
        media_inputs.push(MediaInputRequest {
            item_id: item_id.to_owned(),
            asset_id: asset_id.to_owned(),
            project_relative_path,
            media_type: match kind {
                EvaluatedMediaKind::Image => crate::MediaType::Image,
                EvaluatedMediaKind::Video => crate::MediaType::Video,
                EvaluatedMediaKind::Audio => crate::MediaType::Audio,
            },
            source_in_ms,
            duration_ms,
            input_index: media_inputs.len() + 2,
        });
    }
    Ok(media_inputs)
}

fn prepare_evaluated_text_layers(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    workspace: &Path,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Result<HashMap<String, PreparedText>, CoreError> {
    let font_bindings = evaluated
        .resource_bindings
        .fonts
        .iter()
        .map(|binding| (binding.font_resource_id.as_str(), binding))
        .collect::<HashMap<_, _>>();
    let mut result = HashMap::new();
    for layer in &evaluated.scene.visual_layers {
        let EvaluatedVisualSource::Text(text) = &layer.source else {
            continue;
        };
        let binding = text
            .font_resource_id
            .as_deref()
            .map(|id| {
                font_bindings.get(id).copied().ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::InternalError,
                        "evaluated font resource binding is missing",
                    )
                })
            })
            .transpose()?;
        let path = workspace.join(format!("text-{}.txt", layer.item_id));
        let font_path = resolve_evaluated_font(
            io,
            &layer.item_id,
            binding,
            default_font_path,
            font_roots,
            warnings,
        );
        let content = wrap_text_with_io(
            io,
            &text.text,
            text.style.wrap_width_px,
            text.font_size,
            font_path.as_deref(),
        );
        io.write(&path, content.as_bytes())
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        let metrics = measure_text_block(io, &content, text.font_size, font_path.as_deref());
        let outline = text.style.outline_width_px;
        let shadow_left =
            text.style.shadow.offset_x.unsigned_abs() * u32::from(text.style.shadow.offset_x < 0);
        let shadow_right = text.style.shadow.offset_x.max(0) as u32;
        let shadow_top =
            text.style.shadow.offset_y.unsigned_abs() * u32::from(text.style.shadow.offset_y < 0);
        let shadow_bottom = text.style.shadow.offset_y.max(0) as u32;
        let text_x = text
            .style
            .padding
            .left
            .saturating_add(outline)
            .saturating_add(shadow_left);
        let text_y = text
            .style
            .padding
            .top
            .saturating_add(outline)
            .saturating_add(shadow_top);
        let layer_width = text_x
            .saturating_add(metrics.width.ceil() as u32)
            .saturating_add(text.style.padding.right)
            .saturating_add(outline)
            .saturating_add(shadow_right)
            .saturating_add(2)
            .max(1);
        let line_spacing = text
            .style
            .line_spacing_px
            .saturating_mul(metrics.line_count.saturating_sub(1) as i32);
        let text_height = (metrics.height + f64::from(line_spacing)).max(1.0);
        let layer_height = text_y
            .saturating_add(text_height.ceil() as u32)
            .saturating_add(text.style.padding.bottom)
            .saturating_add(outline)
            .saturating_add(shadow_bottom)
            .saturating_add(2)
            .max(1);
        let maximum_scale = layer
            .keyframes
            .iter()
            .filter_map(|keyframe| match (keyframe.property, keyframe.value) {
                (EvaluatedProperty::Scale, EvaluatedKeyframeValue::Scalar { value }) => Some(value),
                _ => None,
            })
            .fold(layer.transform.scale, f64::max)
            .max(0.01);
        result.insert(
            layer.item_id.clone(),
            PreparedText {
                file_path: path,
                font_path,
                layer_width,
                layer_height,
                canvas_width: ((f64::from(layer_width) * maximum_scale).ceil() as u32)
                    .saturating_add(2)
                    .max(1),
                canvas_height: ((f64::from(layer_height) * maximum_scale).ceil() as u32)
                    .saturating_add(2)
                    .max(1),
                text_x,
                text_y,
            },
        );
    }
    Ok(result)
}

fn resolve_evaluated_font(
    io: &dyn ArtifactIo,
    item_id: &str,
    binding: Option<&FontResourceBinding>,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Option<PathBuf> {
    if let Some(requested) = binding.and_then(|value| value.requested_path.as_deref()) {
        let requested = PathBuf::from(requested);
        let candidates = if requested.is_absolute() {
            vec![requested]
        } else {
            font_roots
                .iter()
                .map(|root| root.join(&requested))
                .collect()
        };
        for candidate in candidates {
            if let Ok(resolved) = io.canonicalize_artifact_path(&candidate)
                && io.entry_kind(&resolved).ok() == Some(ArtifactEntryKind::File)
                && font_roots
                    .iter()
                    .filter_map(|root| io.canonicalize_artifact_path(root).ok())
                    .any(|root| resolved.starts_with(root))
            {
                return Some(resolved);
            }
        }
        warnings.push(format!(
            "Text item {item_id} requested a font path that could not be resolved; using fallback"
        ));
    }
    if let Some(family) = binding.and_then(|value| value.requested_family.as_deref()) {
        let needle = family.to_lowercase().replace([' ', '-', '_'], "");
        for root in font_roots {
            if let Some(path) = find_font_file(io, root, &needle) {
                return Some(path);
            }
        }
        warnings.push(format!(
            "Text item {item_id} requested font family {family:?} that could not be resolved; using fallback"
        ));
    }
    default_font_path.map(Path::to_path_buf)
}

pub(crate) fn write_filter_script(
    io: &dyn ArtifactIo,
    path: &Path,
    contents: &str,
) -> Result<(), CoreError> {
    io.write(path, contents.as_bytes())
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))
}

#[cfg(test)]
fn resolve_text_font(
    io: &dyn ArtifactIo,
    text: &crate::TextItem,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Option<PathBuf> {
    if let Some(requested) = text.font_path.as_deref() {
        let requested = PathBuf::from(requested);
        let candidates = if requested.is_absolute() {
            vec![requested]
        } else {
            font_roots
                .iter()
                .map(|root| root.join(&requested))
                .collect()
        };
        for candidate in candidates {
            if let Ok(resolved) = io.canonicalize_artifact_path(&candidate)
                && io.entry_kind(&resolved).ok() == Some(ArtifactEntryKind::File)
                && font_roots
                    .iter()
                    .filter_map(|root| io.canonicalize_artifact_path(root).ok())
                    .any(|root| resolved.starts_with(root))
            {
                return Some(resolved);
            }
        }
        warnings.push(format!(
            "Text item {} requested a font path that could not be resolved; using fallback",
            text.id
        ));
    }
    if let Some(family) = text.font_family.as_deref() {
        let needle = family.to_lowercase().replace([' ', '-', '_'], "");
        for root in font_roots {
            if let Some(path) = find_font_file(io, root, &needle) {
                return Some(path);
            }
        }
        warnings.push(format!("Text item {} requested font family {family:?} that could not be resolved; using fallback", text.id));
    }
    default_font_path.map(Path::to_path_buf)
}

struct TextMetrics {
    width: f64,
    height: f64,
    line_count: usize,
}

#[cfg(test)]
pub(crate) fn wrap_text(
    text: &str,
    width_px: Option<u32>,
    font_size: u32,
    font_path: Option<&Path>,
) -> String {
    wrap_text_with_io(&FileSystemArtifactIo, text, width_px, font_size, font_path)
}

fn wrap_text_with_io(
    io: &dyn ArtifactIo,
    text: &str,
    width_px: Option<u32>,
    font_size: u32,
    font_path: Option<&Path>,
) -> String {
    let Some(width_px) = width_px else {
        return text.to_owned();
    };
    let font_data = font_path.and_then(|path| io.read(path).ok());
    let face = font_data
        .as_deref()
        .and_then(|data| ttf_parser::Face::parse(data, 0).ok());
    wrap_text_with_measure(text, f64::from(width_px), |value| {
        measure_text_run(value, font_size, face.as_ref())
    })
}

pub(crate) fn wrap_text_with_measure(
    text: &str,
    maximum_width: f64,
    measure: impl Fn(&str) -> f64,
) -> String {
    text.split('\n')
        .map(|line| {
            let mut lines = Vec::new();
            let mut current = String::new();
            for word in line.split_whitespace() {
                let proposed = if current.is_empty() {
                    word.to_owned()
                } else {
                    format!("{current} {word}")
                };
                if measure(&proposed) <= maximum_width {
                    current = proposed;
                    continue;
                }
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                }
                if measure(word) <= maximum_width {
                    current.push_str(word);
                    continue;
                }
                for character in word.chars() {
                    let mut candidate = current.clone();
                    candidate.push(character);
                    if !current.is_empty() && measure(&candidate) > maximum_width {
                        lines.push(std::mem::take(&mut current));
                    }
                    current.push(character);
                }
            }
            lines.push(current);
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn measure_text_block(
    io: &dyn ArtifactIo,
    text: &str,
    font_size: u32,
    font_path: Option<&Path>,
) -> TextMetrics {
    let font_data = font_path.and_then(|path| io.read(path).ok());
    let face = font_data
        .as_deref()
        .and_then(|data| ttf_parser::Face::parse(data, 0).ok());
    let line_count = text.split('\n').count().max(1);
    let width = text
        .split('\n')
        .map(|line| measure_text_run(line, font_size, face.as_ref()))
        .fold(0.0_f64, f64::max);
    let line_height = face.as_ref().map_or(f64::from(font_size) * 1.2, |face| {
        let units = f64::from(face.units_per_em());
        let height = f64::from(face.ascender() - face.descender() + face.line_gap());
        (height / units * f64::from(font_size)).max(1.0)
    });
    TextMetrics {
        width,
        height: line_height * line_count as f64,
        line_count,
    }
}

fn measure_text_run(text: &str, font_size: u32, face: Option<&ttf_parser::Face<'_>>) -> f64 {
    let Some(face) = face else {
        return text.chars().count() as f64 * f64::from(font_size) * 0.6;
    };
    let units = f64::from(face.units_per_em());
    let fallback = face
        .glyph_index('\u{fffd}')
        .or_else(|| face.glyph_index('?'));
    let advance = text
        .chars()
        .map(|character| {
            face.glyph_index(character)
                .or(fallback)
                .and_then(|glyph| face.glyph_hor_advance(glyph))
                .map_or(units * 0.6, f64::from)
        })
        .sum::<f64>();
    advance / units * f64::from(font_size)
}

fn find_font_file(io: &dyn ArtifactIo, root: &Path, normalized_family: &str) -> Option<PathBuf> {
    let entries = io.list(root).ok()?;
    for path in entries {
        if io.entry_kind(&path).ok() == Some(ArtifactEntryKind::Directory) {
            if let Some(found) = find_font_file(io, &path, normalized_family) {
                return Some(found);
            }
        } else if io.entry_kind(&path).ok() == Some(ArtifactEntryKind::File)
            && matches!(
                path.extension()
                    .and_then(|value| value.to_str())
                    .map(str::to_ascii_lowercase)
                    .as_deref(),
                Some("ttf" | "otf" | "ttc")
            )
        {
            let stem = path
                .file_stem()?
                .to_string_lossy()
                .to_lowercase()
                .replace([' ', '-', '_'], "");
            if stem.contains(normalized_family) {
                return io.canonicalize_artifact_path(&path).ok();
            }
        }
    }
    None
}

#[cfg(test)]
pub(crate) fn publish_output(
    temporary: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), CoreError> {
    publish_output_with(&FileSystemArtifactIo, temporary, output, overwrite)
}

pub(crate) fn publish_output_with(
    io: &dyn ArtifactIo,
    temporary: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), CoreError> {
    if io.artifact_path_exists(output) {
        if !overwrite {
            let _ = io.remove(temporary);
            return Err(CoreError::new(
                ErrorCode::ExportExists,
                "export already exists; pass overwrite=true only with explicit permission",
            ));
        }
        if io.remove(output).is_err() {
            let _ = io.remove(temporary);
            return Err(CoreError::render_failure(PUBLISH_STAGE, None, None));
        }
    }
    io.rename(temporary, output).map_err(|_| {
        let _ = io.remove(temporary);
        CoreError::render_failure(PUBLISH_STAGE, None, None)
    })
}

pub(crate) fn resolve_project_asset(
    io: &dyn ArtifactIo,
    project_dir: &Path,
    relative: &Path,
) -> Result<PathBuf, CoreError> {
    validate_project_relative_path(relative)?;
    let root = io
        .canonicalize_artifact_path(project_dir)
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
    let resolved = io
        .canonicalize_artifact_path(&project_dir.join(relative))
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
    if !resolved.starts_with(root) {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "project asset escapes the project directory",
        ));
    }
    Ok(resolved)
}

fn validate_project_relative_path(relative: &Path) -> Result<(), CoreError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "project asset path is not allowed",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn artifact(
    path: &Path,
    relative_path: String,
    mime_type: &str,
    warnings: Vec<String>,
) -> Result<RenderArtifact, CoreError> {
    artifact_with(
        &FileSystemArtifactIo,
        path,
        relative_path,
        mime_type,
        warnings,
    )
}

pub(crate) fn artifact_with(
    io: &dyn ArtifactIo,
    path: &Path,
    relative_path: String,
    mime_type: &str,
    warnings: Vec<String>,
) -> Result<RenderArtifact, CoreError> {
    let size_bytes = match io.size(path) {
        Ok(size) => size,
        Err(_) => {
            let _ = io.remove(path);
            return Err(CoreError::render_failure(PUBLISH_STAGE, None, None));
        }
    };
    if size_bytes == 0 {
        let _ = io.remove(path);
        return Err(CoreError::render_failure(PUBLISH_STAGE, None, None));
    }
    Ok(RenderArtifact {
        relative_path,
        mime_type: mime_type.into(),
        size_bytes,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FailingIo;
    impl ArtifactIo for FailingIo {
        fn request_id(&self) -> String {
            "injected".into()
        }
        fn create_dir(&self, path: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.create_dir(path)
        }
        fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.remove_dir_all(path)
        }
        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            FileSystemArtifactIo.read(path)
        }
        fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
            FileSystemArtifactIo.write(path, contents)
        }
        fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
            FileSystemArtifactIo.list(path)
        }
        fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind> {
            FileSystemArtifactIo.entry_kind(path)
        }
        fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf> {
            FileSystemArtifactIo.canonicalize_artifact_path(path)
        }
        fn artifact_path_exists(&self, _path: &Path) -> bool {
            false
        }
        fn remove(&self, _path: &Path) -> std::io::Result<()> {
            Ok(())
        }
        fn rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
            Err(std::io::Error::other("injected publish failure"))
        }
        fn size(&self, _path: &Path) -> std::io::Result<u64> {
            Err(std::io::Error::other("injected metadata failure"))
        }
    }

    #[test]
    fn publication_and_metadata_failures_are_injectable() {
        let temporary = Path::new("temporary.mp4");
        let output = Path::new("output.mp4");
        assert_eq!(
            publish_output_with(&FailingIo, temporary, output, false)
                .unwrap_err()
                .failed_stage
                .as_deref(),
            Some(PUBLISH_STAGE)
        );
        assert_eq!(
            artifact_with(&FailingIo, output, "output.mp4".into(), "video/mp4", vec![])
                .unwrap_err()
                .failed_stage
                .as_deref(),
            Some(PUBLISH_STAGE)
        );
    }
}
