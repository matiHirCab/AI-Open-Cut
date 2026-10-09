use std::path::PathBuf;

use opencut_editor_core::{
    CoreError, EditorCore, PathPolicy, PreviewPreset, PreviewResolution, PreviewReviewOptions,
    RenderArtifact, Renderer,
};

use crate::session::Startup;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum InspectorTab {
    #[default]
    Layer,
    Cues,
    Audio,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Request {
    Frame(u64),
    Range(PreviewReviewOptions),
}

#[derive(Clone, Debug)]
pub(crate) struct Artifact {
    pub project_id: String,
    pub revision: u64,
    pub request: Request,
    pub dimensions: (u32, u32),
    pub fps: u32,
    pub path: PathBuf,
    pub rendered: RenderArtifact,
}

impl Artifact {
    pub fn stale(&self, project: &opencut_editor_core::Project) -> bool {
        self.project_id != project.id || self.revision != project.revision
    }
}

pub(crate) struct Review {
    pub values: [String; 3],
    pub field: Option<usize>,
    pub preset: PreviewPreset,
    pub include_audio: bool,
    pub frame: Option<Artifact>,
    pub range: Option<Artifact>,
    pub feedback: Option<String>,
}

impl Default for Review {
    fn default() -> Self {
        Self {
            values: ["0".into(), "0".into(), "1000".into()],
            field: None,
            preset: PreviewPreset::P540,
            include_audio: true,
            frame: None,
            range: None,
            feedback: None,
        }
    }
}

impl Review {
    pub fn request(&self, range: bool) -> Result<Request, String> {
        let parse = |index: usize| {
            self.values[index]
                .parse::<u64>()
                .map_err(|_| "Review times must be unsigned integer milliseconds.".to_owned())
        };
        if range {
            Ok(Request::Range(PreviewReviewOptions {
                start_ms: parse(1)?,
                end_ms: parse(2)?,
                resolution: PreviewResolution::Preset(self.preset),
                fps: None,
                include_audio: Some(self.include_audio),
            }))
        } else {
            Ok(Request::Frame(parse(0)?))
        }
    }

    pub fn publish(&mut self, artifact: Artifact) {
        match artifact.request {
            Request::Frame(_) => self.frame = Some(artifact),
            Request::Range(_) => self.range = Some(artifact),
        }
    }
}

pub(crate) fn configured_renderer() -> Renderer {
    Renderer::new(
        std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into()),
        std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap_or_else(|| "ffprobe".into()),
        std::env::var_os("OPENCUT_DEFAULT_FONT_PATH").map(PathBuf::from),
    )
}

pub(crate) fn execute(
    startup: &Startup,
    renderer: &Renderer,
    revision: u64,
    request: Request,
) -> Result<Artifact, CoreError> {
    let core = EditorCore::new(PathPolicy::new(
        &startup.store,
        [&startup.store],
        &startup.store,
    )?);
    let project = core.validate_revision(&startup.project_id, revision)?;
    let directory = core.project_directory(&project.id)?;
    let (rendered, dimensions, fps) = match request {
        Request::Frame(time) => (
            renderer.render_preview(&project, &directory, time)?,
            (project.settings.width, project.settings.height),
            project.settings.fps,
        ),
        Request::Range(options) => {
            let options = options.resolve(&project)?;
            (
                renderer.render_preview_range(&project, &directory, options, |_| {})?,
                (options.width, options.height),
                options.fps,
            )
        }
    };
    Ok(Artifact {
        project_id: project.id,
        revision: project.revision,
        request,
        dimensions,
        fps,
        path: directory.join(&rendered.relative_path),
        rendered,
    })
}
