use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use syn::{
    Attribute, Block, ExprField, ExprMethodCall, Field, FieldPat, ForeignItem, ImplItem, Item,
    ItemFn, ItemMod, ItemStruct, LitStr, Member, Meta, Path as SynPath, Stmt, Token, TraitItem,
    Type, UseTree, Variant, punctuated::Punctuated, visit, visit::Visit,
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

fn validate_motion_graphics_adr(source: &str) -> Result<(), String> {
    const REQUIRED_SEMANTICS: &[&str] = &[
        "### 1. Additive root-track model",
        "### 2. Canonical EvaluatedScene seam",
        "### 3. Hybrid renderer boundary",
        "#### Fallback policy",
        "### 4. Normative ordering and compositing",
        "### 5. Presets compile to persisted primitives",
        "### 6. Additive schema-version policy",
        "top-left origin",
        "half-open intervals `[start_ms, end_ms)`",
        "ascending explicit `z_index`",
        "premultiplied alpha in linear light",
        "raw FFmpeg expressions",
        "executable SVG content",
        "arbitrary paths",
        "network resources",
        "deterministic local priority",
        "complete `EvaluatedScene`",
        "must not omit, approximate, downgrade, reorder, or remotely acquire resources",
        "`DEPENDENCY_UNAVAILABLE`",
        "No partial or degraded artifact is published",
        "current state and every retained undo and redo snapshot",
        "contracts/contract-ownership-v1.json",
        "all versioned public fixtures remain unchanged",
    ];

    let missing = REQUIRED_SEMANTICS
        .iter()
        .copied()
        .filter(|required| !source.contains(required))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "ADR 0004 is missing required motion-graphics semantics: {}",
            missing.join(", ")
        ))
    }
}

#[test]
fn motion_graphics_adr_locks_required_architecture_semantics() {
    let adr = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/adr/0004-motion-graphics-architecture.md"),
    )
    .expect("ADR 0004 must be readable");

    validate_motion_graphics_adr(&adr).unwrap_or_else(|message| panic!("{message}"));
}

#[test]
fn motion_graphics_adr_validation_rejects_an_omitted_decision() {
    let incomplete = "### 1. Additive root-track model";
    let message = validate_motion_graphics_adr(incomplete)
        .expect_err("an ADR fixture missing five decisions and semantics must fail");

    assert!(message.contains("### 2. Canonical EvaluatedScene seam"));
    assert!(message.contains("premultiplied alpha in linear light"));
    assert!(message.contains("all versioned public fixtures remain unchanged"));
}

#[test]
fn motion_graphics_adr_validation_rejects_an_omitted_fallback_policy() {
    let adr = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/adr/0004-motion-graphics-architecture.md"),
    )
    .expect("ADR 0004 must be readable");
    let (before_fallback, after_heading) = adr
        .split_once("#### Fallback policy")
        .expect("ADR fixture must contain the fallback heading");
    let (_, after_fallback) = after_heading
        .split_once("We rejected an FFmpeg-only implementation")
        .expect("ADR fixture must contain the paragraph after the fallback policy");
    let without_fallback =
        format!("{before_fallback}We rejected an FFmpeg-only implementation{after_fallback}");

    let message = validate_motion_graphics_adr(&without_fallback)
        .expect_err("an ADR fixture missing only the fallback policy must fail");

    assert!(message.contains("#### Fallback policy"));
    assert!(message.contains("deterministic local priority"));
    assert!(message.contains("`DEPENDENCY_UNAVAILABLE`"));
    assert!(message.contains("No partial or degraded artifact is published"));
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
        attribute.path().is_ident("test")
            || attribute.path().is_ident("cfg")
                && attribute
                    .parse_args::<Meta>()
                    .is_ok_and(|meta| evaluate_cfg(&meta) == CfgValue::False)
    })
}

fn selects_custom_module_path_in_production(attribute: &Attribute) -> bool {
    if attribute.path().is_ident("path") {
        return true;
    }
    if !attribute.path().is_ident("cfg_attr") {
        return false;
    }
    let Ok(arguments) = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
    else {
        return true;
    };
    let mut arguments = arguments.iter();
    let Some(predicate) = arguments.next() else {
        return true;
    };
    evaluate_cfg(predicate) != CfgValue::False && arguments.any(meta_selects_custom_module_path)
}

