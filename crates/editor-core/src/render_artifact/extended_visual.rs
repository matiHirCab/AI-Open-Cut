//! Bounded local raster effects and sampled visual resources.
use super::{ArtifactIo, GRAPH_BUILD_STAGE, PreparedRenderResources};
use crate::evaluated_scene::{EvaluatedScene, EvaluatedVisualSource, extended_visual};
use crate::render_plan::{MediaInputRequest, RenderIntent};
use crate::{CoreError, ErrorCode, MediaCrop, MediaType, VisualEffect};
use std::{collections::HashMap, path::Path};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}
fn linear(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(v: f64) -> f64 {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
const MAX_PIXELS: usize = 16_777_216;

#[derive(Clone)]
struct Raster {
    width: usize,
    height: usize,
    pixels: Vec<[f32; 4]>,
}
impl Raster {
    fn empty(width: usize, height: usize) -> Result<Self, CoreError> {
        let count = width
            .checked_mul(height)
            .filter(|n| *n <= MAX_PIXELS && width <= 16384 && height <= 16384)
            .ok_or_else(|| invalid("effect raster exceeds limits"))?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(count)
            .map_err(|_| invalid("effect raster allocation failed"))?;
        pixels.resize(count, [0.0; 4]);
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    fn rgba(width: usize, height: usize, bytes: &[u8]) -> Result<Self, CoreError> {
        let mut raster = Self::empty(width, height)?;
        if bytes.len() != width * height * 4 {
            return Err(invalid("decoded raster size mismatch"));
        }
        for (pixel, bytes) in raster.pixels.iter_mut().zip(bytes.as_chunks::<4>().0) {
            let a = f64::from(bytes[3]) / 255.0;
            *pixel = [
                (linear(f64::from(bytes[0]) / 255.0) * a) as f32,
                (linear(f64::from(bytes[1]) / 255.0) * a) as f32,
                (linear(f64::from(bytes[2]) / 255.0) * a) as f32,
                a as f32,
            ];
        }
        Ok(raster)
    }
    fn pam(bytes: &[u8]) -> Result<Self, CoreError> {
        let end = bytes
            .windows(7)
            .position(|v| v == b"ENDHDR\n")
            .map(|i| i + 7)
            .ok_or_else(|| invalid("missing PAM header"))?;
        let header =
            std::str::from_utf8(&bytes[..end]).map_err(|_| invalid("invalid PAM header"))?;
        let dimension = |key: &str| {
            header
                .lines()
                .find_map(|line| line.strip_prefix(key)?.parse::<usize>().ok())
                .ok_or_else(|| invalid("invalid PAM dimensions"))
        };
        Self::rgba(dimension("WIDTH ")?, dimension("HEIGHT ")?, &bytes[end..])
    }
    fn bilinear(&self, x: f64, y: f64) -> [f32; 4] {
        let x = x - 0.5;
        let y = y - 0.5;
        let ix = x.floor() as i64;
        let iy = y.floor() as i64;
        let fx = (x - ix as f64) as f32;
        let fy = (y - iy as f64) as f32;
        let mut result = [0.0; 4];
        for (dx, dy, weight) in [
            (0, 0, (1.0 - fx) * (1.0 - fy)),
            (1, 0, fx * (1.0 - fy)),
            (0, 1, (1.0 - fx) * fy),
            (1, 1, fx * fy),
        ] {
            let (x, y) = (ix + dx, iy + dy);
            if x >= 0 && y >= 0 && x < self.width as i64 && y < self.height as i64 {
                let pixel = self.pixels[y as usize * self.width + x as usize];
                for i in 0..4 {
                    result[i] += pixel[i] * weight;
                }
            }
        }
        result
    }
    fn crop(&self, crop: MediaCrop) -> Result<Self, CoreError> {
        let mut result = Self::empty(self.width, self.height)?;
        let coordinate = |origin: f64, extent: f64, position: f64| {
            if extent >= 1.0 {
                position.clamp(origin + 0.5, origin + extent - 0.5)
            } else {
                origin + extent * 0.5
            }
        };
        for y in 0..self.height {
            for x in 0..self.width {
                result.pixels[y * self.width + x] = self.bilinear(
                    coordinate(
                        crop.x * self.width as f64,
                        crop.width * self.width as f64,
                        (crop.x * self.width as f64) + (x as f64 + 0.5) * crop.width,
                    ),
                    coordinate(
                        crop.y * self.height as f64,
                        crop.height * self.height as f64,
                        (crop.y * self.height as f64) + (y as f64 + 0.5) * crop.height,
                    ),
                );
            }
        }
        Ok(result)
    }
    fn padded(&self, pad: usize) -> Result<Self, CoreError> {
        let mut result = Self::empty(self.width + 2 * pad, self.height + 2 * pad)?;
        for y in 0..self.height {
            let start = (y + pad) * result.width + pad;
            result.pixels[start..start + self.width]
                .copy_from_slice(&self.pixels[y * self.width..(y + 1) * self.width]);
        }
        Ok(result)
    }
    fn blur(&self, sigma: f64) -> Result<Self, CoreError> {
        if sigma == 0.0 {
            return Ok(self.clone());
        }
        let radius = (3.0 * sigma).ceil() as i64;
        let mut kernel: Vec<f32> = (-radius..=radius)
            .map(|x| (-0.5 * (x as f64 / sigma).powi(2)).exp() as f32)
            .collect();
        let sum: f32 = kernel.iter().sum();
        for weight in &mut kernel {
            *weight /= sum;
        }
        let mut horizontal = Self::empty(self.width, self.height)?;
        let mut result = Self::empty(self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                for (i, w) in kernel.iter().enumerate() {
                    let sx = x as i64 + i as i64 - radius;
                    if sx >= 0 && sx < self.width as i64 {
                        let p = self.pixels[y * self.width + sx as usize];
                        for (c, value) in p.iter().enumerate() {
                            horizontal.pixels[y * self.width + x][c] += value * w;
                        }
                    }
                }
            }
        }
        for y in 0..self.height {
            for x in 0..self.width {
                for (i, w) in kernel.iter().enumerate() {
                    let sy = y as i64 + i as i64 - radius;
                    if sy >= 0 && sy < self.height as i64 {
                        let p = horizontal.pixels[sy as usize * self.width + x];
                        for (c, value) in p.iter().enumerate() {
                            result.pixels[y * self.width + x][c] += value * w;
                        }
                    }
                }
            }
        }
        Ok(result)
    }
    fn pam_bytes(&self) -> Vec<u8> {
        let mut bytes = format!(
            "P7\nWIDTH {}\nHEIGHT {}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n",
            self.width, self.height
        )
        .into_bytes();
        bytes.reserve(self.width * self.height * 4);
        for p in &self.pixels {
            let a = f64::from(p[3]).clamp(0.0, 1.0);
            for c in &p[..3] {
                let v = if a == 0.0 {
                    0.0
                } else {
                    srgb((f64::from(*c) / a).clamp(0.0, 1.0))
                };
                bytes.push((v * 255.0).round() as u8);
            }
            bytes.push((a * 255.0).round() as u8);
        }
        bytes
    }
}

fn support(effects: &[VisualEffect], density: f64) -> usize {
    effects
        .iter()
        .map(|e| match e {
            VisualEffect::GaussianBlur { radius_px, .. } | VisualEffect::Glow { radius_px, .. } => {
                (3.0 * radius_px * density).ceil() as usize
            }
            _ => 0,
        })
        .sum()
}

fn effects(
    mut raster: Raster,
    effects: &[VisualEffect],
    density: f64,
    bounds: Option<[f64; 4]>,
) -> Result<(Raster, usize), CoreError> {
    let pad = support(effects, density);
    let bounds = bounds.unwrap_or([0.0, 0.0, raster.width as f64, raster.height as f64]);
    raster = raster.padded(pad)?;
    for effect in effects {
        match effect {
            VisualEffect::GaussianBlur { radius_px, .. } => {
                raster = raster.blur(radius_px * density)?
            }
            VisualEffect::Glow {
                radius_px,
                intensity,
                color,
                ..
            } => {
                let halo = raster.blur(radius_px * density)?;
                for (dst, blurred) in raster.pixels.iter_mut().zip(halo.pixels) {
                    let alpha = f64::from(blurred[3]) * intensity * color.a;
                    let behind = [
                        (linear(color.r) * alpha) as f32,
                        (linear(color.g) * alpha) as f32,
                        (linear(color.b) * alpha) as f32,
                        alpha as f32,
                    ];
                    for c in 0..4 {
                        dst[c] += behind[c] * (1.0 - dst[3]);
                    }
                }
            }
            VisualEffect::ColorTint { color, .. } => {
                for p in &mut raster.pixels {
                    for (c, v) in [color.r, color.g, color.b].into_iter().enumerate() {
                        p[c] = p[c] * (1.0 - color.a as f32) + (linear(v) * color.a) as f32 * p[3];
                    }
                }
            }
            VisualEffect::Vignette { amount, .. } => {
                for y in 0..raster.height {
                    for x in 0..raster.width {
                        let u = 2.0 * (x as f64 + 0.5 - pad as f64 - bounds[0])
                            / (bounds[2] - bounds[0]).max(f64::EPSILON)
                            - 1.0;
                        let v = 2.0 * (y as f64 + 0.5 - pad as f64 - bounds[1])
                            / (bounds[3] - bounds[1]).max(f64::EPSILON)
                            - 1.0;
                        let factor =
                            (1.0 - amount * ((u * u + v * v) / 2.0).clamp(0.0, 1.0)) as f32;
                        for c in 0..3 {
                            raster.pixels[y * raster.width + x][c] *= factor;
                        }
                    }
                }
            }
        }
    }
    Ok((raster, pad))
}

fn sample_times(scene: &EvaluatedScene, intent: RenderIntent) -> (u64, u64, bool) {
    match intent {
        RenderIntent::Frame { at_ms } => (at_ms, at_ms + 1, true),
        RenderIntent::Range {
            start_ms, end_ms, ..
        } => (start_ms, end_ms, false),
        RenderIntent::Export => (0, scene.duration_ms, false),
    }
}

type VisualDecoder<'a> = dyn Fn(&Path, u64, (u32, u32)) -> Result<Vec<u8>, CoreError> + 'a;
type FrameProducer<'a> = dyn FnMut(u64) -> Result<Vec<u8>, CoreError> + 'a;
type VisualEncoder<'a> =
    dyn Fn(&Path, u32, u64, &mut FrameProducer<'_>) -> Result<(), CoreError> + 'a;

pub(crate) type CaptionRasterizer<'a> = dyn Fn(
        &crate::evaluated_scene::EvaluatedVisualLayer,
        Option<&crate::render_plan::PreparedText>,
        (u32, u32),
    ) -> Result<Vec<u8>, CoreError>
    + 'a;

pub(crate) struct VisualPreparation<'a> {
    pub(crate) decode: &'a VisualDecoder<'a>,
    pub(crate) encode: &'a VisualEncoder<'a>,
    pub(crate) caption: &'a CaptionRasterizer<'a>,
}

pub(crate) fn prepare(
    io: &dyn ArtifactIo,
    scene: &mut EvaluatedScene,
    workspace: &Path,
    resources: &mut PreparedRenderResources,
    intent: RenderIntent,
    preparation: &VisualPreparation<'_>,
) -> Result<(), CoreError> {
    let VisualPreparation {
        decode,
        encode,
        caption,
    } = preparation;
    let (start, end, frame) = sample_times(scene, intent);
    let canvas = (scene.canvas.width, scene.canvas.height);
    let fps = scene.canvas.fps;
    let duration = scene.duration_ms;
    let frames = if frame {
        1
    } else {
        (end - start)
            .saturating_mul(u64::from(scene.canvas.fps))
            .div_ceil(1000)
            .max(1)
    };
    let mut bindings = HashMap::new();
    for (input, path) in resources.media_inputs.iter().zip(&resources.media_paths) {
        bindings.insert(input.item_id.clone(), (input.clone(), path.clone()));
    }
    for (index, layer) in scene.visual_layers.iter_mut().enumerate() {
        if !extended_visual::sampling_required(layer, start, fps) {
            continue;
        }
        let binding_id = format!("extended-sample-{index}");
        let file = if frame {
            format!("sampled-{index}.pam")
        } else {
            format!("sampled-{index}.mkv")
        };
        // One static source per layer; drop it after that layer's streamed output.
        let caption_raster = if matches!(layer.source, EvaluatedVisualSource::Caption(_)) {
            let size = layer
                .source_size
                .ok_or_else(|| invalid("sampled source measurement missing"))?;
            extended_visual::validate_sampled_source_size(size)?;
            Some(Raster::rgba(
                size.0 as usize,
                size.1 as usize,
                &caption(layer, resources.text_layers.get(&layer.item_id), size)?,
            )?)
        } else {
            None
        };
        let draw = |at| {
            let (mut sampled, crop, effect_stack) = extended_visual::sample(layer, at)?;
            let result = if !layer.visible_at(at) {
                Raster::empty(canvas.0 as usize, canvas.1 as usize)?
            } else {
                let (mut raster, density) = match &sampled.source {
                    EvaluatedVisualSource::Shape(shape) => (
                        Raster::pam(&super::shapes::rasterize(shape)?)?,
                        shape.density,
                    ),
                    EvaluatedVisualSource::SolidColor { color }
                    | EvaluatedVisualSource::Rectangle { color, .. } => {
                        let size = sampled.source_size.unwrap_or(canvas);
                        let mut raster = Raster::empty(size.0 as usize, size.1 as usize)?;
                        let rgb = [
                            u8::from_str_radix(&color[1..3], 16).unwrap(),
                            u8::from_str_radix(&color[3..5], 16).unwrap(),
                            u8::from_str_radix(&color[5..7], 16).unwrap(),
                        ];
                        let pixel = [
                            linear(rgb[0] as f64 / 255.0) as f32,
                            linear(rgb[1] as f64 / 255.0) as f32,
                            linear(rgb[2] as f64 / 255.0) as f32,
                            1.0,
                        ];
                        raster.pixels.fill(pixel);
                        (raster, 1.0)
                    }
                    EvaluatedVisualSource::Text(_) => {
                        let (_, path) = bindings
                            .get(&layer.item_id)
                            .ok_or_else(|| invalid("text sample raster unavailable"))?;
                        (
                            Raster::pam(&io.read(path).map_err(|_| {
                                CoreError::render_failure(GRAPH_BUILD_STAGE, None, None)
                            })?)?,
                            1.0,
                        )
                    }
                    EvaluatedVisualSource::Media { source_in_ms, .. } => {
                        let (input, path) = bindings
                            .get(&layer.item_id)
                            .ok_or_else(|| invalid("media sample binding unavailable"))?;
                        let size = sampled
                            .source_size
                            .ok_or_else(|| invalid("media sample geometry unavailable"))?;
                        let local = layer
                            .instance
                            .map_or(at as f64, |i| i.rate * at as f64 + i.offset)
                            - layer.span.start_ms as f64;
                        let time = if input.media_type == MediaType::Image {
                            0
                        } else {
                            source_in_ms + local.max(0.0).floor() as u64
                        };
                        (
                            Raster::rgba(
                                size.0 as usize,
                                size.1 as usize,
                                &decode(path, time, size)?,
                            )?,
                            1.0,
                        )
                    }
                    EvaluatedVisualSource::Caption(_) => (
                        caption_raster
                            .as_ref()
                            .ok_or_else(|| invalid("caption sample raster unavailable"))?
                            .clone(),
                        1.0,
                    ),
                };
                let mut budget = 0;
                crate::evaluated_scene::extended_certification::effect_budget(
                    (raster.width as u32, raster.height as u32),
                    &effect_stack,
                    density,
                    &mut budget,
                )?;
                let affine = extended_visual::sample_transform(
                    &mut sampled,
                    at,
                    (raster.width as u32, raster.height as u32),
                    canvas,
                )?;
                if let Some(crop) = crop {
                    raster = raster.crop(crop)?;
                }
                let bounds = if let EvaluatedVisualSource::Shape(shape) = &sampled.source {
                    Some([
                        (shape.bounds[0] - shape.origin.0) * density,
                        (shape.bounds[1] - shape.origin.1) * density,
                        (shape.bounds[2] - shape.origin.0) * density,
                        (shape.bounds[3] - shape.origin.1) * density,
                    ])
                } else {
                    None
                };
                let (raster, pad) = effects(raster, &effect_stack, density, bounds)?;
                let mut output = Raster::empty(canvas.0 as usize, canvas.1 as usize)?;
                let [a, b, c, d, tx, ty] = affine.inverse;
                let opacity = (affine.opacity * layer.transition_gain(at)) as f32;
                for y in 0..output.height {
                    for x in 0..output.width {
                        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                        let mut pixel = raster.bilinear(
                            a * px + c * py + tx + pad as f64,
                            b * px + d * py + ty + pad as f64,
                        );
                        for component in &mut pixel {
                            *component *= opacity;
                        }
                        output.pixels[y * output.width + x] = pixel;
                    }
                }
                output
            };
            Ok::<Raster, CoreError>(result)
        };
        let mut produce = |n| {
            let at = start + n * 1000 / u64::from(fps);
            let times = if let Some(settings) = layer.extended.as_ref().and_then(|v| v.motion_blur)
            {
                settings.sample_times(at, layer.extended.as_ref().unwrap().frame_rate, duration)?
            } else {
                vec![at]
            };
            if times.len() == 1 {
                return Ok(draw(times[0])?.pam_bytes());
            }
            let mut averaged = Raster::empty(canvas.0 as usize, canvas.1 as usize)?;
            let weight = 1.0 / times.len() as f32;
            for time in times {
                let raster = draw(time)?;
                for (dst, src) in averaged.pixels.iter_mut().zip(raster.pixels) {
                    for c in 0..4 {
                        dst[c] += src[c] * weight;
                    }
                }
            }
            Ok(averaged.pam_bytes())
        };
        if frame {
            io.write(&workspace.join(&file), &produce(0)?)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        } else {
            encode(
                &workspace.join(&file),
                scene.canvas.fps,
                frames,
                &mut produce,
            )?;
        }
        resources.media_inputs.push(MediaInputRequest {
            item_id: binding_id.clone(),
            asset_id: binding_id.clone(),
            project_relative_path: file.clone().into(),
            media_type: if frame {
                MediaType::Image
            } else {
                MediaType::Video
            },
            source_in_ms: 0,
            duration_ms: if frame {
                scene.duration_ms
            } else {
                end - start
            },
            input_index: resources.media_inputs.len() + 2,
        });
        resources.media_paths.push(workspace.join(file));
        layer.sampled_input = Some((binding_id, if frame { 0 } else { start }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn asymmetric_crop_excludes_outside_source_and_preserves_destination_extent() {
        let source = Raster::rgba(2, 1, &[255, 0, 0, 255, 0, 255, 0, 255]).unwrap();
        let identity = source.crop(MediaCrop::default()).unwrap();
        assert_eq!(identity.pixels, source.pixels);
        let cropped = source
            .crop(MediaCrop {
                x: 0.5,
                y: 0.0,
                width: 0.5,
                height: 1.0,
            })
            .unwrap();
        assert_eq!((cropped.width, cropped.height), (2, 1));
        assert_eq!(cropped.pixels, vec![[0.0, 1.0, 0.0, 1.0]; 2]);
    }
    #[test]
    fn identities_order_and_vignette_preserve_declared_local_bounds() {
        let mut source = Raster::empty(3, 3).unwrap();
        source.pixels.fill([0.2, 0.1, 0.0, 0.5]);
        let tint = VisualEffect::ColorTint {
            id: "t".into(),
            color: crate::VectorColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        };
        let vignette = VisualEffect::Vignette {
            id: "v".into(),
            amount: 1.0,
        };
        let (result, pad) =
            effects(source.clone(), std::slice::from_ref(&vignette), 1.0, None).unwrap();
        assert_eq!(pad, 0);
        assert_eq!(result.pixels[4], source.pixels[4]);
        assert!((result.pixels[0][0] - 0.2 * (1.0 - 4.0 / 9.0)).abs() < 1e-6);
        assert!(result.pixels.iter().all(|p| p[3] == 0.5));
        let identities = [
            VisualEffect::GaussianBlur {
                id: "b".into(),
                radius_px: 0.0,
            },
            VisualEffect::Vignette {
                id: "v".into(),
                amount: 0.0,
            },
            VisualEffect::ColorTint {
                id: "t".into(),
                color: crate::VectorColor {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            },
            VisualEffect::Glow {
                id: "g".into(),
                radius_px: 0.0,
                intensity: 0.0,
                color: crate::VectorColor {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            },
        ];
        assert_eq!(
            effects(source.clone(), &identities, 1.0, None)
                .unwrap()
                .0
                .pixels,
            source.pixels
        );
        let glow = VisualEffect::Glow {
            id: "g".into(),
            radius_px: 1.0,
            intensity: 1.0,
            color: crate::VectorColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        };
        let first = effects(source.clone(), &[glow.clone(), tint.clone()], 1.0, None)
            .unwrap()
            .0;
        let second = effects(source, &[tint, glow], 1.0, None).unwrap().0;
        assert_ne!(first.pixels, second.pixels);
        assert!(
            first
                .pixels
                .iter()
                .zip(second.pixels)
                .all(|(a, b)| a[3] == b[3])
        );
    }
    #[test]
    fn gaussian_is_normalized_and_glow_keeps_original_alpha() {
        let mut raster = Raster::empty(1, 1).unwrap();
        raster.pixels[0] = [1.0, 0.0, 0.0, 1.0];
        let stack = [VisualEffect::GaussianBlur {
            id: "blur".into(),
            radius_px: 1.0,
        }];
        let (blurred, pad) = effects(raster.clone(), &stack, 1.0, None).unwrap();
        assert_eq!(pad, 3);
        let sum: f32 = blurred.pixels.iter().map(|p| p[3]).sum();
        assert!((sum - 1.0).abs() < 1e-6);
        let glow = [VisualEffect::Glow {
            id: "glow".into(),
            radius_px: 1.0,
            intensity: 1.0,
            color: crate::VectorColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        }];
        let (glowed, pad) = effects(raster, &glow, 1.0, None).unwrap();
        assert_eq!(
            glowed.pixels[pad * glowed.width + pad],
            [1.0, 0.0, 0.0, 1.0]
        );
        assert!(glowed.pixels[pad * glowed.width + pad + 1][1] > 0.0);
    }
}
