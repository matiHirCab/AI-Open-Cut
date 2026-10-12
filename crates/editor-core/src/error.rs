use serde::Serialize;
use thiserror::Error;

// Serde erases typed errors while buffering nested project and operation values.
// Only the layout field deserializer emits this anchored internal classification.
pub(crate) const GROUP_COMPOSITING_DECODE_ERROR_PREFIX: &str =
    "\u{1e}OPENCUT_GROUP_COMPOSITING_DECODE:";
pub(crate) const COLOR_EFFECT_DECODE_ERROR_PREFIX: &str = "\u{1e}OPENCUT_COLOR_EFFECT_DECODE:";
pub(crate) const BLEND_DECODE_ERROR_PREFIX: &str = "\u{1e}OPENCUT_BLEND_DECODE:";
pub(crate) const MATTE_DECODE_ERROR_PREFIX: &str = "\u{1e}OPENCUT_MATTE_DECODE:";
pub(crate) const MASK_ACTIVATION_DECODE_ERROR_PREFIX: &str =
    "\u{1e}OPENCUT_MASK_ACTIVATION_DECODE:";
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegSpawnKind {
    NotFound,
    PermissionDenied,
    InvalidInput,
    InvalidFilename,
    ArgumentListTooLong,
    ResourceBusy,
    OutOfMemory,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegSpawnPlatform {
    Linux,
    Macos,
    Windows,
    Other,
}

impl FfmpegSpawnPlatform {
    pub const fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegSpawnDiagnostic {
    pub kind: FfmpegSpawnKind,
    pub native_code: Option<i32>,
    pub platform: FfmpegSpawnPlatform,
}

impl FfmpegSpawnDiagnostic {
    fn from_io(error: &std::io::Error) -> Self {
        use std::io::ErrorKind;
        let kind = match error.kind() {
            ErrorKind::NotFound => FfmpegSpawnKind::NotFound,
            ErrorKind::PermissionDenied => FfmpegSpawnKind::PermissionDenied,
            ErrorKind::InvalidInput => FfmpegSpawnKind::InvalidInput,
            ErrorKind::InvalidFilename => FfmpegSpawnKind::InvalidFilename,
            ErrorKind::ArgumentListTooLong => FfmpegSpawnKind::ArgumentListTooLong,
            ErrorKind::ResourceBusy => FfmpegSpawnKind::ResourceBusy,
            ErrorKind::OutOfMemory => FfmpegSpawnKind::OutOfMemory,
            _ => FfmpegSpawnKind::Other,
        };
        Self {
            kind,
            native_code: error.raw_os_error(),
            platform: FfmpegSpawnPlatform::current(),
        }
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
    pub ffmpeg_spawn: Option<Box<FfmpegSpawnDiagnostic>>,
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
            ffmpeg_spawn: None,
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
            ffmpeg_spawn: None,
        }
    }

    pub fn ffmpeg_spawn_failure(error: std::io::Error) -> Self {
        let mut result = Self::render_failure("spawn", None, None);
        result.ffmpeg_spawn = Some(Box::new(FfmpegSpawnDiagnostic::from_io(&error)));
        result
    }

    pub(crate) fn io(context: &str, error: std::io::Error) -> Self {
        Self::new(ErrorCode::InternalError, format!("{context}: {error}"))
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        let message = error.to_string();
        if let Some(detail) = message.strip_prefix(GROUP_COMPOSITING_DECODE_ERROR_PREFIX) {
            return Self::new(
                ErrorCode::InvalidArgument,
                format!("invalid group compositing metadata: {detail}"),
            );
        }
        if let Some(detail) = message.strip_prefix(COLOR_EFFECT_DECODE_ERROR_PREFIX) {
            return Self::new(
                ErrorCode::InvalidArgument,
                format!("invalid color effect metadata: {detail}"),
            );
        }
        if let Some(detail) = message.strip_prefix(BLEND_DECODE_ERROR_PREFIX) {
            return Self::new(
                ErrorCode::InvalidArgument,
                format!("invalid blend metadata: {detail}"),
            );
        }
        if let Some(detail) = message.strip_prefix(MATTE_DECODE_ERROR_PREFIX) {
            return Self::new(
                ErrorCode::InvalidArgument,
                format!("invalid track matte metadata: {detail}"),
            );
        }
        if let Some(detail) = message.strip_prefix(MASK_ACTIVATION_DECODE_ERROR_PREFIX) {
            return Self::new(
                ErrorCode::InvalidArgument,
                format!("invalid mask activation: {detail}"),
            );
        }
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
    fn spawn_kind_vocabulary_and_native_codes_match_the_canonical_fixture() {
        use std::io::ErrorKind;
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/ffmpeg-spawn-diagnostics-v1.json"
        ))
        .unwrap();
        let kinds = [
            ErrorKind::NotFound,
            ErrorKind::PermissionDenied,
            ErrorKind::InvalidInput,
            ErrorKind::InvalidFilename,
            ErrorKind::ArgumentListTooLong,
            ErrorKind::ResourceBusy,
            ErrorKind::OutOfMemory,
            ErrorKind::Other,
        ];
        for (index, kind) in kinds.into_iter().enumerate() {
            let error = CoreError::ffmpeg_spawn_failure(std::io::Error::new(kind, "private"));
            let value = serde_json::to_value(error.ffmpeg_spawn.unwrap()).unwrap();
            assert_eq!(value["kind"], fixture["kinds"][index]);
            assert!(value["nativeCode"].is_null());
        }
        let platform = serde_json::to_value(FfmpegSpawnPlatform::current()).unwrap();
        if let Some(cases) = fixture["nativeCases"][platform.as_str().unwrap()].as_array() {
            for case in cases {
                let code = i32::try_from(case["nativeCode"].as_i64().unwrap()).unwrap();
                let error =
                    CoreError::ffmpeg_spawn_failure(std::io::Error::from_raw_os_error(code));
                let value = serde_json::to_value(error.ffmpeg_spawn.unwrap()).unwrap();
                assert_eq!(value["kind"], case["kind"]);
                assert_eq!(value["nativeCode"], code);
                assert_eq!(value["platform"], platform);
                assert_eq!(value.as_object().unwrap().len(), 3);
                assert_eq!(error.code, ErrorCode::FfmpegFailed);
                assert_eq!(error.failed_stage.as_deref(), Some("spawn"));
                assert!(!error.retryable);
                assert!(error.ffmpeg_exit_code.is_none());
                assert!(error.ffmpeg_stderr_excerpt.is_none());
            }
        }
    }

    #[test]
    fn spawn_diagnostics_never_retain_custom_error_text_and_other_errors_stay_absent() {
        for kind in [std::io::ErrorKind::BrokenPipe, std::io::ErrorKind::Other] {
            let error = CoreError::ffmpeg_spawn_failure(std::io::Error::new(
                kind,
                "private /home/project/media C:\\secret\\clip.mp4 --password=secret ENV=secret",
            ));
            assert_eq!(error.message, "FFmpeg render failed");
            let diagnostic = error.ffmpeg_spawn.as_ref().unwrap();
            assert_eq!(diagnostic.kind, FfmpegSpawnKind::Other);
            assert_eq!(diagnostic.native_code, None);
            let text = format!("{error:?}");
            assert!(!text.contains("secret"));
            assert!(!text.contains("private"));
            assert!(serde_json::to_vec(diagnostic).unwrap().len() < 128);
        }
        for error in [
            CoreError::new(ErrorCode::RevisionConflict, "stale"),
            CoreError::new(ErrorCode::DependencyUnavailable, "missing tooling"),
            CoreError::render_failure("spawn", None, None),
            CoreError::render_failure("render", Some(7), Some("safe child stderr".into())),
        ] {
            assert!(error.ffmpeg_spawn.is_none());
        }
    }

    #[test]
    fn color_decode_classification_requires_the_exact_anchored_marker() {
        use serde::de::Error;
        let error = CoreError::from(serde_json::Error::custom(format!(
            "{COLOR_EFFECT_DECODE_ERROR_PREFIX}premature color"
        )));
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        for message in [
            format!("user field {COLOR_EFFECT_DECODE_ERROR_PREFIX}premature color"),
            "OPENCUT_COLOR_EFFECT_DECODE: ordinary text".into(),
        ] {
            assert_eq!(
                CoreError::from(serde_json::Error::custom(message)).code,
                ErrorCode::InternalError
            );
        }
    }
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
