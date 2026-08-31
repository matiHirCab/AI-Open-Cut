use std::{collections::BTreeSet, fs, path::PathBuf};

use syn::{
    Attribute, Block, ExprField, Field, ForeignItem, ImplItem, Item, ItemFn, ItemMod, ItemStruct,
    LitStr, Member, Meta, Path, Stmt, Token, TraitItem, UseTree, Variant, punctuated::Punctuated,
    visit, visit::Visit,
};

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

fn known_owner(candidate: &str) -> Option<&'static str> {
    OWNER_MATRIX
        .iter()
        .map(|(owner, _)| *owner)
        .find(|owner| *owner == candidate)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CfgValue {
    True,
    False,
    Unknown,
}

fn evaluate_cfg(meta: &Meta) -> CfgValue {
    match meta {
        Meta::Path(path) if path.is_ident("test") => CfgValue::False,
        Meta::Path(_) | Meta::NameValue(_) => CfgValue::Unknown,
        Meta::List(list) => {
            let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                return CfgValue::Unknown;
            };
            if list.path.is_ident("all") {
                nested
                    .iter()
                    .map(evaluate_cfg)
                    .fold(CfgValue::True, |combined, value| match (combined, value) {
                        (CfgValue::False, _) | (_, CfgValue::False) => CfgValue::False,
                        (CfgValue::True, CfgValue::True) => CfgValue::True,
                        _ => CfgValue::Unknown,
                    })
            } else if list.path.is_ident("any") {
                nested
                    .iter()
                    .map(evaluate_cfg)
                    .fold(CfgValue::False, |combined, value| match (combined, value) {
                        (CfgValue::True, _) | (_, CfgValue::True) => CfgValue::True,
                        (CfgValue::False, CfgValue::False) => CfgValue::False,
                        _ => CfgValue::Unknown,
                    })
            } else if list.path.is_ident("not") && nested.len() == 1 {
                match evaluate_cfg(&nested[0]) {
                    CfgValue::True => CfgValue::False,
                    CfgValue::False => CfgValue::True,
                    CfgValue::Unknown => CfgValue::Unknown,
                }
            } else {
                CfgValue::Unknown
            }
        }
    }
}

fn item_attributes(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

fn attributes_are_test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<Meta>()
                .is_ok_and(|meta| evaluate_cfg(&meta) == CfgValue::False)
    })
}

fn is_test_only(item: &Item) -> bool {
    attributes_are_test_only(item_attributes(item))
}

fn impl_item_attributes(item: &ImplItem) -> &[Attribute] {
    match item {
        ImplItem::Const(item) => &item.attrs,
        ImplItem::Fn(item) => &item.attrs,
        ImplItem::Type(item) => &item.attrs,
        ImplItem::Macro(item) => &item.attrs,
        _ => &[],
    }
}

fn trait_item_attributes(item: &TraitItem) -> &[Attribute] {
    match item {
        TraitItem::Const(item) => &item.attrs,
        TraitItem::Fn(item) => &item.attrs,
        TraitItem::Type(item) => &item.attrs,
        TraitItem::Macro(item) => &item.attrs,
        _ => &[],
    }
}

fn foreign_item_attributes(item: &ForeignItem) -> &[Attribute] {
    match item {
        ForeignItem::Fn(item) => &item.attrs,
        ForeignItem::Static(item) => &item.attrs,
        ForeignItem::Type(item) => &item.attrs,
        ForeignItem::Macro(item) => &item.attrs,
        _ => &[],
    }
}

