//! Render workspace and artifact publication owner.

pub(crate) mod raster_cache;
mod request_scope;
pub(crate) use request_scope::with_request_id;
mod blend;
pub(crate) mod extended_visual;
mod masks;
mod mattes;
mod shapes;
#[cfg(test)]
pub(crate) use shapes::fill_allocation_tests::observe as observe_allocations;
mod text;
pub(crate) mod text_measurement_memory;
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
    /// The admitted active-matte reader may allocate at most `capacity` bytes,
    /// once, with no moving growth. Alternate ports must explicitly own this
    /// contract; generic read/read_font is not a permitted fallback.
    fn read_admitted_font(
        &self,
        _path: &Path,
        _size: u64,
        _capacity: usize,
    ) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "bounded font read unavailable",
        ))
    }
    /// Active-only streaming lookup. Implementations must admit all cursor,
    /// ancestor, entry and path-normalization heap before opening/advancing.
    fn admitted_font_lookup(
        &self,
        _root: &Path,
        _family: &str,
        _remaining: u64,
    ) -> std::io::Result<Option<PathBuf>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "admitted font lookup unavailable",
        ))
    }
    fn media_digest(&self, _path: &Path) -> std::io::Result<(String, u64)> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "bounded media fingerprint unavailable",
        ))
    }
    fn read_font(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        self.read(path)
    }
    fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()>;
    fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;
    fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind>;
    fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf>;
    fn artifact_path_exists(&self, path: &Path) -> bool;
    fn remove(&self, path: &Path) -> std::io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn size(&self, path: &Path) -> std::io::Result<u64>;
}

