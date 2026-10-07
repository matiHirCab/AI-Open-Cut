//! Signed local domains and one immutable query program for a controlled frame.
use super::*;
use crate::evaluated_scene::mattes::{
    MatteFrameSchedule, OutputFrameBudget, QueryFrame, QueryScope,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SignedDomain {
    pub origin: [f64; 2],
    pub size: (u32, u32),
}
impl SignedDomain {
    fn bounds(bounds: [f64; 4]) -> Result<Self, CoreError> {
        if bounds.iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite aggregate support"));
        }
        let [left, top, right, bottom] = bounds;
        let width = (right.ceil() - left.floor()).max(1.);
        let height = (bottom.ceil() - top.floor()).max(1.);
        if width > 16384. || height > 16384. || width * height > 16_777_216. {
            return Err(invalid("signed aggregate surface exceeds bounds"));
        }
        Ok(Self {
            origin: [left.floor(), top.floor()],
            size: (width as u32, height as u32),
        })
    }
    pub(crate) fn rectangle(self) -> [f64; 4] {
        [
            self.origin[0],
            self.origin[1],
            self.origin[0] + f64::from(self.size.0),
            self.origin[1] + f64::from(self.size.1),
        ]
    }
}
#[derive(Debug)]
pub(crate) struct AggregateFrame {
    pub domain: SignedDomain,
    pub output: SignedDomain,
    pub outward: [f64; 6],
    pub inverse: [f64; 6],
    pub gain: f64,
    pub visible: bool,
    pub schedule: Option<MatteFrameSchedule>,
}
#[derive(Debug)]
pub(crate) struct FrameProgram {
    pub root: MatteFrameSchedule,
    pub aggregates: Vec<AggregateFrame>,
    pub retained_plane_bytes: u64,
}
fn union(a: Option<[f64; 4]>, b: [f64; 4]) -> Option<[f64; 4]> {
    Some(a.map_or(b, |a| {
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    }))
}
fn mapped(bounds: [f64; 4], matrix: [f64; 6]) -> [f64; 4] {
    let [left, top, right, bottom] = bounds;
    let [a, b, c, d, tx, ty] = matrix;
    let corners = [(left, top), (right, top), (left, bottom), (right, bottom)];
    let xs = corners.map(|(x, y)| a * x + c * y + tx);
    let ys = corners.map(|(x, y)| b * x + d * y + ty);
    [
        xs.into_iter().fold(f64::INFINITY, f64::min),
        ys.into_iter().fold(f64::INFINITY, f64::min),
        xs.into_iter().fold(f64::NEG_INFINITY, f64::max),
        ys.into_iter().fold(f64::NEG_INFINITY, f64::max),
    ]
}
fn padding(effects: &[crate::VisualEffect]) -> f64 {
    effects
        .iter()
        .map(|effect| match effect {
            crate::VisualEffect::GaussianBlur { radius_px, .. }
            | crate::VisualEffect::Glow { radius_px, .. } => (3. * radius_px).ceil(),
            crate::VisualEffect::ParticleOverlay { radius_px, .. } => radius_px.ceil(),
            _ => 0.,
        })
        .sum()
}
fn local_frame(
    scene: &EvaluatedScene,
    index: usize,
    at: u64,
    children: &[Option<AggregateFrame>],
) -> Result<AggregateFrame, CoreError> {
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate graph absent"))?;
    let node = &graph.nodes[index];
    let (outward, inverse, gain) = extended_visual::sample_stages(outward_stages(node)?, at)?;
    let visible = node.visible_at(at);
    let mut bounds = None;
    if visible {
        for (child, _) in graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.parent == Some(index))
        {
            let frame = children[child]
                .as_ref()
                .ok_or_else(|| invalid("aggregate child frame absent"))?;
            if frame.visible {
                bounds = union(bounds, mapped(frame.output.rectangle(), frame.outward));
            }
        }
        for (leaf, layer) in scene
            .visual_layers
            .iter()
            .enumerate()
            .filter(|(_, l)| leaf_owner(l) == Some(index))
        {
            if scene
                .mattes
                .as_ref()
                .is_some_and(|g| !g.roles[leaf].contributes || g.roles[leaf].matte_only)
            {
                continue;
            }
            let relative = relative_layer(layer, index)?;
            let times = layer
                .extended
                .as_ref()
                .and_then(|v| v.motion_blur)
                .map_or_else(
                    || Ok(vec![at]),
                    |blur| {
                        blur.sample_times(
                            at,
                            layer.extended.as_ref().unwrap().frame_rate,
                            scene.duration_ms,
                        )
                    },
                )?;
            for time in times {
                if relative_visible_at(scene, leaf, index, time)? {
                    bounds = union(
                        bounds,
                        extended_visual::sampled_support_bounds(&relative, time, node.basis)?,
                    );
                }
            }
        }
    }
    let basis = [0., 0., f64::from(node.basis.0), f64::from(node.basis.1)];
    if node.clip {
        bounds = Some(basis);
    } else if node
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::VisualEffect::ParticleOverlay { .. }))
    {
        bounds = union(bounds, basis);
    }
    let domain = SignedDomain::bounds(bounds.unwrap_or([0., 0., 1., 1.]))?;
    let pad = padding(&node.effects);
    let [left, top, right, bottom] = domain.rectangle();
    let output = SignedDomain::bounds([left - pad, top - pad, right + pad, bottom + pad])?;
    Ok(AggregateFrame {
        domain,
        output,
        outward,
        inverse,
        gain,
        visible,
        schedule: None,
    })
}
/// Plan every domain and query before source callbacks or raster allocation.
pub(crate) fn frame_program(scene: &EvaluatedScene, at: u64) -> Result<FrameProgram, CoreError> {
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate graph absent"))?;
    let descriptors = graph
        .nodes
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(scene.visual_layers.len()))
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n <= MAX_EVALUATED_VISUAL_LAYERS)
        .ok_or_else(|| invalid("aggregate/query descriptor limit exceeded"))?;
    let fixed = add(
        composition_resources::composition_heap_bytes(scene)?,
        mattes::MATTE_CACHE_RESERVATION,
    )?;
    let clone_reserve = scene
        .visual_layers
        .iter()
        .try_fold(0u64, |maximum, layer| {
            Ok::<_, CoreError>(
                maximum.max(mul(composition_resources::layer_heap_bytes(layer)?, 3)?),
            )
        })?;
    if add(add(fixed, mul(descriptors as u64, 8192)?)?, clone_reserve)?
        > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("aggregate query planning exceeds shared memory"));
    }
    let mut frames = Vec::new();
    frames
        .try_reserve_exact(graph.nodes.len())
        .map_err(|_| invalid("aggregate frame allocation failed"))?;
    // Resolve the private DAG iteratively: authored ordinal order need not put
    // parents before children, and nested controls add no new depth limit.
    frames.resize_with(graph.nodes.len(), || None);
    let mut remaining = frames.len();
    while remaining > 0 {
        let mut progressed = false;
        for index in (0..frames.len()).rev() {
            if frames[index].is_some()
                || graph
                    .nodes
                    .iter()
                    .enumerate()
                    .any(|(child, node)| node.parent == Some(index) && frames[child].is_none())
            {
                continue;
            }
            frames[index] = Some(local_frame(scene, index, at, &frames)?);
            remaining -= 1;
            progressed = true;
        }
        if !progressed {
            return Err(invalid("aggregate occurrence graph is cyclic"));
        }
    }
    let mut frames: Vec<_> = frames.into_iter().map(Option::unwrap).collect();
    let mut budget = OutputFrameBudget::new(scene)?;
    let mut plane_bytes = mul(
        mul(
            u64::from(scene.canvas.width),
            u64::from(scene.canvas.height),
        )?,
        16,
    )?;
    for (index, frame) in frames.iter().enumerate() {
        let node = &graph.nodes[index];
        plane_bytes = add(
            plane_bytes,
            mul(
                mul(
                    u64::from(frame.output.size.0),
                    u64::from(frame.output.size.1),
                )?,
                16,
            )?,
        )?;
        budget
            .sampled
            .certify_aggregate(frame.domain.size, &node.effects)?;
    }
    let mut world = Vec::new();
    world
        .try_reserve_exact(frames.len())
        .map_err(|_| invalid("aggregate world frame allocation failed"))?;
    world.resize(frames.len(), None);
    let mut remaining = world.len();
    while remaining > 0 {
        let mut progressed = false;
        for index in 0..world.len() {
            if world[index].is_some() {
                continue;
            }
            let parent = graph.nodes[index].parent;
            if parent.is_some_and(|parent| world.get(parent).is_none_or(Option::is_none)) {
                continue;
            }
            world[index] = Some(multiply_matrix(
                parent.map_or(IDENTITY_MATRIX, |p| world[p].unwrap()),
                frames[index].outward,
            ));
            remaining -= 1;
            progressed = true;
        }
        if !progressed {
            return Err(invalid("aggregate world graph is cyclic"));
        }
    }
    let headers = add(
        std::mem::size_of::<FrameProgram>() as u64,
        add(capacity_bytes(&frames)?, capacity_bytes(&world)?)?,
    )?;
    budget.retained_live_bytes = headers;
    let mut retained = add(add(fixed, plane_bytes)?, headers)?;
    let mut maximum_transient = 0;
    for (index, frame) in frames.iter_mut().enumerate() {
        if !frame.visible {
            continue;
        }
        let factors = world[index].unwrap();
        if factors.iter().any(|factor| !factor.is_finite()) {
            return Err(invalid("aggregate composed world frame must be finite"));
        }
        let [a, b, c, d, tx, ty] = factors;
        let [x, y] = frame.domain.origin;
        let query = QueryFrame::new(
            frame.domain.size,
            [a * x + c * y + tx, b * x + d * y + ty],
            [[a, b], [c, d]],
        )?;
        let schedule = mattes::frame_schedule_in_domain(
            scene,
            at,
            Some(QueryScope {
                frame: query,
                owner: Some(index),
                local_origin: frame.domain.origin,
            }),
            &mut budget,
        )?;
        retained = add(retained, schedule.certificate.descriptor_bytes)?;
        // All completed direct planes may remain live while nested aggregates
        // interleave; source/provider transient peaks include that overlap.
        let temporary = schedule
            .certificate
            .peak_live_bytes
            .checked_sub(schedule.certificate.fixed_live_bytes)
            .ok_or_else(|| invalid("aggregate query peak certificate invalid"))?;
        maximum_transient = maximum_transient.max(temporary);
        let owner_memory = extended_visual::certify_composition_memory(
            (scene.canvas.width, scene.canvas.height),
            frame.domain.size,
            &graph.nodes[index].effects,
            1.,
        )?
        .checked_sub(mattes::MATTE_CACHE_RESERVATION)
        .ok_or_else(|| invalid("aggregate owner memory invalid"))?;
        maximum_transient = maximum_transient.max(owner_memory);
        budget.retained_live_bytes = add(
            budget.retained_live_bytes,
            schedule.certificate.descriptor_bytes,
        )?;
        if add(retained, maximum_transient)? > mattes::MAX_MATTE_LIVE_BYTES {
            return Err(invalid("aggregate query live overlap exceeds bounds"));
        }
        frame.schedule = Some(schedule);
    }
    let root_query = QueryFrame::new(
        (scene.canvas.width, scene.canvas.height),
        [0., 0.],
        [[1., 0.], [0., 1.]],
    )?;
    let root = mattes::frame_schedule_in_domain(
        scene,
        at,
        Some(QueryScope {
            frame: root_query,
            owner: None,
            local_origin: [0., 0.],
        }),
        &mut budget,
    )?;
    retained = add(retained, root.certificate.descriptor_bytes)?;
    let root_transient = root
        .certificate
        .peak_live_bytes
        .checked_sub(root.certificate.fixed_live_bytes)
        .ok_or_else(|| invalid("root query peak certificate invalid"))?;
    maximum_transient = maximum_transient.max(root_transient);
    if add(retained, maximum_transient)? > mattes::MAX_MATTE_LIVE_BYTES {
        return Err(invalid("aggregate root query live overlap exceeds bounds"));
    }
    Ok(FrameProgram {
        root,
        aggregates: frames,
        retained_plane_bytes: plane_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn scene(clip: bool, leaf: bool) -> EvaluatedScene {
        let mut group = json!({"type":"group","id":"owner","startMs":0,"durationMs":500,"zIndex":0,"stackOrder":0,
            "effects":[{"id":"halo","type":"glow","radiusPx":1,"intensity":0.5,"color":{"r":0,"g":1,"b":0,"a":1}}]});
        if clip {
            group["clip"] = json!({"type":"composition_bounds"});
        }
        let mut items = vec![group];
        if leaf {
            items.push(json!({"type":"rectangle","id":"leaf","startMs":0,"durationMs":1000,"width":4,"height":3,"color":"#ff0000",
            "keyframes":[],"zIndex":0,"stackOrder":1,"parent":{"scope":"root","id":"owner"},
            "motionBlur":{"shutterAngleDeg":180,"sampleCount":2},
            "transform2d":{"position":{"x":-6,"y":-4,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}}));
        }
        let project: Project = serde_json::from_value(json!({"schemaVersion":37,"id":"p","revision":0,"name":"Local","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":32,"height":24,"fps":10},"assets":[],"components":[],"fonts":{},"markers":[],
            "tracks":[{"id":"t","name":"T","trackType":"overlay","items":items}]})).unwrap();
        let mut scene = evaluate_project(&project, 32, 24, 10).unwrap().scene;
        finalize_affine_geometry(&mut scene, &Default::default()).unwrap();
        scene
    }
    #[test]
    fn clip_precedes_owner_halo_and_signed_support_is_certified_before_world_culling() {
        let unclipped = frame_program(&scene(false, true), 400).unwrap();
        assert!(unclipped.aggregates[0].domain.origin[0] <= -6.);
        assert!(unclipped.aggregates[0].domain.origin[1] <= -4.);
        assert_eq!(
            unclipped.aggregates[0].output.origin[0],
            unclipped.aggregates[0].domain.origin[0] - 3.
        );
        let clipped = frame_program(&scene(true, true), 400).unwrap();
        assert_eq!(
            clipped.aggregates[0].domain,
            SignedDomain {
                origin: [0., 0.],
                size: (32, 24)
            }
        );
        assert_eq!(
            clipped.aggregates[0].output,
            SignedDomain {
                origin: [-3., -3.],
                size: (38, 30)
            }
        );
        assert_eq!(
            clipped.aggregates[0].schedule.as_ref().unwrap().canvas,
            (32, 24)
        );
        assert!(clipped.root.direct_draw.is_empty());
    }
    #[test]
    fn controlled_visibility_is_frozen_at_owner_output_but_descendants_keep_original_shutters() {
        let scene = scene(true, true);
        assert!(!scene.visual_layers[0].visible_at(515));
        assert!(relative_visible_at(&scene, 0, 0, 515).unwrap());
        let program = frame_program(&scene, 490).unwrap();
        let schedule = program.aggregates[0].schedule.as_ref().unwrap();
        let times: Vec<_> = schedule
            .tasks
            .iter()
            .filter_map(|task| {
                if let mattes::MatteTask::LeafSample {
                    at_ms,
                    relative_owner,
                    ..
                } = task
                {
                    assert_eq!(*relative_owner, Some(0));
                    Some(*at_ms)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(times.len(), 2);
        assert!(times.iter().any(|time| *time >= 500));
        assert!(!frame_program(&scene, 500).unwrap().aggregates[0].visible);
    }
    #[test]
    fn empty_owner_remains_a_real_query_descriptor_with_a_fixed_composition_basis() {
        let mut scene = scene(false, false);
        scene.aggregates.as_mut().unwrap().nodes[0].effects =
            vec![crate::VisualEffect::ParticleOverlay {
                id: "empty-particles".into(),
                count: 2,
                seed: 17,
                radius_px: 2.,
                speed_px_per_second: 10.,
                lifetime_ms: 997,
                color: crate::VectorColor {
                    r: 1.,
                    g: 0.,
                    b: 0.,
                    a: 0.5,
                },
            }];
        let program = frame_program(&scene, 400).unwrap();
        assert!(scene.visual_layers.is_empty());
        assert_eq!(program.aggregates.len(), 1);
        assert_eq!(
            program.aggregates[0].domain,
            SignedDomain {
                origin: [0., 0.],
                size: (32, 24)
            }
        );
        assert_eq!(
            program.aggregates[0].output,
            SignedDomain {
                origin: [-2., -2.],
                size: (36, 28)
            }
        );
        assert!(
            program.aggregates[0]
                .schedule
                .as_ref()
                .unwrap()
                .tasks
                .is_empty()
        );
        assert!(program.retained_plane_bytes >= (32 * 24 + 36 * 28) * 16);
    }
}
