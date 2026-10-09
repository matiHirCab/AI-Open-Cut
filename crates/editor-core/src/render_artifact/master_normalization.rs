//! Select already admitted audio occurrences without re-reading project records.
use super::{PreparedMediaResources, audio_analysis::PreparedAudioMedia};
use crate::evaluated_scene::EvaluatedScene;

pub(crate) fn select_audio(
    scene: &EvaluatedScene,
    media: &PreparedMediaResources,
) -> PreparedAudioMedia {
    let mut inputs = Vec::new();
    let mut paths = Vec::new();
    for (input, path) in media.media_inputs.iter().zip(&media.media_paths) {
        if scene
            .audio_layers
            .iter()
            .any(|layer| layer.item_id == input.item_id)
        {
            let mut input = input.clone();
            input.input_index = inputs.len() + 2;
            inputs.push(input);
            paths.push(path.clone());
        }
    }
    PreparedAudioMedia { inputs, paths }
}
