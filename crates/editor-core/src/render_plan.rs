//! Deterministic scene evaluation and render planning owner.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    Asset, CoreError, Easing, ErrorCode, Keyframe, KeyframeProperty, KeyframeValue, MediaType,
    Project, TextItem, TimelineItem, Track, animation::positive_scalar_ranges,
};

/// The inward handoff consumed by render planning. Issue #12 may replace the
/// current project-backed input with its canonical `EvaluatedScene`.
pub(crate) struct SceneInput<'a> {
    pub(crate) project: &'a Project,
    pub(crate) intent: RenderIntent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MediaInputRequest {
    pub(crate) item_id: String,
    pub(crate) asset_id: String,
    pub(crate) project_relative_path: PathBuf,
    pub(crate) media_type: MediaType,
    pub(crate) source_in_ms: u64,
    pub(crate) duration_ms: u64,
    pub(crate) input_index: usize,
}

#[derive(Debug)]
pub(crate) struct SceneEvaluation<'a> {
    asset_by_id: HashMap<&'a str, &'a Asset>,
    input_indexes: HashMap<String, usize>,
    active_tracks: Vec<&'a Track>,
    pub(crate) media_inputs: Vec<MediaInputRequest>,
    pub(crate) text_resources: Vec<&'a TextItem>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: u32,
    pub(crate) duration_ms: u64,
    pub(crate) intent: RenderIntent,
}

