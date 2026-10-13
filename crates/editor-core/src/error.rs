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

/// Closed native accounting identifiers. Values describe admission work, not RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderBudget {
    ShapeSegments,
    SceneShapeSegments,
    SceneShapeRasterBytes,
    CandidateAnalysisNodes,
    EffectPixelPasses,
    MotionBlurPixelWork,
    CompositionPixelWork,
    CompositionBytes,
}

impl RenderBudget {
    const fn units(self) -> &'static str {
        match self {
            Self::ShapeSegments | Self::SceneShapeSegments => "segments",
            Self::SceneShapeRasterBytes | Self::CompositionBytes => "bytes",
            Self::CandidateAnalysisNodes => "nodes",
            Self::EffectPixelPasses => "pixel_passes",
            Self::MotionBlurPixelWork | Self::CompositionPixelWork => "pixel_samples",
        }
    }

    const fn measurement(self) -> &'static str {
        match self {
            Self::SceneShapeRasterBytes | Self::EffectPixelPasses | Self::CompositionBytes => {
                "conservative"
            }
            _ => "charged",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBudgetDiagnostic {
    pub budget: RenderBudget,
    pub limit: String,
    pub work: Option<String>,
    pub units: &'static str,
    pub measurement: &'static str,
    pub overflow: bool,
    pub contributors: Vec<RenderBudgetContributor>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderContributorKind {
    Effect,
    Grid,
    MotionBlur,
    Shape,
    Svg,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RenderBudgetContributor {
    pub kind: RenderContributorKind,
    pub work: String,
}

/// Retains only the largest eight owner-computed charges, never source identities.
#[derive(Default)]
pub(crate) struct BudgetContributions(Vec<(RenderContributorKind, u64)>);

impl BudgetContributions {
    pub(crate) fn add(&mut self, kind: RenderContributorKind, work: u64) {
        self.0.push((kind, work));
        self.0.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        self.0.truncate(8);
    }

    pub(crate) fn values(&self) -> Vec<RenderBudgetContributor> {
        self.0
            .iter()
            .map(|&(kind, work)| RenderBudgetContributor {
                kind,
                work: work.to_string(),
            })
            .collect()
    }
}

pub(crate) fn budget_contributor(
    kind: RenderContributorKind,
    work: u64,
) -> Vec<RenderBudgetContributor> {
    vec![RenderBudgetContributor {
        kind,
        work: work.to_string(),
    }]
}

/// Report the same checked owner charge while preserving its original messages.
pub(crate) fn charge_render_work(
    total: &mut u64,
    charge: Option<u64>,
    budget: RenderBudget,
    limit: u64,
    messages: (&'static str, &'static str),
    kind: RenderContributorKind,
) -> Result<(), CoreError> {
    let attempted = charge.and_then(|value| total.checked_add(value));
    let message = if charge.is_none() {
        messages.0
    } else {
        messages.1
    };
    let value = attempted.filter(|value| *value <= limit).ok_or_else(|| {
        let contributors = charge.map_or_else(Vec::new, |value| budget_contributor(kind, value));
        CoreError::budget(message, budget, limit, attempted, contributors)
    })?;
    *total = value;
    #[cfg(test)]
    dense_scene_observations::charge(budget, limit, value);
    Ok(())
}

#[cfg(test)]
pub(crate) mod dense_scene_observations;

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
    pub render_budget: Option<Box<RenderBudgetDiagnostic>>,
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
            render_budget: None,
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
            render_budget: None,
        }
    }

    pub fn ffmpeg_spawn_failure(error: std::io::Error) -> Self {
        let mut result = Self::render_failure("spawn", None, None);
        result.ffmpeg_spawn = Some(Box::new(FfmpegSpawnDiagnostic::from_io(&error)));
        result
    }

    pub(crate) fn budget(
        message: &'static str,
        budget: RenderBudget,
        limit: u64,
        work: Option<u64>,
        contributors: Vec<RenderBudgetContributor>,
    ) -> Self {
        let mut error = Self::new(ErrorCode::InvalidArgument, message);
        error.render_budget = Some(Box::new(RenderBudgetDiagnostic {
            budget,
            limit: limit.to_string(),
            work: work.map(|value| value.to_string()),
            units: budget.units(),
            measurement: budget.measurement(),
            overflow: work.is_none(),
            contributors,
        }));
        error
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
    fn native_budget_catalog_matches_typed_identifiers_and_lossless_error_payload() {
        let catalog: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/render-budget-diagnostics-v1.json"
        ))
        .unwrap();
        for budget in [
            RenderBudget::ShapeSegments,
            RenderBudget::SceneShapeSegments,
            RenderBudget::SceneShapeRasterBytes,
            RenderBudget::CandidateAnalysisNodes,
            RenderBudget::EffectPixelPasses,
            RenderBudget::MotionBlurPixelWork,
            RenderBudget::CompositionPixelWork,
            RenderBudget::CompositionBytes,
        ] {
            let name = serde_json::to_value(budget).unwrap();
            assert_eq!(
                catalog["budgets"][name.as_str().unwrap()],
                serde_json::json!([budget.units(), budget.measurement()])
            );
        }
        let error = CoreError::budget(
            "scene shape segment limit exceeded",
            RenderBudget::SceneShapeSegments,
            1_048_576,
            Some(1_048_577),
            budget_contributor(RenderContributorKind::Shape, 1),
        );
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert!(error.failed_stage.is_none());
        assert_eq!(
            serde_json::to_value(error.render_budget).unwrap(),
            catalog["fixtures"][0]["value"]
        );
        let large = CoreError::budget(
            "bound",
            RenderBudget::SceneShapeSegments,
            u64::MAX,
            Some(u64::MAX),
            vec![],
        );
        assert_eq!(
            large.render_budget.unwrap().work.as_deref(),
            Some("18446744073709551615")
        );
        assert!(
            CoreError::new(ErrorCode::RevisionConflict, "stale revision")
                .render_budget
                .is_none()
        );
        assert!(
            CoreError::render_failure("render", None, None)
                .render_budget
                .is_none()
        );
    }

    #[test]
    fn native_budget_charges_preserve_exact_boundary_overflow_and_contributor_bounds() {
        let mut total = 9;
        charge_render_work(
            &mut total,
            Some(1),
            RenderBudget::MotionBlurPixelWork,
            10,
            ("overflow", "limit"),
            RenderContributorKind::MotionBlur,
        )
        .unwrap();
        let error = charge_render_work(
            &mut total,
            Some(1),
            RenderBudget::MotionBlurPixelWork,
            10,
            ("overflow", "limit"),
            RenderContributorKind::MotionBlur,
        )
        .unwrap_err();
        assert_eq!(total, 10);
        let details = error.render_budget.unwrap();
        assert_eq!(details.work.as_deref(), Some("11"));
        assert_eq!(details.limit, "10");
        assert!(!details.overflow);
        total = u64::MAX;
        let error = charge_render_work(
            &mut total,
            Some(1),
            RenderBudget::MotionBlurPixelWork,
            10,
            ("overflow", "limit"),
            RenderContributorKind::MotionBlur,
        )
        .unwrap_err();
        let details = error.render_budget.unwrap();
        assert!(details.overflow);
        assert!(details.work.is_none());
        assert_eq!(total, u64::MAX);
        let mut contributions = BudgetContributions::default();
        for value in 0..24 {
            contributions.add(RenderContributorKind::Shape, value);
            contributions.add(RenderContributorKind::Grid, value);
        }
        let values = contributions.values();
        assert_eq!(values.len(), 8);
        assert_eq!(values[0].kind, RenderContributorKind::Grid);
        assert_eq!(values[0].work, "23");
        assert_eq!(values[1].kind, RenderContributorKind::Shape);
        assert_eq!(values[7].work, "20");
        assert!(!serde_json::to_string(&values).unwrap().contains("/"));
    }

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