/// Active-only legacy measurement adapter. Rich-run measurement can retain
/// one face Vec while measuring the same run with a second Vec; all other
/// generic-read measurement paths are sequential. The caller reserves two
/// maximum bounded faces before invoking any of these legacy helpers.
#[derive(Debug)]
pub(crate) struct MatteMeasurementIo<'a> {
    inner: &'a dyn ArtifactIo,
    failure: std::sync::Mutex<Option<CoreError>>,
}
impl<'a> MatteMeasurementIo<'a> {
    pub(crate) fn new(inner: &'a dyn ArtifactIo) -> Self {
        Self {
            inner,
            failure: std::sync::Mutex::new(None),
        }
    }
    fn rejected(&self, code: ErrorCode, message: &str) -> std::io::Error {
        let mut failure = self.failure.lock().unwrap_or_else(|p| p.into_inner());
        if failure.is_none() {
            *failure = Some(CoreError::new(code, message));
        }
        std::io::Error::other(message)
    }
    pub(crate) fn finish(&self) -> Result<(), CoreError> {
        self.failure
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
            .map_or(Ok(()), Err)
    }
}
#[cfg(test)]
thread_local! { static MEASUREMENT_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(crate) fn observed_measurement_reads(reset: bool) -> usize {
    MEASUREMENT_READS.with(|n| {
        let value = n.get();
        if reset {
            n.set(0);
        }
        value
    })
}
impl ArtifactIo for MatteMeasurementIo<'_> {
    fn request_id(&self) -> String {
        self.inner.request_id()
    }
    fn create_dir(&self, p: &Path) -> std::io::Result<()> {
        self.inner.create_dir(p)
    }
    fn remove_dir_all(&self, p: &Path) -> std::io::Result<()> {
        self.inner.remove_dir_all(p)
    }
    fn read(&self, p: &Path) -> std::io::Result<Vec<u8>> {
        #[cfg(test)]
        MEASUREMENT_READS.with(|n| n.set(n.get() + 1));
        let size = self.inner.size(p)?;
        if size > crate::MAX_FONT_BYTES as u64 {
            return Err(self.rejected(
                ErrorCode::InvalidArgument,
                "matte measurement font exceeds bounded size",
            ));
        }
        let capacity = usize::try_from(size).map_err(|_| {
            self.rejected(
                ErrorCode::InvalidArgument,
                "matte measurement font size overflow",
            )
        })?;
        let bytes = self
            .inner
            .read_admitted_font(p, size, capacity)
            .map_err(|error| {
                self.rejected(
                    if error.kind() == std::io::ErrorKind::Unsupported {
                        ErrorCode::DependencyUnavailable
                    } else {
                        ErrorCode::AssetIntegrityFailed
                    },
                    "matte measurement font cannot be read within admission",
                )
            })?;
        if bytes.capacity() > capacity || bytes.len() as u64 != size {
            return Err(self.rejected(
                ErrorCode::InvalidArgument,
                "matte measurement font exceeded admitted payload",
            ));
        }
        Ok(bytes)
    }
    fn read_admitted_font(&self, p: &Path, s: u64, c: usize) -> std::io::Result<Vec<u8>> {
        self.inner.read_admitted_font(p, s, c)
    }
    fn admitted_font_lookup(
        &self,
        root: &Path,
        family: &str,
        remaining: u64,
    ) -> std::io::Result<Option<PathBuf>> {
        self.inner.admitted_font_lookup(root, family, remaining)
    }
    fn media_digest(&self, p: &Path) -> std::io::Result<(String, u64)> {
        self.inner.media_digest(p)
    }
    fn write(&self, p: &Path, b: &[u8]) -> std::io::Result<()> {
        self.inner.write(p, b)
    }
    fn list(&self, p: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.inner.list(p)
    }
    fn entry_kind(&self, p: &Path) -> std::io::Result<ArtifactEntryKind> {
        self.inner.entry_kind(p)
    }
    fn canonicalize_artifact_path(&self, p: &Path) -> std::io::Result<PathBuf> {
        self.inner.canonicalize_artifact_path(p)
    }
    fn artifact_path_exists(&self, p: &Path) -> bool {
        self.inner.artifact_path_exists(p)
    }
    fn remove(&self, p: &Path) -> std::io::Result<()> {
        self.inner.remove(p)
    }
    fn rename(&self, p: &Path, q: &Path) -> std::io::Result<()> {
        self.inner.rename(p, q)
    }
    fn size(&self, p: &Path) -> std::io::Result<u64> {
        self.inner.size(p)
    }
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
    pub(crate) matte_integrity: Vec<crate::evaluated_scene::MatteMediaIntegrityBinding>,
    pub(crate) matte_font_payload_bytes: u64,
    pub(crate) font_faces: std::collections::BTreeMap<String, Vec<u8>>,
    pub(crate) media_inputs: Vec<MediaInputRequest>,
    pub(crate) media_paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileSystemArtifactIo;

impl ArtifactIo for FileSystemArtifactIo {
    fn read_admitted_font(
        &self,
        path: &Path,
        size: u64,
        capacity: usize,
    ) -> std::io::Result<Vec<u8>> {
        use std::io::Read;
        let size =
            usize::try_from(size).map_err(|_| std::io::Error::other("font size overflow"))?;
        if size > crate::MAX_FONT_BYTES || size > capacity {
            return Err(std::io::Error::other("font capacity admission exceeded"));
        }
        let mut bytes = vec![0u8; size];
        let mut file = std::fs::File::open(path)?;
        file.read_exact(&mut bytes)?;
        let mut extra = [0u8; 1];
        if file.read(&mut extra)? != 0 {
            return Err(std::io::Error::other("font size changed"));
        }
        Ok(bytes)
    }
    fn admitted_font_lookup(
        &self,
        root: &Path,
        family: &str,
        remaining: u64,
    ) -> std::io::Result<Option<PathBuf>> {
        text_measurement_memory::admitted_font_lookup(root, family, remaining)
    }
    fn media_digest(&self, path: &Path) -> std::io::Result<(String, u64)> {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;
        let mut digest = Sha256::new();
        let mut size = 0u64;
        let mut buffer = [0u8; 65_536];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            size = size
                .checked_add(count as u64)
                .ok_or_else(|| std::io::Error::other("media size overflow"))?;
            digest.update(&buffer[..count]);
        }
        Ok((format!("{:x}", digest.finalize()), size))
    }
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
    fn read_font(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(crate::MAX_FONT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_layouts: Vec<crate::TextLayoutDiagnostic>,
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
                rich_runs: None,
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

pub(crate) struct MeasuredText {
    pub(crate) shaped: Option<(
        crate::fonts::shaping::ShapedText,
        crate::evaluated_scene::EvaluatedTextStyle,
    )>,
    pub(crate) prepared: PreparedText,
    pub(crate) content: String,
}

pub(crate) fn prepare_render_resources(
    io: &dyn ArtifactIo,
    media: PreparedMediaResources,
    workspace: &Path,
    measured: HashMap<String, MeasuredText>,
    scene: &crate::evaluated_scene::EvaluatedScene,
    cache: (&raster_cache::RasterCache, [u8; 32]),
) -> Result<PreparedRenderResources, CoreError> {
    let mut text_layers = HashMap::new();
    let mut media_inputs = media.media_inputs;
    let mut media_paths = media.media_paths;
    let mut measured: Vec<_> = measured.into_iter().collect();
    measured.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (index, (id, mut text)) in measured.into_iter().enumerate() {
        if let Some((shaped, style)) = &text.shaped {
            // Expanded component/repeater IDs contain scope separators that are
            // not portable filenames. Keep identity in the plan, not the path.
            let file_name = format!("glyphs-{index}.pam");
            let path = workspace.join(&file_name);
            let source = scene
                .visual_layers
                .iter()
                .find_map(|layer| {
                    if layer.item_id == id
                        && let EvaluatedVisualSource::Text(source) = &layer.source
                    {
                        Some(source)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    CoreError::new(ErrorCode::InternalError, "missing evaluated text raster")
                })?;
            let key = raster_cache::text_key(cache.1, source, shaped, &text.prepared)?;
            let bytes = cache.0.raster(key, || {
                text::rasterize(shaped, &media.font_faces, &text.prepared, style)
            })?;
            io.write(&path, &bytes)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
            media_inputs.push(MediaInputRequest {
                item_id: id.clone(),
                asset_id: format!("glyphs-{id}"),
                project_relative_path: PathBuf::from(file_name),
                media_type: crate::MediaType::Image,
                source_in_ms: 0,
                duration_ms: scene.duration_ms,
                input_index: media_inputs.len() + 2,
            });
            media_paths.push(path);
            text_layers.insert(id, text.prepared);
            continue;
        }
        text.prepared.file_path = workspace.join(&text.prepared.file_path);
        io.write(&text.prepared.file_path, text.content.as_bytes())
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        if let Some(runs) = &mut text.prepared.rich_runs {
            for run in runs {
                run.file_path = workspace.join(&run.file_path);
                io.write(&run.file_path, run.content.as_bytes())
                    .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
            }
        }
        text_layers.insert(id, text.prepared);
    }
    Ok(PreparedRenderResources {
        media_inputs,
        media_paths,
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
    let mut font_faces = std::collections::BTreeMap::new();
    let font_limit = if evaluated.scene.composition_resources.is_some() {
        Some(
            crate::evaluated_scene::composition_resources::font_payload_admission(
                &evaluated.scene,
            )?,
        )
    } else {
        None
    };
    let mut font_payload = 0u64;
    for catalog in std::iter::once(&evaluated.resource_bindings.retained_fonts).chain(
        evaluated
            .resource_bindings
            .fonts
            .iter()
            .map(|b| &b.pinned_faces),
    ) {
        crate::fonts::validate_catalog(catalog)?;
        for (hash, face) in catalog {
            if font_faces.contains_key(hash) {
                continue;
            }
            let path = resolve_project_asset(io, project_dir, Path::new(&face.relative_path))
                .map_err(|error| {
                    if error.code == ErrorCode::PathNotAllowed {
                        error
                    } else {
                        CoreError::new(
                            ErrorCode::AssetIntegrityFailed,
                            "managed font reference is missing",
                        )
                    }
                })?;
            if io.size(&path).ok() != Some(face.size_bytes) {
                return Err(CoreError::new(
                    ErrorCode::AssetIntegrityFailed,
                    "managed font size changed",
                ));
            }
            let bytes = if let Some(limit) = font_limit {
                if font_payload
                    .checked_add(face.size_bytes)
                    .is_none_or(|n| n > limit)
                {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "matte font read exceeds shared memory bounds",
                    ));
                }
                let capacity = usize::try_from(face.size_bytes).map_err(|_| {
                    CoreError::new(ErrorCode::InvalidArgument, "matte font read size overflow")
                })?;
                let bytes = io
                    .read_admitted_font(&path, face.size_bytes, capacity)
                    .map_err(|error| {
                        CoreError::new(
                            if error.kind() == std::io::ErrorKind::Unsupported {
                                ErrorCode::DependencyUnavailable
                            } else {
                                ErrorCode::AssetIntegrityFailed
                            },
                            "managed font cannot be read within admission",
                        )
                    })?;
                if bytes.capacity() > capacity {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "matte font reader exceeded admitted capacity",
                    ));
                }
                bytes
            } else {
                io.read_font(&path).map_err(|_| {
                    CoreError::new(
                        ErrorCode::AssetIntegrityFailed,
                        "managed font cannot be read",
                    )
                })?
            };
            crate::fonts::verify_bytes(face, &bytes)?;
            if let Some(limit) = font_limit {
                font_payload = font_payload
                    .checked_add(bytes.capacity() as u64)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::InvalidArgument, "matte font payload overflow")
                    })?;
                if font_payload > limit {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "matte font payload exceeds shared memory bounds",
                    ));
                }
            }
            font_faces.insert(hash.clone(), bytes);
        }
    }
    Ok(PreparedMediaResources {
        matte_integrity: evaluated.resource_bindings.matte_integrity.clone(),
        matte_font_payload_bytes: font_payload,
        font_faces,
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

#[cfg(test)]
pub(crate) fn measure_evaluated_text_layers(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Result<HashMap<String, MeasuredText>, CoreError> {
    measure_evaluated_text_layers_with_fonts(
        io,
        evaluated,
        default_font_path,
        font_roots,
        warnings,
        &Default::default(),
    )
}

#[cfg(test)]
pub(crate) fn measure_evaluated_text_layers_with_fonts(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
    font_faces: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<HashMap<String, MeasuredText>, CoreError> {
    measure_evaluated_text_layers_with_budget(
        io,
        evaluated,
        default_font_path,
        font_roots,
        warnings,
        font_faces,
        &mut Default::default(),
    )
}

pub(crate) fn measure_evaluated_text_layers_with_budget(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
    font_faces: &std::collections::BTreeMap<String, Vec<u8>>,
    layout_work: &mut crate::evaluated_scene::text_layout::GlyphBudget,
) -> Result<HashMap<String, MeasuredText>, CoreError> {
    measure_evaluated_text_layers_with_admission(
        io,
        evaluated,
        default_font_path,
        font_roots,
        warnings,
        font_faces,
        (layout_work, None),
    )
}

pub(crate) fn measure_evaluated_text_layers_with_admission(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    default_font_path: Option<&Path>,
    font_roots: &[PathBuf],
    warnings: &mut Vec<String>,
    font_faces: &std::collections::BTreeMap<String, Vec<u8>>,
    admission: (
        &mut crate::evaluated_scene::text_layout::GlyphBudget,
        Option<&mut text_measurement_memory::MeasurementMemory>,
    ),
) -> Result<HashMap<String, MeasuredText>, CoreError> {
    let (layout_work, mut memory) = admission;
    let binding_bytes = if let Some(memory) = memory.as_deref_mut() {
        let bytes = text_measurement_memory::map_bytes::<&str, &FontResourceBinding>(
            evaluated.resource_bindings.fonts.len(),
        )?;
        memory.admit(bytes)?;
        memory.adopt(bytes)?;
        bytes
    } else {
        0
    };
    let font_bindings = evaluated
        .resource_bindings
        .fonts
        .iter()
        .map(|binding| (binding.font_resource_id.as_str(), binding))
        .collect::<HashMap<_, _>>();
    let mut result = HashMap::new();
    let mut shaped_cache = HashMap::new();
    for layer in &evaluated.scene.visual_layers {
        if !matches!(layer.source, EvaluatedVisualSource::Text(_))
            && !(matches!(layer.source, EvaluatedVisualSource::Caption(_))
                && layer.requires_affine())
        {
            continue;
        }

        let layer_scratch = if let Some(memory) = memory.as_deref_mut() {
            let scratch = text_measurement_memory::key_scratch(layer)?;
            let growth =
                text_measurement_memory::map_bytes::<String, MeasuredText>(result.len() + 1)?
                    .checked_add(text_measurement_memory::map_bytes::<
                        String,
                        crate::fonts::shaping::ShapedText,
                    >(shaped_cache.len() + 1)?)
                    .and_then(|n| n.checked_add(scratch))
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::InvalidArgument, "matte text map memory overflow")
                    })?;
            memory.admit(growth)?;
            growth
        } else {
            0
        };
        if let EvaluatedVisualSource::Caption(caption) = &layer.source
            && layer.requires_affine()
        {
            if let Some(memory) = memory.as_deref_mut() {
                let clone = default_font_path
                    .map_or(0, |p| p.as_os_str().len() as u64)
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(layer_scratch))
                    .ok_or_else(|| {
                        CoreError::new(
                            ErrorCode::InvalidArgument,
                            "caption font path clone overflow",
                        )
                    })?;
                memory.admit(clone)?;
            }
            let metrics =
                measure_text_block(io, &caption.text, caption.font_size, default_font_path);
            let width = (metrics.width.ceil() as u32).max(1).saturating_add(24);
            let height = (metrics.height.ceil() as u32).max(1).saturating_add(24);
            result.insert(
                layer.item_id.clone(),
                MeasuredText {
                    shaped: None,
                    prepared: PreparedText {
                        rich_runs: None,
                        file_path: PathBuf::from(format!("text-{}.txt", layer.item_id)),
                        font_path: default_font_path.map(Path::to_path_buf),
                        layer_width: width,
                        layer_height: height,
                        canvas_width: width,
                        canvas_height: height,
                        text_x: 12,
                        text_y: 12,
                    },
                    content: caption.text.clone(),
                },
            );
            adopt_text_measurement(
                &mut memory,
                &result,
                &shaped_cache,
                warnings,
                binding_bytes,
                layer_scratch,
            )?;
            continue;
        }
        let EvaluatedVisualSource::Text(text) = &layer.source else {
            adopt_text_measurement(
                &mut memory,
                &result,
                &shaped_cache,
                warnings,
                binding_bytes,
                layer_scratch,
            )?;
            continue;
        };
        if let Some(binding) = &text.font_binding {
            let document = crate::RichTextDocument {
                spans: text.spans.clone(),
                runs: text.rich_runs.clone().ok_or_else(|| {
                    CoreError::new(ErrorCode::InvalidArgument, "pinned text document missing")
                })?,
            };
            let cache_key = serde_json::to_string(&(
                &document,
                binding,
                text.font_size,
                &text.color,
                text.style.wrap_width_px,
                text.style.line_spacing_px,
            ))
            .map_err(|_| CoreError::new(ErrorCode::InternalError, "cannot identify text layout"))?;
            let shaped = if text.style.layout.is_some() {
                {
                    if let Some(memory) = memory.as_deref_mut() {
                        let remaining =
                            memory
                                .remaining()?
                                .checked_sub(layer_scratch)
                                .ok_or_else(|| {
                                    CoreError::new(
                                        ErrorCode::InvalidArgument,
                                        "matte text scratch memory overflow",
                                    )
                                })?;
                        memory.admit(
                            layer_scratch
                                .checked_add(text_measurement_memory::shaping_scratch(
                                    text, font_faces, remaining,
                                )?)
                                .ok_or_else(|| {
                                    CoreError::new(
                                        ErrorCode::InvalidArgument,
                                        "matte text scratch memory overflow",
                                    )
                                })?,
                        )?;
                        crate::evaluated_scene::text_layout::resolve_admitted(
                            text,
                            font_faces,
                            layout_work,
                        )?
                    } else {
                        crate::evaluated_scene::text_layout::resolve(text, font_faces, layout_work)?
                    }
                }
            } else if let Some(shaped) = shaped_cache.get(&cache_key) {
                if let Some(memory) = memory.as_deref_mut() {
                    text_measurement_memory::clone_cached_shape(memory, shaped, layer_scratch)?
                } else {
                    Clone::clone(shaped)
                }
            } else {
                if let Some(memory) = memory.as_deref_mut() {
                    let remaining =
                        memory
                            .remaining()?
                            .checked_sub(layer_scratch)
                            .ok_or_else(|| {
                                CoreError::new(
                                    ErrorCode::InvalidArgument,
                                    "matte text scratch memory overflow",
                                )
                            })?;
                    memory.admit(
                        layer_scratch
                            .checked_add(text_measurement_memory::shaping_scratch(
                                text, font_faces, remaining,
                            )?)
                            .ok_or_else(|| {
                                CoreError::new(
                                    ErrorCode::InvalidArgument,
                                    "matte text scratch memory overflow",
                                )
                            })?,
                    )?;
                }
                let shape = if memory.is_some() {
                    crate::fonts::shaping::shape_admitted
                } else {
                    crate::fonts::shaping::shape
                };
                let shaped = shape(
                    &document,
                    binding,
                    font_faces,
                    text.font_size,
                    &text.color,
                    text.style.wrap_width_px,
                    text.style.line_spacing_px,
                )?;
                shaped_cache.insert(cache_key, shaped.clone());
                shaped
            };
            result.insert(
                layer.item_id.clone(),
                text::measure(shaped, text, font_faces)?,
            );
            warnings.extend(binding.warnings.clone());
            adopt_text_measurement(
                &mut memory,
                &result,
                &shaped_cache,
                warnings,
                binding_bytes,
                layer_scratch,
            )?;
            continue;
        }
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
        let path = PathBuf::from(format!("text-{}.txt", layer.item_id));
        let font_path = if let Some(memory) = memory.as_deref_mut() {
            resolve_evaluated_font_admitted(
                io,
                &layer.item_id,
                binding,
                default_font_path,
                font_roots,
                warnings,
                (memory, layer_scratch),
            )?
        } else {
            resolve_evaluated_font(
                io,
                &layer.item_id,
                binding,
                default_font_path,
                font_roots,
                warnings,
            )
        };
        if let Some(memory) = memory.as_deref_mut() {
            let path_bytes = font_path.as_ref().map_or(0, |p| p.capacity() as u64);
            let characters = text.text.chars().count() as u64;
            let per_run = path_bytes
                .checked_add(layer.item_id.capacity() as u64)
                .and_then(|n| n.checked_add(256))
                .ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::InvalidArgument,
                        "legacy font path memory overflow",
                    )
                })?;
            let scratch = characters
                .checked_add(1)
                .and_then(|n| n.checked_mul(per_run))
                .and_then(|n| n.checked_mul(6))
                .and_then(|n| n.checked_add(layer_scratch))
                .ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::InvalidArgument,
                        "legacy text run memory overflow",
                    )
                })?;
            memory.admit(scratch)?;
        }
        if let Some(runs) = &text.rich_runs {
            let measured = measure_rich_text(io, &layer.item_id, text, runs, font_path.as_deref())?;
            result.insert(layer.item_id.clone(), measured);
            adopt_text_measurement(
                &mut memory,
                &result,
                &shaped_cache,
                warnings,
                binding_bytes,
                layer_scratch,
            )?;
            continue;
        }
        let content = wrap_text_with_io(
            io,
            &text.text,
            text.style.wrap_width_px,
            text.font_size,
            font_path.as_deref(),
        );
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
            MeasuredText {
                shaped: None,
                content,
                prepared: PreparedText {
                    rich_runs: None,
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
            },
        );
        adopt_text_measurement(
            &mut memory,
            &result,
            &shaped_cache,
            warnings,
            binding_bytes,
            layer_scratch,
        )?;
    }
    drop(shaped_cache);
    drop(font_bindings);
    if let Some(memory) = memory {
        memory.adopt(
            text_measurement_memory::measured_bytes(&result)?
                .checked_add(text_measurement_memory::warnings_bytes(warnings)?)
                .ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::InvalidArgument,
                        "matte text result memory overflow",
                    )
                })?,
        )?;
    }
    Ok(result)
}

