use std::{fs, path::PathBuf};

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_source(relative: &str) -> String {
    fs::read_to_string(source_root().join(relative))
        .unwrap_or_else(|error| panic!("cannot read editor-core source {relative}: {error}"))
}

#[test]
fn required_owners_are_private_modules() {
    let lib = read_source("lib.rs");
    for owner in [
        "assets",
        "drafts",
        "migrations",
        "persistence",
        "render_artifact",
        "render_plan",
        "render_process",
        "timeline",
        "validation",
    ] {
        assert!(
            lib.contains(&format!("mod {owner};")),
            "ADR 0003 owner `{owner}` must be declared as a private module"
        );
        assert!(
            !lib.contains(&format!("pub mod {owner};")),
            "ADR 0003 owner `{owner}` must remain behind the stable facade"
        );
    }
}

#[test]
fn stable_facade_remains_reexported() {
    let lib = read_source("lib.rs");
    for public_name in [
        "EditorCore",
        "WriteResult",
        "EditDraft",
        "Renderer",
        "ExportOptions",
        "PreviewRangeOptions",
        "RenderArtifact",
        "RenderProgress",
    ] {
        assert!(
            lib.contains(public_name),
            "stable editor-core facade lost `{public_name}`"
        );
    }
}

#[test]
fn inward_modules_do_not_import_outer_or_infrastructure_layers() {
    let forbidden_outer = [
        "opencut_agent_bridge",
        "opencut_headless",
        "opencut_desktop",
        "@modelcontextprotocol",
    ];
    for owner in [
        "model.rs",
        "validation.rs",
        "timeline.rs",
        "assets.rs",
        "migrations.rs",
        "render_plan.rs",
    ] {
        let source = read_source(owner);
        for token in forbidden_outer {
            assert!(
                !source.contains(token),
                "ADR 0003 forbids `{token}` in inward owner `{owner}`"
            );
        }
    }

    let render_plan = read_source("render_plan.rs");
    for token in [
        "std::process",
        "Command::new",
        "std::env",
        "render_process",
        "render_artifact",
    ] {
        assert!(
            !render_plan.contains(token),
            "render planning must not depend on execution/publication token `{token}`"
        );
    }

    let persistence = read_source("persistence.rs");
    for token in ["crate::timeline", "crate::renderer", "crate::render_"] {
        assert!(
            !persistence.contains(token),
            "persistence must not depend on `{token}`"
        );
    }

    let assets = read_source("assets.rs");
    let assets_production = assets.split("#[cfg(test)]").next().unwrap_or(&assets);
    for token in ["File::open", "std::fs::copy", "std::fs::rename"] {
        assert!(
            !assets_production.contains(token),
            "managed asset I/O must use the persistence port, not `{token}`"
        );
    }

    let store = read_source("store.rs");
    for token in ["struct ProjectTransaction", "fn recover_transaction("] {
        assert!(
            !store.contains(token),
            "persistence orchestration must not be reimplemented in store via `{token}`"
        );
    }

    let renderer = read_source("renderer.rs");
    for token in ["fn prepare_text_layers(", "fn resolve_text_font("] {
        assert!(
            !renderer.contains(token),
            "artifact preparation must not be reimplemented in renderer via `{token}`"
        );
    }
}
