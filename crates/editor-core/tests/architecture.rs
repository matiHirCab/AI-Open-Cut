use std::{collections::BTreeSet, fs, path::PathBuf};

const OWNER_MATRIX: &[(&str, &[&str])] = &[
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

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn mask_non_code(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut masked = bytes.to_vec();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                masked[index] = b' ';
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            let mut depth = 0;
            while index < bytes.len() {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    masked[index] = b' ';
                    masked[index + 1] = b' ';
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    masked[index] = b' ';
                    masked[index + 1] = b' ';
                    index += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if bytes[index] != b'\n' {
                        masked[index] = b' ';
                    }
                    index += 1;
                }
            }
        } else if bytes[index] == b'"' {
            masked[index] = b' ';
            index += 1;
            while index < bytes.len() {
                let byte = bytes[index];
                if byte != b'\n' {
                    masked[index] = b' ';
                }
                index += 1;
                if byte == b'\\' && index < bytes.len() {
                    if bytes[index] != b'\n' {
                        masked[index] = b' ';
                    }
                    index += 1;
                } else if byte == b'"' {
                    break;
                }
            }
        } else {
            index += 1;
        }
    }

    String::from_utf8(masked).expect("masking Rust source must preserve UTF-8")
}

fn crate_import_trees(source: &str) -> Vec<String> {
    let source = mask_non_code(source);
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index..].starts_with(b"use")
            && (index == 0 || !is_identifier_byte(bytes[index - 1]))
            && (index + 3 == bytes.len() || !is_identifier_byte(bytes[index + 3]))
        {
            let mut cursor = index + 3;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes[cursor..].starts_with(b"crate")
                && (cursor + 5 == bytes.len() || !is_identifier_byte(bytes[cursor + 5]))
            {
                cursor += 5;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if bytes[cursor..].starts_with(b"::") {
                    cursor += 2;
                    let tree_start = cursor;
                    while cursor < bytes.len() && bytes[cursor] != b';' {
                        cursor += 1;
                    }
                    trees.push(source[tree_start..cursor].to_owned());
                    index = cursor;
                }
            }
        }
        index += 1;
    }

    trees
}

fn first_identifier<'a>(source: &'a str, index: &mut usize) -> Option<&'a str> {
    let bytes = source.as_bytes();
    while *index < bytes.len() && bytes[*index].is_ascii_whitespace() {
        *index += 1;
    }
    let start = *index;
    while *index < bytes.len() && is_identifier_byte(bytes[*index]) {
        *index += 1;
    }
    (start != *index).then_some(&source[start..*index])
}

fn root_imports(tree: &str) -> Vec<&str> {
    let bytes = tree.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    if bytes.get(index) != Some(&b'{') {
        return first_identifier(tree, &mut index).into_iter().collect();
    }

    index += 1;
    let mut depth = 1;
    let mut expects_root = true;
    let mut roots = Vec::new();
    while index < bytes.len() && depth > 0 {
        match bytes[index] {
            b'{' => {
                depth += 1;
                index += 1;
            }
            b'}' => {
                depth -= 1;
                index += 1;
            }
            b',' if depth == 1 => {
                expects_root = true;
                index += 1;
            }
            _ if depth == 1 && expects_root => {
                if let Some(root) = first_identifier(tree, &mut index) {
                    roots.push(root);
                    expects_root = false;
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }
    roots
}

fn known_owner(candidate: &str) -> Option<&'static str> {
    OWNER_MATRIX
        .iter()
        .map(|(owner, _)| *owner)
        .find(|owner| *owner == candidate)
}

fn imported_owner_dependencies(source: &str) -> BTreeSet<&'static str> {
    let mut dependencies = BTreeSet::new();
    for tree in crate_import_trees(source) {
        for candidate in root_imports(&tree) {
            if let Some(owner) = known_owner(candidate) {
                dependencies.insert(owner);
            }
        }
    }
    dependencies
}

fn qualified_owner_dependencies(source: &str) -> BTreeSet<&'static str> {
    let source = mask_non_code(source);
    let bytes = source.as_bytes();
    let mut dependencies = BTreeSet::new();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index..].starts_with(b"crate")
            && (index == 0 || !is_identifier_byte(bytes[index - 1]))
            && (index + 5 == bytes.len() || !is_identifier_byte(bytes[index + 5]))
        {
            let mut cursor = index + 5;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes[cursor..].starts_with(b"::") {
                cursor += 2;
                if let Some(candidate) = first_identifier(&source, &mut cursor)
                    && let Some(owner) = known_owner(candidate)
                {
                    dependencies.insert(owner);
                }
            }
        }
        index += 1;
    }

    dependencies
}

fn referenced_owner_dependencies(source: &str) -> BTreeSet<&'static str> {
    let mut dependencies = imported_owner_dependencies(source);
    dependencies.extend(qualified_owner_dependencies(source));
    dependencies
}

fn validate_owner_dependencies(owner: &str, allowed: &[&str], source: &str) -> Result<(), String> {
    for dependency in referenced_owner_dependencies(source) {
        if dependency != owner && !allowed.contains(&dependency) {
            return Err(format!(
                "owner `{owner}` imports `{dependency}`, but the ADR matrix allows only {allowed:?}"
            ));
        }
    }
    Ok(())
}

#[test]
fn crate_local_owner_dependencies_are_extracted_from_supported_forms() {
    let source = r#"
        use crate::persistence;
        use crate::render_plan as plan;
        use crate::{assets, drafts as pending};
        use crate::{
            render_artifact::{self, ArtifactIo},
            render_process as process,
        };
        use crate::persistence_cache::Cache;
        crate::validation::validate();
        crate::validation_rules::validate();
        // use crate::timeline;
        let example = "use crate::model;";
    "#;

    assert_eq!(
        referenced_owner_dependencies(source),
        [
            "assets",
            "drafts",
            "persistence",
            "render_artifact",
            "render_plan",
            "render_process",
            "validation",
        ]
        .into_iter()
        .collect()
    );
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
    for (owner, allowed) in OWNER_MATRIX {
        let source = read_source(&format!("{owner}.rs"));
        let production = production_source(&source);
        validate_owner_dependencies(owner, allowed, production)
            .unwrap_or_else(|message| panic!("{message}"));
    }
}

#[test]
fn dependency_violation_diagnostics_name_the_boundary() {
    let message =
        validate_owner_dependencies("validation", &[], "use crate::{persistence as storage};")
            .expect_err("the aliased persistence dependency must be forbidden");

    assert!(message.contains("owner `validation`"));
    assert!(message.contains("imports `persistence`"));
    assert!(message.contains("allows only []"));
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
