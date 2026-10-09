//! Bounded presentation of saved narration state. No inference or domain resolution.
use opencut_editor_core::{
    GeneratedAssetOrigin, Marker, Project, SpeechAlignment, SpeechTimedText, TimeExpression,
    TimelineItem, Track,
};

pub(crate) const MARKER_PAGE: usize = 32;
pub(crate) const BUS_PAGE: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub scope: usize,
    pub marker_page: usize,
    pub marker_id: Option<String>,
    pub revision: Option<u64>,
    pub granularity: usize,
    pub segment: usize,
}

impl Cursor {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn scope<'a>(&self, project: &'a Project) -> (&'a str, &'a [Marker]) {
        self.scope
            .checked_sub(1)
            .and_then(|index| project.components.get(index))
            .map_or(("root", project.markers.as_slice()), |component| {
                (component.id.as_str(), component.markers.as_slice())
            })
    }

    pub fn rows<'a>(&self, project: &'a Project) -> &'a [Marker] {
        let (_, markers) = self.scope(project);
        let page = self
            .marker_page
            .min(markers.len().saturating_sub(1) / MARKER_PAGE);
        let start = page * MARKER_PAGE;
        &markers[start..(start + MARKER_PAGE).min(markers.len())]
    }

    pub fn selected<'a>(&self, project: &'a Project) -> Option<&'a Marker> {
        (self.revision == Some(project.revision))
            .then(|| {
                self.scope(project)
                    .1
                    .iter()
                    .find(|marker| Some(marker.id.as_str()) == self.marker_id.as_deref())
            })
            .flatten()
    }

    pub fn navigate(
        &mut self,
        project: &Project,
        item: Option<&TimelineItem>,
        axis: u8,
        next: bool,
    ) {
        // Navigation changes only presentation cursors over already validated collections.
        match axis {
            0 => {
                self.scope = step(self.scope, project.components.len() + 1, next);
                self.marker_page = 0;
                self.marker_id = None;
            }
            1 => {
                self.marker_page = step(
                    self.marker_page,
                    self.scope(project).1.len().div_ceil(MARKER_PAGE),
                    next,
                );
                self.marker_id = None;
            }
            2 => {
                self.granularity = step(self.granularity, 3, next);
                self.segment = 0;
            }
            3 => {
                let count = item
                    .and_then(|item| alignment(project, item))
                    .map_or(0, |alignment| segments(alignment, self.granularity).len());
                self.segment = step(self.segment, count, next);
            }
            _ => {}
        }
        self.revision = Some(project.revision);
    }
}

fn step(current: usize, count: usize, next: bool) -> usize {
    if count == 0 {
        return 0;
    }
    let current = current.min(count - 1);
    if next {
        (current + 1) % count
    } else {
        (current + count - 1) % count
    }
}

pub(crate) fn alignment<'a>(
    project: &'a Project,
    item: &TimelineItem,
) -> Option<&'a SpeechAlignment> {
    let TimelineItem::Media(media) = item else {
        return None;
    };
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.id == media.asset_id)?;
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = asset.origin.as_ref()?;
    generation.alignment.as_ref()
}

pub(crate) fn segments(alignment: &SpeechAlignment, granularity: usize) -> &[SpeechTimedText] {
    match granularity {
        1 => &alignment.words,
        2 => &alignment.phonemes,
        _ => &alignment.sentences,
    }
}

