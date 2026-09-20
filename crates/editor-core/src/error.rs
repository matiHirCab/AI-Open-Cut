use serde::Serialize;
use thiserror::Error;

// Serde erases typed errors while buffering nested project and operation values.
// Only the layout field deserializer emits this anchored internal classification.
pub(crate) const LAYOUT_DECODE_ERROR_PREFIX: &str = "\u{1e}OPENCUT_LAYOUT_DECODE:";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidArgument,
    ValidationFailed,
    RevisionConflict,
    PathNotAllowed,
    PathTraversal,
    ProjectNotFound,
    ProjectRecoveryFailed,
    AssetNotFound,
    AssetInUse,
    AssetIntegrityFailed,
    ItemNotFound,
    TrackNotFound,
    TrackLocked,
    DraftNotFound,
    DraftLimitReached,
    ExportExists,
    DependencyUnavailable,
    UnsupportedMedia,
    FfmpegFailed,
    JobFailed,
    InternalError,
}

impl ErrorCode {
    pub const fn retryable(self) -> bool {
        matches!(self, Self::RevisionConflict)
    }
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct CoreError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub failed_stage: Option<String>,
    pub ffmpeg_exit_code: Option<i32>,
    pub ffmpeg_stderr_excerpt: Option<String>,
}

impl CoreError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: code.retryable(),
            failed_stage: None,
            ffmpeg_exit_code: None,
            ffmpeg_stderr_excerpt: None,
        }
    }

    pub fn render_failure(stage: &str, exit_code: Option<i32>, stderr: Option<String>) -> Self {
        Self {
            code: ErrorCode::FfmpegFailed,
            message: "FFmpeg render failed".into(),
            retryable: false,
            failed_stage: Some(stage.into()),
            ffmpeg_exit_code: exit_code,
            ffmpeg_stderr_excerpt: stderr,
        }
    }

    pub(crate) fn io(context: &str, error: std::io::Error) -> Self {
        Self::new(ErrorCode::InternalError, format!("{context}: {error}"))
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        let message = error.to_string();
        if let Some(detail) = message.strip_prefix(LAYOUT_DECODE_ERROR_PREFIX) {
            return Self::new(
                ErrorCode::InvalidArgument,
                format!("invalid text layout: {detail}"),
            );
        }
        Self::new(
            ErrorCode::InternalError,
            format!("invalid project data: {message}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_decode_classification_requires_the_exact_anchored_marker() {
        use serde::de::Error;
        let classified = CoreError::from(serde_json::Error::custom(format!(
            "{LAYOUT_DECODE_ERROR_PREFIX}invalid type"
        )));
        assert_eq!(classified.code, ErrorCode::InvalidArgument);
        assert!(!classified.retryable);
        assert_eq!(classified.message, "invalid text layout: invalid type");
        for message in [
            "layout: invalid type".to_owned(),
            format!("user field {LAYOUT_DECODE_ERROR_PREFIX}invalid type"),
            "OPENCUT_LAYOUT_DECODE: user text".to_owned(),
        ] {
            let error = CoreError::from(serde_json::Error::custom(&message));
            assert_eq!(error.code, ErrorCode::InternalError);
            assert_eq!(error.message, format!("invalid project data: {message}"));
        }
    }

    #[test]
    fn every_core_error_is_defined_by_the_shared_catalog() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/error-codes-v1.json")).unwrap();
        for code in [
            ErrorCode::InvalidArgument,
            ErrorCode::ValidationFailed,
            ErrorCode::RevisionConflict,
            ErrorCode::PathNotAllowed,
            ErrorCode::PathTraversal,
            ErrorCode::ProjectNotFound,
            ErrorCode::ProjectRecoveryFailed,
            ErrorCode::AssetNotFound,
            ErrorCode::AssetInUse,
            ErrorCode::AssetIntegrityFailed,
            ErrorCode::ItemNotFound,
            ErrorCode::TrackNotFound,
            ErrorCode::TrackLocked,
            ErrorCode::DraftNotFound,
            ErrorCode::DraftLimitReached,
            ErrorCode::ExportExists,
            ErrorCode::DependencyUnavailable,
            ErrorCode::UnsupportedMedia,
            ErrorCode::FfmpegFailed,
            ErrorCode::JobFailed,
            ErrorCode::InternalError,
        ] {
            let wire = serde_json::to_value(code)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned();
            let definition = &catalog["codes"][&wire];
            assert!(!definition.is_null(), "missing {wire}");
            assert_eq!(definition["retryable"].as_bool(), Some(code.retryable()));
        }

        for warning in ["PERSISTENCE_RECOVERY_PENDING", "DRAFT_CLEANUP_FAILED"] {
            let definition = &catalog["codes"][warning];
            assert_eq!(definition["layer"].as_str(), Some("warning"));
            assert_eq!(definition["retryable"].as_bool(), Some(false));
        }
    }
}
