//! Render workspace and artifact publication owner.

use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    CoreError, ErrorCode, KeyframeProperty, KeyframeValue,
    render_plan::{PreparedText, SceneEvaluation},
};

pub(crate) const PUBLISH_STAGE: &str = "publish";
pub(crate) const GRAPH_BUILD_STAGE: &str = "graph_build";

pub(crate) trait ArtifactIo: Debug + Send + Sync {
    fn exists(&self, path: &Path) -> bool;
    fn remove(&self, path: &Path) -> std::io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn size(&self, path: &Path) -> std::io::Result<u64>;
}

pub(crate) struct PreparedRenderResources {
    pub(crate) media_paths: Vec<PathBuf>,
    pub(crate) text_layers: HashMap<String, PreparedText>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileSystemArtifactIo;

impl ArtifactIo for FileSystemArtifactIo {
    fn exists(&self, path: &Path) -> bool {
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
}

impl Drop for RenderWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn temporary_output(parent: &Path, extension: &str) -> PathBuf {
    let request_id = env::var("OPENCUT_REQUEST_ID")
        .ok()
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
        })
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    parent.join(format!(".opencut-{request_id}.{extension}"))
}

impl RenderWorkspace {
    pub(crate) fn create(project_dir: &Path) -> Result<Self, CoreError> {
        let request_id = env::var("OPENCUT_REQUEST_ID")
            .ok()
            .filter(|value| {
                !value.is_empty()
                    && value
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '-')
            })
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let path = project_dir.join(format!(".opencut-work-{request_id}"));
        std::fs::create_dir(&path)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

pub(crate) fn prepare_text_layers(
    text_resources: &[&crate::TextItem],
    workspace: &Path,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Result<HashMap<String, PreparedText>, CoreError> {
    let mut result = HashMap::new();
    for text in text_resources {
        let path = workspace.join(format!("text-{}.txt", text.id));
        let font_path = resolve_text_font(text, default_font_path, font_roots, warnings);
        let content = wrap_text(
            &text.text,
            text.style.wrap_width_px,
            text.font_size,
            font_path.as_deref(),
        );
        std::fs::write(&path, content.as_bytes())
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        let metrics = measure_text_block(&content, text.font_size, font_path.as_deref());
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
    scene: &SceneEvaluation<'_>,
    project_dir: &Path,
    workspace: &Path,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Result<PreparedRenderResources, CoreError> {
    let media_paths = scene
        .media_inputs
        .iter()
        .map(|input| resolve_project_asset(project_dir, &input.project_relative_path))
        .collect::<Result<Vec<_>, _>>()?;
    let text_layers = prepare_text_layers(
        &scene.text_resources,
        workspace,
        default_font_path,
        font_roots,
        warnings,
    )?;
    debug_assert!(
        scene
            .text_resources
            .iter()
            .all(|text| text_layers.contains_key(&text.id))
    );
    Ok(PreparedRenderResources {
        media_paths,
        text_layers,
    })
}

pub(crate) fn write_filter_script(path: &Path, contents: &str) -> Result<(), CoreError> {
    std::fs::write(path, contents)
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))
}

fn resolve_text_font(
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
            if let Ok(resolved) = candidate.canonicalize()
                && resolved.is_file()
                && font_roots
                    .iter()
                    .filter_map(|root| root.canonicalize().ok())
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
            if let Some(path) = find_font_file(root, &needle) {
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

pub(crate) fn wrap_text(
    text: &str,
    width_px: Option<u32>,
    font_size: u32,
    font_path: Option<&Path>,
) -> String {
    let Some(width_px) = width_px else {
        return text.to_owned();
    };
    let font_data = font_path.and_then(|path| std::fs::read(path).ok());
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

fn measure_text_block(text: &str, font_size: u32, font_path: Option<&Path>) -> TextMetrics {
    let font_data = font_path.and_then(|path| std::fs::read(path).ok());
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

fn find_font_file(root: &Path, normalized_family: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_font_file(&path, normalized_family) {
                return Some(found);
            }
        } else if matches!(
            path.extension()
                .and_then(|value| value.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("ttf" | "otf" | "ttc")
        ) {
            let stem = path
                .file_stem()?
                .to_string_lossy()
                .to_lowercase()
                .replace([' ', '-', '_'], "");
            if stem.contains(normalized_family) {
                return path.canonicalize().ok();
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
    if io.exists(output) {
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
    project_dir: &Path,
    relative: &Path,
) -> Result<PathBuf, CoreError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "project asset path is not allowed",
        ));
    }
    let root = project_dir
        .canonicalize()
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
    let resolved = project_dir
        .join(relative)
        .canonicalize()
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
    if !resolved.starts_with(root) {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "project asset escapes the project directory",
        ));
    }
    Ok(resolved)
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
        fn exists(&self, _path: &Path) -> bool {
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
