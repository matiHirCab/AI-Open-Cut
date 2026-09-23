#[path = "support/rules_screen.rs"]
mod fixture;

use fixture::{operation, seed};
use opencut_editor_core::{BatchEditOperation, EditorCore, ErrorCode, ShapeGeometry, TimelineItem};
use serde_json::{Value, json};

fn rgb(value: &Value) -> [u8; 3] {
    ["r", "g", "b"].map(|name| (value[name].as_f64().unwrap() * 255.0).round() as u8)
}

fn files(f: &fixture::Fixture) -> [Vec<u8>; 2] {
    let dir = f.core.paths().project_dir(&f.id).unwrap();
    ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap())
}

#[test]
fn rules_screen_has_independently_expected_native_motifs_and_lifecycle() {
    let root = tempfile::tempdir().unwrap();
    let mut f = seed(root.path());
    let original = f.project();
    assert_eq!(original.settings.width, 1920);
    assert_eq!(original.settings.height, 1080);
    assert_eq!(original.settings.fps, 10);
    assert_eq!(
        original.assets.len(),
        1,
        "only the synthetic tone is imported"
    );
    assert_eq!(original.tracks[1].items.len(), 18);
    let expected_order = [
        "background",
        "grid",
        "card1",
        "accent1",
        "card2",
        "accent2",
        "card3",
        "accent3",
        "brackets",
        "circle1",
        "circle2",
        "circle3",
        "word1_shadow",
        "word1_face",
        "word2_shadow",
        "word2_face",
        "word3_shadow",
        "word3_face",
    ];
    assert_eq!(
        original.tracks[1]
            .items
            .iter()
            .map(|item| item.id())
            .collect::<Vec<_>>(),
        expected_order
            .iter()
            .map(|alias| f.aliases[*alias].as_str())
            .collect::<Vec<_>>()
    );
    let background =
        serde_json::to_value(original.find_item(&f.aliases["background"]).unwrap()).unwrap();
    assert_eq!(rgb(&background["fill"]["color"]), [0x10, 0x19, 0x25]);
    let grid = serde_json::to_value(original.find_item(&f.aliases["grid"]).unwrap()).unwrap();
    assert_eq!(grid["grid"]["pattern"]["type"], "diagonal");
    assert_eq!(grid["grid"]["pattern"]["spacing"], 72.0);
    assert_eq!(
        rgb(&grid["grid"]["pattern"]["stroke"]["paint"]["color"]),
        [0x6E, 0x8D, 0xA6]
    );
    let TimelineItem::Shape(brackets) = original.find_item(&f.aliases["brackets"]).unwrap() else {
        panic!("brackets must be a native path")
    };
    assert!(
        matches!(&brackets.geometry, ShapeGeometry::Path { path } if path.commands.len() == 48)
    );
    let bracket_value = serde_json::to_value(brackets).unwrap();
    assert_eq!(
        bracket_value["geometry"]["path"]["commands"][0]["to"],
        json!({"x":0.0,"y":0.0})
    );
    assert_eq!(
        bracket_value["geometry"]["path"]["commands"][1]["to"],
        json!({"x":42.0,"y":0.0})
    );
    assert_eq!(bracket_value["transform2d"]["position"]["x"], 80.0);
    assert_eq!(bracket_value["transform2d"]["position"]["y"], 68.0);
    assert_eq!(bracket_value["stroke"]["width"], 6.0);
    assert_eq!(
        rgb(&bracket_value["stroke"]["paint"]["color"]),
        [0x46, 0xD4, 0xCC]
    );
    assert_eq!(original.tracks[2].items.len(), 1);
    for (index, x) in [96.0, 710.0, 1324.0].into_iter().enumerate() {
        let n = index + 1;
        let TimelineItem::Shape(card) =
            original.find_item(&f.aliases[&format!("card{n}")]).unwrap()
        else {
            panic!("card must be a native shape")
        };
        assert_eq!(card.visual_properties.transform2d.unwrap().position.x, x);
        assert!(matches!(
            card.geometry,
            ShapeGeometry::RoundedRectangle {
                width: 500.0,
                height: 270.0,
                ..
            }
        ));
        let card_value = serde_json::to_value(card).unwrap();
        assert_eq!(card_value["stroke"]["width"], 5.0);
        assert_eq!(
            rgb(&card_value["stroke"]["paint"]["color"]),
            [0xDC, 0xE9, 0xF2]
        );
        let TimelineItem::Shape(accent) = original
            .find_item(&f.aliases[&format!("accent{n}")])
            .unwrap()
        else {
            panic!("accent must be a native shape")
        };
        assert!(matches!(
            accent.geometry,
            ShapeGeometry::Rectangle {
                width: 14.0,
                height: 188.0
            }
        ));
        let accent_value = serde_json::to_value(accent).unwrap();
        assert_eq!(
            rgb(&accent_value["fill"]["color"]),
            if index == 1 {
                [0xF6, 0xBB, 0x5C]
            } else {
                [0x46, 0xD4, 0xCC]
            }
        );
    }
    for (index, diameter) in [440.0, 320.0, 200.0].into_iter().enumerate() {
        let TimelineItem::Shape(circle) = original
            .find_item(&f.aliases[&format!("circle{}", index + 1)])
            .unwrap()
        else {
            panic!("circle must be native")
        };
        assert!(
            matches!(circle.geometry, ShapeGeometry::Ellipse { width, height } if width == diameter && height == diameter)
        );
    }
    assert!(matches!(
        original.find_item(&f.aliases["grid"]).unwrap(),
        TimelineItem::Grid(_)
    ));
    for (index, word) in ["EVERY.", "SINGLE.", "ONE."].into_iter().enumerate() {
        for layer in ["shadow", "face"] {
            let TimelineItem::Text(item) = original
                .find_item(&f.aliases[&format!("word{}_{}", index + 1, layer)])
                .unwrap()
            else {
                panic!("word must be native text")
            };
            assert_eq!(item.text, word);
            assert_eq!(item.document.text(), word);
            assert_eq!(
                item.color,
                if layer == "face" {
                    "#FFFFFF"
                } else {
                    "#235767"
                }
            );
        }
    }
    let word_id = &f.aliases["word1_face"];
    let TimelineItem::Text(item) = original.find_item(word_id).unwrap() else {
        panic!()
    };
    assert_eq!(item.font_size, 132, "batch alias update must apply");
    f.resize_impact_word();
    let edited = f.project();
    let TimelineItem::Text(item) = edited.find_item(word_id).unwrap() else {
        panic!()
    };
    assert_eq!(item.font_size, 144);
    f.core.undo(&f.id, edited.revision).unwrap();
    assert_eq!(
        serde_json::to_value(f.project().tracks).unwrap(),
        serde_json::to_value(&original.tracks).unwrap()
    );
    f.core.redo(&f.id, f.project().revision).unwrap();
    f.core = EditorCore::new(f.core.paths().clone());
    assert_eq!(
        serde_json::to_value(f.project().tracks).unwrap(),
        serde_json::to_value(&edited.tracks).unwrap()
    );
}

