//! Canonical bounded audio routing facts. No process or filter expressions.
use super::{EvaluatedScene, invalid};
use crate::{CoreError, Project};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedEqBand {
    pub(crate) frequency_hz: f64,
    pub(crate) q: f64,
    pub(crate) gain_db: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedCompressor {
    pub(crate) threshold: f64,
    pub(crate) ratio: f64,
    pub(crate) attack_ms: f64,
    pub(crate) release_ms: f64,
    pub(crate) makeup: f64,
}
#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedBus {
    pub(crate) index: usize,
    pub(crate) output: Option<usize>,
    pub(crate) linear_gain: f64,
    pub(crate) balance: [f64; 2],
    pub(crate) eq: Vec<EvaluatedEqBand>,
    pub(crate) compressor: Option<EvaluatedCompressor>,
    pub(crate) ducking: Option<Vec<super::audio_bus_ducking::EnvelopeSegment>>,
}
impl std::fmt::Debug for EvaluatedBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = f.debug_struct("EvaluatedBus");
        value
            .field("index", &self.index)
            .field("output", &self.output)
            .field("linear_gain", &self.linear_gain)
            .field("balance", &self.balance)
            .field("eq", &self.eq)
            .field("compressor", &self.compressor);
        if let Some(ducking) = &self.ducking {
            value.field("ducking", ducking);
        }
        value.finish()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedBusGraph {
    /// Child-before-parent, ties in the fixed canonical bus order.
    pub(crate) buses: Vec<EvaluatedBus>,
}
impl EvaluatedBusGraph {
    pub(crate) fn requires_dsp(&self) -> bool {
        self.buses.iter().any(|b| {
            b.linear_gain != 1.0
                || b.balance != [1.0, 1.0]
                || !b.eq.is_empty()
                || b.compressor.is_some()
        })
    }
    pub(crate) fn heap_bytes(&self) -> u64 {
        (self.buses.capacity() * std::mem::size_of::<EvaluatedBus>()
            + self
                .buses
                .iter()
                .map(|b| {
                    b.eq.capacity() * std::mem::size_of::<EvaluatedEqBand>()
                        + b.ducking.as_ref().map_or(0, |segments| {
                            segments.capacity()
                                * std::mem::size_of::<super::audio_bus_ducking::EnvelopeSegment>()
                        })
                })
                .sum::<usize>()) as u64
    }
}

pub(crate) fn finalize(project: &Project, scene: &mut EvaluatedScene) -> Result<(), CoreError> {
    // Omission retains the complete legacy direct-render path, including native
    // historical fixtures. Persistence validates routing independently; only
    // authored DSP introduces the DSP model/render guards here.
    if project
        .audio_buses
        .iter()
        .all(|bus| bus.dsp.is_none() && bus.ducking.is_none())
    {
        return Ok(());
    }
    project.validate_audio_bus_model()?;
    let mut reachable = [false; 4];
    let mut audible_reachable = [false; 4];
    for audio in &scene.audio_layers {
        let Some(mut index) = audio.bus_index else {
            continue;
        };
        for _ in 0..4 {
            let bus = project
                .audio_buses
                .get(index)
                .ok_or_else(|| invalid("evaluated audio bus was not found"))?;
            reachable[index] = true;
            // Item gain multiplies every automation control. A zero item gain
            // cannot activate DSP, but its stream must remain connected when
            // another potentially audible input activates the bus graph.
            audible_reachable[index] |= audio.volume > 0.0;
            match &bus.output_bus_id {
                Some(id) => {
                    index = crate::AUDIO_BUS_IDS
                        .iter()
                        .position(|v| v == id)
                        .ok_or_else(|| invalid("evaluated audio bus output was not found"))?
                }
                None => break,
            }
        }
    }
    let mut ducking = super::audio_bus_ducking::prepare(project, scene, &audible_reachable)?;
    if ducking.iter().all(Vec::is_empty)
        && !project
            .audio_buses
            .iter()
            .enumerate()
            .any(|(i, b)| audible_reachable[i] && b.dsp.as_ref().is_some_and(|d| !d.is_identity()))
    {
        for audio in &mut scene.audio_layers {
            audio.bus_index = None;
        }
        return Ok(());
    }
    let mut order = Vec::with_capacity(4);
    for (index, bus) in project.audio_buses.iter().enumerate() {
        if !reachable[index] {
            continue;
        }
        let mut depth = 0;
        let mut current = bus;
        while let Some(id) = &current.output_bus_id {
            current = project
                .audio_buses
                .iter()
                .find(|b| &b.id == id)
                .ok_or_else(|| invalid("evaluated audio route was not found"))?;
            depth += 1;
        }
        order.push((index, depth));
    }
    order.sort_by_key(|&(index, depth)| (std::cmp::Reverse(depth), index));
    let mut buses = Vec::with_capacity(order.len());
    for (index, _) in order {
        let bus = &project.audio_buses[index];
        let dsp = bus.dsp.as_ref();
        let pan = dsp.map_or(0.0, |d| d.pan);
        buses.push(EvaluatedBus {
            index,
            output: bus
                .output_bus_id
                .as_ref()
                .and_then(|id| crate::AUDIO_BUS_IDS.iter().position(|v| v == id)),
            linear_gain: dsp.map_or(1.0, |d| 10_f64.powf(d.gain_db / 20.0)),
            balance: [
                if pan > 0.0 { 1.0 - pan } else { 1.0 },
                if pan < 0.0 { 1.0 + pan } else { 1.0 },
            ],
            eq: dsp.map_or_else(Vec::new, |d| {
                d.eq.iter()
                    .filter(|b| b.gain_db != 0.0)
                    .map(|b| EvaluatedEqBand {
                        frequency_hz: b.frequency_hz,
                        q: b.q,
                        gain_db: b.gain_db,
                    })
                    .collect()
            }),
            ducking: (!ducking[index].is_empty()).then(|| std::mem::take(&mut ducking[index])),
            compressor: dsp
                .and_then(|d| d.compressor.as_ref())
                .map(|c| EvaluatedCompressor {
                    threshold: 10_f64.powf(c.threshold_db / 20.0),
                    ratio: c.ratio,
                    attack_ms: c.attack_ms,
                    release_ms: c.release_ms,
                    makeup: 10_f64.powf(c.makeup_gain_db / 20.0),
                }),
        });
    }
    scene.audio_bus_graph = Some(EvaluatedBusGraph { buses });
    Ok(())
}