fn adopt_text_measurement(
    memory: &mut Option<&mut text_measurement_memory::MeasurementMemory>,
    result: &HashMap<String, MeasuredText>,
    cache: &HashMap<String, crate::fonts::shaping::ShapedText>,
    warnings: &Vec<String>,
    binding_bytes: u64,
    transient: u64,
) -> Result<(), CoreError> {
    if let Some(memory) = memory.as_deref_mut() {
        let actual = text_measurement_memory::measured_bytes(result)?
            .checked_add(text_measurement_memory::shaped_cache_bytes(cache)?)
            .and_then(|n| n.checked_add(binding_bytes))
            .and_then(|n| n.checked_add(text_measurement_memory::warnings_bytes(warnings).ok()?))
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::InvalidArgument,
                    "matte text adoption memory overflow",
                )
            })?;
        memory.adopt(actual)?;
        memory.admit(transient)?;
    }
    Ok(())
}

fn admitted_font_metadata<T>(result: std::io::Result<T>) -> Result<Option<T>, CoreError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::OutOfMemory | std::io::ErrorKind::Unsupported
            ) =>
        {
            Err(CoreError::new(
                if error.kind() == std::io::ErrorKind::OutOfMemory {
                    ErrorCode::InvalidArgument
                } else {
                    ErrorCode::DependencyUnavailable
                },
                "font resolver resource operation failed",
            ))
        }
        Err(_) => Ok(None),
    }
}
fn resolve_evaluated_font_admitted(
    io: &dyn ArtifactIo,
    item_id: &str,
    binding: Option<&FontResourceBinding>,
    default: Option<&Path>,
    roots: &[PathBuf],
    warnings: &mut Vec<String>,
    admission: (&mut text_measurement_memory::MeasurementMemory, u64),
) -> Result<Option<PathBuf>, CoreError> {
    let (memory, layer_scratch) = admission;
    let binding_payload = if let Some(binding) = binding {
        binding
            .requested_path
            .as_ref()
            .map_or(0, |s| s.capacity() as u64)
            .checked_add(
                binding
                    .requested_family
                    .as_ref()
                    .map_or(0, |s| s.capacity() as u64),
            )
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::InvalidArgument,
                    "requested font payload overflow",
                )
            })?
    } else {
        0
    };
    let path_payload =
        roots
            .iter()
            .try_fold(default.map_or(0, |p| p.as_os_str().len() as u64), |n, p| {
                n.checked_add(p.capacity() as u64).ok_or_else(|| {
                    CoreError::new(ErrorCode::InvalidArgument, "font resolver path overflow")
                })
            })?;
    let scratch = path_payload
        .checked_add(binding_payload)
        .and_then(|n| n.checked_mul(16))
        .and_then(|n| n.checked_add(layer_scratch))
        .and_then(|n| n.checked_add(262144))
        .ok_or_else(|| {
            CoreError::new(ErrorCode::InvalidArgument, "font resolver scratch overflow")
        })?;
    memory.admit(scratch)?;
    if let Some(requested) = binding.and_then(|b| b.requested_path.as_deref()) {
        let payload = scratch
            .checked_add((requested.len() as u64).checked_mul(16).ok_or_else(|| {
                CoreError::new(ErrorCode::InvalidArgument, "requested font path overflow")
            })?)
            .ok_or_else(|| {
                CoreError::new(ErrorCode::InvalidArgument, "requested font path overflow")
            })?;
        memory.admit(payload)?;
        let requested = Path::new(requested);
        for root in roots {
            let candidate = if requested.is_absolute() {
                requested.to_path_buf()
            } else {
                root.join(requested)
            };
            if let Some(found) = admitted_font_metadata(io.canonicalize_artifact_path(&candidate))?
                && admitted_font_metadata(io.entry_kind(&found))? == Some(ArtifactEntryKind::File)
            {
                let mut permitted = false;
                for root in roots {
                    if let Some(root) = admitted_font_metadata(io.canonicalize_artifact_path(root))?
                        && found.starts_with(root)
                    {
                        permitted = true;
                        break;
                    }
                }
                if permitted {
                    memory.admit(payload.checked_add(found.capacity() as u64).ok_or_else(
                        || CoreError::new(ErrorCode::InvalidArgument, "font path overflow"),
                    )?)?;
                    return Ok(Some(found));
                }
            }
            if requested.is_absolute() {
                break;
            }
        }
        warnings.push(format!(
            "Text item {item_id} requested a font path that could not be resolved; using fallback"
        ));
    }
    if let Some(family) = binding.and_then(|b| b.requested_family.as_deref()) {
        let needle = family.to_lowercase().replace([' ', '-', '_'], "");
        let remaining = memory.remaining()?.checked_sub(scratch).ok_or_else(|| {
            CoreError::new(ErrorCode::InvalidArgument, "font lookup memory excess")
        })?;
        for root in roots {
            let found = io
                .admitted_font_lookup(root, &needle, remaining)
                .map_err(|error| {
                    CoreError::new(
                        if error.kind() == std::io::ErrorKind::OutOfMemory {
                            ErrorCode::InvalidArgument
                        } else {
                            ErrorCode::DependencyUnavailable
                        },
                        "admitted font lookup failed",
                    )
                })?;
            if let Some(found) = found {
                memory.admit(
                    scratch
                        .checked_add(found.capacity() as u64)
                        .ok_or_else(|| {
                            CoreError::new(ErrorCode::InvalidArgument, "font path overflow")
                        })?,
                )?;
                return Ok(Some(found));
            }
        }
        warnings.push(format!("Text item {item_id} requested font family {family:?} that could not be resolved; using fallback"));
    }
    Ok(default.map(Path::to_path_buf))
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