pub(crate) fn evaluate_scene(
    scene: SceneInput<'_>,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<SceneEvaluation<'_>, CoreError> {
    let project = scene.project;
    // Issue #12 establishes the canonical, bounded, renderer-neutral evaluation
    // boundary. Issue #13 will replace this borrowed compatibility representation
    // with the returned scene-and-binding envelope after render-plan parity is proven.
    let _evaluated = crate::evaluated_scene::evaluate_project(project, width, height, fps)?;
    let asset_by_id = project
        .assets
        .iter()
        .map(|asset| (asset.id.as_str(), asset))
        .collect::<HashMap<_, _>>();
    let mut input_indexes = HashMap::new();
    let mut media_inputs = Vec::new();
    let text_resources = project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .filter_map(|item| match item {
            TimelineItem::Text(text) if !text.hidden => Some(text),
            _ => None,
        })
        .collect();
    for item in project
        .tracks
        .iter()
        .filter(|track| !track.hidden)
        .flat_map(|track| &track.items)
        .filter(|item| !item.hidden())
    {
        match item {
            TimelineItem::Media(media) if !input_indexes.contains_key(&media.id) => {
                let asset = asset_by_id.get(media.asset_id.as_str()).ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::AssetNotFound,
                        "timeline references a missing asset",
                    )
                })?;
                let input_index = media_inputs.len() + 2;
                input_indexes.insert(media.id.clone(), input_index);
                media_inputs.push(MediaInputRequest {
                    item_id: media.id.clone(),
                    asset_id: media.asset_id.clone(),
                    project_relative_path: PathBuf::from(&asset.project_relative_path),
                    media_type: asset.media_type,
                    source_in_ms: media.source_in_ms,
                    duration_ms: media.duration_ms,
                    input_index,
                });
            }
            _ => {}
        }
    }
    let active_tracks = project
        .tracks
        .iter()
        .filter(|track| !track.hidden)
        .collect();
    Ok(SceneEvaluation {
        asset_by_id,
        input_indexes,
        active_tracks,
        media_inputs,
        text_resources,
        width,
        height,
        fps,
        duration_ms: project.duration_ms().max(1),
        intent: scene.intent,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderIntent {
    Frame {
        at_ms: u64,
    },
    Range {
        start_ms: u64,
        end_ms: u64,
        include_audio: bool,
    },
    Export,
}

pub(crate) struct PreparedText {
    pub(crate) file_path: PathBuf,
    pub(crate) font_path: Option<PathBuf>,
    pub(crate) layer_width: u32,
    pub(crate) layer_height: u32,
    pub(crate) canvas_width: u32,
    pub(crate) canvas_height: u32,
    pub(crate) text_x: u32,
    pub(crate) text_y: u32,
}

#[cfg(test)]
pub(crate) struct FilterContext<'a> {
    pub(crate) text_layers: &'a HashMap<String, PreparedText>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderPlan {
    pub(crate) filter_graph: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: u32,
    pub(crate) duration_ms: u64,
    pub(crate) intent: RenderIntent,
    pub(crate) media_inputs: Vec<MediaInputRequest>,
    pub(crate) media_paths: Vec<PathBuf>,
}

pub(crate) fn build_render_plan(
    scene: &SceneEvaluation<'_>,
    text_layers: &HashMap<String, PreparedText>,
    media_paths: Vec<PathBuf>,
    default_font_path: Option<&Path>,
    _warnings: &mut Vec<String>,
) -> Result<RenderPlan, CoreError> {
    let asset_by_id = &scene.asset_by_id;
    let input_indexes = &scene.input_indexes;
    let (width, height, fps) = (scene.width, scene.height, scene.fps);
    let mut filters = vec!["[0:v]format=yuv420p[base0]".to_owned()];
    let mut current_video = "base0".to_owned();
    let mut visual_count = 0_usize;
    let mut audio_labels = vec!["[1:a]".to_owned()];
    let transitions = scene
        .active_tracks
        .iter()
        .flat_map(|track| &track.items)
        .filter_map(|item| match item {
            TimelineItem::Transition(value) if !value.hidden => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    let voiceover_intervals =
        audible_voiceover_intervals_for_tracks(&scene.active_tracks, asset_by_id);

    for track in &scene.active_tracks {
        for item in &track.items {
            if item.hidden() {
                continue;
            }
            match item {
                TimelineItem::Media(media) => {
                    let asset = asset_by_id.get(media.asset_id.as_str()).ok_or_else(|| {
                        CoreError::new(
                            ErrorCode::AssetNotFound,
                            "timeline references a missing asset",
                        )
                    })?;
                    let input = input_indexes.get(&media.id).ok_or_else(|| {
                        CoreError::new(
                            ErrorCode::InternalError,
                            "renderer input mapping is missing",
                        )
                    })?;
                    if asset.media_type != MediaType::Audio {
                        visual_count += 1;
                        let prepared = format!("visual{visual_count}");
                        let composited = format!("base{visual_count}");
                        let scale = scalar_expression(
                            &media.keyframes,
                            KeyframeProperty::Scale,
                            media.transform.scale,
                            media.start_ms,
                        );
                        let x = position_expression(
                            &media.keyframes,
                            true,
                            media.transform.position_x,
                            media.start_ms,
                        );
                        let y = position_expression(
                            &media.keyframes,
                            false,
                            media.transform.position_y,
                            media.start_ms,
                        );
                        let opacity = scalar_expression_for(
                            &media.keyframes,
                            KeyframeProperty::Opacity,
                            media.transform.opacity,
                            media.start_ms,
                            "T",
                        );
                        let fade = transition_filters(&media.id, &transitions, media.start_ms);
                        filters.push(format!(
                            "[{input}:v]setpts=PTS-STARTPTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{fade}[{prepared}]",
                            seconds(media.start_ms)
                        ));
                        filters.push(format!(
                            "[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]",
                            seconds(media.start_ms),
                            seconds(media.start_ms.saturating_add(media.duration_ms))
                        ));
                        current_video = composited;
                    }
                    if asset.has_audio && !track.muted && !media.audio.muted {
                        let label = format!("audio{}", audio_labels.len());
                        let automation =
                            scalar_expression(&media.keyframes, KeyframeProperty::Volume, 1.0, 0);
                        let volume =
                            format!("({})*({automation})", format_number(media.audio.volume));
                        let ducking = ducking_expression(track, &voiceover_intervals);
                        let mut chain = format!(
                            "[{input}:a]atrim=duration={},asetpts=PTS-STARTPTS,volume='{volume}':eval=frame",
                            seconds(media.duration_ms),
                        );
                        if media.audio.fade_in_ms > 0 {
                            chain.push_str(&format!(
                                ",afade=t=in:st=0:d={}",
                                seconds(media.audio.fade_in_ms)
                            ));
                        }
                        if media.audio.fade_out_ms > 0
                            && media.audio.fade_out_ms < media.duration_ms
                        {
                            chain.push_str(&format!(
                                ",afade=t=out:st={}:d={}",
                                seconds(media.duration_ms - media.audio.fade_out_ms),
                                seconds(media.audio.fade_out_ms)
                            ));
                        }
                        chain.push_str(&format!(
                            ",asetpts=PTS+{}/TB,volume='{ducking}':eval=frame",
                            seconds(media.start_ms)
                        ));
                        chain.push_str(&format!("[{label}]"));
                        filters.push(chain);
                        audio_labels.push(format!("[{label}]"));
                    }
                }
                TimelineItem::Text(text) => {
                    visual_count += 1;
                    let prepared = format!("visual{visual_count}");
                    let composited = format!("base{visual_count}");
                    let x = position_expression(
                        &text.keyframes,
                        true,
                        text.transform.position_x,
                        text.start_ms,
                    );
                    let y = position_expression(
                        &text.keyframes,
                        false,
                        text.transform.position_y,
                        text.start_ms,
                    );
                    let scale = scalar_expression(
                        &text.keyframes,
                        KeyframeProperty::Scale,
                        text.transform.scale,
                        text.start_ms,
                    );
                    let opacity = scalar_expression_for(
                        &text.keyframes,
                        KeyframeProperty::Opacity,
                        text.transform.opacity,
                        text.start_ms,
                        "T",
                    );
                    let prepared_text = text_layers.get(&text.id).ok_or_else(|| {
                        CoreError::new(ErrorCode::InternalError, "renderer text file is missing")
                    })?;
                    let font = prepared_text
                        .font_path
                        .as_ref()
                        .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
                        .unwrap_or_default();
                    let (x, y) = anchored_layer_position(&x, &y, text.style.anchor);
                    let alignment = match text.style.alignment {
                        crate::TextAlignment::Left => "L",
                        crate::TextAlignment::Center => "C",
                        crate::TextAlignment::Right => "R",
                    };
                    let padding = &text.style.padding;
                    let (pad_x, pad_y) = text_layer_padding(text.style.anchor);
                    let transition = transition_filters(&text.id, &transitions, text.start_ms);
                    filters.push(format!(
                            "color=c=black@0.0:s={}x{}:r={fps}:d={},format=rgba,drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:borderw={}:bordercolor={}:shadowx={}:shadowy={}:shadowcolor={}@{}:box=1:boxcolor={}@{}:boxborderw={}|{}|{}|{}:line_spacing={}:text_align={alignment}:x={}:y={},scale=w='iw*({scale})':h='ih*({scale})':eval=frame,pad={}:{}:{pad_x}:{pad_y}:color=black@0:eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]",
                            prepared_text.layer_width,
                            prepared_text.layer_height,
                            seconds(scene.duration_ms),
                            escape_filter_path(&prepared_text.file_path),
                            text.font_size,
                            text.color,
                            text.style.outline_width_px,
                            text.style.outline_color,
                            text.style.shadow.offset_x,
                            text.style.shadow.offset_y,
                            text.style.shadow.color,
                            text.style.shadow.opacity,
                            text.style.background_color,
                            text.style.background_opacity,
                            padding.top, padding.right, padding.bottom, padding.left,
                            text.style.line_spacing_px,
                            prepared_text.text_x,
                            prepared_text.text_y,
                            prepared_text.canvas_width,
                            prepared_text.canvas_height,
                        ));
                    filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(text.start_ms), seconds(text.start_ms.saturating_add(text.duration_ms))));
                    current_video = composited;
                }
                TimelineItem::SolidColor(shape) => {
                    visual_count += 1;
                    let prepared = format!("visual{visual_count}");
                    let composited = format!("base{visual_count}");
                    let scale = scalar_expression(
                        &shape.keyframes,
                        KeyframeProperty::Scale,
                        shape.transform.scale,
                        shape.start_ms,
                    );
                    let opacity = scalar_expression_for(
                        &shape.keyframes,
                        KeyframeProperty::Opacity,
                        shape.transform.opacity,
                        shape.start_ms,
                        "T",
                    );
                    let x = position_expression(
                        &shape.keyframes,
                        true,
                        shape.transform.position_x,
                        shape.start_ms,
                    );
                    let y = position_expression(
                        &shape.keyframes,
                        false,
                        shape.transform.position_y,
                        shape.start_ms,
                    );
                    let transition = transition_filters(&shape.id, &transitions, shape.start_ms);
                    filters.push(format!("color=c={}:s={width}x{height}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]", ffmpeg_color(&shape.color), seconds(shape.duration_ms), seconds(shape.start_ms)));
                    filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(shape.start_ms), seconds(shape.start_ms.saturating_add(shape.duration_ms))));
                    current_video = composited;
                }
                TimelineItem::Rectangle(shape) => {
                    visual_count += 1;
                    let prepared = format!("visual{visual_count}");
                    let composited = format!("base{visual_count}");
                    let scale = scalar_expression(
                        &shape.keyframes,
                        KeyframeProperty::Scale,
                        shape.transform.scale,
                        shape.start_ms,
                    );
                    let opacity = scalar_expression_for(
                        &shape.keyframes,
                        KeyframeProperty::Opacity,
                        shape.transform.opacity,
                        shape.start_ms,
                        "T",
                    );
                    let x = position_expression(
                        &shape.keyframes,
                        true,
                        shape.transform.position_x,
                        shape.start_ms,
                    );
                    let y = position_expression(
                        &shape.keyframes,
                        false,
                        shape.transform.position_y,
                        shape.start_ms,
                    );
                    let transition = transition_filters(&shape.id, &transitions, shape.start_ms);
                    filters.push(format!("color=c={}:s={}x{}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]", ffmpeg_color(&shape.color), shape.width, shape.height, seconds(shape.duration_ms), seconds(shape.start_ms)));
                    filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(shape.start_ms), seconds(shape.start_ms.saturating_add(shape.duration_ms))));
                    current_video = composited;
                }
                TimelineItem::Caption(caption) => {
                    visual_count += 1;
                    let composited = format!("base{visual_count}");
                    let font = default_font_path
                        .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
                        .unwrap_or_default();
                    filters.push(format!(
                        "[{current_video}]drawtext={font}text='{}':fontsize={}:fontcolor={}:box=1:boxcolor={}@0.75:boxborderw=12:x='(w-text_w)/2':y='h-text_h-{}':enable='between(t,{},{})'[{composited}]",
                        escape_filter(&caption.text),
                        caption.style.font_size,
                        caption.style.color,
                        caption.style.background_color,
                        caption.style.bottom_margin_px,
                        seconds(caption.start_ms),
                        seconds(caption.start_ms.saturating_add(caption.duration_ms))
                    ));
                    current_video = composited;
                }
                TimelineItem::Transition(_) => {}
            }
        }
    }
    filters.push(format!(
            "[{current_video}]scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]"
        ));
    filters.push(format!(
        "{}amix=inputs={}:duration=longest:normalize=0[audio]",
        audio_labels.join(""),
        audio_labels.len()
    ));
    Ok(RenderPlan {
        filter_graph: filters.join(";\n"),
        width,
        height,
        fps,
        duration_ms: scene.duration_ms,
        intent: scene.intent,
        media_inputs: scene.media_inputs.clone(),
        media_paths,
    })
}
pub(crate) fn ffmpeg_color(color: &str) -> String {
    format!("0x{}", color.trim_start_matches('#'))
}

