//! Owned, renderer-neutral evaluation of the current flat timeline model.
//!
//! This module deliberately contains no filesystem, process, renderer, or artifact
//! concerns. Production render planning consumes this representation through a
//! separate path-bearing resource-binding sidecar.

use std::collections::{HashMap, HashSet};

use crate::{
    AnchorPoint, Asset, AudioTrackRole, CoreError, Easing, ErrorCode, Keyframe, KeyframeProperty,
    KeyframeValue, MediaType, Project, TextAlignment, TextStyle, TimelineItem, Track, Transform,
    TransitionItem, TransitionType, animation::positive_scalar_ranges,
    validation::validate_project_stacking,
};

pub(crate) const MAX_EVALUATED_VISUAL_LAYERS: usize = 4_096;
pub(crate) const MAX_EVALUATED_MEDIA_RESOURCES: usize = 4_096;
pub(crate) const MAX_EVALUATED_AUDIO_LAYERS: usize = 4_096;
pub(crate) const MAX_EVALUATED_TRANSITION_FACTS: usize = 4_096;
pub(crate) const MAX_EVALUATED_KEYFRAMES_PER_CHANNEL: usize = 10_000;
pub(crate) const MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES: usize = 10_000;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedSceneResult {
    pub(crate) scene: EvaluatedScene,
    pub(crate) resource_bindings: SceneResourceBindings,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SceneResourceBindings {
    pub(crate) media: Vec<MediaResourceBinding>,
    pub(crate) fonts: Vec<FontResourceBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MediaResourceBinding {
    pub(crate) asset_id: String,
    pub(crate) project_relative_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FontResourceBinding {
    pub(crate) font_resource_id: String,
    pub(crate) requested_path: Option<String>,
    pub(crate) requested_family: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedScene {
    pub(crate) canvas: EvaluatedCanvas,
    pub(crate) duration_ms: u64,
    pub(crate) resources: Vec<EvaluatedMediaResource>,
    pub(crate) visual_layers: Vec<EvaluatedVisualLayer>,
    pub(crate) audio_layers: Vec<EvaluatedAudioLayer>,
    pub(crate) voiceover_intervals: Vec<EvaluatedTimeSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedCanvas {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedMediaKind {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedMediaResource {
    pub(crate) asset_id: String,
    pub(crate) kind: EvaluatedMediaKind,
    pub(crate) has_audio: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct EvaluatedLayerOrder {
    pub(crate) track_index: usize,
    pub(crate) item_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedTimeSpan {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedTransform {
    pub(crate) position_x: f64,
    pub(crate) position_y: f64,
    pub(crate) scale: f64,
    pub(crate) opacity: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedProperty {
    Position,
    Scale,
    Opacity,
    Volume,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedEasing {
    Hold,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EvaluatedKeyframeValue {
    Position { x: f64, y: f64 },
    Scalar { value: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedKeyframe {
    pub(crate) property: EvaluatedProperty,
    pub(crate) time_ms: u64,
    pub(crate) value: EvaluatedKeyframeValue,
    pub(crate) easing: EvaluatedEasing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedTransitionRole {
    In,
    Out,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedTransitionKind {
    Fade,
    Crossfade,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedTransition {
    pub(crate) role: EvaluatedTransitionRole,
    pub(crate) kind: EvaluatedTransitionKind,
    pub(crate) span: EvaluatedTimeSpan,
}

#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedVisualLayer {
    pub(crate) item_id: String,
    pub(crate) order: EvaluatedLayerOrder,
    pub(crate) span: EvaluatedTimeSpan,
    pub(crate) transform: EvaluatedTransform,
    pub(crate) transform2d: Option<crate::Transform2D>,
    pub(crate) affine: Option<EvaluatedAffine>,
    pub(crate) sampling_tiles: Option<Vec<EvaluatedAffine>>,
    pub(crate) ancestors: Option<EvaluatedAncestors>,
    pub(crate) source_size: Option<(u32, u32)>,
    pub(crate) keyframes: Vec<EvaluatedKeyframe>,
    pub(crate) transitions: Vec<EvaluatedTransition>,
    pub(crate) source: EvaluatedVisualSource,
}

// Absent additive facts carry no semantics; keep legacy scene diagnostics stable.
impl std::fmt::Debug for EvaluatedVisualLayer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut layer = formatter.debug_struct("EvaluatedVisualLayer");
        layer
            .field("item_id", &self.item_id)
            .field("order", &self.order)
            .field("span", &self.span)
            .field("transform", &self.transform);
        if let Some(value) = self.transform2d {
            layer.field("transform2d", &value);
        }
        if let Some(value) = self.ancestors {
            layer.field("ancestors", &value);
        }
        if let Some(value) = &self.sampling_tiles {
            layer.field("sampling_tiles", value);
        }
        if let Some(value) = self.affine {
            layer.field("affine", &value);
        }
        if let Some(value) = self.source_size {
            layer.field("source_size", &value);
        }
        layer
            .field("keyframes", &self.keyframes)
            .field("transitions", &self.transitions)
            .field("source", &self.source)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EvaluatedVisualSource {
    Media {
        asset_id: String,
        source_in_ms: u64,
    },
    Text(Box<EvaluatedText>),
    SolidColor {
        color: String,
    },
    Rectangle {
        color: String,
        width: u32,
        height: u32,
    },
    Caption(EvaluatedCaption),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedCaption {
    pub(crate) text: String,
    pub(crate) font_size: u32,
    pub(crate) color: String,
    pub(crate) background_color: String,
    pub(crate) bottom_margin_px: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedText {
    pub(crate) text: String,
    pub(crate) font_size: u32,
    pub(crate) color: String,
    pub(crate) font_resource_id: Option<String>,
    pub(crate) style: EvaluatedTextStyle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedAnchorPoint {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedTextPadding {
    pub(crate) top: u32,
    pub(crate) right: u32,
    pub(crate) bottom: u32,
    pub(crate) left: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedTextShadow {
    pub(crate) color: String,
    pub(crate) opacity: f64,
    pub(crate) offset_x: i32,
    pub(crate) offset_y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedTextStyle {
    pub(crate) alignment: EvaluatedTextAlignment,
    pub(crate) wrap_width_px: Option<u32>,
    pub(crate) line_spacing_px: i32,
    pub(crate) outline_color: String,
    pub(crate) outline_width_px: u32,
    pub(crate) shadow: EvaluatedTextShadow,
    pub(crate) background_color: String,
    pub(crate) background_opacity: f64,
    pub(crate) padding: EvaluatedTextPadding,
    pub(crate) anchor: EvaluatedAnchorPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedAudioRole {
    Unassigned,
    Voiceover,
    Music,
    SoundEffects,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedDucking {
    pub(crate) gain: f64,
    pub(crate) attack_ms: u64,
    pub(crate) release_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedAudioLayer {
    pub(crate) item_id: String,
    pub(crate) order: EvaluatedLayerOrder,
    pub(crate) asset_id: String,
    pub(crate) span: EvaluatedTimeSpan,
    pub(crate) source_in_ms: u64,
    pub(crate) volume: f64,
    pub(crate) fade_in_ms: u64,
    pub(crate) fade_out_ms: u64,
    pub(crate) volume_keyframes: Vec<EvaluatedKeyframe>,
    pub(crate) role: EvaluatedAudioRole,
    pub(crate) ducking: Option<EvaluatedDucking>,
}

struct EvaluationPreflight<'a> {
    visual_item_ids: HashSet<&'a str>,
    visual_layer_count: usize,
    media_resource_count: usize,
    audio_layer_count: usize,
    voiceover_activity_range_count: usize,
}

pub(crate) fn evaluate_project(
    project: &Project,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<EvaluatedSceneResult, CoreError> {
    if width == 0 || height == 0 || fps == 0 {
        return Err(invalid(
            "evaluated canvas dimensions and frame rate must be positive",
        ));
    }

    let asset_by_id = project
        .assets
        .iter()
        .map(|asset| (asset.id.as_str(), asset))
        .collect::<HashMap<_, _>>();
    validate_referenced_assets(project, &asset_by_id)?;
    validate_media_source_ranges(project, &asset_by_id)?;
    let preflight = preflight_project(project, &asset_by_id)?;
    validate_project_stacking(project)?;
    crate::validation::validate_parent_graph(project)?;
    let duration_ms = checked_project_duration(project)?.max(1);
    let transition_index = index_transitions(project, &preflight.visual_item_ids)?;
    let voiceover_intervals = audible_voiceover_intervals(
        project,
        &asset_by_id,
        preflight.voiceover_activity_range_count,
    )?;

    let mut resources = Vec::with_capacity(preflight.media_resource_count);
    let mut media_bindings = Vec::with_capacity(preflight.media_resource_count);
    let mut resource_indexes = HashSet::with_capacity(preflight.media_resource_count);
    let mut font_bindings = Vec::new();
    let mut visual_layers = Vec::with_capacity(preflight.visual_layer_count);
    let mut audio_layers = Vec::with_capacity(preflight.audio_layer_count);

    for (track_index, track) in project.tracks.iter().enumerate() {
        if track.hidden {
            continue;
        }
        for (item_index, item) in track.items.iter().enumerate() {
            if item.hidden() {
                continue;
            }
            let order = EvaluatedLayerOrder {
                track_index,
                item_index,
            };
            match item {
                TimelineItem::Media(media) => {
                    let span = checked_span(media.start_ms, media.duration_ms)?;
                    let asset = asset_by_id[media.asset_id.as_str()];
                    add_resource(
                        asset,
                        &mut resources,
                        &mut media_bindings,
                        &mut resource_indexes,
                    );
                    let keyframes = evaluate_keyframes(&media.keyframes)?;
                    let volume_keyframes = keyframes
                        .iter()
                        .copied()
                        .filter(|keyframe| keyframe.property == EvaluatedProperty::Volume)
                        .collect();
                    let transform = evaluate_transform(&media.transform)?;
                    if asset.media_type != MediaType::Audio {
                        visual_layers.push(EvaluatedVisualLayer {
                            transform2d: item.visual_properties().transform2d,
                            affine: None,
                            sampling_tiles: None,
                            ancestors: None,
                            source_size: None,
                            item_id: media.id.clone(),
                            order,
                            span,
                            transform,
                            keyframes,
                            transitions: transitions_for(&media.id, &transition_index),
                            source: EvaluatedVisualSource::Media {
                                asset_id: media.asset_id.clone(),
                                source_in_ms: media.source_in_ms,
                            },
                        });
                    }
                    if asset.has_audio && !track.muted && !media.audio.muted {
                        if !media.audio.volume.is_finite() {
                            return Err(invalid("evaluated audio volume must be finite"));
                        }
                        let ducking = evaluate_ducking(track, !voiceover_intervals.is_empty())?;
                        audio_layers.push(EvaluatedAudioLayer {
                            item_id: media.id.clone(),
                            order,
                            asset_id: media.asset_id.clone(),
                            span,
                            source_in_ms: media.source_in_ms,
                            volume: media.audio.volume,
                            fade_in_ms: media.audio.fade_in_ms,
                            fade_out_ms: media.audio.fade_out_ms,
                            volume_keyframes,
                            role: evaluate_audio_role(track.audio_role),
                            ducking,
                        });
                    }
                }
                TimelineItem::Text(text) => {
                    let font_resource_id = (text.font_path.is_some() || text.font_family.is_some())
                        .then(|| format!("text-font:{}", text.id));
                    if let Some(font_resource_id) = &font_resource_id {
                        font_bindings.push(FontResourceBinding {
                            font_resource_id: font_resource_id.clone(),
                            requested_path: text.font_path.clone(),
                            requested_family: text.font_family.clone(),
                        });
                    }
                    visual_layers.push(EvaluatedVisualLayer {
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: text.id.clone(),
                        order,
                        span: checked_span(text.start_ms, text.duration_ms)?,
                        transform: evaluate_transform(&text.transform)?,
                        keyframes: evaluate_keyframes(&text.keyframes)?,
                        transitions: transitions_for(&text.id, &transition_index),
                        source: EvaluatedVisualSource::Text(Box::new(EvaluatedText {
                            text: text.text.clone(),
                            font_size: text.font_size,
                            color: text.color.clone(),
                            font_resource_id,
                            style: evaluate_text_style(&text.style)?,
                        })),
                    });
                }
                TimelineItem::SolidColor(color) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: color.id.clone(),
                        order,
                        span: checked_span(color.start_ms, color.duration_ms)?,
                        transform: evaluate_transform(&color.transform)?,
                        keyframes: evaluate_keyframes(&color.keyframes)?,
                        transitions: transitions_for(&color.id, &transition_index),
                        source: EvaluatedVisualSource::SolidColor {
                            color: color.color.clone(),
                        },
                    });
                }
                TimelineItem::Rectangle(rectangle) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: rectangle.id.clone(),
                        order,
                        span: checked_span(rectangle.start_ms, rectangle.duration_ms)?,
                        transform: evaluate_transform(&rectangle.transform)?,
                        keyframes: evaluate_keyframes(&rectangle.keyframes)?,
                        transitions: transitions_for(&rectangle.id, &transition_index),
                        source: EvaluatedVisualSource::Rectangle {
                            color: rectangle.color.clone(),
                            width: rectangle.width,
                            height: rectangle.height,
                        },
                    });
                }
                TimelineItem::Caption(caption) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: caption.id.clone(),
                        order,
                        span: checked_span(caption.start_ms, caption.duration_ms)?,
                        transform: EvaluatedTransform {
                            position_x: 0.0,
                            position_y: 0.0,
                            scale: 1.0,
                            opacity: 1.0,
                        },
                        keyframes: vec![],
                        transitions: vec![],
                        source: EvaluatedVisualSource::Caption(EvaluatedCaption {
                            text: caption.text.clone(),
                            font_size: caption.style.font_size,
                            color: caption.style.color.clone(),
                            background_color: caption.style.background_color.clone(),
                            bottom_margin_px: caption.style.bottom_margin_px,
                        }),
                    });
                }
                TimelineItem::Transition(_) | TimelineItem::Group(_) => {}
            }
        }
    }

    apply_ancestors(project, &mut visual_layers, (width, height))?;
    for layer in &mut visual_layers {
        if let Some(value) = layer.transform2d {
            value.validate()?;
            if layer
                .keyframes
                .iter()
                .any(|key| key.property != EvaluatedProperty::Volume)
            {
                return Err(invalid("Transform2D cannot use legacy transform keyframes"));
            }
        }
        if layer.requires_affine() {
            layer.source_size = match &layer.source {
                EvaluatedVisualSource::Rectangle { width, height, .. } => Some((*width, *height)),
                EvaluatedVisualSource::SolidColor { .. } => Some((width, height)),
                _ => None,
            };
            if let Some(size) = layer.source_size {
                layer.affine = Some(evaluate_layer_affine(layer, size, (width, height))?);
            }
        }
    }
    sort_visual_layers(project, &mut visual_layers);
    Ok(EvaluatedSceneResult {
        scene: EvaluatedScene {
            canvas: EvaluatedCanvas { width, height, fps },
            duration_ms,
            resources,
            visual_layers,
            audio_layers,
            voiceover_intervals,
        },
        resource_bindings: SceneResourceBindings {
            media: media_bindings,
            fonts: font_bindings,
        },
    })
}

fn sort_visual_layers(project: &Project, visual_layers: &mut [EvaluatedVisualLayer]) {
    visual_layers.sort_by(|left, right| {
        let key = |layer: &EvaluatedVisualLayer| {
            let item = &project.tracks[layer.order.track_index].items[layer.order.item_index];
            (
                layer.order.track_index,
                item.visual_properties().z_index,
                layer.order.item_index,
            )
        };
        key(left)
            .cmp(&key(right))
            .then_with(|| left.item_id.cmp(&right.item_id))
    });
}

fn validate_referenced_assets(
    project: &Project,
    asset_by_id: &HashMap<&str, &Asset>,
) -> Result<(), CoreError> {
    for track in project.tracks.iter().filter(|track| !track.hidden) {
        for item in track.items.iter().filter(|item| !item.hidden()) {
            if let TimelineItem::Media(media) = item
                && !asset_by_id.contains_key(media.asset_id.as_str())
            {
                return Err(CoreError::new(
                    ErrorCode::AssetNotFound,
                    "timeline references a missing asset",
                ));
            }
        }
    }
    Ok(())
}

fn validate_media_source_ranges(
    project: &Project,
    asset_by_id: &HashMap<&str, &Asset>,
) -> Result<(), CoreError> {
    for track in project.tracks.iter().filter(|track| !track.hidden) {
        for item in track.items.iter().filter(|item| !item.hidden()) {
            let TimelineItem::Media(media) = item else {
                continue;
            };
            let asset = asset_by_id[media.asset_id.as_str()];
            if asset.media_type == MediaType::Image {
                continue;
            }
            let source_end_ms = media
                .source_in_ms
                .checked_add(media.duration_ms)
                .ok_or_else(|| invalid("evaluated media source interval overflows milliseconds"))?;
            if asset
                .duration_ms
                .is_some_and(|duration_ms| source_end_ms > duration_ms)
            {
                return Err(invalid(
                    "evaluated media source interval exceeds asset duration",
                ));
            }
        }
    }
    Ok(())
}

fn preflight_project<'a>(
    project: &'a Project,
    asset_by_id: &HashMap<&str, &Asset>,
) -> Result<EvaluationPreflight<'a>, CoreError> {
    let mut visual_item_ids = HashSet::new();
    let mut media_resource_ids = HashSet::new();
    let mut visual_layer_count = 0_usize;
    let mut audio_layer_count = 0_usize;
    let mut voiceover_activity_range_count = 0_usize;

    for track in project.tracks.iter().filter(|track| !track.hidden) {
        for item in track.items.iter().filter(|item| !item.hidden()) {
            match item {
                TimelineItem::Media(media) => {
                    validate_keyframe_limit(&media.keyframes)?;
                    let asset = asset_by_id[media.asset_id.as_str()];
                    if media_resource_ids.insert(media.asset_id.as_str())
                        && media_resource_ids.len() > MAX_EVALUATED_MEDIA_RESOURCES
                    {
                        return Err(invalid("evaluated media resource limit exceeded"));
                    }
                    if asset.media_type != MediaType::Audio {
                        increment_bounded(
                            &mut visual_layer_count,
                            MAX_EVALUATED_VISUAL_LAYERS,
                            "evaluated visual layer limit exceeded",
                        )?;
                        visual_item_ids.insert(media.id.as_str());
                    }
                    if asset.has_audio && !track.muted && !media.audio.muted {
                        increment_bounded(
                            &mut audio_layer_count,
                            MAX_EVALUATED_AUDIO_LAYERS,
                            "evaluated audio layer limit exceeded",
                        )?;
                    }
                    if track.audio_role == AudioTrackRole::Voiceover
                        && !track.muted
                        && !media.audio.muted
                        && media.audio.volume != 0.0
                        && asset.has_audio
                    {
                        let item_range_count = positive_scalar_ranges(
                            &media.keyframes,
                            KeyframeProperty::Volume,
                            media.duration_ms,
                        )
                        .len();
                        voiceover_activity_range_count = voiceover_activity_range_count
                            .checked_add(item_range_count)
                            .ok_or_else(|| {
                                invalid("evaluated voiceover activity range limit exceeded")
                            })?;
                        if voiceover_activity_range_count > MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES
                        {
                            return Err(invalid(
                                "evaluated voiceover activity range limit exceeded",
                            ));
                        }
                    }
                }
                TimelineItem::Text(text) => {
                    validate_keyframe_limit(&text.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(text.id.as_str());
                }
                TimelineItem::SolidColor(color) => {
                    validate_keyframe_limit(&color.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(color.id.as_str());
                }
                TimelineItem::Rectangle(rectangle) => {
                    validate_keyframe_limit(&rectangle.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(rectangle.id.as_str());
                }
                TimelineItem::Caption(caption) => {
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(caption.id.as_str());
                }
                TimelineItem::Transition(_) | TimelineItem::Group(_) => {}
            }
        }
    }

    let mut transition_fact_count = 0_usize;
    for transition in visible_transitions(project) {
        if visual_item_ids.contains(transition.from_item_id.as_str()) {
            increment_bounded(
                &mut transition_fact_count,
                MAX_EVALUATED_TRANSITION_FACTS,
                "evaluated transition fact limit exceeded",
            )?;
        }
        if transition
            .to_item_id
            .as_deref()
            .is_some_and(|item_id| visual_item_ids.contains(item_id))
        {
            increment_bounded(
                &mut transition_fact_count,
                MAX_EVALUATED_TRANSITION_FACTS,
                "evaluated transition fact limit exceeded",
            )?;
        }
    }

    Ok(EvaluationPreflight {
        visual_item_ids,
        visual_layer_count,
        media_resource_count: media_resource_ids.len(),
        audio_layer_count,
        voiceover_activity_range_count,
    })
}

fn visible_transitions(project: &Project) -> impl Iterator<Item = &TransitionItem> {
    project
        .tracks
        .iter()
        .filter(|track| !track.hidden)
        .flat_map(|track| &track.items)
        .filter_map(|item| match item {
            TimelineItem::Transition(transition) if !transition.hidden => Some(transition),
            _ => None,
        })
}

fn index_transitions(
    project: &Project,
    visual_item_ids: &HashSet<&str>,
) -> Result<HashMap<String, Vec<EvaluatedTransition>>, CoreError> {
    let mut index = HashMap::with_capacity(visual_item_ids.len());
    for transition in visible_transitions(project) {
        let evaluated = EvaluatedTransition {
            role: EvaluatedTransitionRole::Out,
            kind: evaluate_transition_kind(transition.transition_type),
            span: checked_span(transition.start_ms, transition.duration_ms)?,
        };
        if visual_item_ids.contains(transition.from_item_id.as_str()) {
            index
                .entry(transition.from_item_id.clone())
                .or_insert_with(Vec::new)
                .push(evaluated);
        }
        if let Some(item_id) = transition
            .to_item_id
            .as_ref()
            .filter(|item_id| visual_item_ids.contains(item_id.as_str()))
        {
            index
                .entry(item_id.clone())
                .or_insert_with(Vec::new)
                .push(EvaluatedTransition {
                    role: EvaluatedTransitionRole::In,
                    ..evaluated
                });
        }
    }
    Ok(index)
}

fn transitions_for(
    item_id: &str,
    index: &HashMap<String, Vec<EvaluatedTransition>>,
) -> Vec<EvaluatedTransition> {
    index.get(item_id).cloned().unwrap_or_default()
}

fn add_resource(
    asset: &Asset,
    resources: &mut Vec<EvaluatedMediaResource>,
    bindings: &mut Vec<MediaResourceBinding>,
    indexes: &mut HashSet<String>,
) {
    if !indexes.insert(asset.id.clone()) {
        return;
    }
    resources.push(EvaluatedMediaResource {
        asset_id: asset.id.clone(),
        kind: evaluate_media_kind(asset.media_type),
        has_audio: asset.has_audio,
    });
    bindings.push(MediaResourceBinding {
        asset_id: asset.id.clone(),
        project_relative_path: asset.project_relative_path.clone(),
    });
}

fn checked_project_duration(project: &Project) -> Result<u64, CoreError> {
    project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .try_fold(0, |duration, item| {
            let end = item
                .start_ms()
                .checked_add(item.duration_ms())
                .ok_or_else(|| invalid("evaluated timeline interval overflows milliseconds"))?;
            Ok(duration.max(end))
        })
}

fn checked_span(start_ms: u64, duration_ms: u64) -> Result<EvaluatedTimeSpan, CoreError> {
    if duration_ms == 0 {
        return Err(invalid("evaluated timeline interval must be non-empty"));
    }
    let end_ms = start_ms
        .checked_add(duration_ms)
        .ok_or_else(|| invalid("evaluated timeline interval overflows milliseconds"))?;
    Ok(EvaluatedTimeSpan { start_ms, end_ms })
}

fn evaluate_transform(transform: &Transform) -> Result<EvaluatedTransform, CoreError> {
    if !transform.position_x.is_finite()
        || !transform.position_y.is_finite()
        || !transform.scale.is_finite()
        || !transform.opacity.is_finite()
    {
        return Err(invalid("evaluated transform values must be finite"));
    }
    Ok(EvaluatedTransform {
        position_x: transform.position_x,
        position_y: transform.position_y,
        scale: transform.scale,
        opacity: transform.opacity,
    })
}

fn evaluate_keyframes(keyframes: &[Keyframe]) -> Result<Vec<EvaluatedKeyframe>, CoreError> {
    validate_keyframe_limit(keyframes)?;
    let mut evaluated = Vec::with_capacity(keyframes.len());
    for keyframe in keyframes {
        let value = match keyframe.value {
            KeyframeValue::Position { x, y } if x.is_finite() && y.is_finite() => {
                EvaluatedKeyframeValue::Position { x, y }
            }
            KeyframeValue::Scalar { value } if value.is_finite() => {
                EvaluatedKeyframeValue::Scalar { value }
            }
            _ => return Err(invalid("evaluated keyframe values must be finite")),
        };
        evaluated.push(EvaluatedKeyframe {
            property: evaluate_property(keyframe.property),
            time_ms: keyframe.time_ms,
            value,
            easing: evaluate_easing(keyframe.easing),
        });
    }
    Ok(evaluated)
}

fn validate_keyframe_limit(keyframes: &[Keyframe]) -> Result<(), CoreError> {
    let mut counts = [0_usize; 4];
    for keyframe in keyframes {
        let count = &mut counts[match keyframe.property {
            KeyframeProperty::Position => 0,
            KeyframeProperty::Scale => 1,
            KeyframeProperty::Opacity => 2,
            KeyframeProperty::Volume => 3,
        }];
        increment_bounded(
            count,
            MAX_EVALUATED_KEYFRAMES_PER_CHANNEL,
            "evaluated keyframe channel limit exceeded",
        )?;
    }
    Ok(())
}

fn evaluate_transition_kind(transition_type: TransitionType) -> EvaluatedTransitionKind {
    match transition_type {
        TransitionType::Fade => EvaluatedTransitionKind::Fade,
        TransitionType::Crossfade => EvaluatedTransitionKind::Crossfade,
    }
}

fn audible_voiceover_intervals(
    project: &Project,
    asset_by_id: &HashMap<&str, &Asset>,
    activity_range_count: usize,
) -> Result<Vec<EvaluatedTimeSpan>, CoreError> {
    let mut intervals = Vec::with_capacity(activity_range_count);
    for track in project.tracks.iter().filter(|track| {
        !track.hidden && !track.muted && track.audio_role == AudioTrackRole::Voiceover
    }) {
        for item in &track.items {
            let TimelineItem::Media(media) = item else {
                continue;
            };
            if media.hidden || media.audio.muted || media.audio.volume == 0.0 {
                continue;
            }
            let asset = asset_by_id.get(media.asset_id.as_str()).ok_or_else(|| {
                CoreError::new(
                    ErrorCode::AssetNotFound,
                    "timeline references a missing asset",
                )
            })?;
            if !asset.has_audio {
                continue;
            }
            for (start, end) in positive_scalar_ranges(
                &media.keyframes,
                KeyframeProperty::Volume,
                media.duration_ms,
            ) {
                let start_ms = media.start_ms.checked_add(start).ok_or_else(|| {
                    invalid("evaluated voiceover interval overflows milliseconds")
                })?;
                let end_ms = media.start_ms.checked_add(end).ok_or_else(|| {
                    invalid("evaluated voiceover interval overflows milliseconds")
                })?;
                if start_ms < end_ms {
                    intervals.push(EvaluatedTimeSpan { start_ms, end_ms });
                }
            }
        }
    }
    intervals.sort_unstable_by_key(|span| (span.start_ms, span.end_ms));
    let mut merged: Vec<EvaluatedTimeSpan> = Vec::with_capacity(intervals.len());
    for span in intervals {
        if let Some(previous) = merged.last_mut()
            && span.start_ms <= previous.end_ms
        {
            previous.end_ms = previous.end_ms.max(span.end_ms);
        } else {
            merged.push(span);
        }
    }
    Ok(merged)
}

fn evaluate_ducking(
    track: &Track,
    has_voiceover_activity: bool,
) -> Result<Option<EvaluatedDucking>, CoreError> {
    let Some(settings) = track.ducking.as_ref().filter(|settings| settings.enabled) else {
        return Ok(None);
    };
    if track.audio_role != AudioTrackRole::Music || !has_voiceover_activity {
        return Ok(None);
    }
    if !settings.gain.is_finite() {
        return Err(invalid("evaluated ducking gain must be finite"));
    }
    Ok(Some(EvaluatedDucking {
        gain: settings.gain,
        attack_ms: settings.attack_ms,
        release_ms: settings.release_ms,
    }))
}

fn evaluate_text_style(style: &TextStyle) -> Result<EvaluatedTextStyle, CoreError> {
    if !style.shadow.opacity.is_finite() || !style.background_opacity.is_finite() {
        return Err(invalid("evaluated text style values must be finite"));
    }
    Ok(EvaluatedTextStyle {
        alignment: match style.alignment {
            TextAlignment::Left => EvaluatedTextAlignment::Left,
            TextAlignment::Center => EvaluatedTextAlignment::Center,
            TextAlignment::Right => EvaluatedTextAlignment::Right,
        },
        wrap_width_px: style.wrap_width_px,
        line_spacing_px: style.line_spacing_px,
        outline_color: style.outline_color.clone(),
        outline_width_px: style.outline_width_px,
        shadow: EvaluatedTextShadow {
            color: style.shadow.color.clone(),
            opacity: style.shadow.opacity,
            offset_x: style.shadow.offset_x,
            offset_y: style.shadow.offset_y,
        },
        background_color: style.background_color.clone(),
        background_opacity: style.background_opacity,
        padding: EvaluatedTextPadding {
            top: style.padding.top,
            right: style.padding.right,
            bottom: style.padding.bottom,
            left: style.padding.left,
        },
        anchor: match style.anchor {
            AnchorPoint::TopLeft => EvaluatedAnchorPoint::TopLeft,
            AnchorPoint::TopCenter => EvaluatedAnchorPoint::TopCenter,
            AnchorPoint::TopRight => EvaluatedAnchorPoint::TopRight,
            AnchorPoint::CenterLeft => EvaluatedAnchorPoint::CenterLeft,
            AnchorPoint::Center => EvaluatedAnchorPoint::Center,
            AnchorPoint::CenterRight => EvaluatedAnchorPoint::CenterRight,
            AnchorPoint::BottomLeft => EvaluatedAnchorPoint::BottomLeft,
            AnchorPoint::BottomCenter => EvaluatedAnchorPoint::BottomCenter,
            AnchorPoint::BottomRight => EvaluatedAnchorPoint::BottomRight,
        },
    })
}

fn evaluate_media_kind(kind: MediaType) -> EvaluatedMediaKind {
    match kind {
        MediaType::Image => EvaluatedMediaKind::Image,
        MediaType::Video => EvaluatedMediaKind::Video,
        MediaType::Audio => EvaluatedMediaKind::Audio,
    }
}

fn evaluate_property(property: KeyframeProperty) -> EvaluatedProperty {
    match property {
        KeyframeProperty::Position => EvaluatedProperty::Position,
        KeyframeProperty::Scale => EvaluatedProperty::Scale,
        KeyframeProperty::Opacity => EvaluatedProperty::Opacity,
        KeyframeProperty::Volume => EvaluatedProperty::Volume,
    }
}

fn evaluate_easing(easing: Easing) -> EvaluatedEasing {
    match easing {
        Easing::Hold => EvaluatedEasing::Hold,
        Easing::Linear => EvaluatedEasing::Linear,
        Easing::EaseIn => EvaluatedEasing::EaseIn,
        Easing::EaseOut => EvaluatedEasing::EaseOut,
        Easing::EaseInOut => EvaluatedEasing::EaseInOut,
    }
}

fn evaluate_audio_role(role: AudioTrackRole) -> EvaluatedAudioRole {
    match role {
        AudioTrackRole::Unassigned => EvaluatedAudioRole::Unassigned,
        AudioTrackRole::Voiceover => EvaluatedAudioRole::Voiceover,
        AudioTrackRole::Music => EvaluatedAudioRole::Music,
        AudioTrackRole::SoundEffects => EvaluatedAudioRole::SoundEffects,
    }
}

fn increment_bounded(
    count: &mut usize,
    limit: usize,
    message: &'static str,
) -> Result<(), CoreError> {
    if *count == limit {
        return Err(invalid(message));
    }
    *count += 1;
    Ok(())
}

fn invalid(message: &'static str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AudioSettings, DuckingSettings, MediaItem, ProjectSettings, RectangleItem, SolidColorItem,
        TextItem, TextStyle, TrackType, TransitionItem,
    };

    #[test]
    fn explicit_stacking_respects_tracks_z_index_array_ties_and_hidden_sources() {
        let mut p = project();
        let visual = |id: &str, z: i32, order: u32, hidden: bool| -> TimelineItem {
            serde_json::from_value(serde_json::json!({"type":"solid_color","id":id,"color":"#ff0000","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":z,"stackOrder":order,"hidden":hidden})).unwrap()
        };
        p.tracks = vec![
            track(
                "lower",
                TrackType::Overlay,
                vec![
                    visual("z-first", 4, 0, false),
                    visual("a-second", 4, 1, false),
                    visual("negative", -1, 2, false),
                    visual("hidden", -99, 3, true),
                ],
            ),
            track(
                "upper",
                TrackType::Overlay,
                vec![visual("upper-negative", i32::MIN, 0, false)],
            ),
        ];
        let before = serde_json::to_value(&p).unwrap();
        for _ in 0..3 {
            let evaluated = evaluate_project(&p, 160, 90, 10).unwrap();
            assert_eq!(
                evaluated
                    .scene
                    .visual_layers
                    .iter()
                    .map(|layer| layer.item_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["negative", "z-first", "a-second", "upper-negative"]
            );
            assert!(evaluated.scene.audio_layers.is_empty());
            let mut synthesized = vec![
                evaluated.scene.visual_layers[0].clone(),
                evaluated.scene.visual_layers[0].clone(),
            ];
            synthesized[0].item_id = "z-synthesized".into();
            synthesized[1].item_id = "a-synthesized".into();
            sort_visual_layers(&p, &mut synthesized);
            assert_eq!(synthesized[0].item_id, "a-synthesized");
        }
        assert_eq!(serde_json::to_value(&p).unwrap(), before);
    }

    #[test]
    fn groups_compose_geometry_clip_visibility_and_preserve_source_time() {
        let mut p = project();
        let mut group_transform = crate::Transform2D::default();
        group_transform.position.x = 40.0;
        group_transform.position.y = 10.0;
        group_transform.rotation_deg = 90.0;
        group_transform.opacity = 0.5;
        p.tracks = serde_json::from_value(serde_json::json!([{
            "id":"overlay","name":"Overlay","trackType":"overlay","items":[
                {"type":"group","id":"outer","startMs":100,"durationMs":600,"zIndex":0,"stackOrder":0,"transform2d":group_transform},
                {"type":"group","id":"inner","startMs":200,"durationMs":600,"zIndex":0,"stackOrder":1,"parent":{"scope":"root","id":"outer"},"transform2d":crate::Transform2D::default()},
                {"type":"rectangle","id":"child","startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","keyframes":[],"zIndex":0,"stackOrder":2,"parent":{"scope":"root","id":"inner"},"transform":{"positionX":5,"positionY":3,"scale":1,"opacity":0.5}}
            ]
        }])).unwrap();
        let before = serde_json::to_value(&p).unwrap();
        let scene = evaluate_project(&p, 64, 64, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 1);
        let layer = &scene.visual_layers[0];
        assert_eq!(
            layer.span,
            EvaluatedTimeSpan {
                start_ms: 0,
                end_ms: 1000
            }
        );
        assert_eq!(
            layer.visible_span(),
            EvaluatedTimeSpan {
                start_ms: 200,
                end_ms: 700
            }
        );
        let affine = layer.affine.unwrap();
        let [a, b, c, d, x, y] = affine.matrix;
        for (px, py) in [(0.0, 0.0), (20.0, 0.0), (0.0, 10.0), (20.0, 10.0)] {
            assert!((a * px + c * py + x - (37.0 - py)).abs() < 1e-9);
            assert!((b * px + d * py + y - (15.0 + px)).abs() < 1e-9);
        }
        assert_eq!(affine.opacity, 0.25);
        assert_eq!(evaluate_project(&p, 64, 64, 30).unwrap().scene, scene);
        assert_eq!(serde_json::to_value(&p).unwrap(), before);
        p.tracks[0].items[0].set_hidden(true);
        assert!(
            evaluate_project(&p, 64, 64, 30)
                .unwrap()
                .scene
                .visual_layers
                .is_empty()
        );
        p.tracks[0].items[0].visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "root".into(),
            id: "inner".into(),
        });
        assert_eq!(
            evaluate_project(&p, 64, 64, 30).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn nested_group_oracle_covers_anchor_skew_scale_audio_and_overflow() {
        let mut p = project();
        let mut outer = crate::Transform2D::default();
        outer.position.unit = crate::PositionUnit::Normalized;
        outer.position.x = 0.4;
        outer.position.y = 0.3;
        outer.anchor.x = 0.2;
        outer.anchor.y = 0.7;
        outer.scale_x = 1.2;
        outer.scale_y = 0.8;
        outer.rotation_deg = 31.0;
        outer.skew_x_deg = 12.0;
        outer.skew_y_deg = -8.0;
        outer.opacity = 0.8;
        let mut inner = outer;
        inner.position.unit = crate::PositionUnit::Pixels;
        inner.position.x = 15.0;
        inner.position.y = 17.0;
        inner.rotation_deg = -18.0;
        inner.opacity = 0.5;
        p.tracks=serde_json::from_value(serde_json::json!([{"id":"parents","name":"Parents","trackType":"overlay","items":[
            {"type":"group","id":"outer","startMs":100,"durationMs":700,"stackOrder":0,"transform2d":outer},
            {"type":"group","id":"inner","startMs":200,"durationMs":700,"stackOrder":1,"transform2d":inner,"parent":{"scope":"root","id":"outer"}}
        ]}])).unwrap();
        p.assets = vec![asset("video", MediaType::Video, true)];
        let mut child = media("child", "video", 0);
        child.visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "root".into(),
            id: "inner".into(),
        });
        child.visual_properties_mut().transform2d = Some(crate::Transform2D::default());
        p.tracks
            .push(track("visual", TrackType::Video, vec![child]));
        let mut scene = evaluate_project(&p, 320, 180, 30).unwrap().scene;
        finalize_affine_geometry(
            &mut scene,
            &HashMap::from([("child".to_string(), (23, 11))]),
        )
        .unwrap();
        let map = |t: crate::Transform2D, (x, y): (f64, f64)| {
            let x = (x - t.anchor.x * 320.0) * t.scale_x;
            let y = (y - t.anchor.y * 180.0) * t.scale_y;
            let x = x + y * t.skew_x_deg.to_radians().tan();
            let y = y + x * t.skew_y_deg.to_radians().tan();
            let (s, c) = t.rotation_deg.to_radians().sin_cos();
            let (fx, fy) = if t.position.unit == crate::PositionUnit::Normalized {
                (320.0, 180.0)
            } else {
                (1.0, 1.0)
            };
            (
                c * x - s * y + t.position.x * fx,
                s * x + c * y + t.position.y * fy,
            )
        };
        let affine = scene.visual_layers[0].affine.unwrap();
        for (x, y) in [(0.0, 0.0), (23.0, 0.0), (0.0, 11.0), (23.0, 11.0)] {
            let expected = map(outer, map(inner, (x, y)));
            let [a, b, c, d, tx, ty] = affine.matrix;
            assert!((a * x + c * y + tx - expected.0).abs() < 1e-9);
            assert!((b * x + d * y + ty - expected.1).abs() < 1e-9);
        }
        assert_eq!(affine.opacity, 0.4);
        let audio = scene.audio_layers.clone();
        p.tracks[0].hidden = true;
        let hidden = evaluate_project(&p, 320, 180, 30).unwrap().scene;
        assert!(hidden.visual_layers.is_empty());
        assert_eq!(hidden.audio_layers, audio);
        p.tracks[0].hidden = false;
        for group in &mut p.tracks[0].items {
            let t = group.visual_properties_mut().transform2d.as_mut().unwrap();
            t.scale_x = 100.0;
            t.scale_y = 100.0;
        }
        let mut overflow = evaluate_project(&p, 320, 180, 30).unwrap().scene;
        assert_eq!(
            finalize_affine_geometry(
                &mut overflow,
                &HashMap::from([("child".to_string(), (23, 11))])
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn legacy_text_anchors_and_animated_sampling_are_composed_before_parents() {
        for (anchor, ax, ay) in [
            ("top_left", 0.0, 0.0),
            ("top_center", 0.5, 0.0),
            ("top_right", 1.0, 0.0),
            ("center_left", 0.0, 0.5),
            ("center", 0.5, 0.5),
            ("center_right", 1.0, 0.5),
            ("bottom_left", 0.0, 1.0),
            ("bottom_center", 0.5, 1.0),
            ("bottom_right", 1.0, 1.0),
        ] {
            let mut p = project();
            let mut t = crate::Transform2D::default();
            t.position.x = 50.0;
            t.rotation_deg = 90.0;
            p.tracks=serde_json::from_value(serde_json::json!([{"id":"track","name":"Overlay","trackType":"overlay","items":[
                {"type":"group","id":"g","startMs":0,"durationMs":1000,"stackOrder":0,"transform2d":t},
                {"type":"text","id":"text","text":"Anchor","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"stackOrder":1,"style":{"anchor":anchor},"transform":{"positionX":200,"positionY":180,"scale":1.5,"opacity":1},"keyframes":[],"parent":{"scope":"root","id":"g"}}
            ]}])).unwrap();
            let mut scene = evaluate_project(&p, 800, 600, 30).unwrap().scene;
            finalize_affine_geometry(
                &mut scene,
                &HashMap::from([("text".to_string(), (110, 30))]),
            )
            .unwrap();
            let [a, b, c, d, x, y] = scene.visual_layers[0].affine.unwrap().matrix;
            for (px, py) in [(0.0, 0.0), (110.0, 30.0)] {
                assert!(
                    (a * px + c * py + x - (50.0 - (180.0 + 1.5 * (py - ay * 30.0)))).abs() < 1e-9
                );
                assert!((b * px + d * py + y - (200.0 + 1.5 * (px - ax * 110.0))).abs() < 1e-9);
            }
        }
        let mut p = project();
        p.tracks=serde_json::from_value(serde_json::json!([{"id":"track","name":"Overlay","trackType":"overlay","items":[
            {"type":"group","id":"g","startMs":0,"durationMs":1000,"stackOrder":0},
            {"type":"rectangle","id":"r","startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","stackOrder":1,"keyframes":[{"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"},{"property":"position","timeMs":1000,"value":{"type":"position","x":20000,"y":0},"easing":"linear"}],"parent":{"scope":"root","id":"g"}}
        ]}])).unwrap();
        let mut scene = evaluate_project(&p, 7680, 4320, 30).unwrap().scene;
        finalize_affine_geometry(&mut scene, &HashMap::new()).unwrap();
        let tiles = scene.visual_layers[0].sampling_tiles.as_ref().unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(
            (tiles[0].left, tiles[0].width, tiles[0].height),
            (0.0, 4096, 10)
        );
        assert_eq!(
            (tiles[1].left, tiles[1].width, tiles[1].height),
            (4096.0, 3584, 10)
        );
        p.tracks[0].items[0].visual_properties_mut().transform2d = Some(crate::Transform2D {
            position: crate::TransformPosition {
                x: 0.0,
                y: 5000.0,
                unit: crate::PositionUnit::Pixels,
            },
            ..Default::default()
        });
        let mut offscreen = evaluate_project(&p, 7680, 4320, 30).unwrap().scene;
        finalize_affine_geometry(&mut offscreen, &HashMap::new()).unwrap();
        assert!(offscreen.visual_layers.is_empty());
    }

    fn project() -> Project {
        Project {
            schema_version: crate::PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 7,
            name: "Evaluated scene".into(),
            created_at_ms: 1,
            updated_at_ms: 2,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        }
    }

    fn asset(id: &str, media_type: MediaType, has_audio: bool) -> Asset {
        Asset {
            id: id.into(),
            media_type,
            file_name: format!("{id}.media"),
            project_relative_path: format!("assets/{id}.media"),
            duration_ms: Some(20_000),
            has_audio,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        }
    }

    fn media(id: &str, asset_id: &str, start_ms: u64) -> TimelineItem {
        TimelineItem::Media(MediaItem {
            id: id.into(),
            asset_id: asset_id.into(),
            start_ms,
            duration_ms: 1_000,
            source_in_ms: 125,
            visual_properties: crate::VisualProperties::default(),
            audio: AudioSettings::default(),
            keyframes: vec![],
        })
    }

    fn track(id: &str, track_type: TrackType, mut items: Vec<TimelineItem>) -> Track {
        for (index, item) in items.iter_mut().enumerate() {
            item.visual_properties_mut().stack_order = u32::try_from(index).unwrap();
        }
        Track {
            id: id.into(),
            name: id.into(),
            track_type,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items,
        }
    }

    #[test]
    fn evaluates_owned_flat_layers_in_stable_order_without_mutating_project() {
        let mut input = project();
        input.assets = vec![
            asset("image", MediaType::Image, false),
            asset("video", MediaType::Video, true),
        ];
        input.tracks = vec![
            track(
                "base",
                TrackType::Video,
                vec![
                    TimelineItem::SolidColor(SolidColorItem {
                        id: "background".into(),
                        color: "#112233".into(),
                        start_ms: 0,
                        duration_ms: 4_000,
                        visual_properties: crate::VisualProperties::default(),
                        keyframes: vec![],
                    }),
                    media("photo", "image", 250),
                ],
            ),
            track(
                "overlay",
                TrackType::Overlay,
                vec![
                    TimelineItem::Rectangle(RectangleItem {
                        id: "panel".into(),
                        color: "#445566".into(),
                        width: 320,
                        height: 180,
                        start_ms: 500,
                        duration_ms: 2_000,
                        visual_properties: crate::VisualProperties::new(
                            Transform {
                                position_x: 20.0,
                                position_y: 30.0,
                                scale: 1.25,
                                opacity: 0.8,
                            },
                            false,
                        ),
                        keyframes: vec![],
                    }),
                    TimelineItem::Text(TextItem {
                        id: "title".into(),
                        text: "Title".into(),
                        start_ms: 750,
                        duration_ms: 1_000,
                        font_size: 48,
                        color: "#ffffff".into(),
                        font_family: Some("Inter".into()),
                        font_path: Some("fonts/private.ttf".into()),
                        style: TextStyle::default(),
                        visual_properties: crate::VisualProperties::default(),
                        keyframes: vec![Keyframe {
                            property: KeyframeProperty::Opacity,
                            time_ms: 0,
                            value: KeyframeValue::Scalar { value: 0.5 },
                            easing: Easing::Linear,
                        }],
                    }),
                    media("clip", "video", 1_000),
                    TimelineItem::Transition(TransitionItem {
                        id: "panel-to-title".into(),
                        transition_type: TransitionType::Crossfade,
                        from_item_id: "panel".into(),
                        to_item_id: Some("title".into()),
                        start_ms: 700,
                        duration_ms: 200,
                        visual_properties: crate::VisualProperties::default(),
                    }),
                ],
            ),
        ];
        let before = serde_json::to_string(&input).unwrap();

        let first = evaluate_project(&input, 1_280, 720, 30).unwrap();
        let second = evaluate_project(&input, 1_280, 720, 30).unwrap();

        assert_eq!(first, second);
        assert_eq!(serde_json::to_string(&input).unwrap(), before);
        assert_eq!(input.revision, 7);
        let scene = &first.scene;
        assert_eq!(
            scene.canvas,
            EvaluatedCanvas {
                width: 1_280,
                height: 720,
                fps: 30
            }
        );
        assert_eq!(scene.duration_ms, 4_000);
        assert_eq!(
            scene
                .visual_layers
                .iter()
                .map(|layer| (layer.item_id.as_str(), layer.order))
                .collect::<Vec<_>>(),
            vec![
                (
                    "background",
                    EvaluatedLayerOrder {
                        track_index: 0,
                        item_index: 0
                    }
                ),
                (
                    "photo",
                    EvaluatedLayerOrder {
                        track_index: 0,
                        item_index: 1
                    }
                ),
                (
                    "panel",
                    EvaluatedLayerOrder {
                        track_index: 1,
                        item_index: 0
                    }
                ),
                (
                    "title",
                    EvaluatedLayerOrder {
                        track_index: 1,
                        item_index: 1
                    }
                ),
                (
                    "clip",
                    EvaluatedLayerOrder {
                        track_index: 1,
                        item_index: 2
                    }
                ),
            ]
        );
        assert_eq!(scene.visual_layers[1].span.start_ms, 250);
        assert_eq!(scene.visual_layers[1].span.end_ms, 1_250);
        assert_eq!(
            scene
                .resources
                .iter()
                .map(|resource| resource.asset_id.as_str())
                .collect::<Vec<_>>(),
            vec!["image", "video"]
        );
        let title = scene
            .visual_layers
            .iter()
            .find(|layer| layer.item_id == "title")
            .unwrap();
        let EvaluatedVisualSource::Text(text) = &title.source else {
            panic!("title must evaluate as text");
        };
        assert_eq!(text.font_resource_id.as_deref(), Some("text-font:title"));
        assert!(!format!("{scene:?}").contains("fonts/private.ttf"));
        assert_eq!(
            first.resource_bindings.media,
            vec![
                MediaResourceBinding {
                    asset_id: "image".into(),
                    project_relative_path: "assets/image.media".into(),
                },
                MediaResourceBinding {
                    asset_id: "video".into(),
                    project_relative_path: "assets/video.media".into(),
                },
            ]
        );
        assert_eq!(
            first.resource_bindings.fonts,
            vec![FontResourceBinding {
                font_resource_id: "text-font:title".into(),
                requested_path: Some("fonts/private.ttf".into()),
                requested_family: Some("Inter".into()),
            }]
        );
        assert_eq!(
            scene.visual_layers[2].transitions[0].role,
            EvaluatedTransitionRole::Out
        );
        assert_eq!(title.transitions[0].role, EvaluatedTransitionRole::In);
        assert_eq!(
            title.transitions[0].kind,
            EvaluatedTransitionKind::Crossfade
        );
    }

    #[test]
    fn omits_hidden_visuals_and_resolves_audio_ducking() {
        let mut input = project();
        input.assets = vec![
            asset("voice", MediaType::Audio, true),
            asset("music", MediaType::Audio, true),
            asset("muted", MediaType::Audio, true),
        ];
        let mut voice = track(
            "voice",
            TrackType::Audio,
            vec![media("voice-item", "voice", 1_000)],
        );
        voice.audio_role = AudioTrackRole::Voiceover;
        let mut music_item = media("music-item", "music", 0);
        let TimelineItem::Media(music_media) = &mut music_item else {
            unreachable!()
        };
        music_media.duration_ms = 4_000;
        music_media.audio.volume = 0.75;
        music_media.audio.fade_in_ms = 100;
        music_media.audio.fade_out_ms = 200;
        music_media.keyframes = vec![Keyframe {
            property: KeyframeProperty::Volume,
            time_ms: 0,
            value: KeyframeValue::Scalar { value: 0.5 },
            easing: Easing::EaseIn,
        }];
        let mut music = track("music", TrackType::Audio, vec![music_item]);
        music.audio_role = AudioTrackRole::Music;
        music.ducking = Some(DuckingSettings {
            enabled: true,
            gain: 0.25,
            attack_ms: 50,
            release_ms: 75,
        });
        let mut muted = track(
            "muted",
            TrackType::Audio,
            vec![media("muted-item", "muted", 0)],
        );
        muted.muted = true;
        let mut hidden = track(
            "hidden",
            TrackType::Video,
            vec![TimelineItem::Rectangle(RectangleItem {
                id: "hidden-shape".into(),
                color: "#000000".into(),
                width: 10,
                height: 10,
                start_ms: 0,
                duration_ms: 100,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        );
        hidden.hidden = true;
        input.tracks = vec![voice, music, muted, hidden];

        let evaluated = evaluate_project(&input, 640, 360, 24).unwrap();
        let scene = evaluated.scene;

        assert!(scene.visual_layers.is_empty());
        assert_eq!(
            scene
                .audio_layers
                .iter()
                .map(|layer| layer.item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["voice-item", "music-item"]
        );
        let music = &scene.audio_layers[1];
        assert_eq!(
            (music.volume, music.fade_in_ms, music.fade_out_ms),
            (0.75, 100, 200)
        );
        assert_eq!(music.volume_keyframes.len(), 1);
        let ducking = music.ducking.as_ref().unwrap();
        assert_eq!(
            (ducking.gain, ducking.attack_ms, ducking.release_ms),
            (0.25, 50, 75)
        );
        assert_eq!(
            scene.voiceover_intervals,
            vec![EvaluatedTimeSpan {
                start_ms: 1_000,
                end_ms: 2_000
            }]
        );
    }

    #[test]
    fn validates_non_image_source_ranges_after_missing_assets() {
        let evaluate_media = |media_type: MediaType,
                              asset_duration_ms: Option<u64>,
                              source_in_ms: u64,
                              duration_ms: u64| {
            let mut input = project();
            let mut source = asset("source", media_type, media_type != MediaType::Image);
            source.duration_ms = asset_duration_ms;
            input.assets = vec![source];
            let mut item = media("item", "source", 0);
            let TimelineItem::Media(media) = &mut item else {
                unreachable!()
            };
            media.source_in_ms = source_in_ms;
            media.duration_ms = duration_ms;
            input.tracks = vec![track("media", TrackType::Video, vec![item])];
            evaluate_project(&input, 16, 16, 1)
        };

        assert!(evaluate_media(MediaType::Video, None, u64::MAX - 1, 1).is_ok());
        assert_eq!(
            evaluate_media(MediaType::Audio, None, u64::MAX, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert!(evaluate_media(MediaType::Video, Some(1_125), 125, 1_000).is_ok());
        assert_eq!(
            evaluate_media(MediaType::Video, Some(1_124), 125, 1_000)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert!(evaluate_media(MediaType::Image, Some(1), u64::MAX, 1).is_ok());

        let mut missing = project();
        let mut missing_item = media("missing", "absent", 0);
        let TimelineItem::Media(media) = &mut missing_item else {
            unreachable!()
        };
        media.source_in_ms = u64::MAX;
        missing.tracks = vec![track("video", TrackType::Video, vec![missing_item])];
        assert_eq!(
            evaluate_project(&missing, 16, 16, 1).unwrap_err().code,
            ErrorCode::AssetNotFound
        );
    }

    #[test]
    fn missing_non_finite_and_invalid_timing_fail_closed() {
        let mut missing = project();
        missing.tracks = vec![track(
            "video",
            TrackType::Video,
            vec![media("missing", "absent", 0)],
        )];
        missing.tracks[0].items[0]
            .visual_properties_mut()
            .transform2d = Some(crate::Transform2D {
            scale_x: 100.0,
            scale_y: 100.0,
            ..Default::default()
        });
        assert_eq!(
            evaluate_project(&missing, 640, 360, 30).unwrap_err().code,
            ErrorCode::AssetNotFound
        );

        let mut non_finite = project();
        non_finite.tracks = vec![track(
            "overlay",
            TrackType::Overlay,
            vec![TimelineItem::Rectangle(RectangleItem {
                id: "bad".into(),
                color: "#ffffff".into(),
                width: 10,
                height: 10,
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::new(
                    Transform {
                        position_x: f64::NAN,
                        ..Transform::default()
                    },
                    false,
                ),
                keyframes: vec![],
            })],
        )];
        assert_eq!(
            evaluate_project(&non_finite, 640, 360, 30)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );

        let mut empty = project();
        empty.tracks = vec![track(
            "base",
            TrackType::Video,
            vec![TimelineItem::SolidColor(SolidColorItem {
                id: "empty".into(),
                color: "#000000".into(),
                start_ms: 0,
                duration_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        )];
        assert_eq!(
            evaluate_project(&empty, 640, 360, 30).unwrap_err().code,
            ErrorCode::InvalidArgument
        );

        let mut overflow = project();
        overflow.tracks = vec![track(
            "base",
            TrackType::Video,
            vec![TimelineItem::SolidColor(SolidColorItem {
                id: "overflow".into(),
                color: "#000000".into(),
                start_ms: u64::MAX,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        )];
        assert_eq!(
            evaluate_project(&overflow, 640, 360, 30).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn accepts_each_scene_limit_and_rejects_boundary_plus_one() {
        let visual_project = |count: usize| {
            let mut input = project();
            input.tracks = vec![track(
                "base",
                TrackType::Video,
                (0..count)
                    .map(|index| {
                        TimelineItem::SolidColor(SolidColorItem {
                            id: format!("visual-{index}"),
                            color: "#000000".into(),
                            start_ms: 0,
                            duration_ms: 1,
                            visual_properties: crate::VisualProperties::default(),
                            keyframes: vec![],
                        })
                    })
                    .collect(),
            )];
            input
        };
        assert!(evaluate_project(&visual_project(MAX_EVALUATED_VISUAL_LAYERS), 16, 16, 1).is_ok());
        assert_eq!(
            evaluate_project(&visual_project(MAX_EVALUATED_VISUAL_LAYERS + 1), 16, 16, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );

        let audio_project = |count: usize| {
            let mut input = project();
            input.assets = vec![asset("shared", MediaType::Audio, true)];
            input.tracks = vec![track(
                "audio",
                TrackType::Audio,
                (0..count)
                    .map(|index| media(&format!("audio-{index}"), "shared", 0))
                    .collect(),
            )];
            input
        };
        assert!(evaluate_project(&audio_project(MAX_EVALUATED_AUDIO_LAYERS), 16, 16, 1).is_ok());
        assert_eq!(
            evaluate_project(&audio_project(MAX_EVALUATED_AUDIO_LAYERS + 1), 16, 16, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );

        let resource_project = |count: usize| {
            let mut input = project();
            input.assets = (0..count)
                .map(|index| asset(&format!("asset-{index}"), MediaType::Audio, false))
                .collect();
            input.tracks = vec![track(
                "audio",
                TrackType::Audio,
                (0..count)
                    .map(|index| media(&format!("item-{index}"), &format!("asset-{index}"), 0))
                    .collect(),
            )];
            input
        };
        assert!(
            evaluate_project(&resource_project(MAX_EVALUATED_MEDIA_RESOURCES), 16, 16, 1).is_ok()
        );
        assert_eq!(
            evaluate_project(
                &resource_project(MAX_EVALUATED_MEDIA_RESOURCES + 1),
                16,
                16,
                1
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );

        let keyframe_project = |count: usize| {
            let mut input = project();
            input.tracks = vec![track(
                "overlay",
                TrackType::Overlay,
                vec![TimelineItem::Text(TextItem {
                    id: "animated".into(),
                    text: "Animated".into(),
                    start_ms: 0,
                    duration_ms: count as u64 + 1,
                    font_size: 20,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    visual_properties: crate::VisualProperties::default(),
                    keyframes: (0..count)
                        .map(|index| Keyframe {
                            property: KeyframeProperty::Position,
                            time_ms: index as u64,
                            value: KeyframeValue::Position {
                                x: index as f64,
                                y: 0.0,
                            },
                            easing: Easing::Linear,
                        })
                        .collect(),
                })],
            )];
            input
        };
        assert!(
            evaluate_project(
                &keyframe_project(MAX_EVALUATED_KEYFRAMES_PER_CHANNEL),
                16,
                16,
                1
            )
            .is_ok()
        );
        assert_eq!(
            evaluate_project(
                &keyframe_project(MAX_EVALUATED_KEYFRAMES_PER_CHANNEL + 1),
                16,
                16,
                1
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn rejects_missing_assets_before_scene_complexity() {
        let mut input = project();
        let mut items = vec![media("missing", "absent", 0)];
        items.extend((0..=MAX_EVALUATED_VISUAL_LAYERS).map(|index| {
            TimelineItem::SolidColor(SolidColorItem {
                id: format!("visual-{index}"),
                color: "#000000".into(),
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })
        }));
        input.tracks = vec![track("base", TrackType::Video, items)];

        assert_eq!(
            evaluate_project(&input, 16, 16, 1).unwrap_err().code,
            ErrorCode::AssetNotFound
        );
    }

    #[test]
    fn preflights_voiceover_keyframes_before_interval_derivation() {
        let mut input = project();
        input.assets = vec![asset("voice", MediaType::Audio, true)];
        let mut voice_item = media("voice-item", "voice", 0);
        {
            let TimelineItem::Media(media) = &mut voice_item else {
                unreachable!()
            };
            media.duration_ms = MAX_EVALUATED_KEYFRAMES_PER_CHANNEL as u64 + 2;
            media.keyframes = (0..=MAX_EVALUATED_KEYFRAMES_PER_CHANNEL)
                .map(|index| Keyframe {
                    property: KeyframeProperty::Volume,
                    time_ms: index as u64,
                    value: KeyframeValue::Scalar { value: 1.0 },
                    easing: Easing::Linear,
                })
                .collect();
        }
        let mut voice = track("voice", TrackType::Audio, vec![voice_item]);
        voice.audio_role = AudioTrackRole::Voiceover;
        input.tracks = vec![voice];

        assert_eq!(
            evaluate_project(&input, 16, 16, 1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn bounds_pre_merge_voiceover_activity_ranges() {
        fn voiceover_item(id: &str, asset_id: &str, start_ms: u64) -> TimelineItem {
            let mut item = media(id, asset_id, start_ms);
            let TimelineItem::Media(media) = &mut item else {
                unreachable!()
            };
            media.duration_ms = MAX_EVALUATED_KEYFRAMES_PER_CHANNEL as u64;
            media.keyframes = (0..MAX_EVALUATED_KEYFRAMES_PER_CHANNEL)
                .map(|index| Keyframe {
                    property: KeyframeProperty::Volume,
                    time_ms: index as u64,
                    value: KeyframeValue::Scalar {
                        value: if index % 2 == 0 { 1.0 } else { 0.0 },
                    },
                    easing: Easing::Hold,
                })
                .collect();
            item
        }

        let mut exact = project();
        let mut voice_asset = asset("voice", MediaType::Audio, true);
        voice_asset.duration_ms = None;
        exact.assets = vec![voice_asset];
        let mut voice = track(
            "voice",
            TrackType::Audio,
            vec![
                voiceover_item("voice-a", "voice", 0),
                voiceover_item("voice-b", "voice", 20_000),
            ],
        );
        voice.audio_role = AudioTrackRole::Voiceover;
        exact.tracks = vec![voice];

        let evaluated = evaluate_project(&exact, 16, 16, 1).unwrap();
        assert_eq!(
            evaluated.scene.voiceover_intervals.len(),
            MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES
        );
        assert_eq!(
            evaluated.scene.voiceover_intervals[0],
            EvaluatedTimeSpan {
                start_ms: 0,
                end_ms: 1,
            }
        );
        assert_eq!(
            evaluated.scene.voiceover_intervals[5_000],
            EvaluatedTimeSpan {
                start_ms: 20_000,
                end_ms: 20_001,
            }
        );

        let mut overflow = exact;
        overflow.tracks[0]
            .items
            .push(media("voice-c", "voice", 40_000));
        assert_eq!(
            evaluate_project(&overflow, 16, 16, 1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn multiple_ducked_layers_use_the_scene_voiceover_table() {
        let mut input = project();
        input.assets = vec![
            asset("voice", MediaType::Audio, true),
            asset("music", MediaType::Audio, true),
        ];
        let mut voice = track(
            "voice",
            TrackType::Audio,
            vec![media("voice-item", "voice", 100)],
        );
        voice.audio_role = AudioTrackRole::Voiceover;
        let music_track = |id: &str, item_id: &str, gain: f64| {
            let mut music = track(id, TrackType::Audio, vec![media(item_id, "music", 0)]);
            music.audio_role = AudioTrackRole::Music;
            music.ducking = Some(DuckingSettings {
                enabled: true,
                gain,
                attack_ms: 25,
                release_ms: 50,
            });
            music
        };
        input.tracks = vec![
            voice,
            music_track("music-a", "music-a-item", 0.2),
            music_track("music-b", "music-b-item", 0.3),
        ];

        let scene = evaluate_project(&input, 16, 16, 1).unwrap().scene;
        assert_eq!(
            scene.voiceover_intervals,
            vec![EvaluatedTimeSpan {
                start_ms: 100,
                end_ms: 1_100,
            }]
        );
        let ducking = scene.audio_layers[1..]
            .iter()
            .map(|layer| {
                let ducking = layer.ducking.as_ref().unwrap();
                (ducking.gain, ducking.attack_ms, ducking.release_ms)
            })
            .collect::<Vec<_>>();
        assert_eq!(ducking, vec![(0.2, 25, 50), (0.3, 25, 50)]);
    }

    #[test]
    fn transition_facts_are_bounded_ordered_and_include_both_self_roles() {
        let transition_project = |count: usize| {
            let mut input = project();
            let mut items = vec![TimelineItem::Rectangle(RectangleItem {
                id: "visual".into(),
                color: "#ffffff".into(),
                width: 1,
                height: 1,
                start_ms: 0,
                duration_ms: count as u64 + 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })];
            items.extend((0..count).map(|index| {
                TimelineItem::Transition(TransitionItem {
                    id: format!("transition-{index}"),
                    transition_type: if index % 2 == 0 {
                        TransitionType::Fade
                    } else {
                        TransitionType::Crossfade
                    },
                    from_item_id: "visual".into(),
                    to_item_id: None,
                    start_ms: index as u64,
                    duration_ms: 1,
                    visual_properties: crate::VisualProperties::default(),
                })
            }));
            input.tracks = vec![track("base", TrackType::Video, items)];
            input
        };

        let exact = evaluate_project(
            &transition_project(MAX_EVALUATED_TRANSITION_FACTS),
            16,
            16,
            1,
        )
        .unwrap();
        let facts = &exact.scene.visual_layers[0].transitions;
        assert_eq!(facts.len(), MAX_EVALUATED_TRANSITION_FACTS);
        assert_eq!(facts[0].kind, EvaluatedTransitionKind::Fade);
        assert_eq!(facts[1].kind, EvaluatedTransitionKind::Crossfade);
        assert_eq!(facts[0].span.start_ms, 0);
        assert_eq!(facts[1].span.start_ms, 1);
        assert_eq!(
            evaluate_project(
                &transition_project(MAX_EVALUATED_TRANSITION_FACTS + 1),
                16,
                16,
                1,
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );

        let mut self_endpoint = transition_project(0);
        self_endpoint.tracks[0]
            .items
            .push(TimelineItem::Transition(TransitionItem {
                id: "self".into(),
                transition_type: TransitionType::Crossfade,
                from_item_id: "visual".into(),
                to_item_id: Some("visual".into()),
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
            }));
        self_endpoint.tracks[0].items[1]
            .visual_properties_mut()
            .stack_order = 1;
        let evaluated = evaluate_project(&self_endpoint, 16, 16, 1).unwrap();
        assert_eq!(
            evaluated.scene.visual_layers[0]
                .transitions
                .iter()
                .map(|fact| fact.role)
                .collect::<Vec<_>>(),
            vec![EvaluatedTransitionRole::Out, EvaluatedTransitionRole::In,]
        );
    }

    #[test]
    fn font_bindings_preserve_selection_outside_the_scene() {
        let text = |id: &str, path: Option<&str>, family: Option<&str>| {
            TimelineItem::Text(TextItem {
                id: id.into(),
                text: id.into(),
                start_ms: 0,
                duration_ms: 1,
                font_size: 20,
                color: "#ffffff".into(),
                font_family: family.map(str::to_owned),
                font_path: path.map(str::to_owned),
                style: TextStyle::default(),
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })
        };
        let mut input = project();
        input.tracks = vec![track(
            "text",
            TrackType::Overlay,
            vec![
                text("path", Some("fonts/path.ttf"), None),
                text("family", None, Some("Inter")),
                text("both", Some("fonts/both.ttf"), Some("Source Sans")),
                text("default", None, None),
            ],
        )];

        let evaluated = evaluate_project(&input, 16, 16, 1).unwrap();
        let font_ids = evaluated
            .scene
            .visual_layers
            .iter()
            .map(|layer| match &layer.source {
                EvaluatedVisualSource::Text(text) => text.font_resource_id.as_deref(),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            font_ids,
            vec![
                Some("text-font:path"),
                Some("text-font:family"),
                Some("text-font:both"),
                None,
            ]
        );
        assert_eq!(
            evaluated.resource_bindings.fonts,
            vec![
                FontResourceBinding {
                    font_resource_id: "text-font:path".into(),
                    requested_path: Some("fonts/path.ttf".into()),
                    requested_family: None,
                },
                FontResourceBinding {
                    font_resource_id: "text-font:family".into(),
                    requested_path: None,
                    requested_family: Some("Inter".into()),
                },
                FontResourceBinding {
                    font_resource_id: "text-font:both".into(),
                    requested_path: Some("fonts/both.ttf".into()),
                    requested_family: Some("Source Sans".into()),
                },
            ]
        );
        let scene_debug = format!("{:?}", evaluated.scene);
        assert!(!scene_debug.contains("fonts/path.ttf"));
        assert!(!scene_debug.contains("fonts/both.ttf"));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedAncestors {
    pub(crate) matrix: [f64; 6],
    pub(crate) inverse: [f64; 6],
    pub(crate) opacity: f64,
    pub(crate) clip: EvaluatedTimeSpan,
}

impl EvaluatedVisualLayer {
    pub(crate) fn requires_affine(&self) -> bool {
        self.transform2d.is_some() || self.ancestors.is_some()
    }
    pub(crate) fn has_animated_geometry(&self) -> bool {
        self.ancestors.is_some()
            && self.transform2d.is_none()
            && self.keyframes.iter().any(|key| {
                matches!(
                    key.property,
                    EvaluatedProperty::Position | EvaluatedProperty::Scale
                )
            })
    }
    pub(crate) fn legacy_anchor(&self, source: (u32, u32)) -> (f64, f64) {
        let EvaluatedVisualSource::Text(text) = &self.source else {
            return (0.0, 0.0);
        };
        use EvaluatedAnchorPoint::*;
        let x = match text.style.anchor {
            TopCenter | Center | BottomCenter => 0.5,
            TopRight | CenterRight | BottomRight => 1.0,
            _ => 0.0,
        };
        let y = match text.style.anchor {
            CenterLeft | Center | CenterRight => 0.5,
            BottomLeft | BottomCenter | BottomRight => 1.0,
            _ => 0.0,
        };
        (x * f64::from(source.0), y * f64::from(source.1))
    }
    pub(crate) fn visible_span(&self) -> EvaluatedTimeSpan {
        self.ancestors.map_or(self.span, |parent| parent.clip)
    }
}

const IDENTITY_MATRIX: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn multiply_matrix(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
    let [a, b, c, d, x, y] = left;
    let [e, f, g, h, u, v] = right;
    [
        a * e + c * f,
        b * e + d * f,
        a * g + c * h,
        b * g + d * h,
        a * u + c * v + x,
        b * u + d * v + y,
    ]
}

fn apply_ancestors(
    project: &Project,
    layers: &mut Vec<EvaluatedVisualLayer>,
    canvas: (u32, u32),
) -> Result<(), CoreError> {
    let index: HashMap<_, _> = project
        .tracks
        .iter()
        .flat_map(|track| {
            track
                .items
                .iter()
                .map(move |item| (item.id(), (track.hidden, item)))
        })
        .collect();
    for layer in layers.iter_mut() {
        let mut node = index[layer.item_id.as_str()].1;
        if node.visual_properties().parent.is_none() {
            continue;
        }
        let mut ancestors = EvaluatedAncestors {
            matrix: IDENTITY_MATRIX,
            inverse: IDENTITY_MATRIX,
            opacity: 1.0,
            clip: layer.span,
        };
        while let Some(parent) = &node.visual_properties().parent {
            let (track_hidden, target) = index[parent.id.as_str()];
            let transform = target.visual_properties().transform2d.unwrap_or_default();
            let (matrix, inverse) = transform_matrices(transform, canvas, canvas)?;
            ancestors.matrix = multiply_matrix(matrix, ancestors.matrix);
            ancestors.inverse = multiply_matrix(ancestors.inverse, inverse);
            ancestors.opacity *= transform.opacity;
            if ancestors
                .matrix
                .iter()
                .chain(&ancestors.inverse)
                .any(|v| !v.is_finite())
            {
                return Err(invalid("non-finite ancestor matrix"));
            }
            ancestors.clip.start_ms = ancestors.clip.start_ms.max(target.start_ms());
            ancestors.clip.end_ms = ancestors.clip.end_ms.min(target.end_ms());
            if track_hidden || target.hidden() {
                ancestors.clip.end_ms = ancestors.clip.start_ms;
            }
            node = target;
        }
        layer.ancestors = Some(ancestors);
    }
    layers.retain(|layer| {
        let span = layer.visible_span();
        span.start_ms < span.end_ms
    });
    Ok(())
}

/// One canonical source-to-composition affine map and its outward-rounded raster bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedAffine {
    pub(crate) matrix: [f64; 6],
    pub(crate) inverse: [f64; 6],
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) opacity: f64,
}

fn transform_matrices(
    transform: crate::Transform2D,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<([f64; 6], [f64; 6]), CoreError> {
    transform.validate()?;
    if source.0 == 0 || source.1 == 0 {
        return Err(invalid("Transform2D source dimensions must be positive"));
    }
    let (sin, cos) = transform.rotation_deg.to_radians().sin_cos();
    let kx = transform.skew_x_deg.to_radians().tan();
    let ky = transform.skew_y_deg.to_radians().tan();
    let a = (cos - sin * ky) * transform.scale_x;
    let b = (sin + cos * ky) * transform.scale_x;
    let c = (cos * kx - sin * (1.0 + ky * kx)) * transform.scale_y;
    let d = (sin * kx + cos * (1.0 + ky * kx)) * transform.scale_y;
    let factor = match transform.position.unit {
        crate::PositionUnit::Pixels => (1.0, 1.0),
        crate::PositionUnit::Normalized => (f64::from(canvas.0), f64::from(canvas.1)),
    };
    let ax = transform.anchor.x * f64::from(source.0);
    let ay = transform.anchor.y * f64::from(source.1);
    let tx = transform.position.x * factor.0 - a * ax - c * ay;
    let ty = transform.position.y * factor.1 - b * ax - d * ay;
    let matrix = [a, b, c, d, tx, ty];
    let det = transform.scale_x * transform.scale_y;
    let inverse = [
        d / det,
        -b / det,
        -c / det,
        a / det,
        (c * ty - d * tx) / det,
        (b * tx - a * ty) / det,
    ];
    if matrix.iter().chain(&inverse).any(|v| !v.is_finite()) {
        return Err(invalid("Transform2D derived matrix must be finite"));
    }
    Ok((matrix, inverse))
}

pub(crate) fn evaluate_affine(
    transform: crate::Transform2D,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    let (matrix, inverse) = transform_matrices(transform, source, canvas)?;
    affine_from_matrices(matrix, inverse, source, transform.opacity)
}

fn affine_from_matrices(
    matrix: [f64; 6],
    inverse: [f64; 6],
    source: (u32, u32),
    opacity: f64,
) -> Result<EvaluatedAffine, CoreError> {
    let [a, b, c, d, tx, ty] = matrix;
    let corners = [
        (0.0, 0.0),
        (f64::from(source.0), 0.0),
        (0.0, f64::from(source.1)),
        (f64::from(source.0), f64::from(source.1)),
    ];
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for (x, y) in corners {
        let px = a * x + c * y + tx;
        let py = b * x + d * y + ty;
        if !px.is_finite() || !py.is_finite() {
            return Err(invalid("Transform2D derived coordinate must be finite"));
        }
        left = left.min(px);
        right = right.max(px);
        top = top.min(py);
        bottom = bottom.max(py);
    }
    // Snap trigonometric roundoff at integer edges, preserving exact right-angle bounds.
    let snap = |v: f64| {
        if (v - v.round()).abs() < 1e-9 {
            v.round()
        } else {
            v
        }
    };
    left = snap(left).floor();
    top = snap(top).floor();
    let w = (snap(right).ceil() - left).max(1.0);
    let h = (snap(bottom).ceil() - top).max(1.0);
    if w > 16_384.0 || h > 16_384.0 || w * h > 16_777_216.0 {
        return Err(invalid(
            "Transform2D transformed raster bounds exceed complexity limits",
        ));
    }
    Ok(EvaluatedAffine {
        matrix,
        inverse,
        left,
        top,
        width: w as u32,
        height: h as u32,
        opacity,
    })
}

pub(crate) fn evaluate_layer_affine(
    layer: &EvaluatedVisualLayer,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    if layer.ancestors.is_none() {
        return evaluate_affine(
            layer
                .transform2d
                .ok_or_else(|| invalid("missing local affine transform"))?,
            source,
            canvas,
        );
    }
    let parent = layer.ancestors.unwrap();
    let (local, inverse, opacity) = if let Some(transform) = layer.transform2d {
        let (matrix, inverse) = transform_matrices(transform, source, canvas)?;
        (matrix, inverse, transform.opacity)
    } else {
        let mut x = layer.transform.position_x;
        let mut y = layer.transform.position_y;
        if let EvaluatedVisualSource::Caption(caption) = &layer.source {
            x = (f64::from(canvas.0) - f64::from(source.0)) / 2.0;
            y = f64::from(canvas.1) - f64::from(source.1) - f64::from(caption.bottom_margin_px)
                + 12.0;
        }
        let scale = layer.transform.scale;
        let (ax, ay) = layer.legacy_anchor(source);
        x -= scale * ax;
        y -= scale * ay;
        (
            [scale, 0.0, 0.0, scale, x, y],
            [1.0 / scale, 0.0, 0.0, 1.0 / scale, -x / scale, -y / scale],
            layer.transform.opacity,
        )
    };
    let matrix = multiply_matrix(parent.matrix, local);
    let inverse = multiply_matrix(inverse, parent.inverse);
    if matrix.iter().chain(&inverse).any(|v| !v.is_finite()) {
        return Err(invalid("non-finite composed matrix"));
    }
    let mut affine = affine_from_matrices(matrix, inverse, source, opacity * parent.opacity)?;
    if layer.has_animated_geometry() {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut scales = Vec::new();
        for key in &layer.keyframes {
            match (key.property, key.value) {
                (EvaluatedProperty::Position, EvaluatedKeyframeValue::Position { x, y }) => {
                    xs.push(x);
                    ys.push(y);
                }
                (EvaluatedProperty::Scale, EvaluatedKeyframeValue::Scalar { value }) => {
                    scales.push(value)
                }
                _ => {}
            }
        }
        let extremes = |values: &[f64], default: f64| {
            if values.is_empty() {
                [default, default]
            } else {
                [
                    values.iter().copied().fold(f64::INFINITY, f64::min),
                    values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                ]
            }
        };
        let mut left = f64::INFINITY;
        let mut top = f64::INFINITY;
        let mut right = f64::NEG_INFINITY;
        let mut bottom = f64::NEG_INFINITY;
        // Existing easings are bounded by endpoint values. This fixed-size envelope
        // bounds all combinations without expanding animation into per-frame facts.
        for x in extremes(&xs, layer.transform.position_x) {
            for y in extremes(&ys, layer.transform.position_y) {
                for scale in extremes(&scales, layer.transform.scale) {
                    if !scale.is_finite() || scale <= 0.0 {
                        return Err(invalid("invalid animated scale"));
                    }
                    let (ax, ay) = layer.legacy_anchor(source);
                    let x = x - scale * ax;
                    let y = y - scale * ay;
                    let matrix = multiply_matrix(parent.matrix, [scale, 0.0, 0.0, scale, x, y]);
                    let inverse = multiply_matrix(
                        [1.0 / scale, 0.0, 0.0, 1.0 / scale, -x / scale, -y / scale],
                        parent.inverse,
                    );
                    let bounds =
                        affine_from_matrices(matrix, inverse, source, opacity * parent.opacity)?;
                    left = left.min(bounds.left);
                    top = top.min(bounds.top);
                    right = right.max(bounds.left + f64::from(bounds.width));
                    bottom = bottom.max(bounds.top + f64::from(bounds.height));
                }
            }
        }
        // Geometry was validated above before clipping. Travel only determines
        // which composition pixels may need sampling, never the object size limit.
        left = left.max(0.0).min(f64::from(canvas.0));
        top = top.max(0.0).min(f64::from(canvas.1));
        right = right.max(0.0).min(f64::from(canvas.0));
        bottom = bottom.max(0.0).min(f64::from(canvas.1));
        affine.left = left;
        affine.top = top;
        affine.width = (right - left).max(0.0) as u32;
        affine.height = (bottom - top).max(0.0) as u32;
    }
    Ok(affine)
}

pub(crate) fn finalize_affine_geometry(
    scene: &mut EvaluatedScene,
    measurements: &HashMap<String, (u32, u32)>,
) -> Result<(), CoreError> {
    // Collect first so a bad measurement cannot partially finalize the scene.
    let resolved = scene
        .visual_layers
        .iter()
        .map(|layer| {
            if !layer.requires_affine() {
                return Ok(None);
            }
            let size = layer
                .source_size
                .or_else(|| measurements.get(&layer.item_id).copied())
                .ok_or_else(|| invalid("Transform2D source measurement is missing"))?;
            Ok(Some((
                size,
                evaluate_layer_affine(layer, size, (scene.canvas.width, scene.canvas.height))?,
            )))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    for (layer, result) in scene.visual_layers.iter_mut().zip(resolved) {
        if let Some((size, affine)) = result {
            layer.source_size = Some(size);
            layer.affine = Some(affine);
            if layer.has_animated_geometry() {
                let mut tiles = Vec::new();
                for y in (0..affine.height).step_by(4096) {
                    for x in (0..affine.width).step_by(4096) {
                        tiles.push(EvaluatedAffine {
                            left: affine.left + f64::from(x),
                            top: affine.top + f64::from(y),
                            width: (affine.width - x).min(4096),
                            height: (affine.height - y).min(4096),
                            ..affine
                        });
                    }
                }
                layer.sampling_tiles = Some(tiles);
            }
        }
    }
    scene
        .visual_layers
        .retain(|layer| !layer.sampling_tiles.as_ref().is_some_and(Vec::is_empty));
    Ok(())
}

#[cfg(test)]
mod affine_tests {
    use super::*;
    #[test]
    fn independent_sequential_oracle_and_units() {
        let mut t = crate::Transform2D {
            anchor: crate::TransformAnchor { x: 0.25, y: 0.75 },
            scale_x: 1.7,
            scale_y: 0.6,
            skew_x_deg: 17.,
            skew_y_deg: -11.,
            rotation_deg: 33.,
            position: crate::TransformPosition {
                x: 70.,
                y: 90.,
                unit: crate::PositionUnit::Pixels,
            },
            ..Default::default()
        };
        let affine = evaluate_affine(t, (53, 29), (200, 120)).unwrap();
        for (x, y) in [(0., 0.), (53., 0.), (0., 29.), (53., 29.), (12.3, 17.1)] {
            let sx = (x - 0.25 * 53.) * 1.7;
            let sy = (y - 0.75 * 29.) * 0.6;
            let kx = sx + 17_f64.to_radians().tan() * sy;
            let ky = sy + (-11_f64).to_radians().tan() * kx;
            let (sin, cos) = 33_f64.to_radians().sin_cos();
            let expected = (kx * cos - ky * sin + 70., kx * sin + ky * cos + 90.);
            let [a, b, c, d, tx, ty] = affine.matrix;
            assert!((a * x + c * y + tx - expected.0).abs() < 1e-9);
            assert!((b * x + d * y + ty - expected.1).abs() < 1e-9);
            let [a, b, c, d, tx, ty] = affine.inverse;
            assert!((a * expected.0 + c * expected.1 + tx - x).abs() < 1e-9);
            assert!((b * expected.0 + d * expected.1 + ty - y).abs() < 1e-9);
        }
        t.position.unit = crate::PositionUnit::Normalized;
        t.position.x = 70. / 200.;
        t.position.y = 90. / 120.;
        assert_eq!(evaluate_affine(t, (53, 29), (200, 120)).unwrap(), affine);
    }
    #[test]
    fn geometry_boundaries_are_checked_before_clipping() {
        let t = crate::Transform2D::default();
        for size in [(16384, 1), (4096, 4096)] {
            assert!(evaluate_affine(t, size, (1, 1)).is_ok());
        }
        for size in [(16385, 1), (4096, 4097), (0, 1)] {
            assert_eq!(
                evaluate_affine(t, size, (1, 1)).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        let mut t = t;
        t.rotation_deg = 90.;
        let a = evaluate_affine(t, (100, 20), (200, 200)).unwrap();
        assert_eq!((a.width, a.height, a.left, a.top), (20, 100, -20., 0.));
    }
}