pub(crate) fn descriptions(
    project: &Project,
    track: &Track,
    item: &TimelineItem,
    cursor: &Cursor,
) -> Vec<String> {
    let mut rows = vec!["Narration timing · authored values".to_owned()];
    rows.push(match &item.visual_properties().start_time {
        Some(TimeExpression::Marker {
            marker_name,
            offset_ms,
        }) => format!(
            "Marker binding: {marker_name} {offset_ms:+} ms · synchronized start {} ms",
            item.start_ms()
        ),
        _ => format!("Numeric start: {} ms", item.start_ms()),
    });
    let TimelineItem::Media(media) = item else {
        return rows;
    };
    rows.push(format!(
        "Audio: volume {} · muted {} · fade in {} ms · fade out {} ms",
        media.audio.volume, media.audio.muted, media.audio.fade_in_ms, media.audio.fade_out_ms
    ));
    rows.push(format!(
        "Track audio: role {:?} · authored bus {} · muted {}",
        track.audio_role,
        track.audio_bus_id.as_deref().unwrap_or("role default"),
        track.muted
    ));
    if let Some(ducking) = &track.ducking {
        rows.push(format!("Track role ducking: {ducking:?}"));
    }
    if let Some(event) = &media.audio_event {
        rows.push(format!(
            "Captured event: {} · variant {} · seed {} · bus {} · default gain {} dB · gain {} dB",
            event.event,
            event.variant_index,
            event.variant_seed,
            event.bus_id,
            event.default_gain_db,
            event.gain_db
        ));
        rows.push(format!(
            "Captured content: {}:{}",
            event.content_hash.algorithm, event.content_hash.digest
        ));
    }
    let Some(alignment) = alignment(project, item) else {
        rows.push("Saved speech alignment: absent".into());
        return rows;
    };
    rows.push(format!(
        "Saved alignment: {:?} · producer {} · model {} · version {}",
        alignment.quality,
        alignment.provider_id,
        alignment.model_id.as_deref().unwrap_or("none"),
        alignment.model_version.as_deref().unwrap_or("none")
    ));
    rows.push(format!(
        "Alignment counts: {} sentences · {} words · {} phonemes",
        alignment.sentences.len(),
        alignment.words.len(),
        alignment.phonemes.len()
    ));
    let segments = segments(alignment, cursor.granularity);
    let name = ["sentence", "word", "phoneme"][cursor.granularity.min(2)];
    if let Some(segment) = segments.get(cursor.segment.min(segments.len().saturating_sub(1))) {
        rows.push(format!(
            "Selected {name} {} / {} · asset-relative {}–{} ms: {}",
            cursor.segment.min(segments.len() - 1) + 1,
            segments.len(),
            segment.start_ms,
            segment.end_ms,
            segment.text
        ));
    } else {
        rows.push(format!("Selected {name}: no segments"));
    }
    rows
}

pub(crate) fn bus_descriptions(project: &Project) -> Vec<String> {
    let mut rows = vec![format!(
        "Audio buses · authored settings · {} total",
        project.audio_buses.len()
    )];
    // Core currently permits exactly four built-in buses. Keep the presentation bound explicit.
    for bus in project.audio_buses.iter().take(BUS_PAGE) {
        rows.push(format!(
            "{} → {}",
            bus.id,
            bus.output_bus_id.as_deref().unwrap_or("output")
        ));
        if let Some(dsp) = &bus.dsp {
            rows.push(format!("Gain {} dB · pan {}", dsp.gain_db, dsp.pan));
            rows.push(format!("EQ: {} bands", dsp.eq.len()));
            for band in &dsp.eq {
                rows.push(format!(
                    "{} Hz · Q {} · {} dB",
                    band.frequency_hz, band.q, band.gain_db
                ));
            }
            if let Some(compressor) = &dsp.compressor {
                rows.push(format!(
                    "Compressor: threshold {} dB",
                    compressor.threshold_db
                ));
                rows.push(format!(
                    "Ratio {} · makeup {} dB",
                    compressor.ratio, compressor.makeup_gain_db
                ));
                rows.push(format!(
                    "Attack {} ms · release {} ms",
                    compressor.attack_ms, compressor.release_ms
                ));
            } else {
                rows.push("Compressor: absent".into());
            }
        } else {
            rows.push("DSP: absent".into());
        }
        if let Some(ducking) = &bus.ducking {
            rows.push(format!("Ducking: enabled {}", ducking.enabled));
            rows.push(format!(
                "Source {} · gain {}",
                ducking.source_bus_id, ducking.gain
            ));
            rows.push(format!(
                "Attack {} ms · release {} ms",
                ducking.attack_ms, ducking.release_ms
            ));
        } else {
            rows.push("Ducking: absent".into());
        }
    }
    if let Some(value) = &project.master_normalization {
        rows.push(format!("Master normalization: enabled {}", value.enabled));
        rows.push(format!("target {} LUFS", value.target_integrated_lufs));
        rows.push(format!(
            "Range {} LU · ceiling {} dBTP",
            value.target_loudness_range_lu, value.target_true_peak_dbtp
        ));
    } else {
        rows.push("Master normalization: absent".into());
    }
    rows
}