fn path_components(path: &Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

fn without_trailing_self(components: &[String]) -> &[String] {
    if components
        .last()
        .is_some_and(|component| component == "self")
    {
        &components[..components.len() - 1]
    } else {
        components
    }
}

#[derive(Default)]
struct SourceAnalysis {
    dependencies: BTreeSet<&'static str>,
    paths: Vec<Vec<String>>,
    function_names: BTreeSet<String>,
    struct_names: BTreeSet<String>,
    field_names: BTreeSet<String>,
    string_literals: BTreeSet<String>,
}

impl SourceAnalysis {
    fn has_path_sequence(&self, expected: &[&str]) -> bool {
        self.paths.iter().any(|path| {
            path.windows(expected.len()).any(|window| {
                window
                    .iter()
                    .map(String::as_str)
                    .eq(expected.iter().copied())
            })
        })
    }

    fn has_identifier(&self, expected: &str) -> bool {
        self.paths
            .iter()
            .flatten()
            .any(|identifier| identifier == expected)
            || self.function_names.contains(expected)
            || self.struct_names.contains(expected)
            || self.field_names.contains(expected)
            || self
                .string_literals
                .iter()
                .any(|literal| literal.contains(expected))
    }
}

struct OwnerVisitor {
    module_depth: usize,
    root_alias_scopes: Vec<BTreeSet<String>>,
    analysis: SourceAnalysis,
}

impl OwnerVisitor {
    fn new(items: &[Item]) -> Self {
        let mut visitor = Self {
            module_depth: 1,
            root_alias_scopes: vec![BTreeSet::new()],
            analysis: SourceAnalysis::default(),
        };
        visitor.root_alias_scopes[0] = visitor.discover_item_aliases(items, &BTreeSet::new());
        visitor
    }

    fn visible_root_aliases(&self) -> BTreeSet<String> {
        self.root_alias_scopes
            .iter()
            .flat_map(|scope| scope.iter().cloned())
            .collect()
    }

    fn is_crate_root(&self, components: &[String], aliases: &BTreeSet<String>) -> bool {
        let components = without_trailing_self(components);
        if components == ["crate"] {
            return true;
        }
        let super_count = components
            .iter()
            .take_while(|component| component.as_str() == "super")
            .count();
        (super_count == self.module_depth && super_count == components.len())
            || (components.len() == 1 && aliases.contains(&components[0]))
    }

    fn root_owner(&self, components: &[String]) -> Option<&'static str> {
        let candidate = if components.first().is_some_and(|root| root == "crate") {
            components.get(1)
        } else {
            let super_count = components
                .iter()
                .take_while(|component| component.as_str() == "super")
                .count();
            if super_count == self.module_depth {
                components.get(super_count)
            } else if components
                .first()
                .is_some_and(|root| self.visible_root_aliases().contains(root))
            {
                components.get(1)
            } else {
                None
            }
        }?;
        known_owner(candidate)
    }

    fn record_path(&mut self, components: &[String]) {
        if let Some(owner) = self.root_owner(components) {
            self.analysis.dependencies.insert(owner);
        }
        self.analysis.paths.push(components.to_vec());
    }

    fn collect_use_tree(&mut self, tree: &UseTree, prefix: &mut Vec<String>) {
        match tree {
            UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.collect_use_tree(&path.tree, prefix);
                prefix.pop();
            }
            UseTree::Name(name) => {
                prefix.push(name.ident.to_string());
                self.record_path(prefix);
                prefix.pop();
            }
            UseTree::Rename(rename) => {
                prefix.push(rename.ident.to_string());
                self.record_path(prefix);
                if self.is_crate_root(prefix, &self.visible_root_aliases())
                    && let Some(scope) = self.root_alias_scopes.last_mut()
                {
                    scope.insert(rename.rename.to_string());
                }
                prefix.pop();
            }
            UseTree::Glob(_) => self.record_path(prefix),
            UseTree::Group(group) => {
                for tree in &group.items {
                    self.collect_use_tree(tree, prefix);
                }
            }
        }
    }

    fn discover_item_aliases(
        &self,
        items: &[Item],
        inherited: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        let uses = items.iter().filter_map(|item| match item {
            Item::Use(item) if !attributes_are_test_only(&item.attrs) => Some(&item.tree),
            _ => None,
        });
        self.discover_aliases(uses, inherited)
    }

    fn discover_block_aliases(
        &self,
        block: &Block,
        inherited: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        let uses = block.stmts.iter().filter_map(|statement| match statement {
            Stmt::Item(Item::Use(item)) if !attributes_are_test_only(&item.attrs) => {
                Some(&item.tree)
            }
            _ => None,
        });
        self.discover_aliases(uses, inherited)
    }

    fn discover_aliases<'a>(
        &self,
        uses: impl Iterator<Item = &'a UseTree> + Clone,
        inherited: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        let mut aliases = inherited.clone();
        loop {
            let previous = aliases.len();
            let known_aliases = aliases.clone();
            for tree in uses.clone() {
                collect_root_renames(
                    tree,
                    &mut Vec::new(),
                    self.module_depth,
                    &known_aliases,
                    &mut aliases,
                );
            }
            if aliases.len() == previous {
                break;
            }
        }
        aliases.difference(inherited).cloned().collect()
    }
}

