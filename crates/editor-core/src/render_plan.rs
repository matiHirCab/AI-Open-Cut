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

// The process owner consumes this checked planning fact rather than importing
// scene/domain validation. Both source paint and decode use canonical limits.
pub(crate) fn decoded_visual_rgba_bytes(size: (u32, u32)) -> Result<usize, CoreError> {
    crate::evaluated_scene::extended_visual::validate_sampled_source_size(size)
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

pub(crate) struct PreparedTextRun {
    pub(crate) file_path: PathBuf,
    pub(crate) font_path: Option<PathBuf>,
    pub(crate) content: String,
    pub(crate) color: String,
    pub(crate) x: f64,
    pub(crate) y: f64,
}
pub(crate) struct PreparedText {
    pub(crate) rich_runs: Option<Vec<PreparedTextRun>>,
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
    pub(crate) text_layout_fidelity: bool,
    /// FFmpeg 6 shares inherited affine and Bézier expression registers across threads.
    pub(crate) serial_bezier_filters: bool,
    /// Fine procedural marks and styled text need export-quality range encoding for parity.
    pub(crate) detail_fidelity: bool,
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
    if let Some((binding, start_ms)) = &scene.composed_input {
        let input = input_indexes.get(binding.as_str()).ok_or_else(|| {
            CoreError::new(ErrorCode::InternalError, "missing composed visual input")
        })?;
        visual_count += 1;
        filters.push(format!(
            "[{input}:v]fps={fps},settb=AVTB,setpts=PTS-STARTPTS+{}/TB,format=rgba[sampled1]",
            seconds(*start_ms)
        ));
        filters.push("[base0][sampled1]overlay=format=auto:x=0:y=0:eof_action=pass[base1]".into());
        current_video = "base1".into();
    }
    for layer in scene
        .visual_layers
        .iter()
        .filter(|_| scene.composed_input.is_none())
    {
        if let Some((binding, start_ms)) = &layer.sampled_input {
            let input = input_indexes.get(binding.as_str()).ok_or_else(|| {
                CoreError::new(ErrorCode::InternalError, "missing sampled visual input")
            })?;
            visual_count += 1;
            let prepared = format!("sampled{visual_count}");
            let composited = format!("base{visual_count}");
            // Still CPU samples otherwise inherit image2's 25fps cadence,
            // which can defer a half-open overlay start on FFmpeg 6. Match the
            // authored sample cadence before placing it on a precise clock.
            filters.push(format!(
                "[{input}:v]fps={fps},settb=AVTB,setpts=PTS-STARTPTS+{}/TB,format=rgba[{prepared}]",
                seconds(*start_ms)
            ));
            // Prepared pixels already enforce canonical sample activity,
            // including shutter samples. A second floating-time enable can
            // discard a valid seam frame on FFmpeg 6.
            filters.push(format!(
                "[{current_video}][{prepared}]overlay=format=auto:x=0:y=0:eof_action=pass[{composited}]"
            ));
            current_video = composited;
            if binding == "linear-scene" {
                break; // A fully composed opaque sample already contains every evaluated visual.
            }
            continue;
        }
        if let Some(affine) = &layer.affine {
            visual_count += 1;
            let composited = format!("base{visual_count}");
            append_affine_layer(
                &mut filters,
                layer,
                affine,
                scene,
                text_layers,
                &input_indexes,
                (&current_video, &composited, visual_count),
            )?;
            current_video = composited;
            continue;
        }
        if layer.requires_affine() {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "Transform2D geometry was not finalized",
            ));
        }
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
                let sampling_rate = if image_media_needs_scene_cadence(layer, scene) {
                    format!(",fps={fps}")
                } else {
                    String::new()
                };
                filters.push(format!(
                    "[{input}:v]setpts=PTS-STARTPTS+{}/TB{sampling_rate},scale=w='iw*({scale})':h='ih*({scale})':eval=frame,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{fade}[{prepared}]",
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
                let historical = historical_fullscene_synthetic(layer, scene, intent);
                let source = if historical {
                    format!(
                        "color=c={}:s={width}x{height}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB",
                        ffmpeg_color(color),
                        seconds(layer.span.end_ms - layer.span.start_ms),
                        seconds(layer.span.start_ms)
                    )
                } else {
                    let (first, count) =
                        synthetic_frame_window(layer.span.start_ms, layer.span.end_ms, fps)?;
                    format!(
                        "color=c={}:s={width}x{height}:r={fps},trim=end_frame={count},format=rgba,setpts=PTS+{first}",
                        ffmpeg_color(color)
                    )
                };
                filters.push(format!("{source},scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]"));
                let activity = if historical {
                    format!(
                        "between(t,{},{})",
                        seconds(layer.span.start_ms),
                        seconds(layer.span.end_ms)
                    )
                } else {
                    format!(
                        "gte(t,{})*lt(t,{})",
                        seconds(layer.span.start_ms),
                        seconds(layer.span.end_ms)
                    )
                };
                filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='{activity}'[{composited}]"));
                current_video = composited;
            }
            EvaluatedVisualSource::Shape(_) => {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "shape affine preparation missing",
                ));
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
                let historical = historical_fullscene_synthetic(layer, scene, intent);
                let source = if historical {
                    format!(
                        "color=c={}:s={layer_width}x{layer_height}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB",
                        ffmpeg_color(color),
                        seconds(layer.span.end_ms - layer.span.start_ms),
                        seconds(layer.span.start_ms)
                    )
                } else {
                    let (first, count) =
                        synthetic_frame_window(layer.span.start_ms, layer.span.end_ms, fps)?;
                    format!(
                        "color=c={}:s={layer_width}x{layer_height}:r={fps},trim=end_frame={count},format=rgba,setpts=PTS+{first}",
                        ffmpeg_color(color)
                    )
                };
                filters.push(format!("{source},scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]"));
                let activity = if historical {
                    format!(
                        "between(t,{},{})",
                        seconds(layer.span.start_ms),
                        seconds(layer.span.end_ms)
                    )
                } else {
                    format!(
                        "gte(t,{})*lt(t,{})",
                        seconds(layer.span.start_ms),
                        seconds(layer.span.end_ms)
                    )
                };
                filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='{activity}'[{composited}]"));
                current_video = composited;
            }
            EvaluatedVisualSource::Caption(caption) => {
                visual_count += 1;
                let composited = format!("base{visual_count}");
                let draw = caption_drawtext(caption, default_font_path);
                filters.push(format!(
                    "[{current_video}]{draw}:enable='between(t,{},{})'[{composited}]",
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
            scene.instance_voiceover_intervals.as_deref(),
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
        serial_bezier_filters: scene.visual_layers.iter().any(|layer| {
            // Inherited affine expressions and Bezier easing both use mutable
            // expression registers shared by FFmpeg's parallel blend slices.
            (layer.sampled_input.is_none()
                && layer.affine.is_some()
                && layer.has_animated_ancestors())
                || layer.keyframes.iter().any(|keyframe| {
                    matches!(keyframe.easing, EvaluatedEasing::CubicBezier { .. })
                })
        }) || scene.audio_layers.iter().any(|layer| {
            layer.volume_keyframes.iter().any(|keyframe| {
                matches!(keyframe.easing, EvaluatedEasing::CubicBezier { .. })
            })
        }),
        text_layout_fidelity: scene.visual_layers.iter().any(|layer| crate::evaluated_scene::extended_visual::required(layer) || matches!(&layer.source, EvaluatedVisualSource::Text(text) if text.style.layout.is_some())),
        detail_fidelity: scene.visual_layers.iter().any(|layer| {
            matches!(&layer.source, EvaluatedVisualSource::Shape(shape) if shape.grid_descriptor.is_some())
                || matches!(&layer.source, EvaluatedVisualSource::Text(text)
                    if text.spans.is_some() || text.style.paint_layers.is_some())
        }),
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

// Preserve the reviewed zero-origin graph only inside its uncut root scene.
// Endpoints and every mapped/clipped/partial source retain corrected lowering.
fn historical_fullscene_synthetic(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    scene: &EvaluatedScene,
    intent: RenderIntent,
) -> bool {
    if !matches!(
        layer.source,
        EvaluatedVisualSource::SolidColor { .. } | EvaluatedVisualSource::Rectangle { .. }
    ) || layer.requires_affine()
        || layer.instance.is_some()
        || layer.ancestors.is_some()
        || !layer.ancestor_stages.is_empty()
        || layer.sampled_input.is_some()
        || layer.span.start_ms != 0
        || layer.span.end_ms != scene.duration_ms
        || scene.duration_ms == 0
        || scene.canvas.fps == 0
    {
        return false;
    }
    match intent {
        RenderIntent::Frame { at_ms } => {
            at_ms < scene.duration_ms
                && u128::from(at_ms)
                    .checked_mul(u128::from(scene.canvas.fps))
                    .is_some_and(|ticks| ticks.is_multiple_of(1000))
        }
        RenderIntent::Range {
            start_ms, end_ms, ..
        } => {
            start_ms < end_ms
                && end_ms <= scene.duration_ms
                && u128::from(start_ms)
                    .checked_mul(u128::from(scene.canvas.fps))
                    .is_some_and(|ticks| ticks.is_multiple_of(1000))
        }
        RenderIntent::Export => true,
    }
}

// A synthetic source uses the output grid, including its one possible prefix cell.
// Integer frame ticks avoid floating second-to-timebase truncation at split seams.
fn synthetic_frame_window(start_ms: u64, end_ms: u64, fps: u32) -> Result<(i64, i64), CoreError> {
    let invalid = || CoreError::new(ErrorCode::InvalidArgument, "invalid synthetic frame window");
    if fps == 0 || end_ms <= start_ms {
        return Err(invalid());
    }
    let first = u128::from(start_ms)
        .checked_mul(u128::from(fps))
        .ok_or_else(invalid)?
        / 1000;
    let last = u128::from(end_ms)
        .checked_mul(u128::from(fps))
        .and_then(|n| n.checked_add(999))
        .ok_or_else(invalid)?
        / 1000;
    let count = last
        .checked_sub(first)
        .filter(|n| *n > 0)
        .ok_or_else(invalid)?;
    let first = i64::try_from(first).map_err(|_| invalid())?;
    let _last = i64::try_from(last).map_err(|_| invalid())?;
    let count = i64::try_from(count).map_err(|_| invalid())?;
    Ok((first, count))
}

fn append_audio_layer(
    filters: &mut Vec<String>,
    audio_labels: &mut Vec<String>,
    audio: &EvaluatedAudioLayer,
    voiceover_intervals: &[crate::evaluated_scene::EvaluatedTimeSpan],
    precise_intervals: Option<&[(f64, f64)]>,
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
    let volume = if audio
        .volume_keyframes
        .iter()
        .any(|key| key.property == EvaluatedProperty::GainDb)
    {
        let gain =
            evaluated_scalar_expression(&audio.volume_keyframes, EvaluatedProperty::GainDb, 0.0, 0);
        format!(
            "({})*({automation})*pow(10,({gain})/20)",
            format_number(audio.volume)
        )
    } else {
        format!("({})*({automation})", format_number(audio.volume))
    };
    let ducking = if let Some(intervals) = precise_intervals {
        precise_ducking(audio.ducking.as_ref(), intervals)
    } else {
        evaluated_ducking_expression(audio.ducking.as_ref(), voiceover_intervals)
    };
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
    if let Some(clock) = audio.instance {
        chain.push_str(&tempo_filters(clock.rate)?);
        chain.push_str(&format!(
            ",asetpts=PTS+{}/TB,atrim=start={}:end={},asetpts=PTS-STARTPTS,adelay={:.17}:all=1,volume='{ducking}':eval=frame[{label}]",
            precise_seconds(clock.root_ms(audio.span.start_ms)),
            precise_seconds(clock.start_ms),
            precise_seconds(clock.end_ms),
            clock.start_ms
        ));
    } else if audio.retained_timeline_delay {
        // amix consumes sequential samples, so a timestamp shift alone cannot
        // place edited clips. Physical silence also establishes the global
        // clock for ducking after local source animation has been sampled.
        chain.push_str(&format!(
            ",adelay={}:all=1,volume='{ducking}':eval=frame[{label}]",
            audio.span.start_ms
        ));
    } else {
        chain.push_str(&format!(
            ",asetpts=PTS+{}/TB,volume='{ducking}':eval=frame[{label}]",
            seconds(audio.span.start_ms)
        ));
    }
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

#[derive(Clone, Copy)]
enum LoopClockPrecision {
    IntegerMilliseconds,
    FractionalMilliseconds,
}

fn evaluated_scalar_expression_for(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
    evaluated_scalar_expression_with_precision(
        keyframes,
        property,
        default,
        item_start_ms,
        time_variable,
        LoopClockPrecision::IntegerMilliseconds,
    )
}

fn evaluated_fractional_scalar_expression_for(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
    evaluated_scalar_expression_with_precision(
        keyframes,
        property,
        default,
        item_start_ms,
        time_variable,
        LoopClockPrecision::FractionalMilliseconds,
    )
}

fn evaluated_visual_scalar_expression_for(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
    let precision = if layer.instance.is_some() {
        LoopClockPrecision::FractionalMilliseconds
    } else {
        LoopClockPrecision::IntegerMilliseconds
    };
    evaluated_scalar_expression_with_precision(
        keyframes,
        property,
        default,
        item_start_ms,
        time_variable,
        precision,
    )
}

fn evaluated_scalar_expression_with_precision(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
    precision: LoopClockPrecision,
) -> String {
    let time_variable_unshifted = time_variable;
    let source_time = retained_time_expression(keyframes, property, time_variable);
    let time_variable = source_time.as_str();
    let loop_spec = keyframes
        .iter()
        .find(|keyframe| keyframe.property == property)
        .and_then(|keyframe| keyframe.r#loop);
    let values = keyframes
        .iter()
        .filter_map(|keyframe| match (keyframe.property, keyframe.value) {
            (actual, EvaluatedKeyframeValue::Scalar { value }) if actual == property => {
                Some((keyframe.time_ms, value, keyframe.easing))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let bounds = match property {
        EvaluatedProperty::PositionX | EvaluatedProperty::PositionY => {
            Some((-1_000_000.0, 1_000_000.0))
        }
        EvaluatedProperty::ScaleX | EvaluatedProperty::ScaleY => Some((0.000_001, 100.0)),
        EvaluatedProperty::Opacity => Some((0.0, 1.0)),
        EvaluatedProperty::GainDb => Some((-96.0, 12.0)),
        _ => None,
    };
    if values.is_empty() {
        return format_number(default);
    }
    if let Some(clock) = keyframes
        .iter()
        .find(|key| key.property == property)
        .and_then(|key| key.clock)
    {
        return clocked_scalar_expression(
            &values,
            item_start_ms,
            // Keep the unshifted clock: the integer offset cancels against
            // each key origin before any floating-point arithmetic.
            time_variable_unshifted,
            clock.offset_ms,
            loop_spec,
            precision,
            bounds,
        );
    }
    let mapped_time = loop_spec.and_then(|loop_spec| {
        let first = values.first()?.0;
        let last = values.last()?.0;
        let span = last.checked_sub(first)?;
        (span > 0).then(|| {
            looped_time_expression(
                loop_spec,
                item_start_ms,
                first,
                span,
                time_variable,
                precision,
            )
        })
    });
    evaluated_piecewise_expression_for(
        &values,
        default,
        item_start_ms,
        mapped_time.as_deref().unwrap_or(time_variable),
        bounds,
    )
}

fn looped_time_expression(
    loop_spec: crate::AnimationLoop,
    item_start_ms: u64,
    first_ms: u64,
    span_ms: u64,
    time_variable: &str,
    precision: LoopClockPrecision,
) -> String {
    let first = u128::from(item_start_ms) + u128::from(first_ms);
    let span = u128::from(span_ms);
    let multiplier = if loop_spec.mode == crate::AnimationLoopMode::PingPong {
        2
    } else {
        1
    };
    let period = span * multiplier;
    // Persisted clocks and audio retain integer sampling; affine visual clocks
    // must preserve their fractional phase through seams and finite completion.
    let sample_ms = match precision {
        LoopClockPrecision::IntegerMilliseconds => format!("floor((({time_variable})*1000)+0.5)"),
        LoopClockPrecision::FractionalMilliseconds => format!("(({time_variable})*1000)"),
    };
    let phase = format!("mod(max(0,({sample_ms})-({first})),{period})",);
    let offset = if loop_spec.mode == crate::AnimationLoopMode::PingPong {
        format!("if(lte(({phase}),{span}),({phase}),({period})-({phase}))",)
    } else {
        phase
    };
    let active = format!("(({first})+({offset}))/1000");
    let mapped = if let crate::AnimationLoopIterations::Finite(count) = loop_spec.iterations {
        let finish = first + period * u128::from(count);
        let endpoint = if loop_spec.mode == crate::AnimationLoopMode::PingPong {
            first
        } else {
            first + span
        };
        format!("if(gte(({sample_ms}),{finish}),({endpoint})/1000,({active}))",)
    } else {
        active
    };
    format!("if(lt(({sample_ms}),{first}),({time_variable}),({mapped}))",)
}

fn retained_time_expression(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    time_variable: &str,
) -> String {
    match keyframes
        .iter()
        .find(|key| key.property == property)
        .and_then(|key| key.clock)
    {
        Some(clock) if clock.offset_ms != 0 => format!(
            "(({time_variable})+({:.17}))",
            clock.offset_ms as f64 / 1000.0
        ),
        _ => time_variable.to_owned(),
    }
}

fn evaluated_position_expression(
    keyframes: &[EvaluatedKeyframe],
    x_axis: bool,
    default: f64,
    item_start_ms: u64,
) -> String {
    evaluated_position_expression_for(keyframes, x_axis, default, item_start_ms, "t")
}

fn evaluated_position_expression_for(
    keyframes: &[EvaluatedKeyframe],
    x_axis: bool,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
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
    if let Some(clock) = keyframes
        .iter()
        .find(|key| key.property == EvaluatedProperty::Position)
        .and_then(|key| key.clock)
    {
        let local_ms = format!("(({time_variable})*1000-({item_start_ms}))");
        return clocked_piecewise_expression(
            &values,
            default,
            &local_ms,
            i128::from(clock.offset_ms),
            1,
            None,
        );
    }
    let time = retained_time_expression(keyframes, EvaluatedProperty::Position, time_variable);
    evaluated_piecewise_expression_for(&values, default, item_start_ms, &time, None)
}

// The source origin stays integer until it cancels against a selected key.
// This also handles large key origins, not just large retained offsets.
fn clocked_scalar_expression(
    values: &[(u64, f64, EvaluatedEasing)],
    item_start_ms: u64,
    time_variable: &str,
    offset_ms: i64,
    loop_spec: Option<crate::AnimationLoop>,
    precision: LoopClockPrecision,
    bounds: Option<(f64, f64)>,
) -> String {
    let default = values.first().expect("nonempty values").1;
    let local_ms = format!("(({time_variable})*1000-({item_start_ms}))");
    let plain = || {
        clocked_piecewise_expression(values, default, &local_ms, i128::from(offset_ms), 1, bounds)
    };
    let Some(spec) = loop_spec else {
        return plain();
    };
    let Some((first, _, _)) = values.first().copied() else {
        return plain();
    };
    let last = values.last().expect("nonempty values").0;
    let span = i128::from(last - first);
    if span == 0 {
        return plain();
    }
    let period = span
        * if spec.mode == crate::AnimationLoopMode::PingPong {
            2
        } else {
            1
        };
    let sample_ms = match precision {
        LoopClockPrecision::IntegerMilliseconds => {
            format!("(floor((({time_variable})*1000)+0.5)-({item_start_ms}))")
        }
        LoopClockPrecision::FractionalMilliseconds => local_ms,
    };
    let raw_phase = format!("mod(({sample_ms}),{period})");
    let local_phase = format!("if(lt(({raw_phase}),0),({raw_phase})+({period}),({raw_phase}))");
    let phase_offset = (i128::from(offset_ms) - i128::from(first)).rem_euclid(period);
    let branch = |phase_origin: i128| {
        let forward = clocked_piecewise_expression(
            values,
            default,
            &local_phase,
            i128::from(first) + phase_origin,
            1,
            bounds,
        );
        if spec.mode == crate::AnimationLoopMode::Repeat {
            return forward;
        }
        let reverse = clocked_piecewise_expression(
            values,
            default,
            &local_phase,
            i128::from(first) + period - phase_origin,
            -1,
            bounds,
        );
        format!(
            "if(lte(({local_phase}),{}),({forward}),({reverse}))",
            span - phase_origin
        )
    };
    let active = format!(
        "if(gte(({local_phase}),{}),({}),({}))",
        period - phase_offset,
        branch(phase_offset - period),
        branch(phase_offset)
    );
    let mapped = if let crate::AnimationLoopIterations::Finite(count) = spec.iterations {
        let remaining = i128::from(first) + period * i128::from(count) - i128::from(offset_ms);
        let endpoint = if spec.mode == crate::AnimationLoopMode::PingPong {
            values[0].1
        } else {
            values.last().expect("nonempty values").1
        };
        let endpoint = if spec.mode == crate::AnimationLoopMode::PingPong {
            if parameterized_easing(values[0].2) {
                format_curve_number(endpoint)
            } else {
                format_number(endpoint)
            }
        } else if values.len() > 1 && parameterized_easing(values[values.len() - 2].2) {
            format_curve_number(endpoint)
        } else {
            format_number(endpoint)
        };
        format!("if(gte(({sample_ms}),{remaining}),({endpoint}),({active}))")
    } else {
        active
    };
    let before_first = i128::from(first) - i128::from(offset_ms);
    let first_value = if parameterized_easing(values[0].2) {
        format_curve_number(values[0].1)
    } else {
        format_number(values[0].1)
    };
    format!("if(lt(({sample_ms}),{before_first}),({first_value}),({mapped}))")
}

fn clocked_piecewise_expression(
    values: &[(u64, f64, EvaluatedEasing)],
    default: f64,
    sample_ms: &str,
    origin_ms: i128,
    direction: i8,
    bounds: Option<(f64, f64)>,
) -> String {
    let Some(last) = values.last() else {
        return format_number(default);
    };
    let difference = |key: u64| {
        format!(
            "(({direction})*({sample_ms})+({}))",
            origin_ms - i128::from(key)
        )
    };
    let mut expression = if values.len() > 1 && parameterized_easing(values[values.len() - 2].2) {
        format_curve_number(last.1)
    } else {
        format_number(last.1)
    };
    for pair in values.windows(2).rev() {
        let (start, start_value, easing) = pair[0];
        let (end, end_value, _) = pair[1];
        let start_delta = difference(start);
        let end_delta = difference(end);
        let progress = format!("({start_delta})/({})", (end - start).max(1));
        let eased = evaluated_easing_expression(&progress, easing);
        let format_value: fn(f64) -> String = if parameterized_easing(easing) {
            format_curve_number
        } else {
            format_number
        };
        let mut interpolated = format!(
            "({})+(({})-({}))*({eased})",
            format_value(start_value),
            format_value(end_value),
            format_value(start_value)
        );
        if parameterized_easing(easing)
            && let Some((minimum, maximum)) = bounds
        {
            interpolated = format!(
                "if(eq(({start_delta}),0),{},max({},min({},({interpolated}))))",
                format_value(start_value),
                format_number(minimum),
                format_number(maximum)
            );
        }
        if parameterized_easing(easing) {
            expression = format!(
                "if(eq(({end_delta}),0),{},({expression}))",
                format_curve_number(end_value)
            );
        }
        expression = format!("if(lt(({end_delta}),0),({interpolated}),({expression}))");
    }
    let first_value = if parameterized_easing(values[0].2) {
        format_curve_number(values[0].1)
    } else {
        format_number(values[0].1)
    };
    format!(
        "if(lt(({}),0),({first_value}),({expression}))",
        difference(values[0].0)
    )
}

fn evaluated_piecewise_expression_for(
    values: &[(u64, f64, EvaluatedEasing)],
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
    bounds: Option<(f64, f64)>,
) -> String {
    if values.is_empty() {
        return format_number(default);
    }
    let last = values.last().expect("nonempty keyframes");
    let mut expression = if values.len() > 1 && parameterized_easing(values[values.len() - 2].2) {
        format_curve_number(last.1)
    } else {
        format_number(last.1)
    };
    for pair in values.windows(2).rev() {
        let (start_time, start_value, easing) = pair[0];
        let (end_time, end_value, _) = pair[1];
        let global_start = seconds(item_start_ms.saturating_add(start_time));
        let global_end = seconds(item_start_ms.saturating_add(end_time));
        let span = seconds(end_time.saturating_sub(start_time).max(1));
        let progress = format!("(({time_variable})-({global_start}))/({span})");
        let eased = evaluated_easing_expression(&progress, easing);
        let format_value: fn(f64) -> String = if parameterized_easing(easing) {
            format_curve_number
        } else {
            format_number
        };
        let mut interpolated = format!(
            "({})+(({})-({}))*({eased})",
            format_value(start_value),
            format_value(end_value),
            format_value(start_value)
        );
        if matches!(
            easing,
            EvaluatedEasing::CubicBezier { .. } | EvaluatedEasing::Spring { .. }
        ) && let Some((minimum, maximum)) = bounds
        {
            interpolated = format!(
                "if(eq(({time_variable}),({global_start})),{},max({},min({},({interpolated}))))",
                format_value(start_value),
                format_number(minimum),
                format_number(maximum)
            );
        }
        if parameterized_easing(easing) {
            expression = format!(
                "if(eq(({time_variable}),({global_end})),{},({expression}))",
                format_curve_number(end_value)
            );
        }
        expression =
            format!("if(lt(({time_variable}),({global_end})),{interpolated},{expression})");
    }
    let first_time = seconds(item_start_ms.saturating_add(values[0].0));
    format!(
        "if(lt(({time_variable}),({first_time})),({}),{expression})",
        if parameterized_easing(values[0].2) {
            format_curve_number(values[0].1)
        } else {
            format_number(values[0].1)
        }
    )
}

fn parameterized_easing(easing: EvaluatedEasing) -> bool {
    matches!(
        easing,
        EvaluatedEasing::CubicBezier { .. } | EvaluatedEasing::Spring { .. }
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
        EvaluatedEasing::CubicBezier { x1, y1, x2, y2 } => {
            let midpoint = "(ld(0)+ld(1))/2";
            let x = cubic_bezier_expression("ld(2)", x1, x2);
            let y = cubic_bezier_expression(midpoint, y1, y2);
            let steps = format!(
                "st(0,0);st(1,1);st(3,0);while(lt(ld(3),40),st(2,{midpoint})+0*if(lte(({x}),({progress})),st(0,ld(2)),st(1,ld(2)))+st(3,ld(3)+1));{y}"
            );
            format!("if(lte(({progress}),0),0,if(gte(({progress}),1),1,({steps})))")
        }
        EvaluatedEasing::Spring {
            mass,
            stiffness,
            damping,
            initial_velocity,
        } => {
            use crate::animation::{SpringCoefficients, spring_coefficients};
            let displacement = match spring_coefficients(mass, stiffness, damping, initial_velocity)
            {
                SpringCoefficients::Underdamped {
                    decay,
                    frequency,
                    sine,
                } => format!(
                    "exp(-{}*({progress}))*(-cos({}*({progress}))+{}*sin({}*({progress})))",
                    format_curve_number(decay),
                    format_curve_number(frequency),
                    format_curve_number(sine),
                    format_curve_number(frequency)
                ),
                SpringCoefficients::Critical { decay, linear } => format!(
                    "exp(-{}*({progress}))*(-1+{}*({progress}))",
                    format_curve_number(decay),
                    format_curve_number(linear)
                ),
                SpringCoefficients::Overdamped {
                    slow_root,
                    fast_root,
                    slow,
                    fast,
                } => format!(
                    "{}*exp({}*({progress}))+{}*exp({}*({progress}))",
                    format_curve_number(slow),
                    format_curve_number(slow_root),
                    format_curve_number(fast),
                    format_curve_number(fast_root)
                ),
            };
            format!("if(lte(({progress}),0),0,if(gte(({progress}),1),1,1+({displacement})))")
        }
    }
}

fn cubic_bezier_expression(time: &str, first: f64, second: f64) -> String {
    let first = format_curve_number(first);
    let second = format_curve_number(second);
    format!(
        "3*(1-({time}))*(1-({time}))*({time})*{first}+3*(1-({time}))*({time})*({time})*{second}+({time})*({time})*({time})"
    )
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

fn format_curve_number(value: f64) -> String {
    format!("{value:.17}")
}

/// Preserve the existing direct Caption paint and literal escaping verbatim.
fn caption_drawtext(
    caption: &crate::evaluated_scene::EvaluatedCaption,
    font_path: Option<&Path>,
) -> String {
    let font = font_path
        .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
        .unwrap_or_default();
    format!(
        "drawtext={font}text='{}':fontsize={}:fontcolor={}:box=1:boxcolor={}@0.75:boxborderw=12:x='(w-text_w)/2':y='h-text_h-{}'",
        escape_filter(&caption.text),
        caption.font_size,
        caption.color,
        caption.background_color,
        caption.bottom_margin_px
    )
}

fn affine_caption_source(
    caption: &crate::evaluated_scene::EvaluatedCaption,
    prepared: &PreparedText,
    size: (u32, u32),
    fps: u32,
    duration: &str,
) -> String {
    let font = prepared
        .font_path
        .as_ref()
        .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
        .unwrap_or_default();
    format!(
        "color=c={}@0.75:s={}x{}:r={fps}:d={duration},format=rgba,drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:x=12:y=12",
        caption.background_color,
        size.0,
        size.1,
        escape_filter_path(&prepared.file_path),
        caption.font_size,
        caption.color
    )
}

pub(crate) fn text_raster_source(
    text: &crate::evaluated_scene::EvaluatedText,
    prepared: &PreparedText,
    size: (u32, u32),
) -> String {
    let (sw, sh) = size;
    let fps = 1;
    let duration = "1";
    if let Some(runs) = &prepared.rich_runs {
        let mut source = format!(
            "color=c={}@{}:s={sw}x{sh}:r={fps}:d={duration},format=rgba",
            text.style.background_color, text.style.background_opacity
        );
        for run in runs {
            let font = run
                .font_path
                .as_ref()
                .map(|p| format!("fontfile='{}':", escape_filter_path(p)))
                .unwrap_or_default();
            source.push_str(&format!(",drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:borderw={}:bordercolor={}:shadowx={}:shadowy={}:shadowcolor={}@{}:x={:.17}:y={:.17}",
                        escape_filter_path(&run.file_path),text.font_size,run.color,text.style.outline_width_px,text.style.outline_color,
                        text.style.shadow.offset_x,text.style.shadow.offset_y,text.style.shadow.color,text.style.shadow.opacity,run.x,run.y));
        }
        source
    } else {
        let font = prepared
            .font_path
            .as_ref()
            .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
            .unwrap_or_default();
        let padding = &text.style.padding;
        let alignment = match text.style.alignment {
            EvaluatedTextAlignment::Left => "L",
            EvaluatedTextAlignment::Center => "C",
            EvaluatedTextAlignment::Right => "R",
        };
        format!(
            "color=c=black@0:s={sw}x{sh}:r={fps}:d={duration},format=rgba,drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:borderw={}:bordercolor={}:shadowx={}:shadowy={}:shadowcolor={}@{}:box=1:boxcolor={}@{}:boxborderw={}|{}|{}|{}:line_spacing={}:text_align={alignment}:x={}:y={}",
            escape_filter_path(&prepared.file_path),
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
            padding.top,
            padding.right,
            padding.bottom,
            padding.left,
            text.style.line_spacing_px,
            prepared.text_x,
            prepared.text_y
        )
    }
}

/// A source-equivalent static bitmap; animation and activity are sampled elsewhere.
pub(crate) fn caption_raster_source(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    prepared: Option<&PreparedText>,
    default_font_path: Option<&Path>,
    size: (u32, u32),
) -> Result<String, CoreError> {
    let EvaluatedVisualSource::Caption(caption) = &layer.source else {
        return Err(CoreError::new(
            ErrorCode::InternalError,
            "caption raster source kind mismatch",
        ));
    };
    if layer.requires_affine() {
        let prepared = prepared
            .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "missing affine caption"))?;
        Ok(affine_caption_source(caption, prepared, size, 1, "1"))
    } else {
        Ok(format!(
            "color=c=black@0:s={}x{}:r=1:d=1,format=rgba,{}",
            size.0,
            size.1,
            caption_drawtext(caption, default_font_path)
        ))
    }
}

fn affine_shape_needs_scene_cadence(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    fps: u32,
) -> bool {
    if layer.has_animated_ancestors()
        || !layer.transitions.is_empty()
        || layer.keyframes.iter().enumerate().any(|(index, key)| {
            layer.keyframes[..index]
                .iter()
                .any(|previous| previous.property == key.property)
        })
    {
        return true;
    }
    if let Some(clock) = layer.instance {
        [clock.start_ms, clock.end_ms].into_iter().any(|root_ms| {
            let tick = root_ms * f64::from(fps) / 1000.0;
            !tick.is_finite() || tick.fract() != 0.0
        })
    } else {
        let span = layer.visible_span();
        [span.start_ms, span.end_ms]
            .into_iter()
            .any(|root_ms| !(u128::from(root_ms) * u128::from(fps)).is_multiple_of(1000))
    }
}

fn image_media_needs_scene_cadence(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    scene: &EvaluatedScene,
) -> bool {
    let EvaluatedVisualSource::Media { asset_id, .. } = &layer.source else {
        return false;
    };
    scene.resources.iter().any(|resource| {
        resource.asset_id == *asset_id
            && resource.kind == crate::evaluated_scene::EvaluatedMediaKind::Image
    }) && affine_shape_needs_scene_cadence(layer, scene.canvas.fps)
}

fn append_affine_layer(
    filters: &mut Vec<String>,
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    affine: &crate::evaluated_scene::EvaluatedAffine,
    scene: &EvaluatedScene,
    text_layers: &HashMap<String, PreparedText>,
    input_indexes: &HashMap<&str, usize>,
    destination: (&str, &str, usize),
) -> Result<(), CoreError> {
    let (base, output, index) = destination;
    let (sw, sh) = layer
        .source_size
        .ok_or_else(|| CoreError::new(ErrorCode::InvalidArgument, "missing affine source size"))?;
    let fps = scene.canvas.fps;
    let duration = seconds(scene.duration_ms);
    let label = format!("affine{index}");
    let source = match &layer.source {
        EvaluatedVisualSource::Media { .. } => {
            let input = input_indexes.get(layer.item_id.as_str()).ok_or_else(|| {
                CoreError::new(ErrorCode::InternalError, "missing affine media input")
            })?;
            let rate = layer.instance.map_or(1.0, |c| c.rate);
            let start = layer.instance.map_or(layer.span.start_ms as f64, |c| {
                c.root_ms(layer.span.start_ms)
            });
            let sampling_rate = if layer.has_animated_ancestors()
                || image_media_needs_scene_cadence(layer, scene)
            {
                format!(",fps={fps}")
            } else {
                String::new()
            };
            format!(
                "[{input}:v]setpts=(PTS-STARTPTS)/{rate:.17}+{}/TB{sampling_rate},format=rgba",
                precise_seconds(start)
            )
        }
        EvaluatedVisualSource::Shape(_) => {
            let input = input_indexes
                .get(layer.item_id.as_str())
                .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "missing shape input"))?;
            if affine_shape_needs_scene_cadence(layer, fps) {
                format!("[{input}:v]fps={fps},setpts=PTS-STARTPTS,format=rgba")
            } else {
                format!("[{input}:v]setpts=PTS-STARTPTS,format=rgba")
            }
        }
        EvaluatedVisualSource::SolidColor { color }
        | EvaluatedVisualSource::Rectangle { color, .. } => format!(
            "color=c={}:s={sw}x{sh}:r={fps}:d={duration},format=rgba",
            ffmpeg_color(color)
        ),
        EvaluatedVisualSource::Text(text) => {
            let prepared = text_layers
                .get(&layer.item_id)
                .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "missing affine text"))?;
            if text.shaped.is_some() {
                let input = input_indexes.get(layer.item_id.as_str()).ok_or_else(|| {
                    CoreError::new(ErrorCode::InternalError, "missing shaped glyph input")
                })?;
                format!("[{input}:v]fps={fps},setpts=PTS-STARTPTS,format=rgba")
            } else if let Some(runs) = &prepared.rich_runs {
                let mut source = format!(
                    "color=c={}@{}:s={sw}x{sh}:r={fps}:d={duration},format=rgba",
                    text.style.background_color, text.style.background_opacity
                );
                for run in runs {
                    let font = run
                        .font_path
                        .as_ref()
                        .map(|p| format!("fontfile='{}':", escape_filter_path(p)))
                        .unwrap_or_default();
                    source.push_str(&format!(",drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:borderw={}:bordercolor={}:shadowx={}:shadowy={}:shadowcolor={}@{}:x={:.17}:y={:.17}",
                        escape_filter_path(&run.file_path),text.font_size,run.color,text.style.outline_width_px,text.style.outline_color,
                        text.style.shadow.offset_x,text.style.shadow.offset_y,text.style.shadow.color,text.style.shadow.opacity,run.x,run.y));
                }
                source
            } else {
                let font = prepared
                    .font_path
                    .as_ref()
                    .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
                    .unwrap_or_default();
                let padding = &text.style.padding;
                let alignment = match text.style.alignment {
                    EvaluatedTextAlignment::Left => "L",
                    EvaluatedTextAlignment::Center => "C",
                    EvaluatedTextAlignment::Right => "R",
                };
                format!(
                    "color=c=black@0:s={sw}x{sh}:r={fps}:d={duration},format=rgba,drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:borderw={}:bordercolor={}:shadowx={}:shadowy={}:shadowcolor={}@{}:box=1:boxcolor={}@{}:boxborderw={}|{}|{}|{}:line_spacing={}:text_align={alignment}:x={}:y={}",
                    escape_filter_path(&prepared.file_path),
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
                    padding.top,
                    padding.right,
                    padding.bottom,
                    padding.left,
                    text.style.line_spacing_px,
                    prepared.text_x,
                    prepared.text_y
                )
            }
        }
        EvaluatedVisualSource::Caption(caption) => {
            let prepared = text_layers.get(&layer.item_id).ok_or_else(|| {
                CoreError::new(ErrorCode::InternalError, "missing affine caption")
            })?;
            affine_caption_source(caption, prepared, (sw, sh), fps, &duration)
        }
    };
    if let Some(tiles) = &layer.sampling_tiles
        && tiles.len() > 1
    {
        let labels = (0..tiles.len())
            .map(|n| format!("[{label}input{n}]"))
            .collect::<String>();
        filters.push(format!("{source},split={}{labels}", tiles.len()));
        let mut previous = base.to_string();
        for (n, tile) in tiles.iter().enumerate() {
            let next = if n + 1 == tiles.len() {
                output.to_string()
            } else {
                format!("{label}base{n}")
            };
            append_affine_samples(
                filters,
                layer,
                tile,
                scene,
                &format!("[{label}input{n}]"),
                (&previous, &next, &format!("{label}tile{n}")),
            )?;
            previous = next;
        }
        return Ok(());
    }
    append_affine_samples(
        filters,
        layer,
        affine,
        scene,
        &source,
        (base, output, &label),
    )
}

fn animated_ancestor_coordinates(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    affine: &crate::evaluated_scene::EvaluatedAffine,
    source: (u32, u32),
    local_time: &str,
) -> (String, String) {
    // Cubic-curve evaluation uses registers 0..3. Keep coordinates in 4..7,
    // making expression size linear even when static rotations alternate with motion.
    let mut steps = vec![
        format!("st(4,X+{:.17}+0.5)", affine.left),
        format!("st(5,Y+{:.17}+0.5)", affine.top),
    ];
    for stage in &layer.ancestor_stages {
        let (x, y) = if let Some(animation) = &stage.animation {
            let time = format!(
                "(T*{:.17}+{:.17})",
                animation.clock.rate,
                animation.clock.offset / 1000.0
            );
            let scalar = |property, default| {
                evaluated_fractional_scalar_expression_for(
                    &animation.keyframes,
                    property,
                    default,
                    animation.start_ms,
                    &time,
                )
            };
            let px = scalar(EvaluatedProperty::PositionX, animation.transform.position_x);
            let py = scalar(EvaluatedProperty::PositionY, animation.transform.position_y);
            let sx = scalar(EvaluatedProperty::ScaleX, animation.transform.scale);
            let sy = scalar(EvaluatedProperty::ScaleY, animation.transform.scale);
            (
                format!("(ld(4)-({px}))/({sx})"),
                format!("(ld(5)-({py}))/({sy})"),
            )
        } else {
            let [a, b, c, d, x, y] = stage.inverse;
            (
                format!("{a:.17}*ld(4)+{c:.17}*ld(5)+{x:.17}"),
                format!("{b:.17}*ld(4)+{d:.17}*ld(5)+{y:.17}"),
            )
        };
        steps.extend([
            format!("st(6,{x})"),
            format!("st(7,{y})"),
            "st(4,ld(6))".into(),
            "st(5,ld(7))".into(),
        ]);
    }
    let (x, y) = if layer.has_local_animated_geometry() {
        let position = |x_axis, default| {
            let property = if x_axis {
                EvaluatedProperty::PositionX
            } else {
                EvaluatedProperty::PositionY
            };
            if layer.keyframes.iter().any(|key| key.property == property) {
                evaluated_fractional_scalar_expression_for(
                    &layer.keyframes,
                    property,
                    default,
                    layer.span.start_ms,
                    local_time,
                )
            } else {
                evaluated_position_expression_for(
                    &layer.keyframes,
                    x_axis,
                    default,
                    layer.span.start_ms,
                    local_time,
                )
            }
        };
        let scale = |axis| {
            let property = if layer.keyframes.iter().any(|key| key.property == axis) {
                axis
            } else {
                EvaluatedProperty::Scale
            };
            evaluated_fractional_scalar_expression_for(
                &layer.keyframes,
                property,
                layer.transform.scale,
                layer.span.start_ms,
                local_time,
            )
        };
        let px = position(true, layer.transform.position_x);
        let py = position(false, layer.transform.position_y);
        let sx = scale(EvaluatedProperty::ScaleX);
        let sy = scale(EvaluatedProperty::ScaleY);
        let (ax, ay) = layer.legacy_anchor(source);
        let density = match &layer.source {
            EvaluatedVisualSource::Shape(shape) => shape.density,
            _ => 1.0,
        };
        (
            format!("((ld(4)-({px}))/({sx})+{ax:.17})*{density:.17}-0.5"),
            format!("((ld(5)-({py}))/({sy})+{ay:.17})*{density:.17}-0.5"),
        )
    } else {
        let parent = layer.ancestors.expect("animated ancestry has a parent");
        let [a, b, c, d, x, y] =
            crate::evaluated_scene::multiply_matrix(affine.inverse, parent.matrix);
        (
            format!("{a:.17}*ld(4)+{c:.17}*ld(5)+{x:.17}-0.5"),
            format!("{b:.17}*ld(4)+{d:.17}*ld(5)+{y:.17}-0.5"),
        )
    };
    let sequence = steps.join(";");
    (format!("({sequence};{x})"), format!("({sequence};{y})"))
}

fn animated_ancestor_opacity(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    local_time: &str,
) -> String {
    let mut factors = layer
        .ancestor_stages
        .iter()
        .map(|stage| {
            if let Some(animation) = &stage.animation {
                let time = format!(
                    "(T*{:.17}+{:.17})",
                    animation.clock.rate,
                    animation.clock.offset / 1000.0
                );
                evaluated_fractional_scalar_expression_for(
                    &animation.keyframes,
                    EvaluatedProperty::Opacity,
                    animation.transform.opacity,
                    animation.start_ms,
                    &time,
                )
            } else {
                format!("{:.17}", stage.opacity)
            }
        })
        .collect::<Vec<_>>();
    factors.push(if let Some(transform) = layer.transform2d {
        format!("{:.17}", transform.opacity)
    } else {
        evaluated_fractional_scalar_expression_for(
            &layer.keyframes,
            EvaluatedProperty::Opacity,
            layer.transform.opacity,
            layer.span.start_ms,
            local_time,
        )
    });
    factors
        .into_iter()
        .map(|factor| format!("({factor})"))
        .collect::<Vec<_>>()
        .join("*")
}

fn append_affine_samples(
    filters: &mut Vec<String>,
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    affine: &crate::evaluated_scene::EvaluatedAffine,
    scene: &EvaluatedScene,
    source: &str,
    destination: (&str, &str, &str),
) -> Result<(), CoreError> {
    let (base, output, label) = destination;
    let (sw, sh) = layer
        .source_size
        .ok_or_else(|| CoreError::new(ErrorCode::InvalidArgument, "missing affine source size"))?;
    let fps = scene.canvas.fps;
    let local_time = layer.instance.map_or_else(
        || "T".to_owned(),
        |c| format!("(T*{:.17}+{:.17})", c.rate, c.offset / 1000.0),
    );
    let duration = seconds(scene.duration_ms);
    // Four nearest-neighbor gathers followed by separable bilinear interpolation.
    // Each map has exactly the validated output dimensions, so a rotated thin
    // source never requires a square pad enclosing both source and destination.
    let (x, y) = if layer.has_animated_ancestors() {
        animated_ancestor_coordinates(layer, affine, (sw, sh), &local_time)
    } else if let Some(parent) = layer
        .ancestors
        .filter(|_| layer.has_local_animated_geometry())
    {
        let position = |x_axis, default| {
            let typed_property = if x_axis {
                EvaluatedProperty::PositionX
            } else {
                EvaluatedProperty::PositionY
            };
            if layer
                .keyframes
                .iter()
                .any(|key| key.property == typed_property)
            {
                return evaluated_visual_scalar_expression_for(
                    layer,
                    &layer.keyframes,
                    typed_property,
                    default,
                    layer.span.start_ms,
                    &local_time,
                );
            }
            evaluated_position_expression_for(
                &layer.keyframes,
                x_axis,
                default,
                layer.span.start_ms,
                &local_time,
            )
        };
        let px = position(true, layer.transform.position_x);
        let py = position(false, layer.transform.position_y);
        let scale_axis = |typed_property| {
            let property = if layer
                .keyframes
                .iter()
                .any(|key| key.property == typed_property)
            {
                typed_property
            } else {
                EvaluatedProperty::Scale
            };
            evaluated_visual_scalar_expression_for(
                layer,
                &layer.keyframes,
                property,
                layer.transform.scale,
                layer.span.start_ms,
                &local_time,
            )
        };
        let scale_x = scale_axis(EvaluatedProperty::ScaleX);
        let scale_y = scale_axis(EvaluatedProperty::ScaleY);
        let [a, b, c, d, tx, ty] = parent.inverse;
        let (anchor_x, anchor_y) = layer.legacy_anchor((sw, sh));
        let density = match &layer.source {
            EvaluatedVisualSource::Shape(shape) => shape.density,
            _ => 1.0,
        };
        (
            format!(
                "((({a:.17}*(X+{:.17}+0.5)+{c:.17}*(Y+{:.17}+0.5)+{tx:.17}-({px}))/({scale_x})+{anchor_x:.17})*{density:.17}-0.5)",
                affine.left, affine.top
            ),
            format!(
                "((({b:.17}*(X+{:.17}+0.5)+{d:.17}*(Y+{:.17}+0.5)+{ty:.17}-({py}))/({scale_y})+{anchor_y:.17})*{density:.17}-0.5)",
                affine.left, affine.top
            ),
        )
    } else {
        let [a, b, c, d, tx, ty] = affine.inverse;
        (
            format!(
                "({a:.17}*(X+{:.17}+0.5)+{c:.17}*(Y+{:.17}+0.5)+{tx:.17}-0.5)",
                affine.left, affine.top
            ),
            format!(
                "({b:.17}*(X+{:.17}+0.5)+{d:.17}*(Y+{:.17}+0.5)+{ty:.17}-0.5)",
                affine.left, affine.top
            ),
        )
    };
    let fx = format!("({x}-floor({x}))");
    let fy = format!("({y}-floor({y}))");
    let source_separator = if source.ends_with(']') { "" } else { "," };
    // Rectangle and vector raster borders must survive pointwise alpha
    // conversion. FFmpeg 6 bilinear geq copies the inner neighbor at an edge.
    // Geometry still uses the four gathers and bilinear blends below; retain
    // the existing media/text lookup for exact legacy orientation parity.
    let channel_lookup = if matches!(
        &layer.source,
        EvaluatedVisualSource::Shape(_) | EvaluatedVisualSource::Rectangle { .. }
    ) {
        "interpolation=nearest:"
    } else {
        ""
    };
    filters.push(format!("{source}{source_separator}format=gbrap,geq={channel_lookup}r='r(X,Y)*alpha(X,Y)/255':g='g(X,Y)*alpha(X,Y)/255':b='b(X,Y)*alpha(X,Y)/255':a='alpha(X,Y)',format=rgba,split=4[{label}s0][{label}s1][{label}s2][{label}s3]"));
    for (n, (dx, dy)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
        let sx = format!("floor({x})+{dx}");
        let sy = format!("floor({y})+{dy}");
        let valid = format!("between({sx},0,{})*between({sy},0,{})", sw - 1, sh - 1);
        for (axis, coordinate) in [("x", sx), ("y", sy)] {
            filters.push(format!("nullsrc=s={}x{}:r={fps}:d={duration},format=gray16le,geq=lum='if({valid},{coordinate},65535)'[{label}{axis}{n}]",affine.width,affine.height));
        }
        filters.push(format!(
            "[{label}s{n}][{label}x{n}][{label}y{n}]remap=fill=black@0,format=gbrap[{label}p{n}]"
        ));
    }
    filters.push(format!(
        "[{label}p0][{label}p1]blend=all_expr='A*(1-{fx})+B*{fx}'[{label}row0]"
    ));
    filters.push(format!(
        "[{label}p2][{label}p3]blend=all_expr='A*(1-{fx})+B*{fx}'[{label}row1]"
    ));
    let opacity = if layer.has_animated_ancestors() {
        animated_ancestor_opacity(layer, &local_time)
    } else if let Some(parent) = layer.ancestors.filter(|_| layer.transform2d.is_none()) {
        format!(
            "({:.17})*({})",
            parent.opacity,
            evaluated_visual_scalar_expression_for(
                layer,
                &layer.keyframes,
                EvaluatedProperty::Opacity,
                layer.transform.opacity,
                layer.span.start_ms,
                &local_time
            )
        )
    } else {
        format!("{:.17}", affine.opacity)
    };
    let fade = if layer.instance.is_some() {
        String::new()
    } else {
        evaluated_transition_filters(&layer.transitions)
    };
    let opacity = inherited_transition_opacity(layer, opacity);
    filters.push(format!("[{label}row0][{label}row1]blend=all_expr='A*(1-{fy})+B*{fy}',geq={channel_lookup}r='if(gt(alpha(X,Y),0),r(X,Y)*255/alpha(X,Y),0)':g='if(gt(alpha(X,Y),0),g(X,Y)*255/alpha(X,Y),0)':b='if(gt(alpha(X,Y),0),b(X,Y)*255/alpha(X,Y),0)':a='alpha(X,Y)*({opacity})',format=rgba{fade}[{label}]"));
    filters.push(format!("[{base}][{label}]overlay=x={:.0}:y={:.0}:format=auto:enable='gte(t,{})*lt(t,{})'[{output}]",
        affine.left,affine.top,precise_seconds(layer.instance.map_or(layer.visible_span().start_ms as f64, |c|c.start_ms)),precise_seconds(layer.instance.map_or(layer.visible_span().end_ms as f64, |c|c.end_ms))));
    Ok(())
}

fn inherited_transition_opacity(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
    opacity: String,
) -> String {
    if let Some(clock) = layer.instance {
        let mut expression = opacity;
        for transition in &layer.transitions {
            let start = clock.root_ms(transition.span.start_ms) / 1000.0;
            let end = clock.root_ms(transition.span.end_ms) / 1000.0;
            let progress = format!("clip((T-({start:.17}))/({:.17}),0,1)", end - start);
            let gain = if transition.role == EvaluatedTransitionRole::In {
                progress
            } else {
                format!("(1-({progress}))")
            };
            expression = format!("({expression})*({gain})");
        }
        expression
    } else {
        opacity
    }
}

fn precise_seconds(milliseconds: f64) -> String {
    format!("{:.17}", milliseconds / 1000.0)
}
fn tempo_filters(mut rate: f64) -> Result<String, CoreError> {
    if !rate.is_finite() || !(2f64.powi(-32)..=2f64.powi(32)).contains(&rate) {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "unsupported component media rate",
        ));
    }
    let mut result = String::new();
    for _ in 0..32 {
        if rate == 1.0 {
            return Ok(result);
        }
        let factor = rate.clamp(0.5, 2.0);
        result.push_str(&format!(",atempo={factor:.17}"));
        rate /= factor;
    }
    if rate != 1.0 {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "component tempo stage limit exceeded",
        ));
    }
    Ok(result)
}
fn precise_ducking(settings: Option<&EvaluatedDucking>, intervals: &[(f64, f64)]) -> String {
    let Some(settings) = settings else {
        return "1".into();
    };
    let mut expression = "1".to_owned();
    for &(start, end) in intervals {
        let attack_start = (start - settings.attack_ms as f64).max(0.0);
        let release_end = end + settings.release_ms as f64;
        let attack = seconds(settings.attack_ms.max(1));
        let release = seconds(settings.release_ms.max(1));
        let gain = format_number(settings.gain);
        let envelope = format!(
            "if(between(t,{},{}),1-(1-({gain}))*((t-{})/{attack}),if(between(t,{},{}),({gain}),if(between(t,{},{}),({gain})+(1-({gain}))*((t-{})/{release}),1)))",
            precise_seconds(attack_start),
            precise_seconds(start),
            precise_seconds(attack_start),
            precise_seconds(start),
            precise_seconds(end),
            precise_seconds(end),
            precise_seconds(release_end),
            precise_seconds(end)
        );
        expression = format!("min({expression},{envelope})");
    }
    expression
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Asset, AudioSettings, AudioTrackRole, DuckingSettings, MediaItem,
        ParameterizedAnimationCurve, ProjectSettings, RectangleItem, SolidColorItem, TextItem,
        TextStyle, Track, TrackType, Transform, TransitionItem, TransitionType,
        evaluated_scene::evaluate_project, render_artifact::media_input_requests,
    };

    fn affine_shape_cadence_scene() -> EvaluatedScene {
        let mut project = empty_project();
        let item: TimelineItem = serde_json::from_value(serde_json::json!({
            "type":"shape","id":"cadence-shape","startMs":0,"durationMs":1000,
            "geometry":{"type":"rectangle","width":5,"height":5},
            "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
            "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},
            "keyframes":[]
        }))
        .unwrap();
        project.tracks.push(Track {
            audio_bus_id: None,
            id: "cadence-track".into(),
            name: "Cadence".into(),
            track_type: TrackType::Overlay,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![item],
        });
        evaluate_project(&project, 64, 64, 10).unwrap().scene
    }

    #[test]
    fn affine_shape_cadence_classifies_exact_integer_and_fractional_root_endpoints() {
        use crate::evaluated_scene::{EvaluatedInstance, EvaluatedTimeSpan};
        let scene = affine_shape_cadence_scene();
        let mut layer = scene.visual_layers[0].clone();
        // Direct-root visibility uses the local span only without a clip/instance.
        layer.ancestors = None;
        layer.instance = None;
        for fps in [10, 24, 30, 120] {
            for (start, end, expected) in [
                (0, 1000, false),
                (1000, 2000, false),
                (1, 1000, true),
                (0, 999, true),
                (713, 799, true),
            ] {
                layer.span = EvaluatedTimeSpan {
                    start_ms: start,
                    end_ms: end,
                };
                assert_eq!(
                    affine_shape_needs_scene_cadence(&layer, fps),
                    expected,
                    "integer {fps}: {start}..{end}"
                );
            }
            // These finite integer endpoints exceed f64's exact integer range.
            // The owning integer path must retain divisibility without overflow.
            layer.span = EvaluatedTimeSpan {
                start_ms: u64::MAX - 615,
                end_ms: u64::MAX - 615,
            };
            assert!(!affine_shape_needs_scene_cadence(&layer, fps));
            layer.span.end_ms = u64::MAX;
            assert!(affine_shape_needs_scene_cadence(&layer, fps));
            for (start, end, expected) in [
                (0.0, 1000.0, false),
                (0.25, 1000.0, true),
                (0.0, 999.75, true),
                (-1000.0, 0.0, false),
                (-0.25, 1000.0, true),
                (1000.0 - f64::EPSILON * 1024.0, 2000.0, true),
                (1000.0 + f64::EPSILON * 1024.0, 2000.0, true),
                (f64::NAN, 1000.0, true),
                (0.0, f64::INFINITY, true),
                (f64::MAX, 1000.0, true),
            ] {
                layer.instance = Some(EvaluatedInstance {
                    rate: 1.5,
                    offset: 17.25,
                    start_ms: start,
                    end_ms: end,
                    canvas: (64, 64),
                });
                assert_eq!(
                    affine_shape_needs_scene_cadence(&layer, fps),
                    expected,
                    "fractional {fps}: {start}..{end}"
                );
            }
            // Exact root endpoints override a deliberately off-grid local span.
            layer.instance = Some(EvaluatedInstance {
                rate: 1.5,
                offset: 17.25,
                start_ms: 0.0,
                end_ms: 1000.0,
                canvas: (64, 64),
            });
            assert!(!affine_shape_needs_scene_cadence(&layer, fps));
            layer.instance = None;
        }
    }

    #[test]
    fn affine_shape_cadence_uses_clipped_root_span_and_same_property_keys() {
        use crate::evaluated_scene::{EvaluatedAncestors, EvaluatedTimeSpan};
        let scene = affine_shape_cadence_scene();
        let mut layer = scene.visual_layers[0].clone();
        let identity = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        layer.ancestors = Some(EvaluatedAncestors {
            matrix: identity,
            inverse: identity,
            opacity: 1.0,
            clip: EvaluatedTimeSpan {
                start_ms: 713,
                end_ms: 799,
            },
        });
        assert!(affine_shape_needs_scene_cadence(&layer, 10));
        layer.ancestors.as_mut().unwrap().clip = EvaluatedTimeSpan {
            start_ms: 0,
            end_ms: 1000,
        };
        assert!(!affine_shape_needs_scene_cadence(&layer, 10));
        let key = |property, time_ms| EvaluatedKeyframe {
            property,
            time_ms,
            value: EvaluatedKeyframeValue::Scalar { value: 0.7 },
            easing: EvaluatedEasing::Hold,
            r#loop: None,
            clock: None,
        };
        layer.keyframes = vec![key(EvaluatedProperty::Opacity, 0)];
        assert!(!affine_shape_needs_scene_cadence(&layer, 10));
        layer.keyframes.push(key(EvaluatedProperty::Scale, 0));
        assert!(!affine_shape_needs_scene_cadence(&layer, 10));
        for property in [
            EvaluatedProperty::Position,
            EvaluatedProperty::PositionX,
            EvaluatedProperty::PositionY,
            EvaluatedProperty::Scale,
            EvaluatedProperty::ScaleX,
            EvaluatedProperty::ScaleY,
            EvaluatedProperty::Opacity,
        ] {
            layer.keyframes = vec![key(property, 0), key(property, 900)];
            assert!(affine_shape_needs_scene_cadence(&layer, 10), "{property:?}");
        }
        layer.keyframes.clear();
        layer.transitions.push(EvaluatedTransition {
            role: EvaluatedTransitionRole::In,
            kind: crate::evaluated_scene::EvaluatedTransitionKind::Fade,
            span: EvaluatedTimeSpan {
                start_ms: 0,
                end_ms: 900,
            },
        });
        assert!(affine_shape_needs_scene_cadence(&layer, 10));
    }

    #[test]
    fn affine_shape_cadence_preserves_static_source_graph_and_missing_input_error() {
        let mut scene = affine_shape_cadence_scene();
        let mut original = scene.visual_layers[0].clone();
        original.ancestors = None;
        original.instance = None;
        let affine = crate::evaluated_scene::evaluate_layer_affine(
            &original,
            original.source_size.unwrap(),
            (64, 64),
        )
        .unwrap();
        let indexes = HashMap::from([(original.item_id.as_str(), 0)]);
        let source_graph = |layer: &crate::evaluated_scene::EvaluatedVisualLayer| {
            let mut filters = vec![];
            append_affine_layer(
                &mut filters,
                layer,
                &affine,
                &scene,
                &HashMap::new(),
                &indexes,
                ("base", "output", 0),
            )
            .unwrap();
            filters[0].clone()
        };
        let unchanged = source_graph(&original);
        assert!(unchanged.starts_with("[0:v]setpts=PTS-STARTPTS,format=rgba"));
        assert!(!unchanged.starts_with("[0:v]fps="));
        let mut changed = original.clone();
        changed.span.start_ms = 713;
        assert!(source_graph(&changed).starts_with("[0:v]fps=10,setpts=PTS-STARTPTS,format=rgba"));
        let mut filters = vec![];
        changed.source_size = None;
        let error = append_affine_layer(
            &mut filters,
            &changed,
            &affine,
            &scene,
            &HashMap::new(),
            &HashMap::new(),
            ("base", "output", 0),
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert_eq!(error.message, "missing affine source size");
        changed.source_size = original.source_size;
        let error = append_affine_layer(
            &mut filters,
            &changed,
            &affine,
            &scene,
            &HashMap::new(),
            &HashMap::new(),
            ("base", "output", 0),
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(error.message, "missing shape input");
        assert!(filters.is_empty());
        // The existing ancestor predicate remains an independent reason to
        // normalize even when local keys and both root endpoints are static.
        let mut project = empty_project();
        project.tracks.push(Track { audio_bus_id:None,id:"ancestors".into(),name:"Ancestors".into(),track_type:TrackType::Overlay,locked:false,hidden:false,muted:false,audio_role:AudioTrackRole::Unassigned,ducking:None,items:vec![serde_json::from_value(serde_json::json!({"type":"group","id":"parent","startMs":0,"durationMs":1000,"animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.7},"curve":"hold"}]}]})).unwrap(),serde_json::from_value(serde_json::json!({"type":"shape","id":"child","startMs":0,"durationMs":1000,"stackOrder":1,"geometry":{"type":"rectangle","width":5,"height":5},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,"parent":{"scope":"root","id":"parent"},"keyframes":[]})).unwrap()] });
        scene = evaluate_project(&project, 64, 64, 10).unwrap().scene;
        assert!(scene.visual_layers[0].has_animated_ancestors());
        assert!(affine_shape_needs_scene_cadence(
            &scene.visual_layers[0],
            10
        ));
        scene.visual_layers[0].source = EvaluatedVisualSource::Media {
            asset_id: "video-asset".into(),
            source_in_ms: 0,
        };
        scene.resources.clear(); // No image-resource reason is present.
        let layer = &scene.visual_layers[0];
        let affine = crate::evaluated_scene::evaluate_layer_affine(
            layer,
            layer.source_size.unwrap(),
            (64, 64),
        )
        .unwrap();
        let inputs = HashMap::from([(layer.item_id.as_str(), 2)]);
        let mut filters = vec![];
        append_affine_layer(
            &mut filters,
            layer,
            &affine,
            &scene,
            &HashMap::new(),
            &inputs,
            ("base", "output", 0),
        )
        .unwrap();
        assert!(filters[0].contains("/TB,fps=10,format=rgba"));
    }

    fn image_media_cadence_scene(
        kind: Option<crate::evaluated_scene::EvaluatedMediaKind>,
    ) -> EvaluatedScene {
        let mut scene = affine_shape_cadence_scene();
        let layer = &mut scene.visual_layers[0];
        layer.ancestors = None;
        layer.instance = None;
        layer.affine = None;
        layer.source = EvaluatedVisualSource::Media {
            asset_id: "image-asset".into(),
            source_in_ms: 25,
        };
        scene.resources = kind
            .into_iter()
            .map(|kind| crate::evaluated_scene::EvaluatedMediaResource {
                asset_id: "image-asset".into(),
                kind,
                has_audio: false,
            })
            .collect();
        scene
    }

    fn image_media_cadence_plan(scene: &EvaluatedScene, intent: RenderIntent) -> RenderPlan {
        build_render_plan(
            scene,
            &HashMap::new(),
            vec![MediaInputRequest {
                item_id: scene.visual_layers[0].item_id.clone(),
                asset_id: "image-asset".into(),
                project_relative_path: "assets/image.png".into(),
                media_type: MediaType::Image,
                source_in_ms: 25,
                duration_ms: 1000,
                input_index: 2,
            }],
            vec![],
            None,
            intent,
            &mut vec![],
        )
        .unwrap()
    }

    #[test]
    fn image_media_cadence_requires_actual_image_resource_and_precise_raster_reason() {
        use crate::evaluated_scene::EvaluatedMediaKind;
        for kind in [
            Some(EvaluatedMediaKind::Image),
            Some(EvaluatedMediaKind::Video),
            Some(EvaluatedMediaKind::Audio),
            None,
        ] {
            let mut scene = image_media_cadence_scene(kind);
            assert!(!image_media_needs_scene_cadence(
                &scene.visual_layers[0],
                &scene
            ));
            scene.visual_layers[0].span.start_ms = 713;
            assert_eq!(
                image_media_needs_scene_cadence(&scene.visual_layers[0], &scene),
                kind == Some(EvaluatedMediaKind::Image)
            );
            scene.visual_layers[0].span.start_ms = 0;
            let key = |time_ms| EvaluatedKeyframe {
                property: EvaluatedProperty::Opacity,
                time_ms,
                value: EvaluatedKeyframeValue::Scalar { value: 0.7 },
                easing: EvaluatedEasing::Hold,
                r#loop: None,
                clock: None,
            };
            scene.visual_layers[0].keyframes = vec![key(0)];
            assert!(!image_media_needs_scene_cadence(
                &scene.visual_layers[0],
                &scene
            ));
            scene.visual_layers[0].keyframes.push(key(900));
            assert_eq!(
                image_media_needs_scene_cadence(&scene.visual_layers[0], &scene),
                kind == Some(EvaluatedMediaKind::Image)
            );
            scene.visual_layers[0].keyframes.clear();
            scene.visual_layers[0]
                .transitions
                .push(EvaluatedTransition {
                    role: EvaluatedTransitionRole::Out,
                    kind: crate::evaluated_scene::EvaluatedTransitionKind::Fade,
                    span: crate::evaluated_scene::EvaluatedTimeSpan {
                        start_ms: 700,
                        end_ms: 900,
                    },
                });
            assert_eq!(
                image_media_needs_scene_cadence(&scene.visual_layers[0], &scene),
                kind == Some(EvaluatedMediaKind::Image)
            );
            if let Some(resource) = scene.resources.first_mut() {
                resource.asset_id = "different-asset".into();
            }
            assert!(!image_media_needs_scene_cadence(
                &scene.visual_layers[0],
                &scene
            ));
        }
    }

    #[test]
    fn image_media_cadence_direct_graph_preserves_mapping_static_identity_and_inclusive_end() {
        use crate::evaluated_scene::EvaluatedMediaKind;
        let mut image = image_media_cadence_scene(Some(EvaluatedMediaKind::Image));
        let mut video = image_media_cadence_scene(Some(EvaluatedMediaKind::Video));
        let intents = [
            RenderIntent::Frame { at_ms: 700 },
            RenderIntent::Range {
                start_ms: 700,
                end_ms: 900,
                include_audio: false,
            },
            RenderIntent::Export,
        ];
        for intent in intents {
            let baseline = image_media_cadence_plan(&image, intent);
            assert_eq!(
                baseline.filter_graph,
                image_media_cadence_plan(&video, intent).filter_graph
            );
            assert!(
                baseline
                    .filter_graph
                    .contains("[2:v]setpts=PTS-STARTPTS+0.000/TB,scale=")
            );
            image.visual_layers[0].span.start_ms = 713;
            image.visual_layers[0].span.end_ms = 799;
            video.visual_layers[0].span = image.visual_layers[0].span;
            let corrected = image_media_cadence_plan(&image, intent);
            assert!(
                corrected
                    .filter_graph
                    .contains("[2:v]setpts=PTS-STARTPTS+0.713/TB,fps=10,scale=")
            );
            let unchanged_video = image_media_cadence_plan(&video, intent);
            assert!(
                unchanged_video
                    .filter_graph
                    .contains("[2:v]setpts=PTS-STARTPTS+0.713/TB,scale=")
            );
            assert!(!unchanged_video.filter_graph.contains(",fps=10"));
            assert_eq!(corrected.media_inputs[0].source_in_ms, 25);
            image.resources.clear();
            assert_eq!(
                image_media_cadence_plan(&image, intent).filter_graph,
                unchanged_video.filter_graph
            );
            image.resources = image_media_cadence_scene(Some(EvaluatedMediaKind::Image)).resources;
            image.visual_layers[0].span.start_ms = 700;
            image.visual_layers[0].span.end_ms = 800;
            video.visual_layers[0].span = image.visual_layers[0].span;
            let endpoint = image_media_cadence_plan(&image, intent).filter_graph;
            assert_eq!(
                endpoint,
                image_media_cadence_plan(&video, intent).filter_graph
            );
            assert!(endpoint.contains("enable='between(t,0.700,0.800)'"));
            assert!(!endpoint.contains(",fps=10"));
            image.visual_layers[0].span.start_ms = 0;
            image.visual_layers[0].span.end_ms = 1000;
            video.visual_layers[0].span = image.visual_layers[0].span;
        }
        let error = build_render_plan(
            &image,
            &HashMap::new(),
            vec![],
            vec![],
            None,
            RenderIntent::Export,
            &mut vec![],
        )
        .err()
        .unwrap();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(error.message, "renderer input mapping is missing");
    }

    #[test]
    fn image_media_cadence_affine_graph_preserves_rate_start_video_and_input_errors() {
        use crate::evaluated_scene::{EvaluatedInstance, EvaluatedMediaKind};
        let mut scene = image_media_cadence_scene(Some(EvaluatedMediaKind::Image));
        let layer = &mut scene.visual_layers[0];
        layer.span.start_ms = 713;
        layer.span.end_ms = 799;
        layer.instance = Some(EvaluatedInstance {
            rate: 2.0,
            offset: 13.0,
            start_ms: 713.25,
            end_ms: 799.75,
            canvas: (64, 64),
        });
        let affine = crate::evaluated_scene::evaluate_layer_affine(
            layer,
            layer.source_size.unwrap(),
            (64, 64),
        )
        .unwrap();
        let source = |scene: &EvaluatedScene| {
            let layer = &scene.visual_layers[0];
            let mut filters = vec![];
            let inputs = HashMap::from([(layer.item_id.as_str(), 2)]);
            append_affine_layer(
                &mut filters,
                layer,
                &affine,
                scene,
                &HashMap::new(),
                &inputs,
                ("base", "output", 0),
            )
            .unwrap();
            filters[0].clone()
        };
        let normalized = source(&scene);
        assert!(normalized.starts_with("[2:v]setpts=(PTS-STARTPTS)/2.00000000000000000+0.34999999999999998/TB,fps=10,format=rgba"));
        scene.resources[0].kind = EvaluatedMediaKind::Video;
        let video = source(&scene);
        assert!(!video.contains(",fps=10"));
        assert_eq!(normalized.replacen(",fps=10", "", 1), video);
        scene.resources.clear();
        assert_eq!(source(&scene), video);
        let mut filters = vec![];
        let error = append_affine_layer(
            &mut filters,
            &scene.visual_layers[0],
            &affine,
            &scene,
            &HashMap::new(),
            &HashMap::new(),
            ("base", "output", 0),
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(error.message, "missing affine media input");
        assert!(filters.is_empty());
        scene.visual_layers[0].instance = None;
        scene.visual_layers[0].span = crate::evaluated_scene::EvaluatedTimeSpan {
            start_ms: 0,
            end_ms: 1000,
        };
        scene.resources = image_media_cadence_scene(Some(EvaluatedMediaKind::Image)).resources;
        let static_image = source(&scene);
        assert!(static_image.starts_with(
            "[2:v]setpts=(PTS-STARTPTS)/1.00000000000000000+0.00000000000000000/TB,format=rgba"
        ));
        assert!(!static_image.contains(",fps=10"));
        scene.resources[0].kind = EvaluatedMediaKind::Video;
        assert_eq!(source(&scene), static_image);
    }

    #[test]
    fn epic6_fullscene_synthetic_compatibility_is_structural_and_intent_qualified() {
        let mut p = empty_project();
        p.tracks.push(Track {
            audio_bus_id: None,
            id: "plain".into(),
            name: "plain".into(),
            track_type: TrackType::Overlay,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![TimelineItem::SolidColor(SolidColorItem {
                id: "solid".into(),
                color: "#ff0000".into(),
                start_ms: 0,
                duration_ms: 1000,
                visual_properties: crate::VisualProperties::new(Transform::default(), false),
                keyframes: vec![],
            })],
        });
        let scene = evaluate_project(&p, 64, 64, 10).unwrap().scene;
        assert!(scene.visual_layers[0].source_size.is_none());
        let intents = [
            RenderIntent::Frame { at_ms: 500 },
            RenderIntent::Range {
                start_ms: 0,
                end_ms: 1000,
                include_audio: true,
            },
            RenderIntent::Export,
        ];
        let mut graphs = vec![];
        for intent in intents {
            assert!(historical_fullscene_synthetic(
                &scene.visual_layers[0],
                &scene,
                intent
            ));
            let graph = build_render_plan(
                &scene,
                &HashMap::new(),
                vec![],
                vec![],
                None,
                intent,
                &mut vec![],
            )
            .unwrap()
            .filter_graph;
            assert!(
                graph.contains(
                    "color=c=0xff0000:s=64x64:r=10:d=1.000,format=rgba,setpts=PTS+0.000/TB"
                )
            );
            assert!(graph.contains("enable='between(t,0.000,1.000)'"));
            assert!(!graph.contains("trim=end_frame="));
            graphs.push(graph);
        }
        assert!(graphs.windows(2).all(|pair| pair[0] == pair[1]));
        for intent in [
            RenderIntent::Frame { at_ms: 1000 },
            RenderIntent::Frame { at_ms: 713 },
            RenderIntent::Range {
                start_ms: 713,
                end_ms: 913,
                include_audio: false,
            },
            RenderIntent::Range {
                start_ms: 0,
                end_ms: 0,
                include_audio: false,
            },
            RenderIntent::Range {
                start_ms: 0,
                end_ms: 1001,
                include_audio: false,
            },
        ] {
            assert!(!historical_fullscene_synthetic(
                &scene.visual_layers[0],
                &scene,
                intent
            ));
        }
        let endpoint = build_render_plan(
            &scene,
            &HashMap::new(),
            vec![],
            vec![],
            None,
            RenderIntent::Frame { at_ms: 1000 },
            &mut vec![],
        )
        .unwrap()
        .filter_graph;
        assert!(endpoint.contains("trim=end_frame=10"));
        assert!(endpoint.contains("enable='gte(t,0.000)*lt(t,1.000)'"));
        for (start, end) in [(0, 800), (713, 1000), (1, 1000)] {
            let mut layer = scene.visual_layers[0].clone();
            layer.span.start_ms = start;
            layer.span.end_ms = end;
            assert!(!historical_fullscene_synthetic(
                &layer,
                &scene,
                RenderIntent::Export
            ));
        }
        let mut layer = scene.visual_layers[0].clone();
        layer.ancestors = Some(crate::evaluated_scene::EvaluatedAncestors {
            matrix: [1., 0., 0., 1., 0., 0.],
            inverse: [1., 0., 0., 1., 0., 0.],
            opacity: 1.,
            clip: layer.span,
        });
        assert!(!historical_fullscene_synthetic(
            &layer,
            &scene,
            RenderIntent::Export
        ));
        let mut layer = scene.visual_layers[0].clone();
        layer.instance = Some(crate::evaluated_scene::EvaluatedInstance {
            rate: 0.751,
            offset: 0.,
            start_ms: 0.,
            end_ms: 1000.,
            canvas: (31, 23),
        });
        assert!(!historical_fullscene_synthetic(
            &layer,
            &scene,
            RenderIntent::Export
        ));
        let mut layer = scene.visual_layers[0].clone();
        layer.sampled_input = Some(("prepared".into(), 0));
        assert!(!historical_fullscene_synthetic(
            &layer,
            &scene,
            RenderIntent::Export
        ));
        let mut layer = scene.visual_layers[0].clone();
        layer.transform2d = Some(crate::Transform2D::default());
        assert!(!historical_fullscene_synthetic(
            &layer,
            &scene,
            RenderIntent::Export
        ));
        // Valid retained clocks do not alter this topology decision or their phase expression.
        if let TimelineItem::SolidColor(solid) = &mut p.tracks[0].items[0] {
            solid.visual_properties.animation_channels=vec![serde_json::from_value(serde_json::json!({"property":"transform.opacity","clock":{"offsetMs":700,"sourceDurationMs":1700},"loop":{"mode":"repeat","iterations":4},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"linear"},{"timeMs":100,"value":{"type":"scalar","value":0.8},"curve":"linear"},{"timeMs":200,"value":{"type":"scalar","value":0.2},"curve":"hold"}]})).unwrap()];
        }
        let retained = evaluate_project(&p, 64, 64, 10).unwrap().scene;
        let layer = &retained.visual_layers[0];
        assert!(historical_fullscene_synthetic(
            layer,
            &retained,
            RenderIntent::Export
        ));
        let graph = build_render_plan(
            &retained,
            &HashMap::new(),
            vec![],
            vec![],
            None,
            RenderIntent::Export,
            &mut vec![],
        )
        .unwrap()
        .filter_graph;
        let expression = evaluated_scalar_expression_for(
            &layer.keyframes,
            EvaluatedProperty::Opacity,
            layer.transform.opacity,
            0,
            "T",
        );
        assert!(graph.contains(&format!("alpha(X,Y)*({expression})")));
        assert_eq!(
            crate::animation::sample_scalar_channel(
                &p.tracks[0].items[0].visual_properties().animation_channels[0],
                100
            ),
            Some(0.2)
        );
    }

    #[test]
    fn epic6_synthetic_frame_windows_cover_exact_global_cells_and_fail_closed() {
        for (fps, first, count) in [(10, 7, 2), (24, 17, 3), (30, 21, 4), (120, 85, 13)] {
            assert_eq!(
                synthetic_frame_window(713, 813, fps).unwrap(),
                (first, count)
            );
            // Every generated tick encloses the interval; there is at most one
            // prefix tick and the next tick is beyond the exclusive end.
            assert!((first as u128) * 1000 <= 713 * u128::from(fps));
            assert!(((first + count) as u128) * 1000 >= 813 * u128::from(fps));
            assert!(((first + 1) as u128) * 1000 > 713 * u128::from(fps));
        }
        assert_eq!(synthetic_frame_window(700, 800, 10).unwrap(), (7, 1));
        assert_eq!(synthetic_frame_window(713, 799, 10).unwrap(), (7, 1));
        // This interval has no visible export-grid tick: its sole source cell
        // is the inactive700ms prefix, not a fabricated713ms output sample.
        assert_eq!(synthetic_frame_window(1, 2, 120).unwrap(), (0, 1));
        for fps in [10, 24, 30, 120] {
            let (first, count) = synthetic_frame_window(u64::MAX - 1, u64::MAX, fps).unwrap();
            assert!(first > 0 && count > 0);
        }
        for (start, end, fps) in [
            (0, 0, 10),
            (2, 1, 10),
            (0, 1, 0),
            (u64::MAX - 1, u64::MAX, u32::MAX),
            (0, u64::MAX, u32::MAX),
        ] {
            assert_eq!(
                synthetic_frame_window(start, end, fps).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn native_retained_clock_expressions_match_independent_source_functions() {
        let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
            assert_ne!(
                std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
                Ok("1"),
                "retained clock parity requires FFmpeg"
            );
            return;
        };
        for (offset, local, source) in [
            (200, 200.5, 400.5),
            (-200, 100.5, 0.0),
            (700, 500.25, 1200.25),
        ] {
            for (property, easing, loop_spec, expected) in [
                (
                    EvaluatedProperty::PositionX,
                    EvaluatedEasing::CubicBezier {
                        x1: 1.0 / 3.0,
                        y1: 0.0,
                        x2: 2.0 / 3.0,
                        y2: 0.0,
                    },
                    None,
                    40.0 * (source / 1000.0_f64).min(1.0).powi(3),
                ),
                (
                    EvaluatedProperty::GainDb,
                    EvaluatedEasing::Spring {
                        mass: 1.0,
                        stiffness: 16.0,
                        damping: 8.0,
                        initial_velocity: 0.0,
                    },
                    None,
                    if source >= 1000.0 {
                        0.0
                    } else {
                        -12.0
                            + 12.0
                                * (1.0
                                    - (1.0 + 4.0 * source / 1000.0)
                                        * (-4.0 * source / 1000.0).exp())
                    },
                ),
                (
                    EvaluatedProperty::Opacity,
                    EvaluatedEasing::Linear,
                    Some(crate::AnimationLoop {
                        mode: crate::AnimationLoopMode::PingPong,
                        iterations: crate::AnimationLoopIterations::Finite(1),
                    }),
                    if source >= 2000.0 {
                        0.0
                    } else {
                        (if source <= 1000.0 {
                            source
                        } else {
                            2000.0 - source
                        }) / 1000.0
                    },
                ),
            ] {
                let last_value = match property {
                    EvaluatedProperty::PositionX => 40.0,
                    EvaluatedProperty::GainDb => 0.0,
                    _ => 1.0,
                };
                let first_value = if property == EvaluatedProperty::GainDb {
                    -12.0
                } else {
                    0.0
                };
                let frames = [
                    EvaluatedKeyframe {
                        property,
                        time_ms: 0,
                        value: EvaluatedKeyframeValue::Scalar { value: first_value },
                        easing,
                        r#loop: loop_spec,
                        clock: Some(crate::AnimationClock {
                            offset_ms: offset,
                            source_duration_ms: 3000,
                        }),
                    },
                    EvaluatedKeyframe {
                        property,
                        time_ms: 1000,
                        value: EvaluatedKeyframeValue::Scalar { value: last_value },
                        easing: EvaluatedEasing::Hold,
                        r#loop: loop_spec,
                        clock: Some(crate::AnimationClock {
                            offset_ms: offset,
                            source_duration_ms: 3000,
                        }),
                    },
                ];
                let time = format!("{:.17}", local / 1000.0);
                let expr = evaluated_scalar_expression_with_precision(
                    &frames,
                    property,
                    0.0,
                    0,
                    &time,
                    LoopClockPrecision::FractionalMilliseconds,
                );
                let source = format!("aevalsrc=exprs='{expr}':s=8000:d=0.001");
                let output = std::process::Command::new(&ffmpeg)
                    .args([
                        "-v",
                        "error",
                        "-f",
                        "lavfi",
                        "-i",
                        &source,
                        "-frames:a",
                        "1",
                        "-ac",
                        "1",
                        "-c:a",
                        "pcm_f64le",
                        "-f",
                        "f64le",
                        "-",
                    ])
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let actual = f64::from_le_bytes(output.stdout[..8].try_into().unwrap());
                assert!(
                    (actual - expected).abs() < 1e-7,
                    "{property:?} offset={offset} local={local} actual={actual} expected={expected}"
                );
            }
        }
    }

    #[test]
    fn native_retained_large_clock_expressions_preserve_fractional_segments() {
        let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
            assert_ne!(
                std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(),
                Ok("1"),
                "large clock expression comparison requires FFmpeg"
            );
            return;
        };
        use crate::{
            AnimationInfiniteIterations, AnimationLoop, AnimationLoopIterations as Iterations,
            AnimationLoopMode as Mode,
        };
        let huge = 1_u64 << 52;
        let infinite = Iterations::Infinite(AnimationInfiniteIterations::Infinite);
        let linear = EvaluatedEasing::Linear;
        let hold = EvaluatedEasing::Hold;
        let short = vec![(0, 0.0, linear), (100, 1.0, hold)];
        let origin = vec![(huge, 0.0, linear), (huge + 100, 1.0, hold)];
        let large_period = vec![
            (0, 0.0, linear),
            (huge, 0.0, linear),
            (huge + 1, 1.0, linear),
            (huge + 100, 1.0, hold),
        ];
        let cases = vec![
            (
                short.clone(),
                huge as i64,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: infinite,
                }),
                0.25,
                0.9625,
            ),
            (
                short.clone(),
                huge as i64 + 100,
                Some(AnimationLoop {
                    mode: Mode::PingPong,
                    iterations: infinite,
                }),
                0.25,
                0.0375,
            ),
            (
                short.clone(),
                i64::MAX,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: infinite,
                }),
                0.25,
                0.0725,
            ),
            (short.clone(), -(huge as i64), None, 0.25, 0.0),
            (origin.clone(), huge as i64, None, 0.25, 0.0025),
            (
                origin.clone(),
                huge as i64,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: infinite,
                }),
                0.25,
                0.0025,
            ),
            (
                origin.clone(),
                huge as i64 + 199,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: Iterations::Finite(2),
                }),
                0.25,
                0.9925,
            ),
            (
                origin.clone(),
                huge as i64 + 199,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: Iterations::Finite(2),
                }),
                1.0,
                1.0,
            ),
            (
                origin.clone(),
                huge as i64 + 199,
                Some(AnimationLoop {
                    mode: Mode::PingPong,
                    iterations: Iterations::Finite(1),
                }),
                0.25,
                0.0075,
            ),
            (
                origin,
                huge as i64 + 199,
                Some(AnimationLoop {
                    mode: Mode::PingPong,
                    iterations: Iterations::Finite(1),
                }),
                1.0,
                0.0,
            ),
            (
                short.clone(),
                huge as i64,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: Iterations::Finite(3),
                }),
                0.25,
                1.0,
            ),
            (
                short,
                huge as i64,
                Some(AnimationLoop {
                    mode: Mode::PingPong,
                    iterations: Iterations::Finite(3),
                }),
                0.25,
                0.0,
            ),
            (
                large_period.clone(),
                huge as i64,
                Some(AnimationLoop {
                    mode: Mode::Repeat,
                    iterations: infinite,
                }),
                0.25,
                0.25,
            ),
            (
                large_period,
                huge as i64 + 199,
                Some(AnimationLoop {
                    mode: Mode::PingPong,
                    iterations: infinite,
                }),
                0.25,
                0.75,
            ),
            (
                vec![(huge, 0.0, EvaluatedEasing::EaseIn), (huge + 1, 1.0, hold)],
                huge as i64,
                None,
                0.25,
                0.0625,
            ),
            (
                vec![
                    (
                        huge,
                        0.0,
                        EvaluatedEasing::CubicBezier {
                            x1: 1.0 / 3.0,
                            y1: 0.0,
                            x2: 2.0 / 3.0,
                            y2: 0.0,
                        },
                    ),
                    (huge + 1, 1.0, hold),
                ],
                huge as i64,
                None,
                0.25,
                0.015625,
            ),
            // Independent critical-spring formula: 1-(1+2t)*exp(-2t).
            (
                vec![
                    (
                        huge,
                        0.0,
                        EvaluatedEasing::Spring {
                            mass: 1.0,
                            stiffness: 4.0,
                            damping: 4.0,
                            initial_velocity: 0.0,
                        },
                    ),
                    (huge + 1, 1.0, hold),
                ],
                huge as i64,
                None,
                0.25,
                1.0 - 1.5 * (-0.5_f64).exp(),
            ),
        ];
        let mut cases = cases;
        for (local_ms, expected) in [(0.0, 1.0), (0.25, 0.0), (0.000_001, 0.0)] {
            cases.push((
                vec![(huge, 0.0, hold), (huge + 100, 1.0, hold)],
                huge as i64 + 100,
                Some(AnimationLoop {
                    mode: Mode::PingPong,
                    iterations: infinite,
                }),
                local_ms,
                expected,
            ));
        }
        for (values, offset, loop_spec, local_ms, expected) in cases {
            let time = format!("{:.17}", (10.0 + local_ms) / 1000.0);
            let expression = clocked_scalar_expression(
                &values,
                10,
                &time,
                offset,
                loop_spec,
                LoopClockPrecision::FractionalMilliseconds,
                None,
            );
            let source = format!("aevalsrc=exprs='{expression}':s=8000:d=0.001");
            let output = std::process::Command::new(&ffmpeg)
                .args([
                    "-v",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    &source,
                    "-frames:a",
                    "1",
                    "-ac",
                    "1",
                    "-c:a",
                    "pcm_f64le",
                    "-f",
                    "f64le",
                    "-",
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let actual = f64::from_le_bytes(output.stdout[..8].try_into().unwrap());
            assert!(
                (actual - expected).abs() < 0.000_001,
                "offset{offset} local{local_ms} loop{loop_spec:?}: expected{expected} actual{actual}"
            );
        }
    }

    #[test]
    fn native_loop_fractional_expression_matches_independent_phases() {
        let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
            assert_ne!(
                std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(),
                Ok("1"),
                "fractional loop comparison requires FFmpeg"
            );
            return;
        };
        use crate::{
            AnimationLoop, AnimationLoopIterations as Iterations, AnimationLoopMode as Mode,
        };
        for (mode, iterations, first, values, samples) in [
            (
                Mode::Repeat,
                Iterations::Infinite(crate::AnimationInfiniteIterations::Infinite),
                0,
                vec![
                    (0, 0.0, EvaluatedEasing::Linear),
                    (99, 100.0, EvaluatedEasing::Linear),
                    (100, 0.0, EvaluatedEasing::Hold),
                ],
                vec![
                    (99.25, 75.0),
                    (99.5, 50.0),
                    (99.75, 25.0),
                    (100.0, 0.0),
                    (100.25, 25.0 / 99.0),
                    (199.5, 50.0),
                ],
            ),
            (
                Mode::Repeat,
                Iterations::Finite(1),
                10,
                vec![
                    (10, 0.0, EvaluatedEasing::Linear),
                    (109, 100.0, EvaluatedEasing::Linear),
                    (110, 0.0, EvaluatedEasing::Hold),
                ],
                vec![(9.5, 0.0), (109.5, 50.0), (110.0, 0.0), (110.25, 0.0)],
            ),
            (
                Mode::PingPong,
                Iterations::Infinite(crate::AnimationInfiniteIterations::Infinite),
                10,
                vec![
                    (10, 0.0, EvaluatedEasing::Linear),
                    (110, 100.0, EvaluatedEasing::Hold),
                ],
                vec![
                    (109.5, 99.5),
                    (110.0, 100.0),
                    (110.5, 99.5),
                    (209.5, 0.5),
                    (210.0, 0.0),
                    (210.5, 0.5),
                ],
            ),
            (
                Mode::PingPong,
                Iterations::Finite(1),
                10,
                vec![
                    (10, 0.0, EvaluatedEasing::Linear),
                    (110, 100.0, EvaluatedEasing::Hold),
                ],
                vec![(209.5, 0.5), (210.0, 0.0), (210.5, 0.0)],
            ),
        ] {
            for (local_ms, expected) in samples {
                // The constants are independently derived linear phases, not samples
                // from the production Rust integer sampler.
                let time = format!("{:.17}", local_ms / 1000.0);
                let mapped = looped_time_expression(
                    AnimationLoop { mode, iterations },
                    0,
                    first,
                    100,
                    &time,
                    LoopClockPrecision::FractionalMilliseconds,
                );
                let expression = evaluated_piecewise_expression_for(&values, 0.0, 0, &mapped, None);
                let source = format!("aevalsrc=exprs='{expression}':s=8000:d=0.001");
                let output = std::process::Command::new(&ffmpeg)
                    .args([
                        "-v",
                        "error",
                        "-f",
                        "lavfi",
                        "-i",
                        &source,
                        "-frames:a",
                        "1",
                        "-ac",
                        "1",
                        "-c:a",
                        "pcm_f64le",
                        "-f",
                        "f64le",
                        "-",
                    ])
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let actual = f64::from_le_bytes(output.stdout[..8].try_into().unwrap());
                assert!(
                    (actual - expected).abs() < 1e-9,
                    "{mode:?} at {local_ms}: actual={actual}, independent={expected}"
                );
            }
        }
    }

    #[test]
    fn native_loop_scalar_expression_matches_core_at_seams() {
        let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
            assert_ne!(
                std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
                Ok("1"),
                "native loop scalar comparison requires OPENCUT_FFMPEG_PATH"
            );
            return;
        };
        for (mode, iterations, item_start_ms, values, times) in [
            (
                "repeat",
                serde_json::json!("infinite"),
                0,
                vec![
                    (0, 0.0, EvaluatedEasing::Linear),
                    (250, 20.0, EvaluatedEasing::Linear),
                    (500, 0.0, EvaluatedEasing::Hold),
                ],
                vec![0, 250, 500, 750, 1000],
            ),
            (
                "ping_pong",
                serde_json::json!("infinite"),
                0,
                vec![
                    (0, 0.0, EvaluatedEasing::Linear),
                    (500, 20.0, EvaluatedEasing::Hold),
                ],
                vec![0, 250, 500, 750, 1000],
            ),
            (
                "repeat",
                serde_json::json!(3),
                0,
                vec![
                    (3, 0.0, EvaluatedEasing::Linear),
                    (8, 100.0, EvaluatedEasing::Hold),
                    (13, 0.0, EvaluatedEasing::Hold),
                ],
                vec![12, 13, 14, 22, 23, 24, 32, 33, 34],
            ),
            (
                "repeat",
                serde_json::json!(3),
                0,
                vec![
                    (100, 0.0, EvaluatedEasing::Linear),
                    (200, 100.0, EvaluatedEasing::Hold),
                    (300, 0.0, EvaluatedEasing::Hold),
                ],
                vec![99, 299, 300, 301, 499, 500, 501, 699, 700, 701],
            ),
            (
                "repeat",
                serde_json::json!("infinite"),
                100,
                vec![
                    (3, 0.0, EvaluatedEasing::Linear),
                    (8, 100.0, EvaluatedEasing::Hold),
                    (13, 0.0, EvaluatedEasing::Hold),
                ],
                vec![12, 13, 14, 22, 23, 24],
            ),
            (
                "ping_pong",
                serde_json::json!(2),
                0,
                vec![
                    (
                        3,
                        0.0,
                        EvaluatedEasing::CubicBezier {
                            x1: 0.2,
                            y1: 0.1,
                            x2: 0.8,
                            y2: 0.9,
                        },
                    ),
                    (13, 100.0, EvaluatedEasing::Hold),
                ],
                vec![3, 12, 13, 14, 18, 22, 23, 24, 42, 43, 44],
            ),
            (
                "ping_pong",
                serde_json::json!("infinite"),
                0,
                vec![
                    (
                        3,
                        0.0,
                        EvaluatedEasing::Spring {
                            mass: 1.0,
                            stiffness: 100.0,
                            damping: 20.0,
                            initial_velocity: 0.0,
                        },
                    ),
                    (13, 100.0, EvaluatedEasing::Hold),
                ],
                vec![3, 12, 13, 14, 18, 22, 23, 24],
            ),
        ] {
            let channel: crate::AnimationChannel = serde_json::from_value(serde_json::json!({
                "property":"transform.position_x",
                "keyframes":values.iter().map(|(time, value, easing)| serde_json::json!({
                    "timeMs":time,"value":{"type":"scalar","value":value},
                    "curve":match easing {
                        EvaluatedEasing::Linear => serde_json::json!("linear"),
                        EvaluatedEasing::Hold => serde_json::json!("hold"),
                        EvaluatedEasing::CubicBezier {x1,y1,x2,y2} => serde_json::json!({
                            "type":"cubic_bezier","x1":x1,"y1":y1,"x2":x2,"y2":y2
                        }),
                        EvaluatedEasing::Spring {mass,stiffness,damping,initial_velocity} =>
                            serde_json::json!({"type":"spring","mass":mass,
                                "stiffness":stiffness,"damping":damping,
                                "initialVelocity":initial_velocity}),
                        _ => unreachable!("test uses typed channel curves"),
                    }
                })).collect::<Vec<_>>(),
                "loop":{"mode":mode,"iterations":iterations}
            }))
            .unwrap();
            for time_ms in times {
                let time = format_curve_number((item_start_ms + time_ms) as f64 / 1000.0);
                let mapped = looped_time_expression(
                    channel.r#loop.unwrap(),
                    item_start_ms,
                    values.first().unwrap().0,
                    values.last().unwrap().0 - values.first().unwrap().0,
                    &time,
                    LoopClockPrecision::IntegerMilliseconds,
                );
                let expression =
                    evaluated_piecewise_expression_for(&values, 0.0, item_start_ms, &mapped, None);
                let source = format!("aevalsrc=exprs='{expression}':s=8000:d=0.001");
                let output = std::process::Command::new(&ffmpeg)
                    .args([
                        "-hide_banner",
                        "-loglevel",
                        "error",
                        "-f",
                        "lavfi",
                        "-i",
                        &source,
                        "-frames:a",
                        "1",
                        "-ac",
                        "1",
                        "-c:a",
                        "pcm_f64le",
                        "-f",
                        "f64le",
                        "-",
                    ])
                    .output()
                    .expect("FFmpeg loop scalar comparison should start");
                assert!(
                    output.status.success(),
                    "{mode} at {time_ms}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let sample = f64::from_le_bytes(output.stdout[..8].try_into().unwrap());
                let expected = crate::animation::sample_scalar_channel(&channel, time_ms).unwrap();
                assert!(
                    (sample - expected).abs() < 1e-9,
                    "{mode} at {time_ms}: core={expected}, ffmpeg={sample}"
                );
            }
        }
    }

    #[test]
    fn native_curve_scalar_expressions_match_core_at_f64_precision() {
        let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
            assert_ne!(
                std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
                Ok("1"),
                "native curve comparison requires OPENCUT_FFMPEG_PATH"
            );
            return;
        };
        let curves = [
            ParameterizedAnimationCurve::CubicBezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.25,
                y2: 1.0,
            },
            ParameterizedAnimationCurve::CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
            ParameterizedAnimationCurve::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 8.0,
                initial_velocity: 0.0,
            },
            ParameterizedAnimationCurve::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 20.0,
                initial_velocity: 0.0,
            },
            ParameterizedAnimationCurve::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 30.0,
                initial_velocity: 0.0,
            },
            ParameterizedAnimationCurve::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 19.99999999999999,
                initial_velocity: 0.0,
            },
            ParameterizedAnimationCurve::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 20.00000000000001,
                initial_velocity: 0.0,
            },
        ];
        for curve in curves {
            let easing = match curve {
                ParameterizedAnimationCurve::CubicBezier { x1, y1, x2, y2 } => {
                    EvaluatedEasing::CubicBezier { x1, y1, x2, y2 }
                }
                ParameterizedAnimationCurve::Spring {
                    mass,
                    stiffness,
                    damping,
                    initial_velocity,
                } => EvaluatedEasing::Spring {
                    mass,
                    stiffness,
                    damping,
                    initial_velocity,
                },
            };
            for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let expression =
                    evaluated_easing_expression(&format_curve_number(progress), easing);
                let source = format!("aevalsrc=exprs='{expression}':s=8000:d=0.001");
                let output = std::process::Command::new(&ffmpeg)
                    .args([
                        "-hide_banner",
                        "-loglevel",
                        "error",
                        "-filter_threads",
                        "1",
                        "-f",
                        "lavfi",
                        "-i",
                        &source,
                        "-frames:a",
                        "1",
                        "-ac",
                        "1",
                        "-c:a",
                        "pcm_f64le",
                        "-f",
                        "f64le",
                        "-",
                    ])
                    .output()
                    .expect("FFmpeg numeric comparison should start");
                assert!(
                    output.status.success(),
                    "{curve:?} at {progress}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let sample =
                    f64::from_le_bytes(output.stdout[..8].try_into().expect("one f64 sample"));
                let expected = crate::animation::parameterized_curve_progress(curve, progress);
                assert!(
                    (sample - expected).abs() <= 1e-9,
                    "{curve:?} at {progress}: core={expected:.17}, ffmpeg={sample:.17}"
                );
            }
        }
    }

    #[test]
    fn native_parameterized_segment_values_keep_precision_and_bounds() {
        let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
            assert_ne!(
                std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
                Ok("1"),
                "native curve comparison requires OPENCUT_FFMPEG_PATH"
            );
            return;
        };
        let start = 0.123_456_789_012_345_66;
        let end = 0.987_654_321_098_765_4;
        let curve = ParameterizedAnimationCurve::Spring {
            mass: 1.0,
            stiffness: 100.0,
            damping: 19.999_999_999_999_99,
            initial_velocity: 0.0,
        };
        let easing = EvaluatedEasing::Spring {
            mass: 1.0,
            stiffness: 100.0,
            damping: 19.999_999_999_999_99,
            initial_velocity: 0.0,
        };
        let values = [(0, start, easing), (1000, end, EvaluatedEasing::Hold)];
        for progress in [0.0, 0.5, 1.0] {
            let expression = evaluated_piecewise_expression_for(
                &values,
                0.0,
                0,
                &format_curve_number(progress),
                Some((0.0, 1.0)),
            );
            let source = format!("aevalsrc=exprs='{expression}':s=8000:d=0.001");
            let output = std::process::Command::new(&ffmpeg)
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-filter_threads",
                    "1",
                    "-f",
                    "lavfi",
                    "-i",
                    &source,
                    "-frames:a",
                    "1",
                    "-ac",
                    "1",
                    "-c:a",
                    "pcm_f64le",
                    "-f",
                    "f64le",
                    "-",
                ])
                .output()
                .expect("FFmpeg numeric comparison should start");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let sample = f64::from_le_bytes(output.stdout[..8].try_into().expect("one f64 sample"));
            let expected = if progress == 1.0 {
                end
            } else if progress == 0.0 {
                start
            } else {
                (start
                    + (end - start)
                        * crate::animation::parameterized_curve_progress(curve, progress))
                .clamp(0.0, 1.0)
            };
            assert!(
                (sample - expected).abs() <= 1e-9,
                "at {progress}: core={expected:.17}, ffmpeg={sample:.17}"
            );
        }
        let mixed = [
            (0, start, easing),
            (1000, end, EvaluatedEasing::Linear),
            (2000, 0.5, EvaluatedEasing::Hold),
        ];
        let boundary = evaluated_piecewise_expression_for(
            &mixed,
            0.0,
            0,
            "1.00000000000000000",
            Some((0.0, 1.0)),
        );
        let source = format!("aevalsrc=exprs='{boundary}':s=8000:d=0.001");
        let output = std::process::Command::new(&ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-filter_threads",
                "1",
                "-f",
                "lavfi",
                "-i",
                &source,
                "-frames:a",
                "1",
                "-ac",
                "1",
                "-c:a",
                "pcm_f64le",
                "-f",
                "f64le",
                "-",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let sample = f64::from_le_bytes(output.stdout[..8].try_into().unwrap());
        assert!(
            (sample - end).abs() <= 1e-9,
            "mixed-curve boundary rounded: {sample:.17} vs {end:.17}"
        );
        let legacy = evaluated_piecewise_expression_for(
            &[
                (0, start, EvaluatedEasing::Linear),
                (1000, end, EvaluatedEasing::Hold),
            ],
            0.0,
            0,
            "t",
            None,
        );
        assert!(legacy.contains("0.123457"));
        assert!(!legacy.contains("0.12345678901234566"));
    }

    #[test]
    fn authored_mask_facts_preserve_empty_identity_base_geometry_resources_and_semantic_plans() {
        let mut project = empty_project();
        project.schema_version = crate::PROJECT_SCHEMA_VERSION;
        project.audio_buses = crate::default_audio_buses();
        project.tracks = vec![Track {
            audio_bus_id: None,
            id: "visuals".into(),
            name: "Visuals".into(),
            track_type: TrackType::Video,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![],
        }];
        crate::timeline::apply_operation(
            &mut project,
            serde_json::from_value(serde_json::json!({
                "operation":"add_rectangle", "trackId":"visuals", "startMs":0,
                "durationMs":600, "width":20, "height":16, "color":"#cc3311",
                "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}
            }))
            .unwrap(),
        )
        .unwrap();
        let baseline = evaluate_project(&project, 64, 64, 10).unwrap();
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
        let mask = catalog["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["accepted"] == true)
            .unwrap()["value"]
            .clone();
        let item = project.tracks[0].items[0].id().to_owned();
        crate::timeline::apply_operation(
            &mut project,
            serde_json::from_value(serde_json::json!({
                "operation":"update_item", "itemId":item, "masks":[mask]
            }))
            .unwrap(),
        )
        .unwrap();
        project.revision += 1;
        let masked = evaluate_project(&project, 64, 64, 10).unwrap();
        assert_ne!(baseline.revision, masked.revision);
        assert_eq!(baseline.project_id, masked.project_id);
        assert_ne!(baseline.scene, masked.scene);
        assert_eq!(
            masked.scene.visual_layers[0]
                .extended
                .as_ref()
                .unwrap()
                .masks
                .len(),
            1
        );
        // Activation can add canonical projection facts. Assert the source and
        // geometric/timing invariants directly rather than erasing those facts
        // and pretending the active scene must equal its unmasked input.
        let original = &baseline.scene.visual_layers[0];
        let active = &masked.scene.visual_layers[0];
        assert_eq!(active.source, original.source);
        assert_eq!(active.source_size, Some((20, 16)));
        assert_eq!(active.source_size, original.source_size);
        assert_eq!(active.span, original.span);
        assert_eq!(active.transform, original.transform);
        assert_eq!(active.transform2d, original.transform2d);
        assert_eq!(active.order, original.order);
        assert_eq!(active.keyframes, original.keyframes);
        assert_eq!(active.transitions, original.transitions);
        let ancestors = active.ancestors.unwrap();
        assert_eq!(ancestors.matrix, [1., 0., 0., 1., 0., 0.]);
        assert_eq!(ancestors.inverse, [1., 0., 0., 1., 0., 0.]);
        assert_eq!(ancestors.opacity, 1.);
        assert_eq!(ancestors.clip.start_ms, 0);
        assert_eq!(ancestors.clip.end_ms, 600);
        assert!(active.ancestor_stages.is_empty());
        assert_eq!(masked.scene.instance_voiceover_intervals, Some(vec![]));
        let mut empty_project = project.clone();
        empty_project.tracks[0].items[0]
            .visual_properties_mut()
            .masks
            .clear();
        let empty = evaluate_project(&empty_project, 64, 64, 10).unwrap();
        assert_eq!(baseline.scene, empty.scene);
        assert_eq!(baseline.resource_bindings, empty.resource_bindings);
        assert_eq!(baseline.resource_bindings, masked.resource_bindings);
        assert!(
            !project.tracks[0].items[0]
                .visual_properties()
                .masks
                .is_empty()
        );
        for intent in [
            RenderIntent::Frame { at_ms: 100 },
            RenderIntent::Range {
                start_ms: 0,
                end_ms: 600,
                include_audio: true,
            },
            RenderIntent::Export,
        ] {
            let build = |scene: &EvaluatedScene| {
                build_render_plan(
                    scene,
                    &HashMap::new(),
                    vec![],
                    vec![],
                    None,
                    intent,
                    &mut vec![],
                )
                .unwrap()
            };
            // Absent and explicitly empty programs preserve complete scene and
            // semantic-plan identity for every intent, without normalization.
            assert_eq!(build(&baseline.scene), build(&empty.scene));
        }
    }

    #[test]
    fn active_mask_metadata_preserves_missing_media_error() {
        let mut project = empty_project();
        project.schema_version = crate::PROJECT_SCHEMA_VERSION;
        project.audio_buses = crate::default_audio_buses();
        project.tracks = vec![Track {
            audio_bus_id: None,
            id: "visuals".into(),
            name: "Visuals".into(),
            track_type: TrackType::Video,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![TimelineItem::Media(MediaItem {
                id: "leaf".into(),
                asset_id: "absent".into(),
                start_ms: 0,
                duration_ms: 600,
                source_in_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                audio: AudioSettings::default(),
                keyframes: vec![],
            })],
        }];
        let before = evaluate_project(&project, 64, 64, 10).unwrap_err();
        assert_eq!(before.code, ErrorCode::AssetNotFound);
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
        let mask = catalog["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["accepted"] == true)
            .unwrap()["value"]
            .clone();
        project.tracks[0].items[0].visual_properties_mut().masks =
            vec![serde_json::from_value(mask).unwrap()];
        let after = evaluate_project(&project, 64, 64, 10).unwrap_err();
        assert_eq!(after.code, before.code);
        assert_eq!(after.message, before.message);
    }

    fn empty_project() -> Project {
        Project {
            sound_definitions: Vec::new(),
            audio_buses: Vec::new(),
            markers: Vec::new(),
            fonts: Default::default(),
            components: vec![],
            schema_version: 18, // Historical layout baseline; schema-19 fonts have dedicated fixtures.
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
            audio_bus_id: None,
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
                    visual_properties: crate::VisualProperties::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                }),
                TimelineItem::Media(MediaItem {
                    id: "audio".into(),
                    asset_id: "audio-asset".into(),
                    start_ms: 500,
                    duration_ms: 3_000,
                    source_in_ms: 250,
                    visual_properties: crate::VisualProperties::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                }),
                TimelineItem::Media(MediaItem {
                    id: "video-reuse".into(),
                    asset_id: "video-asset".into(),
                    start_ms: 1_500,
                    duration_ms: 500,
                    source_in_ms: 100,
                    visual_properties: crate::VisualProperties::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                }),
                TimelineItem::Text(TextItem {
                    font_binding: None,
                    id: "title".into(),
                    document: crate::RichTextDocument::plain("Title".into()),
                    text: "Title".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 40,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    visual_properties: crate::VisualProperties::default(),
                    keyframes: vec![],
                }),
            ],
        }];
        for track in &mut project.tracks {
            for (index, item) in track.items.iter_mut().enumerate() {
                item.visual_properties_mut().stack_order = u32::try_from(index).unwrap();
            }
        }
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
                audio_bus_id: None,
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
                        visual_properties: crate::VisualProperties::default(),
                        keyframes: vec![],
                    }),
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
                        keyframes: vec![Keyframe {
                            property: KeyframeProperty::Position,
                            time_ms: 0,
                            value: KeyframeValue::Position { x: 20.0, y: 30.0 },
                            easing: Easing::EaseInOut,
                        }],
                    }),
                    TimelineItem::Text(TextItem {
                        font_binding: None,
                        id: "title".into(),
                        document: crate::RichTextDocument::plain("Evaluated title".into()),
                        text: "Evaluated title".into(),
                        start_ms: 750,
                        duration_ms: 1_500,
                        font_size: 48,
                        color: "#ffffff".into(),
                        font_family: Some("Deterministic Sans".into()),
                        font_path: Some("fonts/deterministic.ttf".into()),
                        style: TextStyle::default(),
                        visual_properties: crate::VisualProperties::default(),
                        keyframes: vec![Keyframe {
                            property: KeyframeProperty::Opacity,
                            time_ms: 0,
                            value: KeyframeValue::Scalar { value: 0.5 },
                            easing: Easing::Linear,
                        }],
                    }),
                    TimelineItem::Media(MediaItem {
                        id: "video-first".into(),
                        asset_id: "video-asset".into(),
                        start_ms: 0,
                        duration_ms: 2_000,
                        source_in_ms: 0,
                        visual_properties: crate::VisualProperties::default(),
                        audio: AudioSettings::default(),
                        keyframes: vec![],
                    }),
                    TimelineItem::Media(MediaItem {
                        id: "video-reuse".into(),
                        asset_id: "video-asset".into(),
                        start_ms: 2_000,
                        duration_ms: 2_000,
                        source_in_ms: 1_000,
                        visual_properties: crate::VisualProperties::default(),
                        audio: AudioSettings::default(),
                        keyframes: vec![],
                    }),
                    TimelineItem::Transition(TransitionItem {
                        id: "panel-title".into(),
                        transition_type: TransitionType::Crossfade,
                        from_item_id: "panel".into(),
                        to_item_id: Some("title".into()),
                        start_ms: 700,
                        duration_ms: 200,
                        visual_properties: crate::VisualProperties::default(),
                    }),
                ],
            },
            Track {
                audio_bus_id: None,
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
                    visual_properties: crate::VisualProperties::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                })],
            },
            Track {
                audio_bus_id: None,
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
                    visual_properties: crate::VisualProperties::default(),
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
                })],
            },
        ];

        for track in &mut project.tracks {
            for (index, item) in track.items.iter_mut().enumerate() {
                item.visual_properties_mut().stack_order = u32::try_from(index).unwrap();
            }
        }
        project.tracks[0].items[0]
            .visual_properties_mut()
            .animation_channels = serde_json::from_value(serde_json::json!([{
            "property": "transform.position_x",
            "keyframes": [
                {"timeMs": 0, "value": {"type": "scalar", "value": 0}, "curve": "linear"},
                {"timeMs": 1000, "value": {"type": "scalar", "value": 100}, "curve": "hold"}
            ]
        }]))
        .unwrap();
        project.tracks[1].items[0]
            .visual_properties_mut()
            .animation_channels = serde_json::from_value(serde_json::json!([{
            "property": "audio.gain_db",
            "keyframes": [
                {"timeMs": 0, "value": {"type": "scalar", "value": -6}, "curve": "linear"},
                {"timeMs": 500, "value": {"type": "scalar", "value": 0}, "curve": "hold"}
            ]
        }]))
        .unwrap();
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
                rich_runs: None,
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
        assert!(plans[0].filter_graph.contains("pow(10"));
        assert!(plans[0].filter_graph.contains("100"));
        assert!(!plans[0].serial_bezier_filters);

        // Independently retain the complete existing scene and role-ducking plan
        // across pre-bus, default-bus and explicitly rerouted generations.
        let mut bus_fixture = project.clone();
        bus_fixture.tracks[0]
            .items
            .retain(|item| !matches!(item, TimelineItem::Text(_) | TimelineItem::Transition(_)));
        for (index, item) in bus_fixture.tracks[0].items.iter_mut().enumerate() {
            item.visual_properties_mut().stack_order = u32::try_from(index).unwrap();
        }
        let bus_baseline = evaluate_project(&bus_fixture, 640, 360, 24).unwrap();
        for generation in ["legacy", "default", "rerouted", "registered"] {
            let mut routed = bus_fixture.clone();
            routed.schema_version = match generation {
                "legacy" => 38,
                "registered" => crate::PROJECT_SCHEMA_VERSION,
                _ => 39,
            };
            routed.audio_buses = if generation == "legacy" {
                vec![]
            } else {
                crate::default_audio_buses()
            };
            if generation == "rerouted" {
                routed.audio_buses[1].output_bus_id = Some("sfx".into());
                routed.tracks[1].audio_bus_id = Some("music".into());
                routed.tracks[2].audio_bus_id = Some("voiceover".into());
            }
            if generation == "registered" {
                for (index, asset) in routed.assets.iter_mut().enumerate() {
                    asset.content_hash = Some(crate::ContentHash {
                        algorithm: "sha256".into(),
                        digest: format!("{:064x}", index + 1),
                    });
                    asset.size_bytes = Some(4096);
                }
                routed.sound_definitions = vec![crate::SoundEventDefinition {
                    event: "impact".into(),
                    variant_asset_ids: vec!["voice-asset".into(), "music-asset".into()],
                    default_gain_db: -120.0,
                    bus_id: "sfx".into(),
                    variant_seed: crate::MAX_SOUND_VARIANT_SEED,
                }];
                routed.resolve_sound_event_variant("impact", None).unwrap();
            }
            let scene = evaluate_project(&routed, 640, 360, 24).unwrap();
            assert!(scene.scene == bus_baseline.scene, "{generation}");
            assert_eq!(
                media_input_requests(&scene).unwrap(),
                media_inputs,
                "{generation}"
            );
            assert!(
                scene.scene.audio_layers[1].ducking.is_some(),
                "{generation}"
            );
            for intent in [
                RenderIntent::Frame { at_ms: 1_500 },
                RenderIntent::Range {
                    start_ms: 500,
                    end_ms: 3_500,
                    include_audio: true,
                },
                RenderIntent::Export,
            ] {
                let mut plan = build_render_plan(
                    &scene.scene,
                    &text,
                    media_inputs.clone(),
                    media_paths.clone(),
                    Some(Path::new("/fonts/deterministic.ttf")),
                    intent,
                    &mut vec![],
                )
                .unwrap();
                plan.intent = RenderIntent::Export;
                let mut expected = build_render_plan(
                    &bus_baseline.scene,
                    &text,
                    media_inputs.clone(),
                    media_paths.clone(),
                    Some(Path::new("/fonts/deterministic.ttf")),
                    intent,
                    &mut vec![],
                )
                .unwrap();
                expected.intent = RenderIntent::Export;
                assert_eq!(plan, expected, "{generation}");
            }
        }

        let curve_channel = |curve: serde_json::Value| {
            serde_json::from_value(serde_json::json!([{
                "property": "transform.position_x", "keyframes": [
                    {"timeMs": 0, "value": {"type": "scalar", "value": 0}, "curve": curve},
                    {"timeMs": 1000, "value": {"type": "scalar", "value": 100}, "curve": "hold"}
                ]
            }]))
            .unwrap()
        };
        for (curve, expected) in [
            (
                serde_json::json!({"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}),
                true,
            ),
            (
                serde_json::json!({"type":"spring","mass":1,"stiffness":100,"damping":20,"initialVelocity":0}),
                false,
            ),
            (serde_json::json!("linear"), false),
        ] {
            project.tracks[0].items[0]
                .visual_properties_mut()
                .animation_channels = curve_channel(curve);
            let curve_scene = evaluate_project(&project, 640, 360, 24).unwrap();
            let mut warnings = Vec::new();
            let plan = build_render_plan(
                &curve_scene.scene,
                &text,
                media_inputs.clone(),
                media_paths.clone(),
                Some(Path::new("/fonts/deterministic.ttf")),
                RenderIntent::Export,
                &mut warnings,
            )
            .unwrap();
            assert_eq!(plan.serial_bezier_filters, expected);
        }
    }

    #[test]
    fn filter_arguments_escape_text_and_paths_without_shell_syntax() {
        assert_eq!(escape_filter("it's: 100%"), "it\\'s\\: 100\\%");
        assert!(!scalar_expression(&[], KeyframeProperty::Scale, 1.0, 0).contains("$"));
        assert_eq!(seconds(1_250), "1.250");
    }
    #[test]
    fn supplied_speech_alignment_never_changes_scene_resources_or_render_plans() {
        let mut project = empty_project();
        project.schema_version = crate::PROJECT_SCHEMA_VERSION;
        project.audio_buses = crate::default_audio_buses();
        project.assets.push(Asset {
            id: "speech".into(),
            media_type: MediaType::Audio,
            file_name: "speech.wav".into(),
            project_relative_path: "assets/speech.wav".into(),
            duration_ms: Some(1000),
            has_audio: true,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        });
        project.tracks.push(Track {
            audio_bus_id: None,
            id: "audio".into(),
            name: "Audio".into(),
            track_type: TrackType::Audio,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Voiceover,
            ducking: None,
            items: vec![TimelineItem::Media(MediaItem {
                id: "speech-item".into(),
                asset_id: "speech".into(),
                start_ms: 0,
                duration_ms: 1000,
                source_in_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                audio: AudioSettings::default(),
                keyframes: vec![],
            })],
        });
        let original = evaluate_project(&project, 64, 64, 10).unwrap();
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/speech-alignment-v1.json"))
                .unwrap();
        for case in catalog["valid"].as_array().unwrap() {
            let generation = crate::SpeechGeneration {
                request: crate::SpeechSynthesisRequest {
                    text: "speech".into(),
                    language: "en".into(),
                    voice_id: crate::SpeechVoiceId("af_heart".into()),
                    speed: 1.0,
                    text_options: crate::SpeechTextOptions::default(),
                },
                provider_id: "synthesis-provider".into(),
                model_id: "synthesis-model".into(),
                model_version: None,
                sample_rate_hz: 24000,
                generated_at_ms: 1,
                alignment: Some(serde_json::from_value(case["alignment"].clone()).unwrap()),
            };
            generation.validate_for_duration(Some(1000)).unwrap();
            project.assets[0].origin =
                Some(crate::GeneratedAssetOrigin::SpeechSynthesis(generation));
            let aligned = evaluate_project(&project, 64, 64, 10).unwrap();
            assert!(aligned.scene == original.scene, "{}", case["id"]);
            let inputs = media_input_requests(&original).unwrap();
            assert_eq!(media_input_requests(&aligned).unwrap(), inputs);
            for intent in [
                RenderIntent::Frame { at_ms: 500 },
                RenderIntent::Export,
                RenderIntent::Range {
                    start_ms: 200,
                    end_ms: 800,
                    include_audio: true,
                },
                RenderIntent::Range {
                    start_ms: 200,
                    end_ms: 800,
                    include_audio: false,
                },
            ] {
                let plan = |scene: &EvaluatedScene| {
                    build_render_plan(
                        scene,
                        &HashMap::new(),
                        inputs.clone(),
                        vec![PathBuf::from("assets/speech.wav")],
                        None,
                        intent,
                        &mut vec![],
                    )
                    .unwrap()
                };
                assert_eq!(
                    plan(&aligned.scene),
                    plan(&original.scene),
                    "{}",
                    case["id"]
                );
            }
        }
    }
}

#[cfg(test)]
mod instance_tempo_tests {
    use super::*;
    #[test]
    fn bounded_tempo_factors_preserve_the_requested_product() {
        for rate in [2f64.powi(-32), 0.5, 0.75, 1.0, 1.5, 2.0, 2f64.powi(32)] {
            let filters = tempo_filters(rate).unwrap();
            let factors = filters
                .split(",atempo=")
                .skip(1)
                .map(|v| v.parse::<f64>().unwrap())
                .collect::<Vec<_>>();
            assert!(factors.len() <= 32);
            assert!(factors.iter().all(|v| (0.5..=2.0).contains(v)));
            assert!((factors.iter().product::<f64>() / rate - 1.0).abs() < 1e-12);
        }
        for rate in [
            0.0,
            -1.0,
            f64::NAN,
            f64::INFINITY,
            2f64.powi(-33),
            2f64.powi(33),
        ] {
            assert!(tempo_filters(rate).is_err());
        }
    }
}
