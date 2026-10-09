mod animation;
#[cfg(test)]
extern crate self as opencut_editor_core;
mod assets;
mod drafts;
mod error;
mod evaluated_scene;
mod fonts;
mod migrations;
mod model;
mod path_policy;
mod persistence;
mod render_artifact;
mod render_plan;
mod render_process;
mod renderer;
mod store;
mod timeline;
mod validation;
mod vector;

pub use vector::*;

pub use assets::fonts::FontConfig;
pub use drafts::EditDraft;
pub use error::{CoreError, ErrorCode};
pub use model::*;
pub use validation::audio_buses::resolve_audio_bus_route;
mod markers;
pub use path_policy::PathPolicy;
pub use render_artifact::RenderArtifact;
pub use render_plan::audio_analysis::{
    AudioAnalysisDocument, AudioAnalysisOptions, AudioAnalysisSummary, AudioChannelStatistics,
    AudioWaveformBin,
};
pub use render_process::{ProbeResult, RenderProgress};
pub use renderer::{
    AudioAnalysisResult, ExportOptions, PreviewDimensions, PreviewPreset, PreviewRangeOptions,
    PreviewResolution, PreviewReviewOptions, Renderer,
};
pub use store::{
    CommitGeneratedAssetRequest, CommitGeneratedAssetResult, CommitTranscriptionRequest,
    EditorCore, ProjectSummary, ReplaceGeneratedAssetRequest, ReplaceGeneratedAssetResult,
    ResolvedAssetInput, TranscriptionSegment, WriteResult,
};