fn collect_root_renames(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    module_depth: usize,
    visible_aliases: &BTreeSet<String>,
    aliases: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_root_renames(&path.tree, prefix, module_depth, visible_aliases, aliases);
            prefix.pop();
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            let root_path = without_trailing_self(prefix);
            let super_count = root_path
                .iter()
                .take_while(|component| component.as_str() == "super")
                .count();
            if (root_path.len() == 1 && root_path[0] == "crate")
                || (super_count == module_depth && super_count == root_path.len())
                || (root_path.len() == 1 && visible_aliases.contains(&root_path[0]))
            {
                aliases.insert(rename.rename.to_string());
            }
            prefix.pop();
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_root_renames(tree, prefix, module_depth, visible_aliases, aliases);
            }
        }
        UseTree::Name(_) | UseTree::Glob(_) => {}
    }
}

impl<'ast> Visit<'ast> for OwnerVisitor {
    fn visit_item(&mut self, item: &'ast Item) {
        if !is_test_only(item) {
            visit::visit_item(self, item);
        }
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.collect_use_tree(&item.tree, &mut Vec::new());
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        if !attributes_are_test_only(impl_item_attributes(item)) {
            if let ImplItem::Fn(item) = item {
                self.analysis
                    .function_names
                    .insert(item.sig.ident.to_string());
            }
            visit::visit_impl_item(self, item);
        }
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        if !attributes_are_test_only(trait_item_attributes(item)) {
            if let TraitItem::Fn(item) = item {
                self.analysis
                    .function_names
                    .insert(item.sig.ident.to_string());
            }
            visit::visit_trait_item(self, item);
        }
    }

    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        if !attributes_are_test_only(foreign_item_attributes(item)) {
            if let ForeignItem::Fn(item) = item {
                self.analysis
                    .function_names
                    .insert(item.sig.ident.to_string());
            }
            visit::visit_foreign_item(self, item);
        }
    }

    fn visit_field(&mut self, field: &'ast Field) {
        if !attributes_are_test_only(&field.attrs) {
            visit::visit_field(self, field);
        }
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        if !attributes_are_test_only(&variant.attrs) {
            visit::visit_variant(self, variant);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let Some((_, items)) = &item.content else {
            return;
        };
        let parent_scopes = std::mem::take(&mut self.root_alias_scopes);
        self.module_depth += 1;
        let aliases = self.discover_item_aliases(items, &BTreeSet::new());
        self.root_alias_scopes = vec![aliases];
        for item in items {
            self.visit_item(item);
        }
        self.module_depth -= 1;
        self.root_alias_scopes = parent_scopes;
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let visible = self.visible_root_aliases();
        let aliases = self.discover_block_aliases(block, &visible);
        self.root_alias_scopes.push(aliases);
        visit::visit_block(self, block);
        self.root_alias_scopes.pop();
    }

    fn visit_path(&mut self, path: &'ast Path) {
        let components = path_components(path);
        self.record_path(&components);
        visit::visit_path(self, path);
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.analysis
            .function_names
            .insert(item.sig.ident.to_string());
        visit::visit_item_fn(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.analysis.struct_names.insert(item.ident.to_string());
        visit::visit_item_struct(self, item);
    }

    fn visit_expr_field(&mut self, field: &'ast ExprField) {
        if let Member::Named(identifier) = &field.member {
            self.analysis.field_names.insert(identifier.to_string());
        }
        visit::visit_expr_field(self, field);
    }

    fn visit_lit_str(&mut self, literal: &'ast LitStr) {
        self.analysis.string_literals.insert(literal.value());
    }
}

fn analyze_source(owner: &str, source: &str) -> Result<SourceAnalysis, String> {
    let file_name = format!("{owner}.rs");
    let file = syn::parse_file(source).map_err(|error| {
        format!("cannot parse owner `{owner}` file `{file_name}` source: {error}")
    })?;
    let mut visitor = OwnerVisitor::new(&file.items);
    for item in &file.items {
        visitor.visit_item(item);
    }
    Ok(visitor.analysis)
}

fn referenced_owner_dependencies(
    owner: &str,
    source: &str,
) -> Result<BTreeSet<&'static str>, String> {
    Ok(analyze_source(owner, source)?.dependencies)
}

