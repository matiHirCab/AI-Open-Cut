//! Pure lowering of immutable evaluated bus facts, never persisted DSP records.
use crate::{
    CoreError, ErrorCode,
    evaluated_scene::{EvaluatedScene, audio_bus_dsp::EvaluatedBusGraph},
};

pub(super) fn compile(
    filters: &mut Vec<String>,
    labels: &[String],
    scene: &EvaluatedScene,
    graph: &EvaluatedBusGraph,
) -> Result<(), CoreError> {
    let mut inputs: [Vec<String>; 4] = std::array::from_fn(|_| Vec::new());
    for (i, (layer, label)) in scene.audio_layers.iter().zip(&labels[1..]).enumerate() {
        let bus = layer.bus_index.filter(|v| *v < 4).ok_or_else(|| {
            CoreError::new(
                ErrorCode::InvalidArgument,
                "missing evaluated audio bus route",
            )
        })?;
        let normalized = format!("dspinput{i}");
        filters.push(format!("{label}aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo[{normalized}]"));
        inputs[bus].push(format!("[{normalized}]"));
    }
    // The complete timeline's silence anchor keeps a master clock and detector
    // state across gaps without creating disconnected, unused bus streams.
    inputs[3].insert(0, labels[0].clone());
    for bus in &graph.buses {
        let sources = &inputs[bus.index];
        if sources.is_empty() {
            continue;
        }
        let mut effect = format!(
            "{}amix=inputs={}:duration=longest:normalize=0,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,volume={:.6}",
            sources.join(""),
            sources.len(),
            bus.linear_gain
        );
        for band in &bus.eq {
            effect.push_str(&format!(
                ",equalizer=f={:.6}:t=q:w={:.6}:g={:.6}",
                band.frequency_hz, band.q, band.gain_db
            ));
        }
        if let Some(c) = &bus.compressor {
            effect.push_str(&format!(",acompressor=threshold={:.6}:ratio={:.6}:attack={:.6}:release={:.6}:makeup={:.6}:mode=downward:detection=peak:link=maximum:knee=1:mix=1:level_in=1",c.threshold,c.ratio,c.attack_ms,c.release_ms,c.makeup));
        }
        if let Some(segments) = &bus.ducking {
            effect.push_str(&format!(
                ",volume='{}':eval=frame",
                super::audio_bus_ducking::expression(segments)
            ));
        }
        effect.push_str(&format!(
            ",pan=stereo|c0={:.6}*c0|c1={:.6}*c1",
            bus.balance[0], bus.balance[1]
        ));
        let destination = if bus.output.is_some() {
            format!("dspbus{}", bus.index)
        } else {
            "audio".to_owned()
        };
        filters.push(format!("{effect}[{destination}]"));
        if let Some(output) = bus.output {
            inputs[output].push(format!("[{destination}]"));
        }
    }
    Ok(())
}