pub(crate) fn verify_matte_media_integrity(
    io: &dyn ArtifactIo,
    media: &PreparedMediaResources,
    project_dir: &Path,
) -> Result<(), CoreError> {
    for pin in &media.matte_integrity {
        // Retained hidden/inactive pins are resource obligations even when no
        // decoder input is published for their zero-coverage occurrence.
        let path = resolve_project_asset(io, project_dir, Path::new(&pin.project_relative_path))?;
        let (digest, size) = io.media_digest(&path).map_err(|error| {
            CoreError::new(
                if error.kind() == std::io::ErrorKind::Unsupported {
                    ErrorCode::DependencyUnavailable
                } else {
                    ErrorCode::AssetIntegrityFailed
                },
                "managed matte media cannot be fingerprinted",
            )
        })?;
        if pin
            .sha256
            .as_ref()
            .is_some_and(|expected| *expected != digest)
            || pin.size_bytes.is_some_and(|expected| expected != size)
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "asset content hash or size does not match project metadata",
            ));
        }
    }
    Ok(())
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
        text_layouts: vec![],
        relative_path,
        mime_type: mime_type.into(),
        size_bytes,
        warnings,
    })
}

fn styled_font(
    io: &dyn ArtifactIo,
    base: Option<&Path>,
    bold: bool,
    italic: bool,
) -> Result<Option<PathBuf>, CoreError> {
    if !bold && !italic {
        return Ok(base.map(Path::to_path_buf));
    }
    let base = base.ok_or_else(|| {
        CoreError::new(
            ErrorCode::DependencyUnavailable,
            "rich text styles require a configured font",
        )
    })?;
    let parent = base.parent().ok_or_else(|| {
        CoreError::new(
            ErrorCode::DependencyUnavailable,
            "rich text font has no parent directory",
        )
    })?;
    let stem = base.file_stem().unwrap_or_default().to_string_lossy();
    let stem = stem
        .trim_end_matches("-Regular")
        .trim_end_matches("Regular");
    let suffixes: &[&str] = match (bold, italic) {
        (true, true) => &["-BoldItalic", "-BoldOblique", "BoldItalic", "bi"],
        (true, false) => &["-Bold", "Bold", "bd", "b"],
        (false, true) => &["-Italic", "-Oblique", "Italic", "i"],
        _ => &[],
    };
    let parent = io.canonicalize_artifact_path(parent).map_err(|_| {
        CoreError::new(
            ErrorCode::DependencyUnavailable,
            "rich text font directory unavailable",
        )
    })?;
    for suffix in suffixes {
        let candidate = parent.join(format!(
            "{stem}{suffix}.{}",
            base.extension().unwrap_or_default().to_string_lossy()
        ));
        if let Ok(path) = io.canonicalize_artifact_path(&candidate)
            && path.starts_with(&parent)
            && io.entry_kind(&path).ok() == Some(ArtifactEntryKind::File)
        {
            return Ok(Some(path));
        }
    }
    Err(CoreError::new(
        ErrorCode::DependencyUnavailable,
        "configured font has no matching bold/italic face",
    ))
}
fn measure_rich_text(
    io: &dyn ArtifactIo,
    id: &str,
    text: &crate::evaluated_scene::EvaluatedText,
    runs: &[crate::RichTextRun],
    base: Option<&Path>,
) -> Result<MeasuredText, CoreError> {
    use crate::render_plan::PreparedTextRun;
    let mut prepared_runs: Vec<PreparedTextRun> = vec![];
    let mut line_widths = vec![0.0f64];
    let mut line = 0usize;
    let mut line_height = f64::from(text.font_size) * 1.2;
    let mut run_lines = vec![];
    for run in runs {
        let font = styled_font(
            io,
            base,
            run.bold.unwrap_or(false),
            run.italic.unwrap_or(false),
        )?;
        let bytes = font.as_deref().and_then(|p| io.read(p).ok());
        let face = bytes
            .as_deref()
            .and_then(|b| ttf_parser::Face::parse(b, 0).ok());
        let metrics = measure_text_block(io, &run.text, text.font_size, font.as_deref());
        line_height = line_height.max(metrics.height / metrics.line_count as f64);
        let mut content = String::new();
        let mut start = line_widths[line];
        let flush = |content: &mut String,
                     start: f64,
                     line: usize,
                     prepared: &mut Vec<PreparedTextRun>,
                     lines: &mut Vec<usize>| {
            if content.is_empty() {
                return;
            }
            prepared.push(PreparedTextRun {
                file_path: PathBuf::from(format!("text-{id}-run-{}.txt", prepared.len())),
                font_path: font.clone(),
                content: std::mem::take(content),
                color: run.color.clone().unwrap_or_else(|| text.color.clone()),
                x: start,
                y: 0.0,
            });
            lines.push(line);
        };
        for character in run.text.chars() {
            let width = measure_text_run(&character.to_string(), text.font_size, face.as_ref());
            if character == '\n'
                || text.style.wrap_width_px.is_some_and(|max| {
                    line_widths[line] > 0.0 && line_widths[line] + width > f64::from(max)
                })
            {
                flush(
                    &mut content,
                    start,
                    line,
                    &mut prepared_runs,
                    &mut run_lines,
                );
                line += 1;
                line_widths.push(0.0);
                start = 0.0;
                if character == '\n' {
                    continue;
                }
            }
            content.push(character);
            line_widths[line] += width;
        }
        flush(
            &mut content,
            start,
            line,
            &mut prepared_runs,
            &mut run_lines,
        );
    }
    let width = line_widths.iter().copied().fold(0.0, f64::max);
    let x = text.style.padding.left
        + text.style.outline_width_px
        + text.style.shadow.offset_x.min(0).unsigned_abs();
    let y = text.style.padding.top
        + text.style.outline_width_px
        + text.style.shadow.offset_y.min(0).unsigned_abs();
    let spacing = line_height + f64::from(text.style.line_spacing_px);
    for (run, line) in prepared_runs.iter_mut().zip(run_lines) {
        use crate::evaluated_scene::EvaluatedTextAlignment::*;
        let offset = match text.style.alignment {
            Left => 0.0,
            Center => (width - line_widths[line]) / 2.0,
            Right => width - line_widths[line],
        };
        run.x += f64::from(x) + offset;
        run.y = f64::from(y) + line as f64 * spacing;
    }
    let w = (width.ceil() as u32)
        .saturating_add(x)
        .saturating_add(text.style.padding.right)
        .saturating_add(text.style.outline_width_px)
        .saturating_add(text.style.shadow.offset_x.max(0) as u32)
        .saturating_add(2)
        .max(1);
    let h = ((line_height + line as f64 * spacing).max(1.0).ceil() as u32)
        .saturating_add(y)
        .saturating_add(text.style.padding.bottom)
        .saturating_add(text.style.outline_width_px)
        .saturating_add(text.style.shadow.offset_y.max(0) as u32)
        .saturating_add(2)
        .max(1);
    Ok(MeasuredText {
        shaped: None,
        content: text.text.clone(),
        prepared: PreparedText {
            rich_runs: Some(prepared_runs),
            file_path: PathBuf::from(format!("text-{id}.txt")),
            font_path: base.map(Path::to_path_buf),
            layer_width: w,
            layer_height: h,
            canvas_width: w,
            canvas_height: h,
            text_x: x,
            text_y: y,
        },
    })
}