pub(crate) fn anchored_layer_position(
    x: &str,
    y: &str,
    anchor: crate::AnchorPoint,
) -> (String, String) {
    use crate::AnchorPoint::*;
    let anchored_x = match anchor {
        TopCenter | Center | BottomCenter => format!("({x})-(overlay_w/2)"),
        TopRight | CenterRight | BottomRight => format!("({x})-overlay_w"),
        _ => x.into(),
    };
    let anchored_y = match anchor {
        CenterLeft | Center | CenterRight => format!("({y})-(overlay_h/2)"),
        BottomLeft | BottomCenter | BottomRight => format!("({y})-overlay_h"),
        _ => y.into(),
    };
    (anchored_x, anchored_y)
}

pub(crate) fn text_layer_padding(anchor: crate::AnchorPoint) -> (&'static str, &'static str) {
    use crate::AnchorPoint::*;
    let x = match anchor {
        TopCenter | Center | BottomCenter => "(ow-iw)/2",
        TopRight | CenterRight | BottomRight => "ow-iw",
        _ => "0",
    };
    let y = match anchor {
        CenterLeft | Center | CenterRight => "(oh-ih)/2",
        BottomLeft | BottomCenter | BottomRight => "oh-ih",
        _ => "0",
    };
    (x, y)
}

