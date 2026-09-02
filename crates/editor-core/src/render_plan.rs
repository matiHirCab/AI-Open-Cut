//! Deterministic scene evaluation and render planning owner.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    CoreError, ErrorCode, MediaType,
    evaluated_scene::{
        EvaluatedAnchorPoint, EvaluatedAudioLayer, EvaluatedDucking, EvaluatedEasing,
        EvaluatedKeyframe, EvaluatedKeyframeValue, EvaluatedProperty, EvaluatedScene,
        EvaluatedTextAlignment, EvaluatedTransition, EvaluatedTransitionRole,
        EvaluatedVisualSource,
    },
};

#[cfg(test)]
use crate::{
    Easing, Keyframe, KeyframeProperty, KeyframeValue, Project, TimelineItem, Track,
    animation::positive_scalar_ranges,
};

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
    scene: &EvaluatedScene,
    text_layers: &HashMap<String, PreparedText>,
    media_inputs: Vec<MediaInputRequest>,
    media_paths: Vec<PathBuf>,
    default_font_path: Option<&Path>,
    intent: RenderIntent,
    _warnings: &mut Vec<String>,
) -> Result<RenderPlan, CoreError> {
    let input_indexes = media_inputs
        .iter()
        .map(|input| (input.item_id.as_str(), input.input_index))
        .collect::<HashMap<_, _>>();
    let (width, height, fps) = (scene.canvas.width, scene.canvas.height, scene.canvas.fps);
    let mut filters = vec!["[0:v]format=yuv420p[base0]".to_owned()];
    let mut current_video = "base0".to_owned();
    let mut visual_count = 0_usize;
    let mut audio_labels = vec!["[1:a]".to_owned()];
    for layer in &scene.visual_layers {
        match &layer.source {
            EvaluatedVisualSource::Media { .. } => {
                let input = input_indexes.get(layer.item_id.as_str()).ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::InternalError,
                        "renderer input mapping is missing",
                    )
                })?;
                visual_count += 1;
                let prepared = format!("visual{visual_count}");
                let composited = format!("base{visual_count}");
                let scale = evaluated_scalar_expression(
                    &layer.keyframes,
                    EvaluatedProperty::Scale,
                    layer.transform.scale,
                    layer.span.start_ms,
                );
                let x = evaluated_position_expression(
                    &layer.keyframes,
                    true,
                    layer.transform.position_x,
                    layer.span.start_ms,
                );
                let y = evaluated_position_expression(
                    &layer.keyframes,
                    false,
                    layer.transform.position_y,
                    layer.span.start_ms,
                );
                let opacity = evaluated_scalar_expression_for(
                    &layer.keyframes,
                    EvaluatedProperty::Opacity,
                    layer.transform.opacity,
                    layer.span.start_ms,
                    "T",
                );
                let fade = evaluated_transition_filters(&layer.transitions);
                filters.push(format!(
                    "[{input}:v]setpts=PTS-STARTPTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{fade}[{prepared}]",
                    seconds(layer.span.start_ms)
                ));
                filters.push(format!(
                    "[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]",
                    seconds(layer.span.start_ms), seconds(layer.span.end_ms)
                ));
                current_video = composited;
            }
            EvaluatedVisualSource::Text(text) => {
                visual_count += 1;
                let prepared = format!("visual{visual_count}");
                let composited = format!("base{visual_count}");
                let x = evaluated_position_expression(
                    &layer.keyframes,
                    true,
                    layer.transform.position_x,
                    layer.span.start_ms,
                );
                let y = evaluated_position_expression(
                    &layer.keyframes,
                    false,
                    layer.transform.position_y,
                    layer.span.start_ms,
                );
                let scale = evaluated_scalar_expression(
                    &layer.keyframes,
                    EvaluatedProperty::Scale,
                    layer.transform.scale,
                    layer.span.start_ms,
                );
                let opacity = evaluated_scalar_expression_for(
                    &layer.keyframes,
                    EvaluatedProperty::Opacity,
                    layer.transform.opacity,
                    layer.span.start_ms,
                    "T",
                );
                let prepared_text = text_layers.get(&layer.item_id).ok_or_else(|| {
                    CoreError::new(ErrorCode::InternalError, "renderer text file is missing")
                })?;
                let font = prepared_text
                    .font_path
                    .as_ref()
                    .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
                    .unwrap_or_default();
                let (x, y) = evaluated_anchored_layer_position(&x, &y, text.style.anchor);
                let alignment = match text.style.alignment {
                    EvaluatedTextAlignment::Left => "L",
                    EvaluatedTextAlignment::Center => "C",
                    EvaluatedTextAlignment::Right => "R",
                };
                let padding = &text.style.padding;
                let (pad_x, pad_y) = evaluated_text_layer_padding(text.style.anchor);
                let transition = evaluated_transition_filters(&layer.transitions);
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
                filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(layer.span.start_ms), seconds(layer.span.end_ms)));
                current_video = composited;
            }
            EvaluatedVisualSource::SolidColor { color } => {
                visual_count += 1;
                let prepared = format!("visual{visual_count}");
                let composited = format!("base{visual_count}");
                let scale = evaluated_scalar_expression(
                    &layer.keyframes,
                    EvaluatedProperty::Scale,
                    layer.transform.scale,
                    layer.span.start_ms,
                );
                let opacity = evaluated_scalar_expression_for(
                    &layer.keyframes,
                    EvaluatedProperty::Opacity,
                    layer.transform.opacity,
                    layer.span.start_ms,
                    "T",
                );
                let x = evaluated_position_expression(
                    &layer.keyframes,
                    true,
                    layer.transform.position_x,
                    layer.span.start_ms,
                );
                let y = evaluated_position_expression(
                    &layer.keyframes,
                    false,
                    layer.transform.position_y,
                    layer.span.start_ms,
                );
                let transition = evaluated_transition_filters(&layer.transitions);
                filters.push(format!("color=c={}:s={width}x{height}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]", ffmpeg_color(color), seconds(layer.span.end_ms - layer.span.start_ms), seconds(layer.span.start_ms)));
                filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(layer.span.start_ms), seconds(layer.span.end_ms)));
                current_video = composited;
            }
            EvaluatedVisualSource::Rectangle {
                color,
                width: layer_width,
                height: layer_height,
            } => {
                visual_count += 1;
                let prepared = format!("visual{visual_count}");
                let composited = format!("base{visual_count}");
                let scale = evaluated_scalar_expression(
                    &layer.keyframes,
                    EvaluatedProperty::Scale,
                    layer.transform.scale,
                    layer.span.start_ms,
                );
                let opacity = evaluated_scalar_expression_for(
                    &layer.keyframes,
                    EvaluatedProperty::Opacity,
                    layer.transform.opacity,
                    layer.span.start_ms,
                    "T",
                );
                let x = evaluated_position_expression(
                    &layer.keyframes,
                    true,
                    layer.transform.position_x,
                    layer.span.start_ms,
                );
                let y = evaluated_position_expression(
                    &layer.keyframes,
                    false,
                    layer.transform.position_y,
                    layer.span.start_ms,
                );
                let transition = evaluated_transition_filters(&layer.transitions);
                filters.push(format!("color=c={}:s={}x{}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]", ffmpeg_color(color), layer_width, layer_height, seconds(layer.span.end_ms - layer.span.start_ms), seconds(layer.span.start_ms)));
                filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(layer.span.start_ms), seconds(layer.span.end_ms)));
                current_video = composited;
            }
            EvaluatedVisualSource::Caption(caption) => {
                visual_count += 1;
                let composited = format!("base{visual_count}");
                let font = default_font_path
                    .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
                    .unwrap_or_default();
                filters.push(format!(
                        "[{current_video}]drawtext={font}text='{}':fontsize={}:fontcolor={}:box=1:boxcolor={}@0.75:boxborderw=12:x='(w-text_w)/2':y='h-text_h-{}':enable='between(t,{},{})'[{composited}]",
                        escape_filter(&caption.text),
                        caption.font_size,
                        caption.color,
                        caption.background_color,
                        caption.bottom_margin_px,
                        seconds(layer.span.start_ms),
                        seconds(layer.span.end_ms)
                    ));
                current_video = composited;
            }
        }
    }
    for audio in &scene.audio_layers {
        append_audio_layer(
            &mut filters,
            &mut audio_labels,
            audio,
            &scene.voiceover_intervals,
            &input_indexes,
        )?;
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
        intent,
        media_inputs,
        media_paths,
    })
}