/// Materialize only already-validated evaluated vector sources in the owned workspace.
pub(crate) fn prepare_shape_resources(
    io: &dyn ArtifactIo,
    scene: &crate::evaluated_scene::EvaluatedScene,
    workspace: &Path,
    resources: &mut PreparedRenderResources,
    cache: (&raster_cache::RasterCache, [u8; 32]),
) -> Result<(), CoreError> {
    for (index, layer) in scene.visual_layers.iter().enumerate() {
        if let EvaluatedVisualSource::Shape(shape) = &layer.source {
            let path = workspace.join(format!("shape-{index}.pam"));
            let key = raster_cache::shape_key(cache.1, shape)?;
            let bytes = cache.0.raster(key, || shapes::rasterize(shape))?;
            io.write(&path, &bytes)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
            resources.media_inputs.push(MediaInputRequest {
                item_id: layer.item_id.clone(),
                asset_id: format!("shape-{index}"),
                project_relative_path: PathBuf::from(format!("shape-{index}.pam")),
                media_type: crate::MediaType::Image,
                source_in_ms: 0,
                duration_ms: scene.duration_ms,
                input_index: resources.media_inputs.len() + 2,
            });
            resources.media_paths.push(path);
        }
    }
    Ok(())
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
    #[test]
    fn matte_measurement_font_port_is_bounded_and_latches_swallowed_errors() {
        let root = tempfile::tempdir().unwrap();
        let regular = root.path().join("font.ttf");
        std::fs::write(&regular, b"bounded font bytes").unwrap();
        let io = MatteMeasurementIo::new(&FileSystemArtifactIo);
        let bytes = io.read(&regular).unwrap();
        assert_eq!(bytes, b"bounded font bytes");
        assert_eq!(bytes.capacity(), bytes.len());
        io.finish().unwrap();
        let oversized = root.path().join("oversized.ttf");
        std::fs::File::create(&oversized)
            .unwrap()
            .set_len(crate::MAX_FONT_BYTES as u64 + 1)
            .unwrap();
        let io = MatteMeasurementIo::new(&FileSystemArtifactIo);
        // Legacy measurement may swallow this I/O error. The owner must still
        // fail before publishing approximate glyph facts or any raster output.
        assert!(io.read(&oversized).is_err());
        let error = io.finish().unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert_eq!(
            std::fs::metadata(oversized).unwrap().len(),
            crate::MAX_FONT_BYTES as u64 + 1
        );
    }
    #[derive(Debug)]
    struct SpareMeasurementFontIo;
    impl ArtifactIo for SpareMeasurementFontIo {
        fn request_id(&self) -> String {
            "spare-measurement".into()
        }
        fn read_admitted_font(
            &self,
            _: &Path,
            size: u64,
            capacity: usize,
        ) -> std::io::Result<Vec<u8>> {
            assert_eq!(size as usize, capacity);
            let mut bytes = Vec::with_capacity(capacity + 1);
            bytes.resize(capacity, 0);
            Ok(bytes)
        }
        fn create_dir(&self, p: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.create_dir(p)
        }
        fn remove_dir_all(&self, p: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.remove_dir_all(p)
        }
        fn read(&self, p: &Path) -> std::io::Result<Vec<u8>> {
            FileSystemArtifactIo.read(p)
        }
        fn write(&self, p: &Path, b: &[u8]) -> std::io::Result<()> {
            FileSystemArtifactIo.write(p, b)
        }
        fn list(&self, p: &Path) -> std::io::Result<Vec<PathBuf>> {
            FileSystemArtifactIo.list(p)
        }
        fn entry_kind(&self, p: &Path) -> std::io::Result<ArtifactEntryKind> {
            FileSystemArtifactIo.entry_kind(p)
        }
        fn canonicalize_artifact_path(&self, p: &Path) -> std::io::Result<PathBuf> {
            FileSystemArtifactIo.canonicalize_artifact_path(p)
        }
        fn artifact_path_exists(&self, p: &Path) -> bool {
            FileSystemArtifactIo.artifact_path_exists(p)
        }
        fn remove(&self, p: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.remove(p)
        }
        fn rename(&self, p: &Path, q: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.rename(p, q)
        }
        fn size(&self, p: &Path) -> std::io::Result<u64> {
            FileSystemArtifactIo.size(p)
        }
    }
    #[test]
    fn matte_measurement_font_spare_capacity_is_rejected_before_metric_fallback() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("font.ttf");
        std::fs::write(&path, b"face").unwrap();
        let io = MatteMeasurementIo::new(&SpareMeasurementFontIo);
        assert!(io.read(&path).is_err());
        let error = io.finish().unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert_eq!(std::fs::read(path).unwrap(), b"face");
    }
    #[test]
    fn retained_matte_media_pin_without_decoder_input_is_verified_before_cache() {
        use sha2::{Digest, Sha256};
        let root = tempfile::tempdir().unwrap();
        let assets = root.path().join("assets");
        std::fs::create_dir(&assets).unwrap();
        let path = assets.join("retained-provider.bin");
        let original = b"retained hidden or inactive provider payload";
        std::fs::write(&path, original).unwrap();
        let media = PreparedMediaResources {
            matte_integrity: vec![crate::evaluated_scene::MatteMediaIntegrityBinding {
                asset_id: "retained-only".into(),
                project_relative_path: "assets/retained-provider.bin".into(),
                sha256: Some(format!("{:x}", Sha256::digest(original))),
                size_bytes: Some(original.len() as u64),
            }],
            matte_font_payload_bytes: 0,
            font_faces: Default::default(),
            media_inputs: vec![],
            media_paths: vec![],
        };
        // Zero-span retained providers intentionally have no decoder input.
        verify_matte_media_integrity(&FileSystemArtifactIo, &media, root.path()).unwrap();
        verify_matte_media_integrity(&FileSystemArtifactIo, &media, root.path()).unwrap();
        let mut changed = original.to_vec();
        *changed.last_mut().unwrap() ^= 1;
        std::fs::write(&path, &changed).unwrap();
        for _ in 0..2 {
            let error = verify_matte_media_integrity(&FileSystemArtifactIo, &media, root.path())
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
            assert!(!error.retryable);
            assert_eq!(std::fs::read(&path).unwrap(), changed);
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
            assert_eq!(std::fs::read_dir(&assets).unwrap().count(), 1);
        }
    }
}