pub(crate) fn ducking_expression(track: &crate::Track, intervals: &[(u64, u64)]) -> String {
    let Some(settings) = track.ducking.as_ref().filter(|settings| settings.enabled) else {
        return "1".into();
    };
    if track.audio_role != crate::AudioTrackRole::Music || intervals.is_empty() {
        return "1".into();
    }
    let mut expression = "1".to_owned();
    for (start, end) in intervals {
        let attack_start = start.saturating_sub(settings.attack_ms);
        let release_end = end.saturating_add(settings.release_ms);
        let attack = seconds(settings.attack_ms.max(1));
        let release = seconds(settings.release_ms.max(1));
        let gain = format_number(settings.gain);
        let envelope = format!(
            "if(between(t,{},{}),1-(1-({gain}))*((t-{})/{attack}),if(between(t,{},{}),({gain}),if(between(t,{},{}),({gain})+(1-({gain}))*((t-{})/{release}),1)))",
            seconds(attack_start),
            seconds(*start),
            seconds(attack_start),
            seconds(*start),
            seconds(*end),
            seconds(*end),
            seconds(release_end),
            seconds(*end),
        );
        expression = format!("min({expression},{envelope})");
    }
    expression
}

#[cfg(test)]
pub(crate) fn audible_voiceover_intervals(
    project: &Project,
    asset_by_id: &HashMap<&str, &crate::Asset>,
) -> Vec<(u64, u64)> {
    let tracks = project
        .tracks
        .iter()
        .filter(|track| !track.hidden)
        .collect::<Vec<_>>();
    audible_voiceover_intervals_for_tracks(&tracks, asset_by_id)
}

