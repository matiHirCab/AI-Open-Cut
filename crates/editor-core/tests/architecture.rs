use std::{fs, path::PathBuf};

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_source(relative: &str) -> String {
    fs::read_to_string(source_root().join(relative))
        .unwrap_or_else(|error| panic!("cannot read editor-core source {relative}: {error}"))
}

fn production_source(source: &str) -> &str {
    let Some(module_index) = source.find("mod tests {") else {
        return source;
    };
    let test_cfg_index = source[..module_index]
        .rfind("#[cfg(test)]")
        .expect("test module must have a cfg(test) guard");
    &source[..test_cfg_index]
}

#[test]
fn required_owners_are_private_modules() {
    let lib = read_source("lib.rs");
    for owner in [
        "animation",
        "assets",
        "drafts",
        "error",
        "migrations",
        "model",
        "path_policy",
        "persistence",
        "render_artifact",
        "render_plan",
        "render_process",
        "renderer",
        "store",
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
fn private_owner_dependencies_match_the_approved_matrix() {
    let matrix: &[(&str, &[&str])] = &[
        ("animation", &[]),
        ("assets", &["persistence"]),
        ("drafts", &["persistence"]),
        ("error", &[]),
        ("migrations", &[]),
        ("model", &["error"]),
        ("path_policy", &[]),
        ("persistence", &[]),
        ("render_artifact", &["render_plan"]),
        ("render_plan", &["animation"]),
        ("render_process", &["render_plan"]),
        (
            "renderer",
            &["render_artifact", "render_plan", "render_process"],
        ),
        (
            "store",
            &[
                "assets",
                "drafts",
                "migrations",
                "persistence",
                "timeline",
                "validation",
            ],
        ),
        ("timeline", &["animation", "validation"]),
        ("validation", &[]),
    ];
    for (owner, allowed) in matrix {
        let source = read_source(&format!("{owner}.rs"));
        let production = production_source(&source);
        for (dependency, _) in matrix {
            if owner == dependency {
                continue;
            }
            let imported = production.contains(&format!("{dependency}::"));
            assert!(
                !imported || allowed.contains(dependency),
                "owner `{owner}` imports `{dependency}`, but the ADR matrix allows only {allowed:?}"
            );
        }
    }
}

#[test]
fn owners_exclude_outer_layers_and_reviewed_responsibilities() {
    let forbidden_outer = [
        "opencut_agent_bridge",
        "opencut_headless",
        "opencut_desktop",
        "@modelcontextprotocol",
    ];
    for owner in [
        "animation.rs",
        "assets.rs",
        "drafts.rs",
        "error.rs",
        "migrations.rs",
        "model.rs",
        "path_policy.rs",
        "persistence.rs",
        "render_artifact.rs",
        "render_plan.rs",
        "render_process.rs",
        "renderer.rs",
        "store.rs",
        "timeline.rs",
        "validation.rs",
    ] {
        let source = read_source(owner);
        let source = production_source(&source);
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
    let assets_production = production_source(&assets);
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
    let renderer = production_source(&renderer);
    for token in [
        "fn prepare_text_layers(",
        "fn resolve_text_font(",
        ".tracks",
        "TimelineItem",
        "Command::new",
        "std::process",
    ] {
        assert!(
            !renderer.contains(token),
            "artifact preparation must not be reimplemented in renderer via `{token}`"
        );
    }

    let store = read_source("store.rs");
    let store = production_source(&store);
    for token in ["fn garbage_collect(", "fn collect_files("] {
        assert!(
            !store.contains(token),
            "store must delegate managed-asset collection instead of owning `{token}`"
        );
    }
}
