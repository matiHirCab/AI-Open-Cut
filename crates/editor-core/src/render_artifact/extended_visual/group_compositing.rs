//! Consume completed certified queries in canonical contiguous owner order.
use super::super::mattes::{LeafSamplePlane, PreparedFrame};
use super::*;
use crate::evaluated_scene::group_compositing::frame::{FrameProgram, SignedDomain};
use crate::evaluated_scene::mattes::{MatteFrameSchedule, QueryFrame};

type QuerySampler<'a> = dyn FnMut(usize, u64, Option<usize>, QueryFrame, SignedDomain) -> Result<LeafSamplePlane, CoreError>
    + 'a;
#[derive(Clone, Copy)]
enum Draw {
    Leaf(usize),
    Aggregate(usize),
}
enum PaintLayer<'a> {
    Leaf(&'a super::super::mattes::LinearPlane, crate::BlendMode),
    Aggregate(
        &'a Raster,
        &'a crate::evaluated_scene::group_compositing::frame::AggregateFrame,
    ),
}
struct PaintContext<'a> {
    scene: &'a EvaluatedScene,
    program: &'a FrameProgram,
    completed: &'a [Option<Raster>],
}
impl PaintContext<'_> {
    fn plan<'a>(
        &'a self,
        owner: Option<usize>,
        prepared: &'a PreparedFrame,
        schedule: &MatteFrameSchedule,
    ) -> Result<Vec<PaintLayer<'a>>, CoreError> {
        let graph = self
            .scene
            .aggregates
            .as_ref()
            .ok_or_else(|| invalid("aggregate graph absent"))?;
        let mut order = Vec::new();
        order
            .try_reserve_exact(schedule.direct_draw.len() + graph.nodes.len())
            .map_err(|_| invalid("aggregate paint order allocation failed"))?;
        order.extend((0..schedule.direct_draw.len()).map(Draw::Leaf));
        order.extend(
            graph
                .nodes
                .iter()
                .enumerate()
                .filter(|(index, node)| {
                    node.parent == owner && self.program.aggregates[*index].visible
                })
                .map(|(index, _)| Draw::Aggregate(index)),
        );
        let key = |draw: &Draw| match draw {
            Draw::Leaf(index) => &graph.leaf_orders[schedule.direct_draw[*index].layer_index],
            Draw::Aggregate(index) => &graph.nodes[*index].order,
        };
        order.sort_by(|a, b| key(a).cmp(key(b)));
        let mut planes = Vec::new();
        planes
            .try_reserve_exact(order.len())
            .map_err(|_| invalid("aggregate completed paint allocation failed"))?;
        for draw in order {
            planes.push(match draw {
                Draw::Leaf(index) => {
                    let draw = &schedule.direct_draw[index];
                    PaintLayer::Leaf(prepared.plane(draw.task)?, draw.blend_mode)
                }
                Draw::Aggregate(index) => {
                    let raster = self.completed[index]
                        .as_ref()
                        .ok_or_else(|| invalid("aggregate child output absent"))?;
                    let frame = &self.program.aggregates[index];
                    if !frame.gain.is_finite()
                        || !(0.0..=1.0).contains(&frame.gain)
                        || frame.inverse.iter().any(|v| !v.is_finite())
                        || (raster.width as u32, raster.height as u32) != frame.output.size
                    {
                        return Err(invalid(
                            "aggregate completed output differs from certificate",
                        ));
                    }
                    super::super::mattes::validate_plane_pixels(&raster.pixels)?;
                    let domain = owner.map_or(
                        SignedDomain {
                            origin: [0., 0.],
                            size: self.program.root.canvas,
                        },
                        |owner| self.program.aggregates[owner].domain,
                    );
                    let [a, b, c, d, tx, ty] = frame.inverse;
                    let [x, y] = domain.origin;
                    QueryFrame::new(
                        domain.size,
                        [
                            a * x + c * y + tx - frame.output.origin[0],
                            b * x + d * y + ty - frame.output.origin[1],
                        ],
                        [[a, b], [c, d]],
                    )?;
                    PaintLayer::Aggregate(raster, frame)
                }
            });
        }
        Ok(planes)
    }
}
/// Every reference, capacity and numeric boundary is admitted by plan(). This
/// final commit has no allocation, callback, admission or fallible operation.
fn paint(planes: &[PaintLayer<'_>], domain: SignedDomain, destination: &mut Raster) {
    for plane in planes {
        match plane {
            PaintLayer::Leaf(plane, mode) => {
                for y in 0..plane.height {
                    for x in 0..plane.width {
                        let pixel = &mut destination.pixels
                            [(plane.top + y) * destination.width + plane.left + x];
                        let source = plane.pixels[y * plane.width + x];
                        if mode.is_normal() {
                            super::super::mattes::source_over(pixel, source);
                        } else {
                            super::super::blend::composite(pixel, source, *mode);
                        }
                    }
                }
            }
            PaintLayer::Aggregate(raster, frame) => {
                let [a, b, c, d, tx, ty] = frame.inverse;
                for y in 0..destination.height {
                    for x in 0..destination.width {
                        let px = x as f64 + 0.5 + domain.origin[0];
                        let py = y as f64 + 0.5 + domain.origin[1];
                        let mut pixel = raster.bilinear(
                            a * px + c * py + tx - frame.output.origin[0],
                            b * px + d * py + ty - frame.output.origin[1],
                        );
                        for channel in &mut pixel {
                            *channel *= frame.gain as f32;
                        }
                        super::super::mattes::source_over(
                            &mut destination.pixels[y * destination.width + x],
                            pixel,
                        );
                    }
                }
            }
        }
    }
}
pub(super) fn compose(
    scene: &EvaluatedScene,
    program: &FrameProgram,
    at: u64,
    destination: &mut Raster,
    sample: &mut QuerySampler<'_>,
) -> Result<(), CoreError> {
    if (destination.width as u32, destination.height as u32) != program.root.canvas {
        return Err(invalid("aggregate destination dimensions differ"));
    }
    super::super::mattes::validate_destination(program.root.canvas, &destination.pixels)?;
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate graph absent"))?;
    let mut completed = Vec::new();
    completed
        .try_reserve_exact(graph.nodes.len())
        .map_err(|_| invalid("aggregate output metadata allocation failed"))?;
    completed.resize_with(graph.nodes.len(), || None);
    let mut retained_planes = (destination.pixels.capacity() as u64)
        .checked_mul(16)
        .ok_or_else(|| invalid("aggregate retained plane capacity overflow"))?;
    if retained_planes > program.retained_plane_bytes {
        return Err(invalid("aggregate caller capacity exceeds certificate"));
    }
    let mut pending = program
        .aggregates
        .iter()
        .filter(|frame| frame.visible)
        .count();
    while pending > 0 {
        let mut progressed = false;
        for index in (0..completed.len()).rev() {
            let frame = &program.aggregates[index];
            if !frame.visible
                || completed[index].is_some()
                || graph.nodes.iter().enumerate().any(|(child, node)| {
                    node.parent == Some(index)
                        && program.aggregates[child].visible
                        && completed[child].is_none()
                })
            {
                continue;
            }
            let schedule = frame
                .schedule
                .as_ref()
                .ok_or_else(|| invalid("aggregate query absent"))?;
            let query = schedule
                .query
                .ok_or_else(|| invalid("aggregate query frame absent"))?;
            let mut local =
                Raster::empty(frame.domain.size.0 as usize, frame.domain.size.1 as usize)?;
            let prepared = super::super::mattes::prepare_frame(
                schedule,
                &local.pixels,
                &mut |index, time, owner| sample(index, time, owner, query, frame.domain),
            )?;
            let _payload = prepared.payload_bytes()?;
            let context = PaintContext {
                scene,
                program,
                completed: &completed,
            };
            let planes = context.plan(Some(index), &prepared, schedule)?;
            paint(&planes, frame.domain, &mut local);
            drop(planes);
            drop(prepared);
            let node = &graph.nodes[index];
            if node.clip {
                for y in 0..local.height {
                    for x in 0..local.width {
                        let px = x as f64 + 0.5 + frame.domain.origin[0];
                        let py = y as f64 + 0.5 + frame.domain.origin[1];
                        if px < 0.
                            || py < 0.
                            || px >= f64::from(node.basis.0)
                            || py >= f64::from(node.basis.1)
                        {
                            local.pixels[y * local.width + x] = [0.; 4];
                        }
                    }
                }
            }
            let bounds = Some([
                -frame.domain.origin[0],
                -frame.domain.origin[1],
                f64::from(node.basis.0) - frame.domain.origin[0],
                f64::from(node.basis.1) - frame.domain.origin[1],
            ]);
            let output =
                effects_in_domain(local, &node.effects, 1., bounds, bounds, node.time(at))?.0;
            retained_planes = retained_planes
                .checked_add(
                    (output.pixels.capacity() as u64)
                        .checked_mul(16)
                        .ok_or_else(|| invalid("aggregate output capacity overflow"))?,
                )
                .ok_or_else(|| invalid("aggregate retained capacity overflow"))?;
            if retained_planes > program.retained_plane_bytes {
                return Err(invalid("aggregate output capacities exceed certificate"));
            }
            completed[index] = Some(output);
            pending -= 1;
            progressed = true;
        }
        if !progressed {
            return Err(invalid("aggregate output graph is cyclic"));
        }
    }
    let domain = SignedDomain {
        origin: [0., 0.],
        size: program.root.canvas,
    };
    let query = program
        .root
        .query
        .ok_or_else(|| invalid("root query frame absent"))?;
    let prepared = super::super::mattes::prepare_frame(
        &program.root,
        &destination.pixels,
        &mut |index, time, owner| sample(index, time, owner, query, domain),
    )?;
    let context = PaintContext {
        scene,
        program,
        completed: &completed,
    };
    let planes = context.plan(None, &prepared, &program.root)?;
    paint(&planes, domain, destination);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn invalid_destination_and_late_source_failure_leave_every_original_bit_unchanged() {
        let (_, scene, _) = prepared_pixels(json!([
            {"type":"group","id":"g","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,"clip":{"type":"composition_bounds"}},
            {"type":"rectangle","id":"inside","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,"width":4,"height":4,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"g"}},
            {"type":"rectangle","id":"outside","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":2,"width":4,"height":4,"color":"#00ff00","keyframes":[]}
        ]));
        let program =
            crate::evaluated_scene::group_compositing::frame::frame_program(&scene, 400).unwrap();
        let bits = |r: &Raster| {
            r.pixels
                .iter()
                .flat_map(|p| p.iter().map(|c| c.to_bits()))
                .collect::<Vec<_>>()
        };
        for bad in [f32::NAN, f32::INFINITY, -0.5, 1.5] {
            let mut destination = Raster::empty(64, 64).unwrap();
            destination.pixels.fill([0.1, 0.2, 0.3, 1.]);
            destination.pixels[4095][0] = bad;
            let before = bits(&destination);
            let mut calls = 0;
            let result = compose(
                &scene,
                &program,
                400,
                &mut destination,
                &mut |_, _, _, _, _| {
                    calls += 1;
                    Err(invalid("must never call source"))
                },
            );
            assert!(result.is_err());
            assert_eq!(calls, 0);
            assert_eq!(bits(&destination), before);
        }
        let mut destination = Raster::empty(64, 64).unwrap();
        destination.pixels.fill([0.1, 0.2, 0.3, 1.]);
        let before = bits(&destination);
        let mut calls = 0;
        let result = compose(
            &scene,
            &program,
            400,
            &mut destination,
            &mut |_, _, _, query, _| {
                calls += 1;
                if calls == 2 {
                    return Err(invalid("late root source failure"));
                }
                let (width, height) = (query.size.0 as usize, query.size.1 as usize);
                Ok(LeafSamplePlane {
                    plane: super::super::super::mattes::LinearPlane {
                        left: 0,
                        top: 0,
                        width,
                        height,
                        pixels: vec![[0.5, 0., 0., 0.5]; width * height],
                    },
                    gain: 1.,
                })
            },
        );
        assert!(result.is_err());
        assert_eq!(calls, 2);
        assert_eq!(bits(&destination), before);
    }
    #[test]
    fn bilinear_signed_extreme_coordinates_are_transparent_before_integer_conversion() {
        let mut raster = Raster::empty(2, 2).unwrap();
        raster.pixels.fill([1.; 4]);
        for coordinate in [
            f64::MAX,
            -f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ] {
            assert_eq!(raster.bilinear(coordinate, 0.5), [0.; 4]);
            assert_eq!(raster.bilinear(0.5, coordinate), [0.; 4]);
        }
        assert_eq!(raster.bilinear(0.5, 0.5), [1.; 4]);
        assert_eq!(raster.bilinear(0., 0.5), [0.5; 4]);
    }
    fn prepared_pixels(
        items: serde_json::Value,
    ) -> (Raster, EvaluatedScene, PreparedRenderResources) {
        prepared_pixels_at(items, 400)
    }
    fn prepared_pixels_at(
        items: serde_json::Value,
        at: u64,
    ) -> (Raster, EvaluatedScene, PreparedRenderResources) {
        let project:crate::Project=serde_json::from_value(json!({"schemaVersion":37,"id":"p","revision":0,"name":"Pure","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":64,"height":64,"fps":10},"fonts":{},"markers":[],"assets":[],"components":[],
            "tracks":[{"id":"t","name":"T","trackType":"overlay","items":items}]})).unwrap();
        let mut scene = crate::evaluated_scene::evaluate_project(&project, 64, 64, 10)
            .unwrap()
            .scene;
        crate::evaluated_scene::finalize_affine_geometry(&mut scene, &Default::default()).unwrap();
        extended_visual::preflight_samples(&scene, at, at + 1, true).unwrap();
        let root = tempfile::tempdir().unwrap();
        let mut resources = PreparedRenderResources {
            media_inputs: vec![],
            media_paths: vec![],
            text_layers: Default::default(),
        };
        super::super::prepare(
            &super::super::super::FileSystemArtifactIo,
            &mut scene,
            root.path(),
            &mut resources,
            RenderIntent::Frame { at_ms: at },
            &VisualPreparation {
                decode: &|_, _, _| panic!("solid sources must not decode"),
                encode: &|_, _, _, _| panic!("frame must not encode stream"),
                caption: &|_, _, _| panic!("solid sources must not paint text"),
            },
        )
        .unwrap();
        let pixels =
            Raster::pam(&std::fs::read(resources.media_paths.last().unwrap()).unwrap()).unwrap();
        (pixels, scene, resources)
    }
    #[test]
    fn animated_path_keeps_original_particle_centers_and_allocates_both_source_domains() {
        let points = |offset: f64| json!([{"x":offset,"y":0},{"x":offset+8.,"y":0},{"x":offset+8.,"y":8},{"x":offset,"y":8}]);
        let items = json!([{"type":"shape","id":"path","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,"keyframes":[],
            "geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":8,"y":0}},{"type":"lineTo","to":{"x":8,"y":8}},{"type":"lineTo","to":{"x":0,"y":8}},{"type":"close"}]}},
            "fill":{"type":"solid","color":{"r":0,"g":1,"b":0,"a":1}},"stroke":null,
            "effects":[{"id":"fixed-particles","type":"particle_overlay","count":6,"seed":173,"radiusPx":2,"speedPxPerSecond":0,"lifetimeMs":997,"color":{"r":1,"g":0,"b":0,"a":1}}],
            "animationChannels":[{"property":"graphic.path_points","target":{"kind":"graphic_geometry","scope":"root","id":"path"},
                "keyframes":[{"timeMs":0,"value":{"type":"path_points","points":points(0.)},"curve":"linear"},{"timeMs":999,"value":{"type":"path_points","points":points(32.)},"curve":"hold"}]}]
        }]);
        let (first, _, _) = prepared_pixels_at(items.clone(), 0);
        let (last, _, _) = prepared_pixels_at(items, 999);
        assert!(first.pixels.iter().any(|p| p[0] > 0.1));
        assert_eq!(
            first.pixels.iter().map(|p| p[0]).collect::<Vec<_>>(),
            last.pixels.iter().map(|p| p[0]).collect::<Vec<_>>()
        );
        for y in 0..64 {
            for x in 12..64 {
                assert_eq!(last.pixels[y * 64 + x][0], 0., "particle moved to {x},{y}");
            }
        }
        assert_eq!(last.pixels[4 * 64 + 36], [0., 1., 0., 1.]);
        assert_eq!(first.pixels[4 * 64 + 36], [0., 0., 0., 1.]);
    }
    #[test]
    fn actual_preparation_composites_owner_opacity_once_and_keeps_children_inside_owner_block() {
        let (pixels, scene, resources) = prepared_pixels(json!([
            {"type":"group","id":"g","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,"clip":{"type":"composition_bounds"},
                "transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.5}},
            {"type":"rectangle","id":"red","startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","zIndex":0,"stackOrder":1,"keyframes":[],
                "parent":{"scope":"root","id":"g"},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":0.5}},
            {"type":"rectangle","id":"blue","startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#0000ff","zIndex":99,"stackOrder":2,"keyframes":[],
                "parent":{"scope":"root","id":"g"},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":0.5}},
            {"type":"rectangle","id":"green","startMs":0,"durationMs":1000,"width":2,"height":2,"color":"#00ff00","zIndex":1,"stackOrder":3,"keyframes":[],
                "transform":{"positionX":1,"positionY":1,"scale":1,"opacity":1}}
        ]));
        // Independently derived linear red=.5*(.5*(1-.5))=.125,
        // blue=.5*.5=.25; PAM roundtrip quantizes to encoded 99/137.
        let actual = pixels.pam_bytes();
        let header = actual.windows(7).position(|w| w == b"ENDHDR\n").unwrap() + 7;
        assert_eq!(&actual[header..header + 4], &[99, 0, 137, 255]);
        assert_eq!(pixels.pixels[65], [0., 1., 0., 1.]);
        assert_eq!(pixels.pixels[8], [0., 0., 0., 1.]);
        assert_eq!(scene.composed_input.as_ref().unwrap().0, "linear-scene");
        assert!(
            scene
                .visual_layers
                .iter()
                .all(|layer| layer.sampled_input.is_none())
        );
        let plan = crate::render_plan::build_render_plan(
            &scene,
            &resources.text_layers,
            resources.media_inputs,
            resources.media_paths,
            None,
            RenderIntent::Frame { at_ms: 400 },
            &mut vec![],
        )
        .unwrap();
        assert!(format!("{plan:?}").contains("[base0][sampled1]overlay"));
    }
    #[test]
    fn empty_particle_owner_prepares_real_pixels_without_fake_authored_leaves() {
        let (pixels, scene, _) = prepared_pixels(json!([
            {"type":"group","id":"empty","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,
                "effects":[{"id":"particles","type":"particle_overlay","count":20,"seed":173,"radiusPx":2,"speedPxPerSecond":12,"lifetimeMs":997,
                    "color":{"r":1,"g":0,"b":0,"a":0.7}}]}
        ]));
        assert!(scene.visual_layers.is_empty());
        assert!(scene.composed_input.is_some());
        assert!(pixels.pixels.iter().any(|p| p[0] > 0.1));
        assert!(
            pixels
                .pixels
                .iter()
                .all(|p| p[3] == 1. && p[1] == 0. && p[2] == 0.)
        );
    }
}