fn audible_voiceover_intervals_for_tracks(
    tracks: &[&Track],
    asset_by_id: &HashMap<&str, &crate::Asset>,
) -> Vec<(u64, u64)> {
    merge_intervals(
        tracks
            .iter()
            .filter(|track| !track.muted && track.audio_role == crate::AudioTrackRole::Voiceover)
            .flat_map(|track| track.items.iter())
            .flat_map(|item| {
                let TimelineItem::Media(media) = item else {
                    return vec![];
                };
                if media.hidden
                    || media.audio.muted
                    || media.audio.volume == 0.0
                    || !asset_by_id
                        .get(media.asset_id.as_str())
                        .is_some_and(|asset| asset.has_audio)
                {
                    return vec![];
                }
                positive_scalar_ranges(
                    &media.keyframes,
                    KeyframeProperty::Volume,
                    media.duration_ms,
                )
                .into_iter()
                .map(|(start, end)| {
                    (
                        media.start_ms.saturating_add(start),
                        media.start_ms.saturating_add(end),
                    )
                })
                .collect()
            })
            .collect(),
    )
}

pub(crate) fn merge_intervals(mut intervals: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    intervals.retain(|(start, end)| start < end);
    intervals.sort_unstable_by_key(|(start, end)| (*start, *end));
    let mut merged: Vec<(u64, u64)> = Vec::with_capacity(intervals.len());
    for (start, end) in intervals {
        if let Some((_, previous_end)) = merged.last_mut()
            && start <= *previous_end
        {
            *previous_end = (*previous_end).max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

#[cfg(test)]
pub(crate) fn ducking_gain_at(
    settings: &crate::DuckingSettings,
    intervals: &[(u64, u64)],
    time_ms: u64,
) -> f64 {
    intervals.iter().fold(1.0, |gain, (start, end)| {
        let attack_start = start.saturating_sub(settings.attack_ms);
        let release_end = end.saturating_add(settings.release_ms);
        let envelope = if (attack_start..*start).contains(&time_ms) {
            let progress = (time_ms - attack_start) as f64 / settings.attack_ms.max(1) as f64;
            1.0 - (1.0 - settings.gain) * progress
        } else if (*start..=*end).contains(&time_ms) {
            settings.gain
        } else if (*end < time_ms) && time_ms <= release_end {
            let progress = (time_ms - end) as f64 / settings.release_ms.max(1) as f64;
            settings.gain + (1.0 - settings.gain) * progress
        } else {
            1.0
        };
        gain.min(envelope)
    })
}

pub(crate) fn transition_filters(
    item_id: &str,
    transitions: &[&crate::TransitionItem],
    _item_start_ms: u64,
) -> String {
    let mut result = String::new();
    for transition in transitions {
        if transition.from_item_id == item_id {
            result.push_str(&format!(
                ",fade=t=out:st={}:d={}:alpha=1",
                seconds(transition.start_ms),
                seconds(transition.duration_ms)
            ));
        }
        if transition.to_item_id.as_deref() == Some(item_id) {
            result.push_str(&format!(
                ",fade=t=in:st={}:d={}:alpha=1",
                seconds(transition.start_ms),
                seconds(transition.duration_ms)
            ));
        }
    }
    result
}

pub(crate) fn scalar_expression(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    default: f64,
    item_start_ms: u64,
) -> String {
    scalar_expression_for(keyframes, property, default, item_start_ms, "t")
}

pub(crate) fn scalar_expression_for(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| {
            if keyframe.property != property {
                return None;
            }
            let KeyframeValue::Scalar { value } = keyframe.value else {
                return None;
            };
            Some((keyframe.time_ms, value, keyframe.easing))
        })
        .collect::<Vec<_>>();
    piecewise_expression_for(&values, default, item_start_ms, time_variable)
}

pub(crate) fn position_expression(
    keyframes: &[Keyframe],
    x_axis: bool,
    default: f64,
    item_start_ms: u64,
) -> String {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| {
            if keyframe.property != KeyframeProperty::Position {
                return None;
            }
            let KeyframeValue::Position { x, y } = keyframe.value else {
                return None;
            };
            Some((
                keyframe.time_ms,
                if x_axis { x } else { y },
                keyframe.easing,
            ))
        })
        .collect::<Vec<_>>();
    piecewise_expression(&values, default, item_start_ms)
}

pub(crate) fn piecewise_expression(
    values: &[(u64, f64, Easing)],
    default: f64,
    item_start_ms: u64,
) -> String {
    piecewise_expression_for(values, default, item_start_ms, "t")
}

pub(crate) fn piecewise_expression_for(
    values: &[(u64, f64, Easing)],
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
    if values.is_empty() {
        return format_number(default);
    }
    let mut expression = format_number(values.last().map_or(default, |value| value.1));
    for pair in values.windows(2).rev() {
        let (start_time, start_value, easing) = pair[0];
        let (end_time, end_value, _) = pair[1];
        let global_start = seconds(item_start_ms.saturating_add(start_time));
        let global_end = seconds(item_start_ms.saturating_add(end_time));
        let span = seconds(end_time.saturating_sub(start_time).max(1));
        let progress = format!("(({time_variable})-({global_start}))/({span})");
        let eased = easing_expression(&progress, easing);
        let interpolated = format!(
            "({})+(({})-({}))*({eased})",
            format_number(start_value),
            format_number(end_value),
            format_number(start_value)
        );
        expression =
            format!("if(lt(({time_variable}),({global_end})),{interpolated},{expression})");
    }
    let first_time = seconds(item_start_ms.saturating_add(values[0].0));
    format!(
        "if(lt(({time_variable}),({first_time})),({}),{expression})",
        format_number(values[0].1)
    )
}

pub(crate) fn easing_expression(progress: &str, easing: Easing) -> String {
    match easing {
        Easing::Hold => "0".into(),
        Easing::Linear => progress.into(),
        Easing::EaseIn => format!("({progress})*({progress})"),
        Easing::EaseOut => format!("1-(1-({progress}))*(1-({progress}))"),
        Easing::EaseInOut => format!(
            "if(lt(({progress}),0.5),2*({progress})*({progress}),1-pow(-2*({progress})+2,2)/2)"
        ),
    }
}

pub(crate) fn escape_filter(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('\'', "\\'")
        .replace('%', "\\%")
        .replace('\n', "\\n")
}

pub(crate) fn escape_filter_path(path: &Path) -> String {
    escape_filter(&path.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn seconds(milliseconds: u64) -> String {
    format!("{:.3}", milliseconds as f64 / 1_000.0)
}

pub(crate) fn format_number(value: f64) -> String {
    format!("{value:.6}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Asset, AudioSettings, AudioTrackRole, MediaItem, PROJECT_SCHEMA_VERSION, ProjectSettings,
        TextItem, TextStyle, Track, TrackType, Transform,
    };

    fn empty_project() -> Project {
        Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "Project".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        }
    }

    #[test]
    fn a_scene_produces_a_deterministic_declarative_plan_without_process_io() {
        let project = empty_project();
        let text = HashMap::new();
        let mut warnings = vec![];
        let scene = evaluate_scene(
            SceneInput {
                project: &project,
                intent: RenderIntent::Export,
            },
            1_920,
            1_080,
            30,
        )
        .unwrap();
        let first = build_render_plan(&scene, &text, vec![], None, &mut warnings).unwrap();
        let second = build_render_plan(&scene, &text, vec![], None, &mut warnings).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            (first.width, first.height, first.fps, first.duration_ms),
            (1_920, 1_080, 30, 1)
        );
        assert!(first.filter_graph.contains("[video]"));
        assert!(first.filter_graph.contains("[audio]"));

        for intent in [
            RenderIntent::Frame { at_ms: 500 },
            RenderIntent::Range {
                start_ms: 100,
                end_ms: 900,
                include_audio: false,
            },
            RenderIntent::Export,
        ] {
            let scene = evaluate_scene(
                SceneInput {
                    project: &project,
                    intent,
                },
                1_920,
                1_080,
                30,
            )
            .unwrap();
            let plan = build_render_plan(&scene, &text, vec![], None, &mut warnings).unwrap();
            assert_eq!(plan.intent, intent);
        }
    }

    #[test]
    fn scene_evaluation_orders_inputs_and_resource_requests_without_io() {
        let mut project = empty_project();
        project.assets = vec![
            Asset {
                id: "video-asset".into(),
                media_type: MediaType::Video,
                file_name: "video.mp4".into(),
                project_relative_path: "assets/video.mp4".into(),
                duration_ms: Some(2_125),
                has_audio: true,
                origin: None,
                content_hash: None,
                size_bytes: None,
                probe: None,
            },
            Asset {
                id: "audio-asset".into(),
                media_type: MediaType::Audio,
                file_name: "audio.wav".into(),
                project_relative_path: "assets/audio.wav".into(),
                duration_ms: Some(3_250),
                has_audio: true,
                origin: None,
                content_hash: None,
                size_bytes: None,
                probe: None,
            },
        ];
        project.tracks = vec![Track {
            id: "track".into(),
            name: "Track".into(),
            track_type: TrackType::Video,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![
                TimelineItem::Media(MediaItem {
                    id: "video".into(),
                    asset_id: "video-asset".into(),
                    start_ms: 0,
                    duration_ms: 2_000,
                    source_in_ms: 125,
                    transform: Transform::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                    hidden: false,
                }),
                TimelineItem::Media(MediaItem {
                    id: "audio".into(),
                    asset_id: "audio-asset".into(),
                    start_ms: 500,
                    duration_ms: 3_000,
                    source_in_ms: 250,
                    transform: Transform::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                    hidden: false,
                }),
                TimelineItem::Text(TextItem {
                    id: "title".into(),
                    text: "Title".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 40,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    transform: Transform::default(),
                    keyframes: vec![],
                    hidden: false,
                }),
            ],
        }];
        let scene = evaluate_scene(
            SceneInput {
                project: &project,
                intent: RenderIntent::Range {
                    start_ms: 500,
                    end_ms: 1_500,
                    include_audio: true,
                },
            },
            640,
            360,
            24,
        )
        .unwrap();
        assert_eq!(
            scene
                .media_inputs
                .iter()
                .map(|input| (input.item_id.as_str(), input.input_index))
                .collect::<Vec<_>>(),
            vec![("video", 2), ("audio", 3)]
        );
        assert_eq!(scene.media_inputs[0].source_in_ms, 125);
        assert_eq!(scene.media_inputs[1].duration_ms, 3_000);
        assert_eq!(
            scene
                .text_resources
                .iter()
                .map(|text| text.id.as_str())
                .collect::<Vec<_>>(),
            vec!["title"]
        );
        assert_eq!((scene.width, scene.height, scene.fps), (640, 360, 24));
        assert_eq!(scene.duration_ms, 3_500);
    }

    #[test]
    fn filter_arguments_escape_text_and_paths_without_shell_syntax() {
        assert_eq!(escape_filter("it's: 100%"), "it\\'s\\: 100\\%");
        assert!(!scalar_expression(&[], KeyframeProperty::Scale, 1.0, 0).contains("$"));
        assert_eq!(seconds(1_250), "1.250");
    }
}
