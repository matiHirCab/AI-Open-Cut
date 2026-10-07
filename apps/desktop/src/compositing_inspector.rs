//! Bounded presentation and typed requests over existing core compositing semantics.
use crate::{
    animation_inspector,
    hierarchy::Selection,
    inspector_edit::{Field, FieldKind, add_value},
};
use opencut_editor_core::{
    EditOperation, Mask, MaskSource, Paint, Project, TimelineItem, VisualEffect,
};
use serde_json::{Value, json};

pub(crate) const BLENDS: &[&str] = &[
    "normal", "multiply", "screen", "overlay", "add", "darken", "lighten",
];
pub(crate) const EFFECT_TYPES: &[&str] = &[
    "gaussian_blur",
    "glow",
    "color_tint",
    "vignette",
    "color_adjustment",
    "screen_flash",
    "particle_overlay",
];
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub mask_id: Option<String>,
    pub effect_id: Option<String>,
    pub command: usize,
    pub stop: usize,
}
impl Cursor {
    pub fn mask_index(&self, item: &TimelineItem) -> Option<usize> {
        let masks = &item.visual_properties().masks;
        self.mask_id
            .as_ref()
            .and_then(|id| masks.iter().position(|m| &m.id == id))
            .or_else(|| (!masks.is_empty()).then_some(0))
    }
    pub fn effect_index(&self, item: &TimelineItem) -> Option<usize> {
        let effects = &item.visual_properties().effects;
        self.effect_id
            .as_ref()
            .and_then(|id| effects.iter().position(|e| e.id() == id))
            .or_else(|| (!effects.is_empty()).then_some(0))
    }
    pub fn resolve(&mut self, item: &TimelineItem) {
        self.mask_id = self
            .mask_index(item)
            .map(|i| item.visual_properties().masks[i].id.clone());
        self.effect_id = self
            .effect_index(item)
            .map(|i| item.visual_properties().effects[i].id().to_owned());
        if let Some(i) = self.mask_index(item) {
            let MaskSource::Path { path, paint } = &item.visual_properties().masks[i].source;
            self.command = self.command.min(path.commands.len().saturating_sub(1));
            self.stop = self.stop.min(stops(paint).len().saturating_sub(1));
        } else {
            self.command = 0;
            self.stop = 0;
        }
    }
    pub fn navigate(&mut self, item: &TimelineItem, axis: u8, next: bool) {
        self.resolve(item);
        let visual = item.visual_properties();
        let step = |index: usize, len: usize| {
            if next {
                index.saturating_add(1).min(len.saturating_sub(1))
            } else {
                index.saturating_sub(1)
            }
        };
        match axis {
            0 => {
                self.mask_id = self
                    .mask_index(item)
                    .map(|i| visual.masks[step(i, visual.masks.len())].id.clone());
                self.command = 0;
                self.stop = 0;
            }
            1 => {
                self.effect_id = self.effect_index(item).map(|i| {
                    visual.effects[step(i, visual.effects.len())]
                        .id()
                        .to_owned()
                })
            }
            2 | 3 => {
                if let Some(i) = self.mask_index(item) {
                    let MaskSource::Path { path, paint } = &visual.masks[i].source;
                    if axis == 2 {
                        self.command = step(self.command, path.commands.len());
                    } else {
                        self.stop = step(self.stop, stops(paint).len());
                    }
                }
            }
            _ => {}
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Eligibility {
    None,
    Leaf,
    Caption,
    Aggregate,
}
pub(crate) fn eligibility(
    project: &Project,
    selection: &Selection,
    item: &TimelineItem,
) -> Eligibility {
    if !animation_inspector::editable(selection) {
        return Eligibility::None;
    }
    match item {
        TimelineItem::Group(_) | TimelineItem::ComponentInstance(_) => Eligibility::Aggregate,
        TimelineItem::Caption(_) => Eligibility::Caption,
        TimelineItem::Repeater(_) | TimelineItem::Transition(_) => Eligibility::None,
        TimelineItem::Media(media) => {
            if project.assets.iter().any(|a| {
                a.id == media.asset_id
                    && matches!(
                        a.media_type,
                        opencut_editor_core::MediaType::Image
                            | opencut_editor_core::MediaType::Video
                    )
            }) {
                Eligibility::Leaf
            } else {
                Eligibility::None
            }
        }
        _ => Eligibility::Leaf,
    }
}
fn stops(paint: &Paint) -> &[opencut_editor_core::GradientStop] {
    match paint {
        Paint::Solid { .. } => &[],
        Paint::LinearGradient { stops, .. } | Paint::RadialGradient { stops, .. } => stops,
    }
}
fn add_pointer(
    fields: &mut Vec<Field>,
    source: &Value,
    prefix: &str,
    path: &str,
    key: &'static str,
    kind: FieldKind,
    label: String,
) {
    add_value(
        fields,
        source.pointer(path).unwrap_or(&Value::Null),
        label,
        format!("{prefix}{path}"),
        key,
        kind,
    );
}
fn rgba(
    fields: &mut Vec<Field>,
    source: &Value,
    prefix: &str,
    path: &str,
    key: &'static str,
    label: &str,
) {
    for component in ["r", "g", "b", "a"] {
        add_pointer(
            fields,
            source,
            prefix,
            &format!("{path}/{component}"),
            key,
            FieldKind::Number,
            format!("{label} {component}"),
        );
    }
}
pub(crate) fn fields(
    project: &Project,
    selection: &Selection,
    item: &TimelineItem,
    cursor: &Cursor,
) -> Vec<Field> {
    let eligible = eligibility(project, selection, item);
    let mut fs = vec![];
    let visual = item.visual_properties();
    if matches!(eligible, Eligibility::Leaf | Eligibility::Caption) {
        add_value(
            &mut fs,
            &serde_json::to_value(visual.blend_mode).unwrap(),
            "Blend mode",
            "/blendMode",
            "blendMode",
            FieldKind::Choice(BLENDS),
        );
    }
    if eligible == Eligibility::Leaf {
        add_value(
            &mut fs,
            &json!(visual.matte_only),
            "Matte only",
            "/matteOnly",
            "matteOnly",
            FieldKind::Boolean,
        );
        add_value(
            &mut fs,
            &json!(
                visual
                    .matte
                    .as_ref()
                    .map(|m| m.source_id.as_str())
                    .unwrap_or("")
            ),
            "Matte source ID · explicit provider",
            "/matte/sourceId",
            "matte",
            FieldKind::Text,
        );
        if let Some(matte) = &visual.matte {
            add_value(
                &mut fs,
                &serde_json::to_value(matte.channel).unwrap(),
                "Matte channel",
                "/matte/channel",
                "matte",
                FieldKind::Choice(&["alpha", "luma"]),
            );
        }
        if let Some(index) = cursor.mask_index(item) {
            let mask = &visual.masks[index];
            let prefix = format!("/masks/{index}");
            // No mask/path/gradient serialization: only bounded common scalars and the selected command.
            let source = json!({"channel":mask.channel,"operation":mask.operation,"inverted":mask.inverted,"featherPx":mask.feather_px,"expansionPx":mask.expansion_px,"transform":mask.transform});
            for (path, kind) in [
                ("/channel", FieldKind::Choice(&["alpha", "luma"])),
                (
                    "/operation",
                    FieldKind::Choice(&["add", "subtract", "intersect", "exclude"]),
                ),
                ("/inverted", FieldKind::Boolean),
                ("/featherPx", FieldKind::Number),
                ("/expansionPx", FieldKind::Number),
                (
                    "/transform/position/unit",
                    FieldKind::Choice(&["pixels", "normalized"]),
                ),
            ] {
                add_pointer(
                    &mut fs,
                    &source,
                    &prefix,
                    path,
                    "masks",
                    kind,
                    format!("Mask {}", path.trim_start_matches('/').replace('/', ".")),
                );
            }
            for path in [
                "/transform/position/x",
                "/transform/position/y",
                "/transform/anchor/x",
                "/transform/anchor/y",
                "/transform/scaleX",
                "/transform/scaleY",
                "/transform/rotationDeg",
                "/transform/skewXDeg",
                "/transform/skewYDeg",
                "/transform/opacity",
            ] {
                add_pointer(
                    &mut fs,
                    &source,
                    &prefix,
                    path,
                    "masks",
                    FieldKind::Number,
                    format!("Mask {}", path.trim_start_matches('/').replace('/', ".")),
                );
            }
            let MaskSource::Path { path, paint } = &mask.source;
            add_value(
                &mut fs,
                &serde_json::to_value(path.fill_rule).unwrap(),
                "Mask fill rule",
                format!("{prefix}/source/path/fillRule"),
                "masks",
                FieldKind::Choice(&["nonzero", "evenodd"]),
            );
            if let Paint::Solid { color } = paint {
                rgba(
                    &mut fs,
                    &json!({"color":color}),
                    &format!("{prefix}/source/paint"),
                    "/color",
                    "masks",
                    "Mask paint",
                );
            }
            if let Some(command) = path
                .commands
                .get(cursor.command.min(path.commands.len().saturating_sub(1)))
            {
                let selected = serde_json::to_value(command).unwrap();
                let command_index = cursor.command.min(path.commands.len().saturating_sub(1));
                for name in ["control1", "control2", "control", "to"] {
                    if selected.get(name).is_some() {
                        for axis in ["x", "y"] {
                            add_pointer(
                                &mut fs,
                                &selected,
                                &format!("{prefix}/source/path/commands/{command_index}"),
                                &format!("/{name}/{axis}"),
                                "masks",
                                FieldKind::Number,
                                format!("Mask command {name}.{axis}"),
                            );
                        }
                    }
                }
            }
        }
    }
    if matches!(eligible, Eligibility::Leaf | Eligibility::Aggregate)
        && let Some(index) = cursor.effect_index(item)
    {
        let source = serde_json::to_value(&visual.effects[index]).unwrap();
        for (name, value) in source.as_object().unwrap() {
            if name == "color" {
                rgba(
                    &mut fs,
                    &source,
                    &format!("/effects/{index}"),
                    "/color",
                    "effects",
                    "Effect color",
                );
            } else if name != "id" && name != "type" {
                let kind = match name.as_str() {
                    "count" => FieldKind::Unsigned16,
                    "seed" | "startMs" | "durationMs" | "lifetimeMs" => FieldKind::Integer,
                    _ => FieldKind::Number,
                };
                add_value(
                    &mut fs,
                    value,
                    format!("Effect {name}"),
                    format!("/effects/{index}/{name}"),
                    "effects",
                    kind,
                );
            }
        }
    }
    fs
}
pub(crate) fn descriptions(item: &TimelineItem, cursor: &Cursor) -> Vec<String> {
    let v = item.visual_properties();
    let mut result=vec!["Controlled Group/Instance: contiguous children → bounds clip → ordered effects → outward transform/opacity. Other layers: track, z-index, stable order; tree indentation describes parentage.".into(),format!("Masks: {} · Effects: {}",v.masks.len(),v.effects.len())];
    if let Some(i) = cursor.mask_index(item) {
        let mask = &v.masks[i];
        let MaskSource::Path { path, paint } = &mask.source;
        result.push(format!(
            "Mask {} / {} · ID {} (immutable)",
            i + 1,
            v.masks.len(),
            mask.id
        ));
        result.push(format!(
            "Mask source: path · paint {} (variants read-only)",
            match paint {
                Paint::Solid { .. } => "solid",
                Paint::LinearGradient { .. } => "linearGradient",
                Paint::RadialGradient { .. } => "radialGradient",
            }
        ));
        let n = cursor.command.min(path.commands.len().saturating_sub(1));
        if let Some(command) = path.commands.get(n) {
            result.push(format!(
                "Path command {} / {} · variant read-only: {}",
                n + 1,
                path.commands.len(),
                serde_json::to_string(command).unwrap()
            ));
        }
        match paint {
            Paint::Solid { .. } => {}
            Paint::LinearGradient { start, end, .. } => result.push(format!(
                "Linear gradient geometry read-only · start ({},{}) end ({},{})",
                start.x, start.y, end.x, end.y
            )),
            Paint::RadialGradient { center, radius, .. } => result.push(format!(
                "Radial gradient geometry read-only · center ({},{}) radius {}",
                center.x, center.y, radius
            )),
        }
        let stops = stops(paint);
        let index = cursor.stop.min(stops.len().saturating_sub(1));
        if let Some(stop) = stops.get(index) {
            result.push(format!(
                "Gradient stop {} / {} · offset/RGBA read-only: {}",
                index + 1,
                stops.len(),
                serde_json::to_string(stop).unwrap()
            ));
        }
    }
    if let Some(i) = cursor.effect_index(item) {
        result.push(format!(
            "Effect {} / {} · ID {} (immutable) · {}",
            i + 1,
            v.effects.len(),
            v.effects[i].id(),
            serde_json::to_string(&v.effects[i]).unwrap()
        ));
    }
    result.push(format!(
        "Clip: {}",
        v.clip.map(|_| "composition_bounds").unwrap_or("none")
    ));
    result
}
pub(crate) fn default_mask(id: String) -> Mask {
    serde_json::from_value(json!({"id":id,"source":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":64,"y":0}},{"type":"lineTo","to":{"x":64,"y":64}},{"type":"lineTo","to":{"x":0,"y":64}},{"type":"close"}]},"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}}},"channel":"alpha","operation":"add","inverted":false,"transform":opencut_editor_core::Transform2D::default(),"featherPx":0,"expansionPx":0})).unwrap()
}
pub(crate) fn default_effect(kind: &str, id: String) -> Result<VisualEffect, String> {
    let white = json!({"r":1,"g":1,"b":1,"a":1});
    let mut value = match kind {
        "gaussian_blur" => json!({"radiusPx":2}),
        "glow" => json!({"radiusPx":2,"intensity":0.5,"color":white}),
        "color_tint" => json!({"color":white}),
        "vignette" => json!({"amount":0.5}),
        "color_adjustment" => json!({"exposureStops":0,"contrast":1,"saturation":1}),
        "screen_flash" => json!({"startMs":0,"durationMs":200,"intensity":0.5,"color":white}),
        "particle_overlay" => {
            json!({"count":16,"seed":1,"radiusPx":2,"speedPxPerSecond":20,"lifetimeMs":1000,"color":white})
        }
        _ => return Err("Unknown effect type.".into()),
    };
    value["type"] = json!(kind);
    value["id"] = json!(id);
    serde_json::from_value(value).map_err(|e| e.to_string())
}
#[derive(Clone, Debug)]
pub(crate) enum Action {
    AddMask,
    AddEffect(&'static str),
    DeleteMask,
    DeleteEffect,
    MoveMask(bool),
    MoveEffect(bool),
    ClearMatte,
    SetClip(bool),
}
fn first_unused<'a>(prefix: &str, ids: impl Iterator<Item = &'a str>) -> String {
    let ids: std::collections::HashSet<_> = ids.collect();
    (1u64..)
        .map(|n| format!("{prefix}-{n}"))
        .find(|id| !ids.contains(id.as_str()))
        .unwrap()
}
fn update(item: &TimelineItem, key: &str, value: Value) -> Result<EditOperation, String> {
    serde_json::from_value(json!({"operation":"update_item","itemId":item.id(),key:value}))
        .map_err(|e| format!("Could not prepare compositing edit: {e}"))
}
pub(crate) fn action(
    project: &Project,
    selection: &Selection,
    item: &TimelineItem,
    cursor: &Cursor,
    action: Action,
) -> Result<(EditOperation, Cursor), String> {
    let eligible = eligibility(project, selection, item);
    let mut next = cursor.clone();
    next.resolve(item);
    let v = item.visual_properties();
    let mask_action = matches!(
        action,
        Action::AddMask | Action::DeleteMask | Action::MoveMask(_)
    );
    let effect_action = matches!(
        action,
        Action::AddEffect(_) | Action::DeleteEffect | Action::MoveEffect(_)
    );
    if (mask_action || matches!(action, Action::ClearMatte)) && eligible != Eligibility::Leaf
        || effect_action && !matches!(eligible, Eligibility::Leaf | Eligibility::Aggregate)
        || matches!(action, Action::SetClip(_)) && eligible != Eligibility::Aggregate
    {
        return Err("Compositing control unavailable in this scope.".into());
    }
    let (key, value) = match action {
        Action::ClearMatte => ("matte", Value::Null),
        Action::SetClip(set) => (
            "clip",
            if set {
                json!({"type":"composition_bounds"})
            } else {
                Value::Null
            },
        ),
        Action::AddMask => {
            let mut masks = v.masks.clone();
            let id = first_unused("mask", masks.iter().map(|m| m.id.as_str()));
            masks.push(default_mask(id.clone()));
            next.mask_id = Some(id);
            next.command = 0;
            next.stop = 0;
            ("masks", serde_json::to_value(masks).unwrap())
        }
        Action::AddEffect(kind) => {
            let mut effects = v.effects.clone();
            let id = first_unused("effect", effects.iter().map(VisualEffect::id));
            effects.push(default_effect(kind, id.clone())?);
            next.effect_id = Some(id);
            ("effects", serde_json::to_value(effects).unwrap())
        }
        Action::DeleteMask | Action::MoveMask(_) => {
            let mut masks = v.masks.clone();
            let i = next.mask_index(item).ok_or("No selected mask.")?;
            if let Action::MoveMask(later) = action {
                let j = if later {
                    i.saturating_add(1).min(masks.len() - 1)
                } else {
                    i.saturating_sub(1)
                };
                masks.swap(i, j);
            } else {
                masks.remove(i);
                next.mask_id = masks
                    .get(i.min(masks.len().saturating_sub(1)))
                    .map(|m| m.id.clone());
                next.command = 0;
                next.stop = 0;
            }
            ("masks", serde_json::to_value(masks).unwrap())
        }
        Action::DeleteEffect | Action::MoveEffect(_) => {
            let mut effects = v.effects.clone();
            let i = next.effect_index(item).ok_or("No selected effect.")?;
            if let Action::MoveEffect(later) = action {
                let j = if later {
                    i.saturating_add(1).min(effects.len() - 1)
                } else {
                    i.saturating_sub(1)
                };
                effects.swap(i, j);
            } else {
                effects.remove(i);
                next.effect_id = effects
                    .get(i.min(effects.len().saturating_sub(1)))
                    .map(|e| e.id().to_owned());
            }
            ("effects", serde_json::to_value(effects).unwrap())
        }
    };
    Ok((update(item, key, value)?, next))
}
