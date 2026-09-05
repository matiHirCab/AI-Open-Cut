use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{CoreError, ErrorCode};

pub const PROJECT_SCHEMA_VERSION: u32 = 9;

fn deserialize_double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[serde(try_from = "ProjectDocument")]
pub struct Project {
    pub schema_version: u32,
    pub id: String,
    pub revision: u64,
    pub name: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub settings: ProjectSettings,
    pub assets: Vec<Asset>,
    pub tracks: Vec<Track>,
}

// Keep older documents readable while requiring explicit stacking in schema 9.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectDocument {
    schema_version: u32,
    id: String,
    revision: u64,
    name: String,
    created_at_ms: u64,
    updated_at_ms: u64,
    settings: ProjectSettings,
    assets: Vec<Asset>,
    tracks: serde_json::Value,
}

impl TryFrom<ProjectDocument> for Project {
    type Error = String;

    fn try_from(value: ProjectDocument) -> Result<Self, Self::Error> {
        if value.schema_version == 9 {
            for track in value.tracks.as_array().into_iter().flatten() {
                for item in track["items"].as_array().into_iter().flatten() {
                    if item.get("zIndex").is_none() || item.get("stackOrder").is_none() {
                        return Err("schema 9 requires zIndex and stackOrder".into());
                    }
                }
            }
        }
        Ok(Self {
            schema_version: value.schema_version,
            id: value.id,
            revision: value.revision,
            name: value.name,
            created_at_ms: value.created_at_ms,
            updated_at_ms: value.updated_at_ms,
            settings: value.settings,
            assets: value.assets,
            tracks: serde_json::from_value(value.tracks).map_err(|error| error.to_string())?,
        })
    }
}

impl Project {
    pub fn duration_ms(&self) -> u64 {
        self.tracks
            .iter()
            .flat_map(|track| &track.items)
            .map(TimelineItem::end_ms)
            .max()
            .unwrap_or(0)
    }