fn meta_selects_custom_module_path(meta: &Meta) -> bool {
    if meta.path().is_ident("path") {
        return true;
    }
    let Meta::List(list) = meta else {
        return false;
    };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    let Ok(arguments) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
    else {
        return true;
    };
    let mut arguments = arguments.iter();
    let Some(predicate) = arguments.next() else {
        return true;
    };
    evaluate_cfg(predicate) != CfgValue::False && arguments.any(meta_selects_custom_module_path)
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

fn path_components(path: &SynPath) -> Vec<String> {
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

#[derive(Debug, Default)]
struct SourceAnalysis {
    dependencies: BTreeSet<&'static str>,
    dependency_sources: BTreeMap<&'static str, BTreeSet<String>>,
    paths: Vec<Vec<String>>,
    function_names: BTreeSet<String>,
    struct_names: BTreeSet<String>,
    field_names: BTreeSet<String>,
    method_names: BTreeSet<String>,
    string_literals: BTreeSet<String>,
}

impl SourceAnalysis {
    fn merge(&mut self, other: Self) {
        self.dependencies.extend(other.dependencies);
        for (dependency, sources) in other.dependency_sources {
            self.dependency_sources
                .entry(dependency)
                .or_default()
                .extend(sources);
        }
        self.paths.extend(other.paths);
        self.function_names.extend(other.function_names);
        self.struct_names.extend(other.struct_names);
        self.field_names.extend(other.field_names);
        self.method_names.extend(other.method_names);
        self.string_literals.extend(other.string_literals);
    }

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
            || self.method_names.contains(expected)
            || self
                .string_literals
                .iter()
                .any(|literal| literal.contains(expected))
    }
}

struct OwnerVisitor {
    module_depth: usize,
    alias_scopes: Vec<AliasScope>,
    inline_modules: Vec<String>,
    external_modules: Vec<ExternalModule>,
    error: Option<String>,
    analysis: SourceAnalysis,
}

#[derive(Clone, Debug)]
struct AliasScope {
    module_depth: usize,
    aliases: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
struct ExternalModule {
    name: String,
    module_depth: usize,
    inline_modules: Vec<String>,
    parent_aliases: Vec<AliasScope>,
}

impl OwnerVisitor {
    fn new(items: &[Item]) -> Self {
        Self::with_context(items, 1, Vec::new(), Vec::new())
    }

    fn with_context(
        items: &[Item],
        module_depth: usize,
        mut alias_scopes: Vec<AliasScope>,
        inline_modules: Vec<String>,
    ) -> Self {
        let mut visitor = Self {
            module_depth,
            alias_scopes: alias_scopes.clone(),
            inline_modules,
            external_modules: Vec::new(),
            error: None,
            analysis: SourceAnalysis::default(),
        };
        let aliases = visitor.discover_item_aliases(items.iter(), &BTreeMap::new());
        alias_scopes.push(AliasScope {
            module_depth,
            aliases,
        });
        visitor.alias_scopes = alias_scopes;
        visitor
    }

    fn visible_aliases(&self) -> BTreeMap<String, Vec<String>> {
        self.alias_scopes
            .iter()
            .filter(|scope| scope.module_depth == self.module_depth)
            .flat_map(|scope| scope.aliases.clone())
            .collect()
    }

    fn alias_at_depth(&self, depth: usize, name: &str) -> Option<Vec<String>> {
        self.alias_scopes
            .iter()
            .rev()
            .find(|scope| scope.module_depth == depth && scope.aliases.contains_key(name))
            .and_then(|scope| scope.aliases.get(name).cloned())
    }

    fn canonical_path(&self, components: &[String]) -> Vec<String> {
        let mut resolved = without_trailing_self(components).to_vec();
        if resolved
            .first()
            .is_some_and(|component| component == "self")
            && resolved
                .get(1)
                .is_some_and(|component| component == "super")
        {
            resolved.remove(0);
        }
        let mut visited = BTreeSet::new();
        for _ in 0..16 {
            if !visited.insert(resolved.clone()) {
                break;
            }
            let super_count = resolved
                .iter()
                .take_while(|component| component.as_str() == "super")
                .count();
            let replacement = if super_count > 0 && super_count < resolved.len() {
                self.module_depth
                    .checked_sub(super_count)
                    .and_then(|depth| self.alias_at_depth(depth, &resolved[super_count]))
                    .map(|target| (super_count + 1, target))
            } else if let Some(first) = resolved.first() {
                self.visible_aliases()
                    .get(first)
                    .cloned()
                    .map(|target| (1, target))
            } else {
                None
            };
            let Some((consumed, mut target)) = replacement else {
                break;
            };
            target.extend_from_slice(&resolved[consumed..]);
            if target == resolved {
                break;
            }
            resolved = target;
        }
        resolved
    }

    fn root_owner(&self, components: &[String]) -> Option<&'static str> {
        let canonical = self.canonical_path(components);
        let components = canonical.as_slice();
        let candidate = if components.first().is_some_and(|root| root == "crate") {
            components.get(1)
        } else {
            let super_count = components
                .iter()
                .take_while(|component| component.as_str() == "super")
                .count();
            if super_count == self.module_depth {
                components.get(super_count)
            } else {
                None
            }
        }?;
        known_owner(candidate)
    }

    fn record_path(&mut self, components: &[String]) {
        let canonical = self.canonical_path(components);
        if let Some(owner) = self.root_owner(&canonical) {
            self.analysis.dependencies.insert(owner);
        }
        self.analysis.paths.push(canonical);
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
                let canonical = self.canonical_path(prefix);
                if let Some(scope) = self.alias_scopes.last_mut() {
                    scope.aliases.insert(rename.rename.to_string(), canonical);
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

    fn discover_item_aliases<'a>(
        &self,
        items: impl Iterator<Item = &'a Item> + Clone,
        inherited: &BTreeMap<String, Vec<String>>,
    ) -> BTreeMap<String, Vec<String>> {
        let uses = items.clone().filter_map(|item| match item {
            Item::Use(item) if !attributes_are_test_only(&item.attrs) => Some(&item.tree),
            _ => None,
        });
        let mut seeded = inherited.clone();
        for item in items {
            match item {
                Item::ExternCrate(item) if !attributes_are_test_only(&item.attrs) => {
                    let alias = item
                        .rename
                        .as_ref()
                        .map_or_else(|| item.ident.to_string(), |(_, alias)| alias.to_string());
                    let target = if item.ident == "self" {
                        vec!["crate".to_owned()]
                    } else {
                        vec![item.ident.to_string()]
                    };
                    seeded.insert(alias, target);
                }
                Item::Type(item) if !attributes_are_test_only(&item.attrs) => {
                    if let Type::Path(target) = item.ty.as_ref() {
                        seeded.insert(item.ident.to_string(), path_components(&target.path));
                    }
                }
                _ => {}
            }
        }
        self.discover_aliases(uses, &seeded)
    }

    fn discover_block_aliases(
        &self,
        block: &Block,
        inherited: &BTreeMap<String, Vec<String>>,
    ) -> BTreeMap<String, Vec<String>> {
        let items = block.stmts.iter().filter_map(|statement| match statement {
            Stmt::Item(item) => Some(item),
            _ => None,
        });
        self.discover_item_aliases(items, inherited)
    }

    fn discover_aliases<'a>(
        &self,
        uses: impl Iterator<Item = &'a UseTree> + Clone,
        inherited: &BTreeMap<String, Vec<String>>,
    ) -> BTreeMap<String, Vec<String>> {
        let mut aliases = inherited.clone();
        for _ in 0..16 {
            let previous = aliases.clone();
            for tree in uses.clone() {
                collect_alias_renames(tree, &mut Vec::new(), self, &mut aliases);
            }
            if aliases == previous {
                break;
            }
        }
        aliases
    }
}

fn collect_alias_renames(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    visitor: &OwnerVisitor,
    aliases: &mut BTreeMap<String, Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_alias_renames(&path.tree, prefix, visitor, aliases);
            prefix.pop();
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            let canonical = expand_discovered_aliases(visitor.canonical_path(prefix), aliases);
            aliases.insert(rename.rename.to_string(), canonical);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let (alias, path) = if name.ident == "self" {
                let Some(alias) = prefix.last().cloned() else {
                    return;
                };
                (alias, prefix.clone())
            } else {
                prefix.push(name.ident.to_string());
                let path = prefix.clone();
                prefix.pop();
                (name.ident.to_string(), path)
            };
            let canonical = expand_discovered_aliases(visitor.canonical_path(&path), aliases);
            aliases.insert(alias, canonical);
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_alias_renames(tree, prefix, visitor, aliases);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn expand_discovered_aliases(
    mut canonical: Vec<String>,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut visited = BTreeSet::new();
    for _ in 0..16 {
        if !visited.insert(canonical.clone()) {
            break;
        }
        let Some(first) = canonical.first() else {
            break;
        };
        let Some(mut target) = aliases.get(first).cloned() else {
            break;
        };
        target.extend_from_slice(&canonical[1..]);
        if target == canonical {
            break;
        }
        canonical = target;
    }
    canonical
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
        if item.content.is_none() {
            if item
                .attrs
                .iter()
                .any(selects_custom_module_path_in_production)
            {
                self.error = Some(format!(
                    "custom #[path] module `{}` is not supported",
                    item.ident
                ));
                return;
            }
            self.external_modules.push(ExternalModule {
                name: item.ident.to_string(),
                module_depth: self.module_depth + 1,
                inline_modules: self.inline_modules.clone(),
                parent_aliases: self.alias_scopes.clone(),
            });
            return;
        }
        let Some((_, items)) = &item.content else {
            unreachable!();
        };
        self.module_depth += 1;
        self.inline_modules.push(item.ident.to_string());
        let aliases = self.discover_item_aliases(items.iter(), &BTreeMap::new());
        self.alias_scopes.push(AliasScope {
            module_depth: self.module_depth,
            aliases,
        });
        for item in items {
            self.visit_item(item);
        }
        self.alias_scopes.pop();
        self.inline_modules.pop();
        self.module_depth -= 1;
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let visible = self.visible_aliases();
        let aliases = self.discover_block_aliases(block, &visible);
        self.alias_scopes.push(AliasScope {
            module_depth: self.module_depth,
            aliases,
        });
        visit::visit_block(self, block);
        self.alias_scopes.pop();
    }

    fn visit_path(&mut self, path: &'ast SynPath) {
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

    fn visit_field_pat(&mut self, field: &'ast FieldPat) {
        if let Member::Named(identifier) = &field.member {
            self.analysis.field_names.insert(identifier.to_string());
        }
        visit::visit_field_pat(self, field);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.analysis.method_names.insert(call.method.to_string());
        visit::visit_expr_method_call(self, call);
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
    if let Some(error) = visitor.error {
        return Err(format!("owner `{owner}` file `{file_name}`: {error}"));
    }
    if let Some(module) = visitor.external_modules.first() {
        return Err(format!(
            "owner `{owner}` file `{file_name}` contains out-of-line module `{}` without a source loader",
            module.name
        ));
    }
    let mut analysis = visitor.analysis;
    for dependency in &analysis.dependencies {
        analysis
            .dependency_sources
            .entry(dependency)
            .or_default()
            .insert(file_name.clone());
    }
    Ok(analysis)
}

fn analyze_owner(owner: &str) -> Result<SourceAnalysis, String> {
    let source_file = source_root().join(format!("{owner}.rs"));
    let module_base = source_root().join(owner);
    analyze_source_file(owner, &source_file, &module_base, 1, Vec::new())
}

fn analyze_source_file(
    owner: &str,
    source_file: &Path,
    module_base: &Path,
    module_depth: usize,
    alias_scopes: Vec<AliasScope>,
) -> Result<SourceAnalysis, String> {
    let source = fs::read_to_string(source_file).map_err(|error| {
        format!(
            "cannot read owner `{owner}` file `{}` source: {error}",
            source_file.display()
        )
    })?;
    let file = syn::parse_file(&source).map_err(|error| {
        format!(
            "cannot parse owner `{owner}` file `{}` source: {error}",
            source_file.display()
        )
    })?;
    let mut visitor = OwnerVisitor::with_context(&file.items, module_depth, alias_scopes, vec![]);
    for item in &file.items {
        visitor.visit_item(item);
    }
    if let Some(error) = visitor.error {
        return Err(format!(
            "owner `{owner}` file `{}`: {error}",
            source_file.display()
        ));
    }
    let mut analysis = visitor.analysis;
    for dependency in &analysis.dependencies {
        analysis
            .dependency_sources
            .entry(dependency)
            .or_default()
            .insert(source_file.display().to_string());
    }
    for module in visitor.external_modules {
        let parent = module
            .inline_modules
            .iter()
            .fold(module_base.to_owned(), |path, component| {
                path.join(component)
            });
        let flat = parent.join(format!("{}.rs", module.name));
        let nested = parent.join(&module.name).join("mod.rs");
        let candidates = [flat.as_path(), nested.as_path()]
            .into_iter()
            .filter(|candidate| candidate.is_file())
            .collect::<Vec<_>>();
        let [child_file] = candidates.as_slice() else {
            return Err(format!(
                "owner `{owner}` file `{}` cannot resolve out-of-line module `{}`",
                source_file.display(),
                module.name
            ));
        };
        let child_base = parent.join(&module.name);
        analysis.merge(analyze_source_file(
            owner,
            child_file,
            &child_base,
            module.module_depth,
            module.parent_aliases,
        )?);
    }
    Ok(analysis)
}

fn referenced_owner_dependencies(
    owner: &str,
    source: &str,
) -> Result<BTreeSet<&'static str>, String> {
    Ok(analyze_source(owner, source)?.dependencies)
}

fn validate_owner_dependencies(owner: &str, allowed: &[&str], source: &str) -> Result<(), String> {
    validate_owner_analysis(owner, allowed, &analyze_source(owner, source)?)
}

fn validate_owner_analysis(
    owner: &str,
    allowed: &[&str],
    analysis: &SourceAnalysis,
) -> Result<(), String> {
    for dependency in &analysis.dependencies {
        if *dependency != owner && !allowed.contains(dependency) {
            let sources = analysis
                .dependency_sources
                .get(dependency)
                .map(|paths| paths.iter().cloned().collect::<Vec<_>>().join(", "))
                .unwrap_or_else(|| "unknown source".into());
            return Err(format!(
                "owner `{owner}` file `{sources}` imports `{dependency}`, but the ADR matrix allows only {allowed:?}"
            ));
        }
    }
    Ok(())
}

fn validate_structural_responsibilities(
    owner: &str,
    analysis: &SourceAnalysis,
) -> Result<(), String> {
    let native_filesystem_methods = [
        "read_dir",
        "read_link",
        "metadata",
        "symlink_metadata",
        "canonicalize",
        "exists",
        "try_exists",
        "is_file",
        "is_dir",
        "is_symlink",
    ];
    let native_filesystem_method = native_filesystem_methods
        .iter()
        .find(|method| analysis.method_names.contains(**method));
    let native_filesystem_qualified_path = analysis.paths.iter().any(|path| {
        path.windows(3).any(|window| {
            window[0] == "std"
                && window[1] == "path"
                && matches!(window[2].as_str(), "Path" | "PathBuf")
        }) && path.last().is_some_and(|method| {
            native_filesystem_methods
                .iter()
                .any(|candidate| method == candidate)
        })
    });
    let uses_native_filesystem = analysis.has_path_sequence(&["std", "fs"])
        || native_filesystem_method.is_some()
        || native_filesystem_qualified_path;
    let uses_storage_primitive = ["list", "entry_kind", "remove_durable"]
        .iter()
        .any(|method| {
            analysis.method_names.contains(*method)
                || analysis
                    .paths
                    .iter()
                    .any(|path| path.last().is_some_and(|segment| segment == method))
        });
    let violation = match owner {
        "assets" if uses_native_filesystem => Some("managed asset filesystem access"),
        "store" if uses_native_filesystem => Some("direct filesystem access"),
        "store" if uses_storage_primitive => Some("primitive listing/type/removal operation"),
        "store" if !analysis.has_path_sequence(&["crate", "assets", "garbage_collect"]) => {
            Some("canonical asset garbage-collection delegation")
        }
        "renderer" if analysis.field_names.contains("tracks") => Some("timeline track planning"),
        "renderer" | "render_plan" if uses_native_filesystem => {
            Some("direct render filesystem access")
        }
        _ => None,
    };
    violation.map_or(Ok(()), |responsibility| {
        Err(format!(
            "owner `{owner}` duplicates forbidden responsibility `{responsibility}`"
        ))
    })
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
fn parent_root_and_chained_external_aliases_are_canonicalized() {
    let source = r#"
        use crate as root;
        use std as standard;
        use standard::fs as disk;

        mod nested {
            use super::root::persistence as storage;
            fn inspect() { super::disk::read("fixture"); }
        }
    "#;

    let analysis = analyze_source("fixture", source).unwrap();
    assert!(analysis.dependencies.contains("persistence"));
    assert!(analysis.has_path_sequence(&["std", "fs", "read"]));
}

#[test]
fn self_super_and_extern_crate_aliases_are_canonicalized() {
    let source = r#"
        extern crate self as root;
        extern crate std as platform;
        use platform::fs as disk;

        mod nested {
            use self::super::root::persistence as storage;
            fn inspect() { super::disk::read("fixture"); }
        }
    "#;

    let analysis = analyze_source("fixture", source).unwrap();
    assert!(analysis.dependencies.contains("persistence"));
    assert!(analysis.has_path_sequence(&["std", "fs", "read"]));

    let prefixed = analyze_source(
        "fixture",
        "extern crate self as root; use root::persistence_cache;",
    )
    .unwrap();
    assert!(!prefixed.dependencies.contains("persistence"));

    let cyclic =
        analyze_source("fixture", "use b as a; use a as b; use crate::validation;").unwrap();
    assert!(cyclic.dependencies.contains("validation"));
}

#[test]
fn block_local_item_aliases_are_canonicalized_and_test_only_aliases_are_excluded() {
    let source = r#"
        fn inspect(path: &std::path::Path) {
            extern crate self as root;
            extern crate std as platform;
            use root::persistence as storage;
            use platform::fs as disk;
            type FsPath = std::path::Path;

            storage::read();
            disk::read("fixture");
            let _ = FsPath::metadata(path);
        }
    "#;
    let analysis = analyze_source("validation", source).unwrap();
    assert!(analysis.dependencies.contains("persistence"));
    assert!(analysis.has_path_sequence(&["std", "fs", "read"]));
    assert!(analysis.has_path_sequence(&["std", "path", "Path", "metadata"]));
    assert!(validate_owner_analysis("validation", &[], &analysis).is_err());

    let test_only = analyze_source(
        "fixture",
        r#"
            fn inspect() {
                #[cfg(test)]
                extern crate self as root;
                #[cfg(test)]
                use root::persistence as storage;
                #[cfg(test)]
                type FsPath = std::path::Path;

                crate::validation::validate();
            }
        "#,
    )
    .unwrap();
    assert_eq!(test_only.dependencies, ["validation"].into_iter().collect());

    let exact_and_cyclic = analyze_source(
        "fixture",
        r#"
            fn inspect() {
                extern crate self as root;
                use root::persistence_cache;
                use b as a;
                use a as b;
                crate::validation::validate();
            }
        "#,
    )
    .unwrap();
    assert_eq!(
        exact_and_cyclic.dependencies,
        ["validation"].into_iter().collect()
    );
}

#[test]
fn structured_patterns_record_named_fields() {
    let analysis = analyze_source(
        "fixture",
        "fn inspect(project: Project) { let Project { tracks, .. } = project; }",
    )
    .unwrap();

    assert!(analysis.field_names.contains("tracks"));
    assert!(validate_structural_responsibilities("renderer", &analysis).is_err());
}

#[test]
fn custom_module_paths_are_rejected_with_owner_and_file() {
    for source in [
        "#[path = \"hidden.rs\"] mod hidden;",
        "#[cfg_attr(not(test), path = \"hidden.rs\")] mod hidden;",
        "#[cfg_attr(feature = \"custom\", path = \"hidden.rs\")] mod hidden;",
        "#[cfg_attr(not(test), cfg_attr(unix, path = \"hidden.rs\"))] mod hidden;",
    ] {
        let message = analyze_source("validation", source)
            .expect_err("custom module paths must not bypass analysis");

        assert!(message.contains("owner `validation`"));
        assert!(message.contains("file `validation.rs`"));
        assert!(message.contains("custom #[path]"));
    }
}

#[test]
fn standard_out_of_line_modules_are_analyzed_recursively() {
    for nested_layout in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let owner_file = directory.path().join("validation.rs");
        let owner_base = directory.path().join("validation");
        fs::create_dir_all(&owner_base).unwrap();
        fs::write(&owner_file, "mod hidden;").unwrap();
        let child = if nested_layout {
            let child_dir = owner_base.join("hidden");
            fs::create_dir_all(&child_dir).unwrap();
            child_dir.join("mod.rs")
        } else {
            owner_base.join("hidden.rs")
        };
        fs::write(child, "use crate::persistence;").unwrap();

        let analysis =
            analyze_source_file("validation", &owner_file, &owner_base, 1, Vec::new()).unwrap();
        let message = validate_owner_analysis("validation", &[], &analysis)
            .expect_err("a forbidden dependency in a child module must fail");
        assert!(message.contains("imports `persistence`"));
        assert!(message.contains("hidden"));
    }
}

#[test]
fn renamed_gc_and_aliased_asset_filesystem_access_are_rejected_structurally() {
    let store = analyze_source(
        "store",
        "fn sweep_unused_assets(storage: &Storage) { storage.list(path); storage.entry_kind(path); storage.remove_durable(path); }",
    )
    .unwrap();
    assert!(validate_structural_responsibilities("store", &store).is_err());
    let indirect_store = analyze_source(
        "store",
        "use crate::assets::garbage_collect; use crate::persistence::Storage as Disk; fn sweep_unused_assets(storage: &dyn Disk) { Disk::list(storage, path); garbage_collect(); }",
    )
    .unwrap();
    assert!(validate_structural_responsibilities("store", &indirect_store).is_err());
    let undelegated_store =
        analyze_source("store", "fn sweep_unused_assets() { retain_everything(); }").unwrap();
    assert!(validate_structural_responsibilities("store", &undelegated_store).is_err());
    let delegated_store = analyze_source(
        "store",
        "use crate::assets::garbage_collect as collect_assets; fn sweep_unused_assets() { collect_assets(); }",
    )
    .unwrap();
    validate_structural_responsibilities("store", &delegated_store).unwrap();

    let assets = analyze_source(
        "assets",
        "use std as standard; use standard::fs as disk; fn load() { disk::read(\"a\"); disk::remove_file(\"b\"); }",
    )
    .unwrap();
    assert!(validate_structural_responsibilities("assets", &assets).is_err());

    for source in [
        "fn load(path: &std::path::Path) { let _ = path.read_dir(); }",
        "fn load(path: &std::path::Path) { let _ = path.metadata(); }",
        "fn load(path: &std::path::Path) { let _ = path.canonicalize(); }",
        "use std::path::Path; fn load(path: &Path) { let _ = Path::read_dir(path); }",
        "use std::path::Path as FsPath; fn load(path: &FsPath) { let _ = FsPath::metadata(path); }",
        "type FsPath = std::path::Path; fn load(path: &FsPath) { let _ = FsPath::canonicalize(path); }",
    ] {
        let analysis = analyze_source("assets", source).unwrap();
        assert!(validate_structural_responsibilities("assets", &analysis).is_err());
    }

    for owner in ["assets", "store", "renderer", "render_plan"] {
        let delegation = if owner == "store" {
            "use crate::assets::garbage_collect;"
        } else {
            ""
        };
        let collect = if owner == "store" {
            "garbage_collect();"
        } else {
            ""
        };
        for operation in [
            "fn inspect(path: &std::path::Path) { let _ = path.is_symlink(); COLLECT }",
            "fn inspect(path: &std::path::Path) { let _ = std::path::Path::is_symlink(path); COLLECT }",
            "use std::path::Path as FsPath; fn inspect(path: &FsPath) { let _ = FsPath::is_symlink(path); COLLECT }",
        ] {
            let source = format!("{delegation} {}", operation.replace("COLLECT", collect));
            let analysis = analyze_source(owner, &source).unwrap();
            assert!(
                validate_structural_responsibilities(owner, &analysis).is_err(),
                "owner `{owner}` accepted native symlink inspection in `{source}`"
            );
        }
    }

    let direct_store = analyze_source(
        "store",
        "use crate::assets::garbage_collect; fn load(path: &std::path::Path) { let _ = path.read_dir(); garbage_collect(); }",
    )
    .unwrap();
    assert!(validate_structural_responsibilities("store", &direct_store).is_err());

    let delegated_store = analyze_source(
        "store",
        "use crate::assets::garbage_collect; fn load(storage: &dyn Storage) { let _ = storage.canonicalize_storage_path(path); garbage_collect(); }",
    )
    .unwrap();
    validate_structural_responsibilities("store", &delegated_store).unwrap();

    let direct_renderer = analyze_source(
        "renderer",
        "fn prepare(path: &std::path::Path) { let _ = path.exists(); }",
    )
    .unwrap();
    assert!(validate_structural_responsibilities("renderer", &direct_renderer).is_err());
    let delegated_renderer = analyze_source(
        "renderer",
        "fn prepare(io: &dyn ArtifactIo) { let _ = io.artifact_path_exists(path); }",
    )
    .unwrap();
    validate_structural_responsibilities("renderer", &delegated_renderer).unwrap();

    let delegated_assets = analyze_source(
        "assets",
        "fn inspect(storage: &dyn Storage) { let _ = storage.entry_kind(path); }",
    )
    .unwrap();
    validate_structural_responsibilities("assets", &delegated_assets).unwrap();
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

        #[test]
        fn standalone_test() {
            crate::persistence::read();
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
    assert!(!analysis.function_names.contains("standalone_test"));
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
        let analysis = analyze_owner(owner).unwrap_or_else(|message| panic!("{message}"));
        validate_owner_analysis(owner, allowed, &analysis)
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
        let owner_name = owner.trim_end_matches(".rs");
        let analysis = analyze_owner(owner_name).unwrap_or_else(|message| panic!("{message}"));
        for token in forbidden_outer {
            assert!(
                !analysis.has_identifier(token),
                "ADR 0003 forbids `{token}` in inward owner `{owner}`"
            );
        }
    }

    let render_plan = analyze_owner("render_plan").unwrap();
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

    let persistence = analyze_owner("persistence").unwrap();
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

    let assets = analyze_owner("assets").unwrap();
    validate_structural_responsibilities("assets", &assets).unwrap();
    for (token, forbidden) in [
        ("std::fs", assets.has_path_sequence(&["std", "fs"])),
        (
            "std::fs::File",
            assets.has_path_sequence(&["std", "fs", "File"]),
        ),
    ] {
        assert!(
            !forbidden,
            "managed asset I/O must use the persistence port, not `{token}`"
        );
    }

    let store_analysis = analyze_owner("store").unwrap();
    validate_structural_responsibilities("store", &store_analysis).unwrap();
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

    let renderer = analyze_owner("renderer").unwrap();
    validate_structural_responsibilities("renderer", &renderer).unwrap();
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
        ("std::fs", store_analysis.has_path_sequence(&["std", "fs"])),
        (
            "File",
            store_analysis.has_path_sequence(&["std", "fs", "File"]),
        ),
        (
            "Storage::list",
            store_analysis.method_names.contains("list"),
        ),
        (
            "Storage::entry_kind",
            store_analysis.method_names.contains("entry_kind"),
        ),
        (
            "Storage::remove_durable",
            store_analysis.method_names.contains("remove_durable"),
        ),
    ] {
        assert!(
            !forbidden,
            "store must delegate managed-asset collection instead of owning `{token}`"
        );
    }
}