#[test]
fn rules_screen_failures_preserve_project_and_history() {
    let root = tempfile::tempdir().unwrap();
    let f = seed(root.path());
    let before = files(&f);
    let revision = f.project().revision;
    for (request, code) in [
        (
            json!({"operation":"update_item","itemId":f.aliases["word1_face"],"fontSize":0}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":f.aliases["card1"],"fontSize":48}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":"missing","fontSize":48}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        assert_eq!(
            f.core
                .edit(&f.id, revision, operation(request))
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(files(&f), before);
    }
    assert_eq!(f.core.edit(&f.id, revision - 1, operation(json!({"operation":"update_item","itemId":f.aliases["word1_face"],"fontSize":144}))).unwrap_err().code, ErrorCode::RevisionConflict);
    assert_eq!(files(&f), before);
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"update_item","itemId":f.aliases["word1_face"],"fontSize":144},
        {"operation":"delete_item","itemId":"missing"}
    ]))
    .unwrap();
    assert_eq!(
        f.core.edit_batch(&f.id, revision, batch).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(files(&f), before);
    let track = f.project().tracks[1].id.clone();
    f.core
        .edit(
            &f.id,
            revision,
            operation(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
    let locked = files(&f);
    assert_eq!(f.core.edit(&f.id, revision + 1, operation(json!({"operation":"update_item","itemId":f.aliases["word1_face"],"fontSize":144}))).unwrap_err().code, ErrorCode::TrackLocked);
    assert_eq!(files(&f), locked);
}

#[test]
fn rules_screen_scaled_variants_keep_every_motif() {
    for (width, height) in [(1280, 720), (960, 540)] {
        let root = tempfile::tempdir().unwrap();
        let f = fixture::seed_at(root.path(), width, height);
        assert_eq!((f.width, f.height), (width, height));
        let project = f.project();
        assert_eq!(
            (project.settings.width, project.settings.height),
            (width, height)
        );
        assert_eq!(project.tracks[1].items.len(), 18);
        let TimelineItem::Shape(card) = project.find_item(&f.aliases["card1"]).unwrap() else {
            panic!()
        };
        assert_eq!(
            card.visual_properties.transform2d.unwrap().position.x,
            96.0 * f64::from(width) / 1920.0
        );
        f.resize_impact_word();
        assert_eq!(f.project().revision, project.revision + 1);
    }
}