fn validate_owner_dependencies(owner: &str, allowed: &[&str], source: &str) -> Result<(), String> {
    for dependency in referenced_owner_dependencies(owner, source)? {
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
        fn qualified() {
            crate::validation::validate();
            crate::validation_rules::validate();
        }
        // use crate::timeline;
        const EXAMPLE: &str = "use crate::model;";
    "#;

    assert_eq!(
        referenced_owner_dependencies("fixture", source).unwrap(),
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
fn relative_grouped_and_root_aliased_owner_dependencies_are_extracted() {
    let source = r#"
        use super::persistence as parent_storage;
        use {
            crate::drafts as pending,
            crate::{render_plan::{self as plan}},
        };
        use crate as root;
        use root::render_process as process;

        fn qualified() {
            crate::validation::validate();
            super::timeline::apply();
        }

        fn locally_aliased() {
            use crate as local_root;
            use local_root::model as domain;

            use super::{self as parent_root};
            use parent_root::error as errors;
        }

        mod nested {
            use super::super::assets as media;
        }

        use crate::persistence_cache::Cache;
        use super::validation_rules::Rules;
    "#;

    assert_eq!(
        referenced_owner_dependencies("fixture", source).unwrap(),
        [
            "assets",
            "drafts",
            "error",
            "model",
            "persistence",
            "render_plan",
            "render_process",
            "timeline",
            "validation",
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn test_only_items_are_excluded_without_hiding_later_production() {
    let source = r#"
        use crate::animation;

        #[cfg(test)]
        mod tests {
            use crate::persistence;
        }

        #[cfg(all(test, unix))]
        fn test_helper() {
            crate::render_process::execute();
        }

        #[cfg(not(test))]
        use crate::timeline;

        #[cfg(any(test, feature = "production"))]
        use crate::model;

        struct Fixture {
            #[cfg(test)]
            hidden: crate::persistence::Storage,
        }

        enum Choice {
            #[cfg(test)]
            Hidden(crate::drafts::EditDraft),
            Visible,
        }

        impl Fixture {
            #[cfg(test)]
            fn test_helper() {
                crate::render_process::execute();
            }

            fn production_helper() {}
        }

        trait FixtureTrait {
            #[cfg(test)]
            fn test_helper() {
                crate::assets::collect();
            }
        }

        use crate::validation;
    "#;

    let analysis = analyze_source("fixture", source).unwrap();
    assert_eq!(
        analysis.dependencies,
        ["animation", "model", "timeline", "validation"]
            .into_iter()
            .collect()
    );
    assert!(!analysis.function_names.contains("test_helper"));
    assert!(analysis.function_names.contains("production_helper"));
}

#[test]
fn parse_failures_identify_the_source_owner() {
    let message = referenced_owner_dependencies("validation", "use crate::{persistence;")
        .expect_err("invalid Rust must fail architecture analysis");

    assert!(message.contains("owner `validation`"));
    assert!(message.contains("file `validation.rs`"));
    assert!(message.contains("cannot parse"));
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
        validate_owner_dependencies(owner, allowed, &source)
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
        let analysis = analyze_source(owner, &source).unwrap_or_else(|message| panic!("{message}"));
        for token in forbidden_outer {
            assert!(
                !analysis.has_identifier(token),
                "ADR 0003 forbids `{token}` in inward owner `{owner}`"
            );
        }
    }

    let render_plan = read_source("render_plan.rs");
    let render_plan = analyze_source("render_plan", &render_plan).unwrap();
    for (token, forbidden) in [
        (
            "std::process",
            render_plan.has_path_sequence(&["std", "process"]),
        ),
        (
            "Command::new",
            render_plan.has_path_sequence(&["Command", "new"]),
        ),
        ("std::env", render_plan.has_path_sequence(&["std", "env"])),
        (
            "render_process",
            render_plan.dependencies.contains("render_process"),
        ),
        (
            "render_artifact",
            render_plan.dependencies.contains("render_artifact"),
        ),
    ] {
        assert!(
            !forbidden,
            "render planning must not depend on execution/publication token `{token}`"
        );
    }

    let persistence = read_source("persistence.rs");
    let persistence = analyze_source("persistence", &persistence).unwrap();
    for (token, forbidden) in [
        (
            "crate::timeline",
            persistence.dependencies.contains("timeline"),
        ),
        (
            "crate::renderer",
            persistence.dependencies.contains("renderer"),
        ),
        (
            "crate::render_",
            persistence
                .dependencies
                .iter()
                .any(|dependency| dependency.starts_with("render_")),
        ),
    ] {
        assert!(!forbidden, "persistence must not depend on `{token}`");
    }

    let assets = read_source("assets.rs");
    let assets = analyze_source("assets", &assets).unwrap();
    for (token, forbidden) in [
        ("File::open", assets.has_path_sequence(&["File", "open"])),
        (
            "std::fs::copy",
            assets.has_path_sequence(&["std", "fs", "copy"]),
        ),
        (
            "std::fs::rename",
            assets.has_path_sequence(&["std", "fs", "rename"]),
        ),
    ] {
        assert!(
            !forbidden,
            "managed asset I/O must use the persistence port, not `{token}`"
        );
    }

    let store = read_source("store.rs");
    let store_analysis = analyze_source("store", &store).unwrap();
    for (token, forbidden) in [
        (
            "struct ProjectTransaction",
            store_analysis.struct_names.contains("ProjectTransaction"),
        ),
        (
            "fn recover_transaction(",
            store_analysis
                .function_names
                .contains("recover_transaction"),
        ),
    ] {
        assert!(
            !forbidden,
            "persistence orchestration must not be reimplemented in store via `{token}`"
        );
    }

    let renderer = read_source("renderer.rs");
    let renderer = analyze_source("renderer", &renderer).unwrap();
    for (token, forbidden) in [
        (
            "fn prepare_text_layers(",
            renderer.function_names.contains("prepare_text_layers"),
        ),
        (
            "fn resolve_text_font(",
            renderer.function_names.contains("resolve_text_font"),
        ),
        (".tracks", renderer.field_names.contains("tracks")),
        ("TimelineItem", renderer.has_identifier("TimelineItem")),
        (
            "Command::new",
            renderer.has_path_sequence(&["Command", "new"]),
        ),
        (
            "std::process",
            renderer.has_path_sequence(&["std", "process"]),
        ),
    ] {
        assert!(
            !forbidden,
            "artifact preparation must not be reimplemented in renderer via `{token}`"
        );
    }

    for (token, forbidden) in [
        (
            "fn garbage_collect(",
            store_analysis.function_names.contains("garbage_collect"),
        ),
        (
            "fn collect_files(",
            store_analysis.function_names.contains("collect_files"),
        ),
    ] {
        assert!(
            !forbidden,
            "store must delegate managed-asset collection instead of owning `{token}`"
        );
    }
}
