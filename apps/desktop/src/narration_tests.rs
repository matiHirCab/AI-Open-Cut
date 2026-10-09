#[path = "../../../crates/editor-core/tests/support/narration_fixture.rs"]
mod fixture;
use crate::narration_inspector::{self, Cursor};
use opencut_editor_core::{GeneratedAssetOrigin, Marker, MarkerKind, SpeechTimedText};
use serde_json::json;

#[test]
fn desktop_narration_inspection_reads_exact_provenance_bindings_and_audio_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let p = f.project();
    let before = fixture::inventory(root.path());
    let cursor = Cursor::default();
    assert_eq!(cursor.rows(&p).len(), 6);
    let event = p
        .tracks
        .iter()
        .flat_map(|track| track.items.iter().map(move |item| (track, item)))
        .find(|(_, item)| item.id() == f.aliases["event0"])
        .unwrap();
    let rows = narration_inspector::descriptions(&p, event.0, event.1, &cursor).join("\n");
    assert!(rows.contains("EVERY +0 ms · synchronized start 500 ms"));
    assert!(rows.contains("variant 1 · seed 1 · bus sfx"));
    assert!(rows.contains("Saved speech alignment: absent"));
    let narration=p.tracks.iter().flat_map(|track|track.items.iter().map(move |item|(track,item))).find(|(_,item)|matches!(item,opencut_editor_core::TimelineItem::Media(media) if media.asset_id==f.asset)).unwrap();
    let rows = narration_inspector::descriptions(&p, narration.0, narration.1, &cursor).join("\n");
    assert!(
        rows.contains(
            "Estimated · producer synthetic-narration-fixture · model none · version none"
        )
    );
    assert!(rows.contains("6 sentences · 7 words · 0 phonemes"));
    assert!(rows.contains("asset-relative 500–900 ms: EVERY"));
    assert!(!rows.contains(root.path().to_str().unwrap()));
    let buses = narration_inspector::bus_descriptions(&p).join("\n");
    assert!(buses.contains("4 total"));
    assert!(buses.contains("target -24 LUFS"));
    assert!(buses.contains("voiceover"));
    assert_eq!(fixture::inventory(root.path()), before);
    assert_eq!(f.dir(), f.core.paths().projects_root().join(&f.id));
    assert!(
        p.assets
            .iter()
            .find(|asset| asset.id == f.plain_asset)
            .unwrap()
            .origin
            .is_none()
    );
}

#[test]
fn desktop_narration_pages_scope_identity_and_stale_revision_are_bounded() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let mut p = f.project();
    p.markers = (0..4096)
        .map(|i| Marker {
            id: format!("cue_{i}"),
            name: format!("cue_{i}"),
            scope: "root".into(),
            time_ms: i,
            kind: MarkerKind::Cue,
        })
        .collect();
    let mut cursor = Cursor::default();
    cursor.navigate(&p, None, 1, true);
    assert_eq!(cursor.rows(&p).len(), 32);
    assert_eq!(cursor.rows(&p)[0].time_ms, 32);
    cursor.marker_id = Some("cue_32".into());
    assert_eq!(cursor.selected(&p).unwrap().time_ms, 32);
    p.revision += 1;
    assert!(cursor.selected(&p).is_none());
    cursor.reset();
    assert_eq!(cursor, Cursor::default());
    f.core.edit(&f.id,f.project().revision,fixture::op(json!({"operation":"component_create","name":"empty","width":64,"height":64,"durationMs":6000,"tracks":[]}))).unwrap();
    let mut p = f.project();
    cursor.navigate(&p, None, 0, true);
    assert_eq!(cursor.scope(&p).0, p.components[0].id);
    assert!(cursor.rows(&p).is_empty());
    let scope = format!("component:{}", p.components[0].id);
    p.components[0].markers.push(Marker {
        id: "local".into(),
        name: "EVERY".into(),
        scope,
        time_ms: 10,
        kind: MarkerKind::Cue,
    });
    assert_eq!(cursor.rows(&p)[0].time_ms, 10);
    assert_eq!(p.markers[0].time_ms, 500);
}

#[test]
fn desktop_narration_maximum_alignment_borrows_only_selected_segment() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let mut p = f.project();
    let asset = p
        .assets
        .iter_mut()
        .find(|asset| asset.id == f.asset)
        .unwrap();
    asset.duration_ms = Some(100000);
    asset.probe.as_mut().unwrap().duration_ms = Some(100000);
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = asset.origin.as_mut().unwrap();
    let alignment = generation.alignment.as_mut().unwrap();
    alignment.sentences.clear();
    alignment.words = (0..100000)
        .map(|i| SpeechTimedText {
            text: "x".into(),
            start_ms: i,
            end_ms: i + 1,
        })
        .collect();
    alignment.validate_for_duration(Some(100000)).unwrap();
    let narration=p.tracks.iter().flat_map(|track|track.items.iter().map(move |item|(track,item))).find(|(_,item)|matches!(item,opencut_editor_core::TimelineItem::Media(media) if media.asset_id==f.asset)).unwrap();
    let mut cursor = Cursor {
        granularity: 1,
        segment: 99999,
        ..Default::default()
    };
    let alignment = narration_inspector::alignment(&p, narration.1).unwrap();
    assert!(std::ptr::eq(
        narration_inspector::segments(alignment, 1),
        alignment.words.as_slice()
    ));
    let rows = narration_inspector::descriptions(&p, narration.0, narration.1, &cursor);
    assert!(rows.len() < 12);
    assert!(rows.iter().map(String::len).sum::<usize>() < 1200);
    assert!(
        rows.iter()
            .any(|row| row.contains("100000 / 100000 · asset-relative 99999–100000 ms: x"))
    );
    cursor.navigate(&p, Some(narration.1), 3, true);
    assert_eq!(cursor.segment, 0);
    cursor.navigate(&p, Some(narration.1), 2, true);
    assert!(
        narration_inspector::descriptions(&p, narration.0, narration.1, &cursor)
            .iter()
            .any(|row| row.contains("phoneme: no segments"))
    );
    cursor.reset();
    assert_eq!(cursor.segment, 0);
}