    pub fn find_item(&self, item_id: &str) -> Option<&TimelineItem> {
        self.tracks
            .iter()
            .flat_map(|track| &track.items)
            .find(|item| item.id() == item_id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            width: 1_920,
            height: 1_080,
            fps: 30,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SpeechVoiceId(pub String);

impl SpeechVoiceId {
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_speech_identifier(&self.0, "speech voice ID")
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSynthesisRequest {
    pub text: String,
    pub language: String,
    pub voice_id: SpeechVoiceId,
    pub speed: f64,
    #[serde(default)]
    pub text_options: SpeechTextOptions,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechNormalization {
    None,
    #[default]
    Basic,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechChunking {
    None,
    #[default]
    Sentence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechPronunciation {
    pub term: String,
    pub spoken: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechTextOptions {
    pub normalization: SpeechNormalization,
    pub pronunciations: Vec<SpeechPronunciation>,
    pub chunking: SpeechChunking,
    pub sentence_pause_ms: u64,
}

impl Default for SpeechTextOptions {
    fn default() -> Self {
        Self {
            normalization: SpeechNormalization::Basic,
            pronunciations: vec![],
            chunking: SpeechChunking::Sentence,
            sentence_pause_ms: 120,
        }
    }
}

impl SpeechTextOptions {
    fn validate(&self) -> Result<(), CoreError> {
        if self.pronunciations.len() > 100 || self.sentence_pause_ms > 5_000 {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "speech text options exceed supported limits",
            ));
        }
        let mut terms = std::collections::HashSet::new();
        for pronunciation in &self.pronunciations {
            let term = pronunciation.term.trim();
            let spoken = pronunciation.spoken.trim();
            if term.is_empty()
                || spoken.is_empty()
                || term.chars().count() > 128
                || spoken.chars().count() > 256
                || !terms.insert(term)
            {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "speech pronunciations must be unique, non-empty, and bounded",
                ));
            }
        }
        Ok(())
    }
}

impl SpeechSynthesisRequest {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.text.trim().is_empty() {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "speech text cannot be empty",
            ));
        }
        validate_speech_identifier(&self.language, "speech language")?;
        self.voice_id.validate()?;
        self.text_options.validate()?;
        if !self.speed.is_finite() || self.speed <= 0.0 {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "speech speed must be finite and positive",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechGeneration {
    pub request: SpeechSynthesisRequest,
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub sample_rate_hz: u32,
    pub generated_at_ms: u64,
}

impl SpeechGeneration {
    pub fn validate(&self) -> Result<(), CoreError> {
        self.request.validate()?;
        validate_speech_identifier(&self.provider_id, "speech provider ID")?;
        validate_speech_identifier(&self.model_id, "speech model ID")?;
        if self
            .model_version
            .as_ref()
            .is_some_and(|version| version.trim().is_empty())
        {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "speech model version cannot be empty",
            ));
        }
        if self.sample_rate_hz == 0 {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "speech sample rate must be positive",
            ));
        }
        if self.generated_at_ms == 0 {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "speech generation timestamp must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "generation", rename_all = "snake_case")]
pub enum GeneratedAssetOrigin {
    SpeechSynthesis(SpeechGeneration),
}

impl GeneratedAssetOrigin {
    pub fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::SpeechSynthesis(generation) => generation.validate(),
        }
    }
}

fn validate_speech_identifier(value: &str, name: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            format!("{name} cannot be empty"),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Asset {
    pub id: String,
    pub media_type: MediaType,
    pub file_name: String,
    pub project_relative_path: String,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub has_audio: bool,
    #[serde(default)]
    pub origin: Option<GeneratedAssetOrigin>,
    #[serde(default)]
    pub content_hash: Option<ContentHash>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub probe: Option<MediaProbeFacts>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentHash {
    pub algorithm: String,
    pub digest: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaProbeFacts {
    pub duration_ms: Option<u64>,
    pub has_audio: bool,
    pub has_video: bool,
    pub format_name: Option<String>,
    pub video_codec: Option<String>,
    pub video_width: Option<u32>,
    pub video_height: Option<u32>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<u32>,
    pub audio_sample_rate_hz: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackType {
    Video,
    Overlay,
    Audio,
    Caption,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub track_type: TrackType,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub audio_role: AudioTrackRole,
    #[serde(default)]
    pub ducking: Option<DuckingSettings>,
    pub items: Vec<TimelineItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimelineItem {
    Media(MediaItem),
    Text(TextItem),
    SolidColor(SolidColorItem),
    Rectangle(RectangleItem),
    Caption(CaptionItem),
    Transition(TransitionItem),
}

impl TimelineItem {
    pub fn id(&self) -> &str {
        match self {
            Self::Media(item) => &item.id,
            Self::Text(item) => &item.id,
            Self::SolidColor(item) => &item.id,
            Self::Rectangle(item) => &item.id,
            Self::Caption(item) => &item.id,
            Self::Transition(item) => &item.id,
        }
    }

    pub fn start_ms(&self) -> u64 {
        match self {
            Self::Media(item) => item.start_ms,
            Self::Text(item) => item.start_ms,
            Self::SolidColor(item) => item.start_ms,
            Self::Rectangle(item) => item.start_ms,
            Self::Caption(item) => item.start_ms,
            Self::Transition(item) => item.start_ms,
        }
    }

    pub fn duration_ms(&self) -> u64 {
        match self {
            Self::Media(item) => item.duration_ms,
            Self::Text(item) => item.duration_ms,
            Self::SolidColor(item) => item.duration_ms,
            Self::Rectangle(item) => item.duration_ms,
            Self::Caption(item) => item.duration_ms,
            Self::Transition(item) => item.duration_ms,
        }
    }

    pub fn end_ms(&self) -> u64 {
        self.start_ms().saturating_add(self.duration_ms())
    }

    pub fn overlaps(&self, start_ms: u64, end_ms: u64) -> bool {
        self.start_ms() < end_ms && self.end_ms() > start_ms
    }

    pub fn keyframes(&self) -> &[Keyframe] {
        match self {
            Self::Media(v) => &v.keyframes,
            Self::Text(v) => &v.keyframes,
            Self::SolidColor(v) => &v.keyframes,
            Self::Rectangle(v) => &v.keyframes,
            Self::Caption(_) | Self::Transition(_) => &[],
        }
    }

    pub fn keyframes_mut(&mut self) -> Option<&mut Vec<Keyframe>> {
        match self {
            Self::Media(item) => Some(&mut item.keyframes),
            Self::Text(item) => Some(&mut item.keyframes),
            Self::SolidColor(item) => Some(&mut item.keyframes),
            Self::Rectangle(item) => Some(&mut item.keyframes),
            Self::Caption(_) => None,
            Self::Transition(_) => None,
        }
    }

    pub fn hidden(&self) -> bool {
        self.visual_properties().hidden
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.visual_properties_mut().hidden = hidden;
    }

    pub fn visual_properties(&self) -> &VisualProperties {
        match self {
            Self::Media(item) => &item.visual_properties,
            Self::Text(item) => &item.visual_properties,
            Self::SolidColor(item) => &item.visual_properties,
            Self::Rectangle(item) => &item.visual_properties,
            Self::Caption(item) => &item.visual_properties,
            Self::Transition(item) => &item.visual_properties,
        }
    }

    pub fn visual_properties_mut(&mut self) -> &mut VisualProperties {
        match self {
            Self::Media(item) => &mut item.visual_properties,
            Self::Text(item) => &mut item.visual_properties,
            Self::SolidColor(item) => &mut item.visual_properties,
            Self::Rectangle(item) => &mut item.visual_properties,
            Self::Caption(item) => &mut item.visual_properties,
            Self::Transition(item) => &mut item.visual_properties,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualProperties {
    #[serde(default)]
    pub z_index: i32,
    #[serde(default)]
    pub stack_order: u32,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform2d: Option<Transform2D>,
    #[serde(default)]
    pub hidden: bool,
}

impl VisualProperties {
    pub fn new(transform: Transform, hidden: bool) -> Self {
        Self {
            transform,
            hidden,
            transform2d: None,
            z_index: 0,
            stack_order: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaItem {
    pub id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub source_in_ms: u64,
    #[serde(flatten)]
    pub visual_properties: VisualProperties,
    pub audio: AudioSettings,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextItem {
    pub id: String,
    pub text: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub font_size: u32,
    pub color: String,
    pub font_family: Option<String>,
    #[serde(default)]
    pub font_path: Option<String>,
    #[serde(default)]
    pub style: TextStyle,
    #[serde(flatten)]
    pub visual_properties: VisualProperties,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorPoint {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextShadow {
    pub color: String,
    pub opacity: f64,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl Default for TextShadow {
    fn default() -> Self {
        Self {
            color: "#000000".into(),
            opacity: 0.0,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextPadding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextStyle {
    #[serde(default)]
    pub alignment: TextAlignment,
    #[serde(default)]
    pub wrap_width_px: Option<u32>,
    #[serde(default)]
    pub line_spacing_px: i32,
    #[serde(default = "default_black")]
    pub outline_color: String,
    #[serde(default)]
    pub outline_width_px: u32,
    #[serde(default)]
    pub shadow: TextShadow,
    #[serde(default = "default_black")]
    pub background_color: String,
    #[serde(default)]
    pub background_opacity: f64,
    #[serde(default)]
    pub padding: TextPadding,
    #[serde(default)]
    pub anchor: AnchorPoint,
}

fn default_black() -> String {
    "#000000".into()
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            alignment: TextAlignment::Left,
            wrap_width_px: None,
            line_spacing_px: 0,
            outline_color: default_black(),
            outline_width_px: 0,
            shadow: TextShadow::default(),
            background_color: default_black(),
            background_opacity: 0.0,
            padding: TextPadding::default(),
            anchor: AnchorPoint::TopLeft,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolidColorItem {
    pub id: String,
    pub color: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    #[serde(flatten)]
    pub visual_properties: VisualProperties,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RectangleItem {
    pub id: String,
    pub color: String,
    pub width: u32,
    pub height: u32,
    pub start_ms: u64,
    pub duration_ms: u64,
    #[serde(flatten)]
    pub visual_properties: VisualProperties,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptionWord {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptionSource {
    pub asset_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub language: String,
    pub generated_at_ms: u64,
    pub original_text: String,
    pub confidence: Option<f64>,
    pub words: Vec<CaptionWord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptionStyle {
    pub font_size: u32,
    pub color: String,
    pub background_color: String,
    pub bottom_margin_px: u32,
}

impl Default for CaptionStyle {
    fn default() -> Self {
        Self {
            font_size: 48,
            color: "#ffffff".into(),
            background_color: "#000000".into(),
            bottom_margin_px: 64,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptionItem {
    pub id: String,
    pub text: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub style: CaptionStyle,
    pub source: CaptionSource,
    #[serde(flatten)]
    pub visual_properties: VisualProperties,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    Fade,
    Crossfade,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionItem {
    pub id: String,
    pub transition_type: TransitionType,
    pub from_item_id: String,
    pub to_item_id: Option<String>,
    pub start_ms: u64,
    pub duration_ms: u64,
    #[serde(flatten)]
    pub visual_properties: VisualProperties,
}

macro_rules! impl_visual_properties_access {
    ($($item:ty),+ $(,)?) => {
        $(
            impl std::ops::Deref for $item {
                type Target = VisualProperties;

                fn deref(&self) -> &Self::Target {
                    &self.visual_properties
                }
            }

            impl std::ops::DerefMut for $item {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.visual_properties
                }
            }
        )+
    };
}

impl_visual_properties_access!(
    MediaItem,
    TextItem,
    SolidColorItem,
    RectangleItem,
    CaptionItem,
    TransitionItem,
);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transform {
    pub position_x: f64,
    pub position_y: f64,
    pub scale: f64,
    pub opacity: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            scale: 1.0,
            opacity: 1.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioSettings {
    pub volume: f64,
    pub muted: bool,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioTrackRole {
    #[default]
    Unassigned,
    Voiceover,
    Music,
    SoundEffects,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuckingSettings {
    pub enabled: bool,
    pub gain: f64,
    pub attack_ms: u64,
    pub release_ms: u64,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            muted: false,
            fade_in_ms: 0,
            fade_out_ms: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyframeProperty {
    Position,
    Scale,
    Opacity,
    Volume,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    Hold,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Keyframe {
    pub property: KeyframeProperty,
    pub time_ms: u64,
    pub value: KeyframeValue,
    pub easing: Easing,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KeyframeValue {
    Position { x: f64, y: f64 },
    Scalar { value: f64 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

impl TimeRange {
    pub fn validate(&self) -> bool {
        self.start_ms < self.end_ms
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectState {
    pub project: Project,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(
    tag = "operation",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum EditOperation {
    ItemSetZIndex {
        item_id: String,
        z_index: i32,
    },
    ItemReorder {
        item_id: String,
        index: usize,
    },
    TrackReorder {
        track_id: String,
        index: usize,
    },
    AddMedia {
        track_id: String,
        asset_id: String,
        start_ms: u64,
        duration_ms: u64,
        source_in_ms: u64,
    },
    AddText {
        track_id: String,
        text: String,
        start_ms: u64,
        duration_ms: u64,
        font_size: u32,
        color: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_family: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_path: Option<String>,
        #[serde(default)]
        style: TextStyle,
        transform: Transform,
    },
    AddSolidColor {
        track_id: String,
        color: String,
        start_ms: u64,
        duration_ms: u64,
        transform: Transform,
    },
    AddRectangle {
        track_id: String,
        color: String,
        width: u32,
        height: u32,
        start_ms: u64,
        duration_ms: u64,
        transform: Transform,
    },
    UpdateItem {
        item_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        transform: Option<Transform>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        transform2d: Option<Option<Transform2D>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        font_family: Option<Option<String>>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        font_path: Option<Option<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<TextStyle>,
    },
    MoveItem {
        item_id: String,
        track_id: String,
        start_ms: u64,
    },
    TrimItem {
        item_id: String,
        start_ms: u64,
        duration_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_in_ms: Option<u64>,
    },
    DeleteItem {
        item_id: String,
    },
    SetKeyframes {
        item_id: String,
        keyframes: Vec<Keyframe>,
    },
    AddTransition {
        track_id: String,
        transition_type: TransitionType,
        from_item_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        to_item_id: Option<String>,
        start_ms: u64,
        duration_ms: u64,
    },
    SetAudio {
        item_id: String,
        audio: AudioSettings,
    },
    SplitItem {
        item_id: String,
        split_ms: u64,
    },
    DuplicateItems {
        item_ids: Vec<String>,
        offset_ms: u64,
    },
    CreateTrack {
        name: String,
        track_type: TrackType,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
        #[serde(default)]
        audio_role: AudioTrackRole,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ducking: Option<DuckingSettings>,
    },
    UpdateTrack {
        track_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        locked: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hidden: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        muted: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        audio_role: Option<AudioTrackRole>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        ducking: Option<Option<DuckingSettings>>,
    },
    DeleteTrack {
        track_id: String,
    },
    SetItemVisibility {
        item_id: String,
        hidden: bool,
    },
}

#[derive(Deserialize)]
#[serde(
    remote = "EditOperation",
    tag = "operation",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum EditOperationDef {
    ItemSetZIndex {
        item_id: String,
        z_index: i32,
    },
    ItemReorder {
        item_id: String,
        index: usize,
    },
    TrackReorder {
        track_id: String,
        index: usize,
    },
    AddMedia {
        track_id: String,
        asset_id: String,
        start_ms: u64,
        duration_ms: u64,
        source_in_ms: u64,
    },
    AddText {
        track_id: String,
        text: String,
        start_ms: u64,
        duration_ms: u64,
        font_size: u32,
        color: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_family: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_path: Option<String>,
        #[serde(default)]
        style: TextStyle,
        transform: Transform,
    },
    AddSolidColor {
        track_id: String,
        color: String,
        start_ms: u64,
        duration_ms: u64,
        transform: Transform,
    },
    AddRectangle {
        track_id: String,
        color: String,
        width: u32,
        height: u32,
        start_ms: u64,
        duration_ms: u64,
        transform: Transform,
    },
    UpdateItem {
        item_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        transform: Option<Transform>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        transform2d: Option<Option<Transform2D>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        font_family: Option<Option<String>>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        font_path: Option<Option<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<TextStyle>,
    },
    MoveItem {
        item_id: String,
        track_id: String,
        start_ms: u64,
    },
    TrimItem {
        item_id: String,
        start_ms: u64,
        duration_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_in_ms: Option<u64>,
    },
    DeleteItem {
        item_id: String,
    },
    SetKeyframes {
        item_id: String,
        keyframes: Vec<Keyframe>,
    },
    AddTransition {
        track_id: String,
        transition_type: TransitionType,
        from_item_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        to_item_id: Option<String>,
        start_ms: u64,
        duration_ms: u64,
    },
    SetAudio {
        item_id: String,
        audio: AudioSettings,
    },
    SplitItem {
        item_id: String,
        split_ms: u64,
    },
    DuplicateItems {
        item_ids: Vec<String>,
        offset_ms: u64,
    },
    CreateTrack {
        name: String,
        track_type: TrackType,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
        #[serde(default)]
        audio_role: AudioTrackRole,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ducking: Option<DuckingSettings>,
    },
    UpdateTrack {
        track_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        locked: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hidden: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        muted: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        audio_role: Option<AudioTrackRole>,
        #[serde(
            default,
            deserialize_with = "deserialize_double_option",
            skip_serializing_if = "Option::is_none"
        )]
        ducking: Option<Option<DuckingSettings>>,
    },
    DeleteTrack {
        track_id: String,
    },
    SetItemVisibility {
        item_id: String,
        hidden: bool,
    },
}

impl<'de> Deserialize<'de> for EditOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let allowed: Option<&[&str]> = match value["operation"].as_str() {
            Some("item_set_z_index") => Some(&["operation", "itemId", "zIndex"]),
            Some("item_reorder") => Some(&["operation", "itemId", "index"]),
            Some("track_reorder") => Some(&["operation", "trackId", "index"]),
            _ => None,
        };
        if let Some(allowed) = allowed
            && value
                .as_object()
                .is_some_and(|fields| fields.keys().any(|key| !allowed.contains(&key.as_str())))
        {
            return Err(serde::de::Error::custom("unknown stacking operation field"));
        }
        EditOperationDef::deserialize(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEditOperation {
    #[serde(flatten)]
    pub edit: EditOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_alias: Option<String>,
}

impl From<EditOperation> for BatchEditOperation {
    fn from(edit: EditOperation) -> Self {
        Self {
            edit,
            result_alias: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[derive(Default)]
pub struct History {
    pub undo: Vec<Project>,
    pub redo: Vec<Project>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nullable_edit_fields_distinguish_missing_clear_and_set() {
        let clear_item: EditOperation = serde_json::from_value(serde_json::json!({
            "operation": "update_item",
            "itemId": "text",
            "fontFamily": null,
            "fontPath": null
        }))
        .unwrap();
        let EditOperation::UpdateItem {
            font_family,
            font_path,
            ..
        } = &clear_item
        else {
            panic!("expected update_item")
        };
        assert_eq!(font_family, &Some(None));
        assert_eq!(font_path, &Some(None));
        let serialized = serde_json::to_value(clear_item).unwrap();
        assert!(serialized["fontFamily"].is_null());
        assert!(serialized["fontPath"].is_null());

        let omitted_item: EditOperation = serde_json::from_value(serde_json::json!({
            "operation": "update_item",
            "itemId": "text"
        }))
        .unwrap();
        let EditOperation::UpdateItem {
            font_family,
            font_path,
            ..
        } = omitted_item
        else {
            panic!("expected update_item")
        };
        assert_eq!(font_family, None);
        assert_eq!(font_path, None);

        let clear_track: EditOperation = serde_json::from_value(serde_json::json!({
            "operation": "update_track",
            "trackId": "music",
            "ducking": null
        }))
        .unwrap();
        let EditOperation::UpdateTrack { ducking, .. } = clear_track else {
            panic!("expected update_track")
        };
        assert_eq!(ducking, Some(None));
    }

    #[test]
    fn speech_generation_uses_provider_neutral_json() {
        let origin = GeneratedAssetOrigin::SpeechSynthesis(SpeechGeneration {
            request: SpeechSynthesisRequest {
                text: "Hello".into(),
                language: "en-US".into(),
                voice_id: SpeechVoiceId("voice-1".into()),
                speed: 1.25,
                text_options: SpeechTextOptions::default(),
            },
            provider_id: "provider-1".into(),
            model_id: "model-1".into(),
            model_version: Some("2026-08".into()),
            sample_rate_hz: 24_000,
            generated_at_ms: 1_777_000_000_000,
        });

        let value = serde_json::to_value(&origin).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "type": "speech_synthesis",
                "generation": {
                    "request": {
                        "text": "Hello",
                        "language": "en-US",
                        "voiceId": "voice-1",
                        "speed": 1.25,
                        "textOptions": {
                            "normalization": "basic",
                            "pronunciations": [],
                            "chunking": "sentence",
                            "sentencePauseMs": 120
                        }
                    },
                    "providerId": "provider-1",
                    "modelId": "model-1",
                    "modelVersion": "2026-08",
                    "sampleRateHz": 24_000,
                    "generatedAtMs": 1_777_000_000_000_u64
                }
            })
        );
        assert_eq!(
            serde_json::from_value::<GeneratedAssetOrigin>(value.clone()).unwrap(),
            origin
        );
        let mut legacy = value;
        legacy["generation"]["request"]
            .as_object_mut()
            .unwrap()
            .remove("textOptions");
        let GeneratedAssetOrigin::SpeechSynthesis(legacy) =
            serde_json::from_value::<GeneratedAssetOrigin>(legacy).unwrap();
        assert_eq!(legacy.request.text_options, SpeechTextOptions::default());

        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project-1".into(),
            revision: 0,
            name: "Speech".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![Asset {
                id: "asset-1".into(),
                media_type: MediaType::Audio,
                file_name: "speech.wav".into(),
                project_relative_path: "assets/speech.wav".into(),
                duration_ms: Some(1_000),
                has_audio: true,
                origin: Some(origin.clone()),
                content_hash: None,
                size_bytes: None,
                probe: None,
            }],
            tracks: vec![],
        };
        let restored: Project =
            serde_json::from_value(serde_json::to_value(project).unwrap()).unwrap();
        let restored_origin = restored.assets[0].origin.as_ref().unwrap();
        assert_eq!(restored.schema_version, PROJECT_SCHEMA_VERSION);
        assert_eq!(restored_origin, &origin);
    }

    #[test]
    fn speech_generation_validation_is_provider_neutral() {
        let valid = SpeechGeneration {
            request: SpeechSynthesisRequest {
                text: "Hello".into(),
                language: "en-US".into(),
                voice_id: SpeechVoiceId("provider-voice".into()),
                speed: 1.0,
                text_options: SpeechTextOptions::default(),
            },
            provider_id: "provider".into(),
            model_id: "model".into(),
            model_version: None,
            sample_rate_hz: 24_000,
            generated_at_ms: 1,
        };
        assert!(valid.validate().is_ok());

        let mut invalid = valid.clone();
        invalid.request.text = "   ".into();
        assert_eq!(
            invalid.validate().unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        invalid = valid.clone();
        invalid.request.speed = f64::NAN;
        assert_eq!(
            invalid.validate().unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        invalid = valid;
        invalid.sample_rate_hz = 0;
        assert_eq!(
            invalid.validate().unwrap_err().code,
            ErrorCode::ValidationFailed
        );
    }

    #[test]
    fn shared_speech_contract_matches_persisted_origin() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../contracts/speech-provider-v1.json");
        let contract: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let origin: GeneratedAssetOrigin =
            serde_json::from_value(contract["origin"].clone()).unwrap();
        let GeneratedAssetOrigin::SpeechSynthesis(generation) = origin;
        assert_eq!(generation.provider_id, contract["status"]["providerId"]);
        assert_eq!(generation.model_id, contract["synthesis"]["modelId"]);
        assert_eq!(
            generation.sample_rate_hz,
            contract["synthesis"]["sampleRateHz"].as_u64().unwrap() as u32
        );
        assert_eq!(
            generation.request.voice_id.0,
            contract["synthesis"]["voiceId"]
        );
        generation.validate().unwrap();
    }

    #[test]
    fn half_open_overlap_excludes_touching_items() {
        let item = TimelineItem::Text(TextItem {
            id: "text".into(),
            text: "hello".into(),
            start_ms: 1_000,
            duration_ms: 1_000,
            font_size: 48,
            color: "#ffffff".into(),
            font_family: None,
            font_path: None,
            style: TextStyle::default(),
            visual_properties: crate::VisualProperties::default(),
            keyframes: vec![],
        });
        assert!(item.overlaps(0, 1_001));
        assert!(!item.overlaps(0, 1_000));
        assert!(!item.overlaps(2_000, 3_000));
    }

    #[test]
    fn every_timeline_item_serializes_flattened_common_visual_properties() {
        let common = serde_json::json!({
            "transform": {
                "positionX": 12.0,
                "positionY": 34.0,
                "scale": 1.5,
                "opacity": 0.75
            },
            "hidden": true
        });
        let cases = [
            serde_json::json!({
                "type": "media", "id": "media", "assetId": "asset", "startMs": 0,
                "durationMs": 100, "sourceInMs": 0, "audio": {
                    "volume": 1.0, "muted": false, "fadeInMs": 0, "fadeOutMs": 0
                }, "keyframes": [], "transform": common["transform"], "hidden": true
            }),
            serde_json::json!({
                "type": "text", "id": "text", "text": "Text", "startMs": 0,
                "durationMs": 100, "fontSize": 24, "color": "#ffffff",
                "fontFamily": null, "fontPath": null, "style": TextStyle::default(),
                "keyframes": [], "transform": common["transform"], "hidden": true
            }),
            serde_json::json!({
                "type": "solid_color", "id": "solid", "color": "#000000",
                "startMs": 0, "durationMs": 100, "keyframes": [],
                "transform": common["transform"], "hidden": true
            }),
            serde_json::json!({
                "type": "rectangle", "id": "rectangle", "color": "#000000",
                "width": 10, "height": 20, "startMs": 0, "durationMs": 100,
                "keyframes": [], "transform": common["transform"], "hidden": true
            }),
            serde_json::json!({
                "type": "caption", "id": "caption", "text": "Caption", "startMs": 0,
                "durationMs": 100, "style": CaptionStyle::default(), "source": {
                    "assetId": "asset", "providerId": "provider", "modelId": "model",
                    "modelVersion": null, "language": "en", "generatedAtMs": 1,
                    "originalText": "Caption", "confidence": null, "words": []
                }, "transform": common["transform"], "hidden": true
            }),
            serde_json::json!({
                "type": "transition", "id": "transition", "transitionType": "fade",
                "fromItemId": "media", "toItemId": null, "startMs": 0,
                "durationMs": 100, "transform": common["transform"], "hidden": true
            }),
        ];

        for value in cases {
            let item: TimelineItem = serde_json::from_value(value).unwrap();
            assert_eq!(item.visual_properties().transform.position_x, 12.0);
            assert!(item.hidden());
            let serialized = serde_json::to_value(item).unwrap();
            assert_eq!(serialized["transform"], common["transform"]);
            assert_eq!(serialized["hidden"], common["hidden"]);
            assert!(serialized.get("visualProperties").is_none());
        }
    }

    #[test]
    fn legacy_caption_and_transition_default_common_visual_properties() {
        let caption: TimelineItem = serde_json::from_value(serde_json::json!({
            "type": "caption", "id": "caption", "text": "Caption", "startMs": 0,
            "durationMs": 100, "style": CaptionStyle::default(), "source": {
                "assetId": "asset", "providerId": "provider", "modelId": "model",
                "modelVersion": null, "language": "en", "generatedAtMs": 1,
                "originalText": "Caption", "confidence": null, "words": []
            }
        }))
        .unwrap();
        let transition: TimelineItem = serde_json::from_value(serde_json::json!({
            "type": "transition", "id": "transition", "transitionType": "fade",
            "fromItemId": "media", "toItemId": null, "startMs": 0, "durationMs": 100
        }))
        .unwrap();

        assert_eq!(caption.visual_properties(), &VisualProperties::default());
        assert_eq!(transition.visual_properties(), &VisualProperties::default());
        for item in [caption, transition] {
            let serialized = serde_json::to_value(item).unwrap();
            assert_eq!(serialized["transform"]["scale"], 1.0);
            assert_eq!(serialized["hidden"], false);
        }
    }
}

/// Static affine transform. Legacy Transform remains a separate compatibility value.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transform2D {
    pub position: TransformPosition,
    pub anchor: TransformAnchor,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation_deg: f64,
    pub skew_x_deg: f64,
    pub skew_y_deg: f64,
    pub opacity: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransformPosition {
    pub x: f64,
    pub y: f64,
    pub unit: PositionUnit,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransformAnchor {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionUnit {
    Pixels,
    Normalized,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: TransformPosition {
                x: 0.0,
                y: 0.0,
                unit: PositionUnit::Pixels,
            },
            anchor: TransformAnchor { x: 0.0, y: 0.0 },
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_deg: 0.0,
            skew_x_deg: 0.0,
            skew_y_deg: 0.0,
            opacity: 1.0,
        }
    }
}

impl Transform2D {
    pub fn validate(&self) -> Result<(), CoreError> {
        let position_limit = match self.position.unit {
            PositionUnit::Pixels => 1_000_000.0,
            PositionUnit::Normalized => 100.0,
        };
        let bounded = |v: f64, lo: f64, hi: f64| v.is_finite() && (lo..=hi).contains(&v);
        if !bounded(self.position.x, -position_limit, position_limit)
            || !bounded(self.position.y, -position_limit, position_limit)
            || !bounded(self.anchor.x, 0.0, 1.0)
            || !bounded(self.anchor.y, 0.0, 1.0)
            || !bounded(self.scale_x, 0.0, 100.0)
            || self.scale_x == 0.0
            || !bounded(self.scale_y, 0.0, 100.0)
            || self.scale_y == 0.0
            || !bounded(self.rotation_deg, -36_000.0, 36_000.0)
            || !bounded(self.skew_x_deg, -80.0, 80.0)
            || !bounded(self.skew_y_deg, -80.0, 80.0)
            || !bounded(self.opacity, 0.0, 1.0)
        {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "Transform2D exceeds its finite numeric bounds",
            ));
        }
        Ok(())
    }
}
