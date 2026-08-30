use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{CoreError, ErrorCode};

pub const PROJECT_SCHEMA_VERSION: u32 = 6;

fn deserialize_double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
        match self {
            Self::Media(item) => item.hidden,
            Self::Text(item) => item.hidden,
            Self::SolidColor(item) => item.hidden,
            Self::Rectangle(item) => item.hidden,
            Self::Caption(item) => item.hidden,
            Self::Transition(item) => item.hidden,
        }
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        match self {
            Self::Media(item) => item.hidden = hidden,
            Self::Text(item) => item.hidden = hidden,
            Self::SolidColor(item) => item.hidden = hidden,
            Self::Rectangle(item) => item.hidden = hidden,
            Self::Caption(item) => item.hidden = hidden,
            Self::Transition(item) => item.hidden = hidden,
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
    pub transform: Transform,
    pub audio: AudioSettings,
    pub keyframes: Vec<Keyframe>,
    #[serde(default)]
    pub hidden: bool,
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
    pub transform: Transform,
    pub keyframes: Vec<Keyframe>,
    #[serde(default)]
    pub hidden: bool,
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
    pub transform: Transform,
    pub keyframes: Vec<Keyframe>,
    #[serde(default)]
    pub hidden: bool,
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
    pub transform: Transform,
    pub keyframes: Vec<Keyframe>,
    #[serde(default)]
    pub hidden: bool,
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
    #[serde(default)]
    pub hidden: bool,
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
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "operation",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum EditOperation {
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
            transform: Transform::default(),
            keyframes: vec![],
            hidden: false,
        });
        assert!(item.overlaps(0, 1_001));
        assert!(!item.overlaps(0, 1_000));
        assert!(!item.overlaps(2_000, 3_000));
    }
}