fn append_audio_layer(
    filters: &mut Vec<String>,
    audio_labels: &mut Vec<String>,
    audio: &EvaluatedAudioLayer,
    voiceover_intervals: &[crate::evaluated_scene::EvaluatedTimeSpan],
    input_indexes: &HashMap<&str, usize>,
) -> Result<(), CoreError> {
    let input = input_indexes.get(audio.item_id.as_str()).ok_or_else(|| {
        CoreError::new(
            ErrorCode::InternalError,
            "renderer input mapping is missing",
        )
    })?;
    let label = format!("audio{}", audio_labels.len());
    let automation =
        evaluated_scalar_expression(&audio.volume_keyframes, EvaluatedProperty::Volume, 1.0, 0);
    let volume = format!("({})*({automation})", format_number(audio.volume));
    let ducking = evaluated_ducking_expression(audio.ducking.as_ref(), voiceover_intervals);
    let duration_ms = audio.span.end_ms - audio.span.start_ms;
    let mut chain = format!(
        "[{input}:a]atrim=duration={},asetpts=PTS-STARTPTS,volume='{volume}':eval=frame",
        seconds(duration_ms),
    );
    if audio.fade_in_ms > 0 {
        chain.push_str(&format!(",afade=t=in:st=0:d={}", seconds(audio.fade_in_ms)));
    }
    if audio.fade_out_ms > 0 && audio.fade_out_ms < duration_ms {
        chain.push_str(&format!(
            ",afade=t=out:st={}:d={}",
            seconds(duration_ms - audio.fade_out_ms),
            seconds(audio.fade_out_ms)
        ));
    }
    chain.push_str(&format!(
        ",asetpts=PTS+{}/TB,volume='{ducking}':eval=frame[{label}]",
        seconds(audio.span.start_ms)
    ));
    filters.push(chain);
    audio_labels.push(format!("[{label}]"));
    Ok(())
}

fn evaluated_transition_filters(transitions: &[EvaluatedTransition]) -> String {
    let mut result = String::new();
    for transition in transitions {
        let direction = match transition.role {
            EvaluatedTransitionRole::In => "in",
            EvaluatedTransitionRole::Out => "out",
        };
        result.push_str(&format!(
            ",fade=t={direction}:st={}:d={}:alpha=1",
            seconds(transition.span.start_ms),
            seconds(transition.span.end_ms - transition.span.start_ms)
        ));
    }
    result
}

fn evaluated_scalar_expression(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    default: f64,
    item_start_ms: u64,
) -> String {
    evaluated_scalar_expression_for(keyframes, property, default, item_start_ms, "t")
}

fn evaluated_scalar_expression_for(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| match (keyframe.property, keyframe.value) {
            (actual, EvaluatedKeyframeValue::Scalar { value }) if actual == property => {
                Some((keyframe.time_ms, value, keyframe.easing))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    evaluated_piecewise_expression_for(&values, default, item_start_ms, time_variable)
}

fn evaluated_position_expression(
    keyframes: &[EvaluatedKeyframe],
    x_axis: bool,
    default: f64,
    item_start_ms: u64,
) -> String {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| match (keyframe.property, keyframe.value) {
            (EvaluatedProperty::Position, EvaluatedKeyframeValue::Position { x, y }) => Some((
                keyframe.time_ms,
                if x_axis { x } else { y },
                keyframe.easing,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    evaluated_piecewise_expression_for(&values, default, item_start_ms, "t")
}

fn evaluated_piecewise_expression_for(
    values: &[(u64, f64, EvaluatedEasing)],
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
        let eased = evaluated_easing_expression(&progress, easing);
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

fn evaluated_easing_expression(progress: &str, easing: EvaluatedEasing) -> String {
    match easing {
        EvaluatedEasing::Hold => "0".into(),
        EvaluatedEasing::Linear => progress.into(),
        EvaluatedEasing::EaseIn => format!("({progress})*({progress})"),
        EvaluatedEasing::EaseOut => format!("1-(1-({progress}))*(1-({progress}))"),
        EvaluatedEasing::EaseInOut => format!(
            "if(lt(({progress}),0.5),2*({progress})*({progress}),1-pow(-2*({progress})+2,2)/2)"
        ),
    }
}

fn evaluated_ducking_expression(
    settings: Option<&EvaluatedDucking>,
    intervals: &[crate::evaluated_scene::EvaluatedTimeSpan],
) -> String {
    let Some(settings) = settings else {
        return "1".into();
    };
    let mut expression = "1".to_owned();
    for interval in intervals {
        let start = interval.start_ms;
        let end = interval.end_ms;
        let attack_start = start.saturating_sub(settings.attack_ms);
        let release_end = end.saturating_add(settings.release_ms);
        let attack = seconds(settings.attack_ms.max(1));
        let release = seconds(settings.release_ms.max(1));
        let gain = format_number(settings.gain);
        let envelope = format!(
            "if(between(t,{},{}),1-(1-({gain}))*((t-{})/{attack}),if(between(t,{},{}),({gain}),if(between(t,{},{}),({gain})+(1-({gain}))*((t-{})/{release}),1)))",
            seconds(attack_start),
            seconds(start),
            seconds(attack_start),
            seconds(start),
            seconds(end),
            seconds(end),
            seconds(release_end),
            seconds(end),
        );
        expression = format!("min({expression},{envelope})");
    }
    expression
}

fn evaluated_anchored_layer_position(
    x: &str,
    y: &str,
    anchor: EvaluatedAnchorPoint,
) -> (String, String) {
    use EvaluatedAnchorPoint::*;
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

fn evaluated_text_layer_padding(anchor: EvaluatedAnchorPoint) -> (&'static str, &'static str) {
    use EvaluatedAnchorPoint::*;
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
pub(crate) fn ffmpeg_color(color: &str) -> String {
    format!("0x{}", color.trim_start_matches('#'))
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn scalar_expression(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    default: f64,
    item_start_ms: u64,
) -> String {
    scalar_expression_for(keyframes, property, default, item_start_ms, "t")
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn piecewise_expression(
    values: &[(u64, f64, Easing)],
    default: f64,
    item_start_ms: u64,
) -> String {
    piecewise_expression_for(values, default, item_start_ms, "t")
}

#[cfg(test)]
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

#[cfg(test)]
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
        Asset, AudioSettings, AudioTrackRole, DuckingSettings, MediaItem, PROJECT_SCHEMA_VERSION,
        ProjectSettings, RectangleItem, SolidColorItem, TextItem, TextStyle, Track, TrackType,
        Transform, TransitionItem, TransitionType, evaluated_scene::evaluate_project,
        render_artifact::media_input_requests,
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
        let evaluated = evaluate_project(&project, 1_920, 1_080, 30).unwrap();
        let first = build_render_plan(
            &evaluated.scene,
            &text,
            vec![],
            vec![],
            None,
            RenderIntent::Export,
            &mut warnings,
        )
        .unwrap();
        let second = build_render_plan(
            &evaluated.scene,
            &text,
            vec![],
            vec![],
            None,
            RenderIntent::Export,
            &mut warnings,
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            (first.width, first.height, first.fps, first.duration_ms),
            (1_920, 1_080, 30, 1)
        );
        assert!(first.filter_graph.contains("[video]"));
        assert!(first.filter_graph.contains("[audio]"));

        let mut intent_plans = Vec::new();
        for intent in [
            RenderIntent::Frame { at_ms: 500 },
            RenderIntent::Range {
                start_ms: 100,
                end_ms: 900,
                include_audio: false,
            },
            RenderIntent::Export,
        ] {
            let plan = build_render_plan(
                &evaluated.scene,
                &text,
                vec![],
                vec![],
                None,
                intent,
                &mut warnings,
            )
            .unwrap();
            assert_eq!(plan.intent, intent);
            intent_plans.push(plan);
        }
        for plan in &mut intent_plans {
            plan.intent = RenderIntent::Export;
        }
        assert!(intent_plans.windows(2).all(|pair| pair[0] == pair[1]));
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
                TimelineItem::Media(MediaItem {
                    id: "video-reuse".into(),
                    asset_id: "video-asset".into(),
                    start_ms: 1_500,
                    duration_ms: 500,
                    source_in_ms: 100,
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
        let scene = evaluate_project(&project, 640, 360, 24).unwrap();
        let media_inputs = media_input_requests(&scene).unwrap();
        assert_eq!(
            media_inputs
                .iter()
                .map(|input| (input.item_id.as_str(), input.input_index))
                .collect::<Vec<_>>(),
            vec![("video", 2), ("audio", 3), ("video-reuse", 4)]
        );
        assert_eq!(media_inputs[0].source_in_ms, 125);
        assert_eq!(media_inputs[1].duration_ms, 3_000);
        assert_eq!(media_inputs[2].asset_id, "video-asset");
        assert_eq!(
            scene
                .scene
                .visual_layers
                .iter()
                .filter(|layer| matches!(layer.source, EvaluatedVisualSource::Text(_)))
                .map(|layer| layer.item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["title"]
        );
        assert_eq!(
            (
                scene.scene.canvas.width,
                scene.scene.canvas.height,
                scene.scene.canvas.fps
            ),
            (640, 360, 24)
        );
        assert_eq!(scene.scene.duration_ms, 3_500);

        let mut inconsistent = scene.clone();
        inconsistent.resource_bindings.media.clear();
        assert_eq!(
            media_input_requests(&inconsistent).unwrap_err().code,
            ErrorCode::InternalError
        );
    }

    #[test]
    fn non_empty_semantic_plan_is_identical_across_render_intents() {
        let mut project = empty_project();
        project.settings = ProjectSettings {
            width: 640,
            height: 360,
            fps: 24,
        };
        project.assets = vec![
            Asset {
                id: "video-asset".into(),
                media_type: MediaType::Video,
                file_name: "video.mp4".into(),
                project_relative_path: "assets/video.mp4".into(),
                duration_ms: Some(4_000),
                has_audio: false,
                origin: None,
                content_hash: None,
                size_bytes: None,
                probe: None,
            },
            Asset {
                id: "voice-asset".into(),
                media_type: MediaType::Audio,
                file_name: "voice.wav".into(),
                project_relative_path: "assets/voice.wav".into(),
                duration_ms: Some(4_000),
                has_audio: true,
                origin: None,
                content_hash: None,
                size_bytes: None,
                probe: None,
            },
            Asset {
                id: "music-asset".into(),
                media_type: MediaType::Audio,
                file_name: "music.wav".into(),
                project_relative_path: "assets/music.wav".into(),
                duration_ms: Some(4_000),
                has_audio: true,
                origin: None,
                content_hash: None,
                size_bytes: None,
                probe: None,
            },
        ];
        project.tracks = vec![
            Track {
                id: "visual".into(),
                name: "Visual".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![
                    TimelineItem::SolidColor(SolidColorItem {
                        id: "background".into(),
                        color: "#112233".into(),
                        start_ms: 0,
                        duration_ms: 4_000,
                        transform: Transform::default(),
                        keyframes: vec![],
                        hidden: false,
                    }),
                    TimelineItem::Rectangle(RectangleItem {
                        id: "panel".into(),
                        color: "#445566".into(),
                        width: 320,
                        height: 180,
                        start_ms: 500,
                        duration_ms: 2_000,
                        transform: Transform {
                            position_x: 20.0,
                            position_y: 30.0,
                            scale: 1.25,
                            opacity: 0.8,
                        },
                        keyframes: vec![Keyframe {
                            property: KeyframeProperty::Position,
                            time_ms: 0,
                            value: KeyframeValue::Position { x: 20.0, y: 30.0 },
                            easing: Easing::EaseInOut,
                        }],
                        hidden: false,
                    }),
                    TimelineItem::Text(TextItem {
                        id: "title".into(),
                        text: "Evaluated title".into(),
                        start_ms: 750,
                        duration_ms: 1_500,
                        font_size: 48,
                        color: "#ffffff".into(),
                        font_family: Some("Deterministic Sans".into()),
                        font_path: Some("fonts/deterministic.ttf".into()),
                        style: TextStyle::default(),
                        transform: Transform::default(),
                        keyframes: vec![Keyframe {
                            property: KeyframeProperty::Opacity,
                            time_ms: 0,
                            value: KeyframeValue::Scalar { value: 0.5 },
                            easing: Easing::Linear,
                        }],
                        hidden: false,
                    }),
                    TimelineItem::Media(MediaItem {
                        id: "video-first".into(),
                        asset_id: "video-asset".into(),
                        start_ms: 0,
                        duration_ms: 2_000,
                        source_in_ms: 0,
                        transform: Transform::default(),
                        audio: AudioSettings::default(),
                        keyframes: vec![],
                        hidden: false,
                    }),
                    TimelineItem::Media(MediaItem {
                        id: "video-reuse".into(),
                        asset_id: "video-asset".into(),
                        start_ms: 2_000,
                        duration_ms: 2_000,
                        source_in_ms: 1_000,
                        transform: Transform::default(),
                        audio: AudioSettings::default(),
                        keyframes: vec![],
                        hidden: false,
                    }),
                    TimelineItem::Transition(TransitionItem {
                        id: "panel-title".into(),
                        transition_type: TransitionType::Crossfade,
                        from_item_id: "panel".into(),
                        to_item_id: Some("title".into()),
                        start_ms: 700,
                        duration_ms: 200,
                        hidden: false,
                    }),
                ],
            },
            Track {
                id: "voice".into(),
                name: "Voice".into(),
                track_type: TrackType::Audio,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Voiceover,
                ducking: None,
                items: vec![TimelineItem::Media(MediaItem {
                    id: "voice-item".into(),
                    asset_id: "voice-asset".into(),
                    start_ms: 1_000,
                    duration_ms: 1_000,
                    source_in_ms: 0,
                    transform: Transform::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                    hidden: false,
                })],
            },
            Track {
                id: "music".into(),
                name: "Music".into(),
                track_type: TrackType::Audio,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Music,
                ducking: Some(DuckingSettings {
                    enabled: true,
                    gain: 0.25,
                    attack_ms: 50,
                    release_ms: 75,
                }),
                items: vec![TimelineItem::Media(MediaItem {
                    id: "music-item".into(),
                    asset_id: "music-asset".into(),
                    start_ms: 0,
                    duration_ms: 4_000,
                    source_in_ms: 0,
                    transform: Transform::default(),
                    audio: AudioSettings {
                        volume: 0.75,
                        fade_in_ms: 100,
                        fade_out_ms: 200,
                        ..AudioSettings::default()
                    },
                    keyframes: vec![Keyframe {
                        property: KeyframeProperty::Volume,
                        time_ms: 0,
                        value: KeyframeValue::Scalar { value: 0.5 },
                        easing: Easing::EaseIn,
                    }],
                    hidden: false,
                })],
            },
        ];

        let evaluated = evaluate_project(&project, 640, 360, 24).unwrap();
        assert_eq!(evaluated.resource_bindings.media.len(), 3);
        assert_eq!(evaluated.resource_bindings.fonts.len(), 1);
        assert_eq!(evaluated.scene.visual_layers.len(), 5);
        assert_eq!(evaluated.scene.audio_layers.len(), 2);
        assert!(evaluated.scene.audio_layers[1].ducking.is_some());
        assert!(!evaluated.scene.visual_layers[1].transitions.is_empty());

        let media_inputs = media_input_requests(&evaluated).unwrap();
        assert_eq!(
            media_inputs
                .iter()
                .filter(|input| input.asset_id == "video-asset")
                .count(),
            2
        );
        let media_paths = media_inputs
            .iter()
            .map(|input| PathBuf::from("/prepared").join(&input.project_relative_path))
            .collect::<Vec<_>>();
        let text = HashMap::from([(
            "title".into(),
            PreparedText {
                file_path: PathBuf::from("/workspace/title.txt"),
                font_path: Some(PathBuf::from("/fonts/deterministic.ttf")),
                layer_width: 300,
                layer_height: 80,
                canvas_width: 300,
                canvas_height: 80,
                text_x: 0,
                text_y: 0,
            },
        )]);
        let mut plans = Vec::new();
        for intent in [
            RenderIntent::Frame { at_ms: 1_500 },
            RenderIntent::Range {
                start_ms: 500,
                end_ms: 3_500,
                include_audio: true,
            },
            RenderIntent::Export,
        ] {
            let mut warnings = Vec::new();
            plans.push(
                build_render_plan(
                    &evaluated.scene,
                    &text,
                    media_inputs.clone(),
                    media_paths.clone(),
                    Some(Path::new("/fonts/deterministic.ttf")),
                    intent,
                    &mut warnings,
                )
                .unwrap(),
            );
        }
        for plan in &mut plans {
            plan.intent = RenderIntent::Export;
        }
        assert!(plans.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn filter_arguments_escape_text_and_paths_without_shell_syntax() {
        assert_eq!(escape_filter("it's: 100%"), "it\\'s\\: 100\\%");
        assert!(!scalar_expression(&[], KeyframeProperty::Scale, 1.0, 0).contains("$"));
        assert_eq!(seconds(1_250), "1.250");
    }
}
