//! Bounded local raster effects and sampled visual resources.
use super::{ArtifactIo, GRAPH_BUILD_STAGE, PreparedRenderResources};
mod group_compositing;
use crate::evaluated_scene::{EvaluatedScene, EvaluatedVisualSource, extended_visual};
use crate::render_plan::{MediaInputRequest, RenderIntent};
use crate::{CoreError, ErrorCode, MediaCrop, MediaType, VisualEffect};
use std::{collections::HashMap, path::Path};

fn matte_sample(
    raster: Raster,
    left: usize,
    top: usize,
    gain: f32,
) -> super::mattes::LeafSamplePlane {
    let plane = if raster.width == 0 || raster.height == 0 {
        super::mattes::LinearPlane {
            left: 0,
            top: 0,
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    } else {
        super::mattes::LinearPlane {
            left,
            top,
            width: raster.width,
            height: raster.height,
            pixels: raster.pixels,
        }
    };
    super::mattes::LeafSamplePlane { plane, gain }
}

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

#[derive(Clone, Debug, PartialEq)]
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
        // Outside this finite support both neighbors are transparent. Check
        // before integer conversion so enormous signed coordinates cannot
        // overflow neighbor arithmetic or introduce NaN interpolation weights.
        if !x.is_finite()
            || !y.is_finite()
            || x <= -0.5
            || y <= -0.5
            || x >= self.width as f64 + 0.5
            || y >= self.height as f64 + 0.5
        {
            return [0.; 4];
        }
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
    fn particle_domain(
        self,
        domain: extended_visual::ParticleSourceDomain,
    ) -> Result<Self, CoreError> {
        if domain.origin == [0., 0.] && domain.size == (self.width as u32, self.height as u32) {
            return Ok(self);
        }
        let mut result = Self::empty(domain.size.0 as usize, domain.size.1 as usize)?;
        let left = (-domain.origin[0]) as usize;
        let top = (-domain.origin[1]) as usize;
        if left
            .checked_add(self.width)
            .is_none_or(|v| v > result.width)
            || top
                .checked_add(self.height)
                .is_none_or(|v| v > result.height)
        {
            return Err(invalid("particle source domain differs from source"));
        }
        for y in 0..self.height {
            result.pixels
                [(top + y) * result.width + left..(top + y) * result.width + left + self.width]
                .copy_from_slice(&self.pixels[y * self.width..(y + 1) * self.width]);
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
    fn source_over_at(&mut self, source: &Self, left: usize, top: usize) -> Result<(), CoreError> {
        if left
            .checked_add(source.width)
            .is_none_or(|v| v > self.width)
            || top
                .checked_add(source.height)
                .is_none_or(|v| v > self.height)
        {
            return Err(invalid("composition raster bounds differ"));
        }
        for y in 0..source.height {
            let start = (top + y) * self.width + left;
            for (destination, source) in self.pixels[start..start + source.width]
                .iter_mut()
                .zip(&source.pixels[y * source.width..(y + 1) * source.width])
            {
                let mut source = *source;
                let alpha = source[3];
                if !alpha.is_finite()
                    || !(-1e-6..=1.0 + 1e-6).contains(&alpha)
                    || source[..3]
                        .iter()
                        .any(|v| !v.is_finite() || *v < -1e-6 || *v > alpha + 1e-6)
                {
                    return Err(invalid("invalid premultiplied source pixel"));
                }
                source[3] = alpha.clamp(0.0, 1.0);
                for c in 0..3 {
                    source[c] = source[c].clamp(0.0, source[3]);
                }
                for c in 0..4 {
                    destination[c] = source[c] + destination[c] * (1.0 - source[3]);
                }
                let alpha = destination[3];
                if !alpha.is_finite()
                    || !(-1e-6..=1.0 + 1e-6).contains(&alpha)
                    || destination[..3]
                        .iter()
                        .any(|v| !v.is_finite() || *v < -1e-6 || *v > alpha + 1e-6)
                {
                    return Err(invalid("non-finite or invalid premultiplied composition"));
                }
                destination[3] = alpha.clamp(0.0, 1.0);
                for c in 0..3 {
                    destination[c] = destination[c].clamp(0.0, destination[3]);
                }
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn source_over(&mut self, source: &Self) -> Result<(), CoreError> {
        self.source_over_at(source, 0, 0)
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
            VisualEffect::ParticleOverlay { radius_px, .. } => {
                (radius_px * density).ceil() as usize
            }
            _ => 0,
        })
        .sum()
}

#[cfg(test)]
fn effects(
    raster: Raster,
    effects: &[VisualEffect],
    density: f64,
    bounds: Option<[f64; 4]>,
) -> Result<(Raster, usize), CoreError> {
    effects_at(
        raster,
        effects,
        density,
        bounds,
        extended_visual::SampleTime::Integer(0),
    )
}

#[cfg(test)]
fn effects_at(
    raster: Raster,
    effects: &[VisualEffect],
    density: f64,
    bounds: Option<[f64; 4]>,
    time: extended_visual::SampleTime,
) -> Result<(Raster, usize), CoreError> {
    effects_in_domain(raster, effects, density, bounds, bounds, time)
}

fn emission_bounds(
    effects: &[VisualEffect],
    source: &EvaluatedVisualSource,
    prepared: Option<&crate::render_plan::PreparedText>,
    original: Option<[f64; 4]>,
) -> Result<Option<[f64; 4]>, CoreError> {
    if effects
        .iter()
        .any(|effect| matches!(effect, VisualEffect::ParticleOverlay { .. }))
        && let EvaluatedVisualSource::Text(text) = source
    {
        let shaped = text
            .shaped
            .as_ref()
            .ok_or_else(|| invalid("particle text domain missing"))?;
        let prepared = prepared.ok_or_else(|| invalid("particle text origin missing"))?;
        Ok(Some([
            f64::from(prepared.text_x),
            f64::from(prepared.text_y),
            f64::from(prepared.text_x) + shaped.width,
            f64::from(prepared.text_y) + shaped.height,
        ]))
    } else {
        Ok(original)
    }
}

fn effects_in_domain(
    mut raster: Raster,
    effects: &[VisualEffect],
    density: f64,
    bounds: Option<[f64; 4]>,
    particle_bounds: Option<[f64; 4]>,
    time: extended_visual::SampleTime,
) -> Result<(Raster, usize), CoreError> {
    let pad = support(effects, density);
    if effects.is_empty() {
        return Ok((raster, 0));
    }
    let bounds = bounds.unwrap_or([0.0, 0.0, raster.width as f64, raster.height as f64]);
    let particle_bounds = particle_bounds.unwrap_or(bounds);
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
            VisualEffect::ColorAdjustment {
                exposure_stops,
                contrast,
                saturation,
                ..
            } => {
                let identity = *exposure_stops == 0.0 && *contrast == 1.0 && *saturation == 1.0;
                let exposure = exposure_stops.exp2();
                for pixel in &mut raster.pixels {
                    let alpha = f64::from(pixel[3]);
                    if alpha == 0.0 {
                        pixel[..3].fill(0.0);
                    } else if !identity {
                        let adjusted = [0, 1, 2].map(|c| {
                            0.18 + contrast * (f64::from(pixel[c]) / alpha * exposure - 0.18)
                        });
                        let luminance =
                            0.2126 * adjusted[0] + 0.7152 * adjusted[1] + 0.0722 * adjusted[2];
                        for c in 0..3 {
                            pixel[c] = ((luminance + saturation * (adjusted[c] - luminance))
                                .clamp(0.0, 1.0)
                                * alpha) as f32;
                        }
                    }
                }
            }
            VisualEffect::ScreenFlash {
                start_ms,
                duration_ms,
                intensity,
                color,
                ..
            } => {
                let start = u64::from(*start_ms);
                let end = start + u64::from(*duration_ms);
                let strength = if time.compare(start).is_some_and(|v| !v.is_lt())
                    && time.compare(end).is_some_and(|v| v.is_lt())
                {
                    intensity * color.a * (1.0 - time.progress(start, end))
                } else {
                    0.0
                };
                if strength != 0.0 {
                    for pixel in &mut raster.pixels {
                        for (c, channel) in [color.r, color.g, color.b].into_iter().enumerate() {
                            pixel[c] = (f64::from(pixel[c])
                                + (f64::from(pixel[3]) - f64::from(pixel[c]))
                                    * linear(channel)
                                    * strength) as f32;
                        }
                    }
                }
            }
            VisualEffect::ParticleOverlay {
                count,
                seed,
                radius_px,
                speed_px_per_second,
                lifetime_ms,
                color,
                ..
            } => {
                particle_overlay(
                    &mut raster,
                    ParticleSample {
                        count: *count,
                        seed: *seed,
                        radius: *radius_px * density,
                        speed: *speed_px_per_second * density,
                        lifetime: *lifetime_ms,
                        color: *color,
                        bounds: particle_bounds,
                        pad,
                        time,
                    },
                )?;
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

fn particle_lane(seed: u32, index: u16, lane: u32) -> u32 {
    let mut value = seed
        ^ (u32::from(index) + 1).wrapping_mul(0x9e37_79b9)
        ^ (lane + 1).wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}
fn particle_lanes(count: u16, seed: u32) -> impl Iterator<Item = (u16, [u32; 3])> {
    (0..count).map(move |index| {
        (
            index,
            [0, 1, 2].map(|lane| particle_lane(seed, index, lane)),
        )
    })
}
struct ParticleSample {
    count: u16,
    seed: u32,
    radius: f64,
    speed: f64,
    lifetime: u32,
    color: crate::VectorColor,
    bounds: [f64; 4],
    pad: usize,
    time: extended_visual::SampleTime,
}
fn particle_overlay(raster: &mut Raster, sample: ParticleSample) -> Result<(), CoreError> {
    let ParticleSample {
        count,
        seed,
        radius,
        speed,
        lifetime,
        color,
        bounds,
        pad,
        time,
    } = sample;
    let width = bounds[2] - bounds[0];
    let height = bounds[3] - bounds[1];
    if count == 0 || radius == 0.0 || color.a == 0.0 || width <= 0.0 || height <= 0.0 {
        return Ok(());
    }
    let modulo = match time {
        extended_visual::SampleTime::Integer(whole) => (whole % u64::from(lifetime)) as f64,
        extended_visual::SampleTime::Split { whole, fraction } => {
            ((whole % u64::from(lifetime)) as f64 + fraction).rem_euclid(f64::from(lifetime))
        }
        extended_visual::SampleTime::Fractional(value) => value.rem_euclid(f64::from(lifetime)),
    };
    if !modulo.is_finite() {
        return Err(invalid("particle clock is nonfinite"));
    }
    let rgb = [linear(color.r), linear(color.g), linear(color.b)];
    for (_, lanes) in particle_lanes(count, seed) {
        let units = lanes.map(|lane| f64::from(lane) / 4294967296.0);
        let phase = (modulo + units[2] * f64::from(lifetime)) % f64::from(lifetime);
        let cx = bounds[0] + units[0] * width + pad as f64;
        let cy = bounds[1]
            + (units[1] * height + speed * phase / 1000.0).rem_euclid(height)
            + pad as f64;
        let left = ((cx - radius).floor() - 1.0)
            .max(0.0)
            .min(raster.width as f64) as usize;
        let top = ((cy - radius).floor() - 1.0)
            .max(0.0)
            .min(raster.height as f64) as usize;
        let right = ((cx + radius).ceil() + 1.0)
            .max(left as f64)
            .min(raster.width as f64) as usize;
        let bottom = ((cy + radius).ceil() + 1.0)
            .max(top as f64)
            .min(raster.height as f64) as usize;
        for y in top..bottom {
            for x in left..right {
                let mut covered = 0;
                for a in 0..4 {
                    for b in 0..4 {
                        let dx = x as f64 + (f64::from(a) + 0.5) / 4.0 - cx;
                        let dy = y as f64 + (f64::from(b) + 0.5) / 4.0 - cy;
                        if dx * dx + dy * dy <= radius * radius {
                            covered += 1;
                        }
                    }
                }
                let alpha = (color.a * f64::from(covered) / 16.0) as f32;
                let dst = &mut raster.pixels[y * raster.width + x];
                for c in 0..3 {
                    dst[c] = (rgb[c] * f64::from(alpha)) as f32 + dst[c] * (1.0 - alpha);
                }
                dst[3] = alpha + dst[3] * (1.0 - alpha);
            }
        }
    }
    Ok(())
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

type VisualDecoder<'a> = dyn Fn(&Path, f64, (u32, u32)) -> Result<Vec<u8>, CoreError> + 'a;
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
    if scene.visual_layers.is_empty()
        && scene
            .aggregates
            .as_ref()
            .is_none_or(|graph| graph.nodes.is_empty())
    {
        return Ok(());
    } // Exact opaque black is supplied by the existing final encoder source.
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
    let binding_id = "linear-scene".to_owned();
    let file = if frame {
        "linear-scene.pam"
    } else {
        "linear-scene.mkv"
    };
    // Source-local paint resources are disposable files, never an unbounded decoded-source cache.
    for (index, layer) in scene.visual_layers.iter().enumerate() {
        if matches!(&layer.source, EvaluatedVisualSource::Caption(_))
            || matches!(&layer.source, EvaluatedVisualSource::Text(text) if text.shaped.is_none())
        {
            let size = layer
                .source_size
                .ok_or_else(|| invalid("local source measurement missing"))?;
            let bytes = caption(layer, resources.text_layers.get(&layer.item_id), size)?;
            if bytes.len() != extended_visual::validate_sampled_source_size(size)? {
                return Err(invalid("local decoded raster size mismatch"));
            }
            let mut pam = format!(
                "P7\nWIDTH {}\nHEIGHT {}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n",
                size.0, size.1
            )
            .into_bytes();
            pam.extend_from_slice(&bytes);
            let path = workspace.join(format!("local-source-{index}.pam"));
            io.write(&path, &pam)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
            bindings.insert(
                layer.item_id.clone(),
                (
                    MediaInputRequest {
                        item_id: layer.item_id.clone(),
                        asset_id: layer.item_id.clone(),
                        project_relative_path: path.clone(),
                        media_type: MediaType::Image,
                        source_in_ms: 0,
                        duration_ms: duration,
                        input_index: 0,
                    },
                    path,
                ),
            );
        }
    }
    let scene_has_masks = scene
        .visual_layers
        .iter()
        .any(|l| l.extended.as_ref().is_some_and(|v| !v.masks.is_empty()));
    let mut produce = |n| {
        let at = start + n * 1000 / u64::from(fps);
        let mut composed = Raster::empty(canvas.0 as usize, canvas.1 as usize)?;
        composed.pixels.fill([0.0, 0.0, 0.0, 1.0]);
        let mut mask_work = crate::evaluated_scene::masks::MaskFrameBudget::default();
        let mut mask_segments = std::collections::HashMap::<u64, usize>::new();
        let mut counted_occurrences = std::collections::HashSet::new();
        let mut draw = |layer_index: usize,
                        at,
                        apply_gain: bool,
                        query: Option<(
            crate::evaluated_scene::mattes::QueryFrame,
            Option<usize>,
            crate::evaluated_scene::group_compositing::frame::SignedDomain,
        )>| {
            let original = &scene.visual_layers[layer_index];
            let relative;
            let owner = query.and_then(|(_, owner, _)| owner);
            let layer = if let Some(owner) = owner {
                relative =
                    crate::evaluated_scene::group_compositing::relative_layer(original, owner)?;
                &relative
            } else {
                original
            };
            let visible = if let Some(owner) = owner {
                crate::evaluated_scene::group_compositing::relative_visible_at(
                    scene,
                    layer_index,
                    owner,
                    at,
                )?
            } else {
                original.visible_at(at)
            };
            let (mut sampled, crop, effect_stack) = extended_visual::sample(layer, at)?;
            let result = if !visible {
                (Raster::empty(1, 1)?, 0, 0, 1.0f32)
            } else {
                let (mut raster, density) = match &sampled.source {
                    EvaluatedVisualSource::Shape(shape) => {
                        let bytes = if sampled.source == layer.source {
                            if let Some((_, path)) = bindings.get(&layer.item_id) {
                                io.read(path).map_err(|_| {
                                    CoreError::render_failure(GRAPH_BUILD_STAGE, None, None)
                                })?
                            } else {
                                super::shapes::rasterize(shape)?
                            }
                        } else {
                            super::shapes::rasterize(shape)?
                        };
                        (Raster::pam(&bytes)?, shape.density)
                    }
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
                    EvaluatedVisualSource::Text(_) | EvaluatedVisualSource::Caption(_) => {
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
                        let time = if input.media_type == MediaType::Image {
                            0.0
                        } else {
                            extended_visual::certified_media_source_time(layer, at, *source_in_ms)?
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
                };
                let source_domain = extended_visual::particle_source_domain(
                    layer,
                    &sampled,
                    &effect_stack,
                    (raster.width as u32, raster.height as u32),
                )?;
                let mut budget = 0;
                crate::evaluated_scene::extended_certification::effect_budget(
                    source_domain.size,
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
                if scene_has_masks {
                    let masks = extended_visual::sampled_masks(layer, at)?;
                    let first = counted_occurrences.insert((at, layer_index));
                    let mut segments = *mask_segments.entry(at).or_default();
                    if first && let EvaluatedVisualSource::Shape(shape) = &sampled.source {
                        segments = segments
                            .checked_add(shape.segments())
                            .filter(|v| *v <= crate::evaluated_scene::shapes::MAX_SCENE_SEGMENTS)
                            .ok_or_else(|| invalid("sampled mask scene segment overflow"))?;
                    }
                    if !masks.is_empty() {
                        let mut certification_segments = if first { segments } else { 0 };
                        let facts = crate::evaluated_scene::masks::certify_sampled_masks(
                            &masks,
                            crate::evaluated_scene::masks::MaskOwnerBasis {
                                size: (raster.width as u32, raster.height as u32),
                                density,
                            },
                            &mut mask_work,
                            &mut certification_segments,
                        )?;
                        if first {
                            segments = certification_segments;
                        }
                        super::masks::apply_stack(&mut raster.pixels, &facts)?;
                    }
                    if first {
                        mask_segments.insert(at, segments);
                    }
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
                let effect_time = extended_visual::leaf_effect_time(layer, at);
                // Particle emission uses the unpainted source domain. Paint
                // gutters remain part of the raster and legacy vignette domain.
                let particle_bounds = source_domain.emission.or(emission_bounds(
                    &effect_stack,
                    &sampled.source,
                    resources.text_layers.get(&layer.item_id),
                    bounds,
                )?);
                let shift = |bounds: Option<[f64; 4]>| {
                    bounds.map(|[a, b, c, d]| {
                        [
                            a - source_domain.origin[0],
                            b - source_domain.origin[1],
                            c - source_domain.origin[0],
                            d - source_domain.origin[1],
                        ]
                    })
                };
                raster = raster.particle_domain(source_domain)?;
                let (raster, pad) = effects_in_domain(
                    raster,
                    &effect_stack,
                    density,
                    shift(bounds),
                    shift(particle_bounds),
                    effect_time,
                )?;
                // Visit only the conservative transformed support, including
                // the transparent bilinear border; certification counts the full canvas.
                let [ma, mb, mc, md, mtx, mty] = affine.matrix;
                let left_source = source_domain.origin[0] - pad as f64;
                let top_source = source_domain.origin[1] - pad as f64;
                let corners = [
                    (left_source - 1., top_source - 1.),
                    (left_source + raster.width as f64 + 1., top_source - 1.),
                    (left_source - 1., top_source + raster.height as f64 + 1.),
                    (
                        left_source + raster.width as f64 + 1.,
                        top_source + raster.height as f64 + 1.,
                    ),
                ];
                let xs = corners.map(|(x, y)| ma * x + mc * y + mtx);
                let ys = corners.map(|(x, y)| mb * x + md * y + mty);
                let left = xs
                    .into_iter()
                    .fold(f64::INFINITY, f64::min)
                    .floor()
                    .clamp(0.0, canvas.0 as f64) as usize;
                let top = ys
                    .into_iter()
                    .fold(f64::INFINITY, f64::min)
                    .floor()
                    .clamp(0.0, canvas.1 as f64) as usize;
                let right = xs
                    .into_iter()
                    .fold(f64::NEG_INFINITY, f64::max)
                    .ceil()
                    .clamp(left as f64, canvas.0 as f64) as usize;
                let bottom = ys
                    .into_iter()
                    .fold(f64::NEG_INFINITY, f64::max)
                    .ceil()
                    .clamp(top as f64, canvas.1 as f64) as usize;
                let (left, top, right, bottom) = query
                    .map_or((left, top, right, bottom), |(query, _, _)| {
                        (0, 0, query.size.0 as usize, query.size.1 as usize)
                    });
                let mut output = Raster::empty(right - left, bottom - top)?;
                let inverse = if let Some((query, owner, domain)) = query {
                    let basis = if owner.is_some() {
                        [1., 0., 0., 1., domain.origin[0], domain.origin[1]]
                    } else {
                        query.matrix()
                    };
                    crate::evaluated_scene::multiply_matrix(affine.inverse, basis)
                } else {
                    affine.inverse
                };
                if inverse.iter().any(|value| !value.is_finite()) {
                    return Err(invalid("sample query inverse must be finite"));
                }
                let [a, b, c, d, tx, ty] = inverse;
                let opacity = (affine.opacity * layer.transition_gain(at)) as f32;
                for y in 0..output.height {
                    for x in 0..output.width {
                        let (px, py) = ((x + left) as f64 + 0.5, (y + top) as f64 + 0.5);
                        let mut pixel = raster.bilinear(
                            a * px + c * py + tx + pad as f64 - source_domain.origin[0],
                            b * px + d * py + ty + pad as f64 - source_domain.origin[1],
                        );
                        for component in &mut pixel {
                            if apply_gain {
                                *component *= opacity;
                            }
                        }
                        output.pixels[y * output.width + x] = pixel;
                    }
                }
                (output, left, top, opacity)
            };
            Ok::<(Raster, usize, usize, f32), CoreError>(result)
        };
        if scene
            .aggregates
            .as_ref()
            .is_some_and(|graph| !graph.nodes.is_empty())
        {
            let program =
                crate::evaluated_scene::group_compositing::frame::frame_program(scene, at)?;
            group_compositing::compose(
                scene,
                &program,
                at,
                &mut composed,
                &mut |index, time, owner, query, domain| {
                    let visible = if let Some(owner) = owner {
                        crate::evaluated_scene::group_compositing::relative_visible_at(
                            scene, index, owner, time,
                        )?
                    } else {
                        scene.visual_layers[index].visible_at(time)
                    };
                    if !visible {
                        return Ok(matte_sample(Raster::empty(0, 0)?, 0, 0, 1.));
                    }
                    let (raster, left, top, gain) =
                        draw(index, time, false, Some((query, owner, domain)))?;
                    Ok(matte_sample(raster, left, top, gain))
                },
            )?;
        } else if scene.composition_resources.is_some() {
            let schedule = crate::evaluated_scene::mattes::frame_schedule(scene, at)?;
            super::mattes::compose_frame(&schedule, &mut composed.pixels, &mut |index, time| {
                let layer = &scene.visual_layers[index];
                if !layer.visible_at(time) {
                    return Ok(super::mattes::LeafSamplePlane {
                        plane: super::mattes::LinearPlane {
                            left: 0,
                            top: 0,
                            width: 0,
                            height: 0,
                            pixels: Vec::new(),
                        },
                        gain: 1.,
                    });
                }
                let (raster, left, top, gain) = draw(index, time, false, None)?;
                Ok(matte_sample(raster, left, top, gain))
            })?;
        } else {
            for (layer_index, layer) in scene.visual_layers.iter().enumerate() {
                let times =
                    if let Some(settings) = layer.extended.as_ref().and_then(|v| v.motion_blur) {
                        settings.sample_times(
                            at,
                            layer.extended.as_ref().unwrap().frame_rate,
                            duration,
                        )?
                    } else {
                        vec![at]
                    };
                if times.len() == 1 {
                    if layer.visible_at(times[0]) {
                        let (raster, left, top, _) = draw(layer_index, times[0], true, None)?;
                        composed.source_over_at(&raster, left, top)?;
                    }
                } else {
                    let mut averaged = Raster::empty(canvas.0 as usize, canvas.1 as usize)?;
                    let weight = 1.0 / times.len() as f32;
                    for time in times {
                        let (raster, left, top, _) = draw(layer_index, time, true, None)?;
                        for y in 0..raster.height {
                            for x in 0..raster.width {
                                let src = raster.pixels[y * raster.width + x];
                                let dst =
                                    &mut averaged.pixels[(top + y) * averaged.width + left + x];
                                for c in 0..4 {
                                    dst[c] += src[c] * weight;
                                }
                            }
                        }
                    }
                    composed.source_over_at(&averaged, 0, 0)?;
                }
            }
        }
        Ok(composed.pam_bytes())
    };
    if frame {
        io.write(&workspace.join(file), &produce(0)?)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
    } else {
        encode(
            &workspace.join(file),
            scene.canvas.fps,
            frames,
            &mut produce,
        )?;
    }
    resources.media_inputs.push(MediaInputRequest {
        item_id: binding_id.clone(),
        asset_id: binding_id.clone(),
        project_relative_path: file.into(),
        media_type: if frame {
            MediaType::Image
        } else {
            MediaType::Video
        },
        source_in_ms: 0,
        duration_ms: end - start,
        input_index: resources.media_inputs.len() + 2,
    });
    resources.media_paths.push(workspace.join(file));
    if scene
        .aggregates
        .as_ref()
        .is_some_and(|graph| !graph.nodes.is_empty())
    {
        scene.composed_input = Some((binding_id, if frame { 0 } else { start }));
    } else if let Some(layer) = scene.visual_layers.first_mut() {
        layer.sampled_input = Some((binding_id, if frame { 0 } else { start }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grade(exposure_stops: f64, contrast: f64, saturation: f64) -> VisualEffect {
        VisualEffect::ColorAdjustment {
            id: "grade".into(),
            exposure_stops,
            contrast,
            saturation,
        }
    }
    #[test]
    fn color_controls_preserve_float_alpha_and_identity_without_quantization() {
        let mut input = Raster::empty(3, 1).unwrap();
        input.pixels = vec![
            [0.2, 0.1, 0.05, 0.25],
            [0.12345679, 0.023_456_79, 0.000123456, 0.7654321],
            [0.0; 4],
        ];
        let (identity, pad) = effects(input.clone(), &[grade(0.0, 1.0, 1.0)], 1.0, None).unwrap();
        assert_eq!(identity.pixels, input.pixels);
        assert_eq!(pad, 0);
        let (actual, pad) = effects(input.clone(), &[grade(-1.0, 0.75, 0.25)], 1.0, None).unwrap();
        assert_eq!(pad, 0);
        for (source, output) in input.pixels.iter().zip(&actual.pixels) {
            assert_eq!(output[3].to_bits(), source[3].to_bits());
            if source[3] == 0.0 {
                assert_eq!(*output, [0.0; 4]);
                continue;
            }
            // Independent scalar reference, retaining the original float sample precision.
            let a = f64::from(source[3]);
            let r = 0.18 + 0.75 * (f64::from(source[0]) / a / 2.0 - 0.18);
            let g = 0.18 + 0.75 * (f64::from(source[1]) / a / 2.0 - 0.18);
            let b = 0.18 + 0.75 * (f64::from(source[2]) / a / 2.0 - 0.18);
            let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            for (c, value) in [r, g, b].into_iter().enumerate() {
                assert!(
                    (f64::from(output[c]) - (y * 0.75 + value * 0.25).clamp(0.0, 1.0) * a).abs()
                        < 2e-7
                );
            }
        }
    }
    #[test]
    fn color_controls_clear_hidden_rgb_at_zero_alpha_for_identity_and_nonidentity() {
        for effect in [grade(0.0, 1.0, 1.0), grade(-1.0, 0.75, 0.25)] {
            let mut input = Raster::empty(1, 1).unwrap();
            input.pixels[0] = [0.4, 0.2, 0.1, 0.0];
            let (output, _) = effects(input, &[effect], 1.0, None).unwrap();
            assert_eq!(output.pixels[0], [0.0; 4]);
        }
    }
    #[test]
    fn color_controls_clamp_only_after_saturation_in_linear_rgb() {
        let mut input = Raster::empty(1, 1).unwrap();
        input.pixels[0] = [1.0, 0.0, 0.0, 1.0];
        let (actual, _) = effects(input.clone(), &[grade(0.0, 2.0, 0.0)], 1.0, None).unwrap();
        for c in 0..3 {
            assert!((f64::from(actual.pixels[0][c]) - 0.2452).abs() < 1e-7);
        }
        let (actual, _) = effects(input, &[grade(-1.0, 0.75, 0.25)], 1.0, None).unwrap();
        for (c, expected) in [0.19854375, 0.10479375, 0.10479375].into_iter().enumerate() {
            assert!((f64::from(actual.pixels[0][c]) - expected).abs() < 1e-7);
        }
    }
    #[test]
    fn zero_area_affine_support_is_canonical_at_matte_callback_seam() {
        for (width, height, left, top) in [(0, 16, 64, 12), (24, 0, 8, 64), (0, 0, 64, 64)] {
            let sample = matte_sample(Raster::empty(width, height).unwrap(), left, top, 0.5);
            assert_eq!(
                (
                    sample.plane.left,
                    sample.plane.top,
                    sample.plane.width,
                    sample.plane.height
                ),
                (0, 0, 0, 0)
            );
            assert!(sample.plane.pixels.is_empty());
            assert_eq!(sample.gain, 0.5);
        }
    }

    #[test]
    fn rejects_invalid_source_even_when_opaque_destination_would_hide_it() {
        for pixel in [
            [0.0, 0.0, 0.0, -0.1],
            [0.8, 0.0, 0.0, 0.5],
            [f32::NAN, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, f32::INFINITY],
        ] {
            let mut destination = Raster::empty(1, 1).unwrap();
            destination.pixels[0] = [0.0, 0.0, 0.0, 1.0];
            let mut source = Raster::empty(1, 1).unwrap();
            source.pixels[0] = pixel;
            assert_eq!(
                destination.source_over(&source).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        let mut destination = Raster::empty(1, 1).unwrap();
        let mut source = Raster::empty(1, 1).unwrap();
        source.pixels[0] = [0.5000005, -0.0000005, 0.0, 0.5];
        destination.source_over(&source).unwrap();
        assert_eq!(destination.pixels[0], [0.5, 0.0, 0.0, 0.5]);
    }

    #[test]
    fn linear_source_over_matches_independent_alpha_oracle() {
        let mut destination = Raster::empty(1, 1).unwrap();
        destination.pixels[0] = [0.0, 0.0, 0.0, 1.0];
        let mut source = Raster::empty(1, 1).unwrap();
        source.pixels[0] = [0.5, 0.5, 0.5, 0.5];
        destination.source_over(&source).unwrap();
        assert_eq!(destination.pixels[0], [0.5, 0.5, 0.5, 1.0]);
        let bytes = destination.pam_bytes();
        assert_eq!(&bytes[bytes.len() - 4..], &[188, 188, 188, 255]);
        let mut lower = Raster::empty(1, 1).unwrap();
        lower.pixels[0] = [0.25, 0.0, 0.0, 0.5];
        source.pixels[0] = [0.0, 0.0, 0.5, 0.5];
        lower.source_over(&source).unwrap();
        assert_eq!(lower.pixels[0], [0.125, 0.0, 0.5, 0.75]);
    }

    #[test]
    fn hidden_rgb_cannot_contaminate_linear_transparent_edges() {
        let source = Raster::rgba(2, 1, &[255, 0, 0, 0, 255, 255, 255, 255]).unwrap();
        assert_eq!(source.pixels[0], [0.0; 4]);
        assert_eq!(source.bilinear(1.0, 0.5), [0.5; 4]);
        let gray = Raster::rgba(1, 1, &[128, 128, 128, 128]).unwrap();
        let expected = ((128.0_f64 / 255.0 + 0.055) / 1.055).powf(2.4) * (128.0 / 255.0);
        assert!((gray.pixels[0][0] as f64 - expected).abs() < 1e-6);
    }

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

#[cfg(test)]
mod overlay_tests {
    use super::*;
    use crate::evaluated_scene::extended_visual::SampleTime;
    fn color() -> crate::VectorColor {
        crate::VectorColor {
            r: 0.5,
            g: 1.0,
            b: 0.0,
            a: 0.6,
        }
    }
    #[test]
    fn independently_pinned_particle_integer_vectors_and_emission_order() {
        let catalog: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/group-compositing-v1.json"
        ))
        .unwrap();
        for vector in catalog["hashVectors"].as_array().unwrap() {
            let seed = vector["seed"].as_u64().unwrap() as u32;
            let index = vector["index"].as_u64().unwrap() as u16;
            let (_, lanes) = particle_lanes(256, seed).nth(index as usize).unwrap();
            for lane in 0..3 {
                assert_eq!(
                    u64::from(lanes[lane]),
                    vector["lanes"][lane].as_u64().unwrap()
                );
            }
            let emitted: Vec<_> = particle_lanes(256, seed).map(|(i, _)| i).collect();
            assert_eq!(emitted, (0..256).collect::<Vec<u16>>());
        }
    }
    #[test]
    fn flash_preserves_alpha_and_exact_half_open_original_clock_envelope() {
        let base = Raster {
            width: 1,
            height: 1,
            pixels: vec![[0.1, 0.2, 0.0, 0.4]],
        };
        let effect = VisualEffect::ScreenFlash {
            id: "flash".into(),
            start_ms: 100,
            duration_ms: 400,
            intensity: 0.8,
            color: color(),
        };
        for (time, strength) in [
            (SampleTime::Integer(99), 0.0),
            (
                SampleTime::Split {
                    whole: 100,
                    fraction: -0.25,
                },
                0.0,
            ),
            (
                SampleTime::Split {
                    whole: 500,
                    fraction: -0.25,
                },
                0.0003,
            ),
            (SampleTime::Integer(100), 0.48),
            (SampleTime::Integer(300), 0.24),
            (
                SampleTime::Split {
                    whole: 499,
                    fraction: 0.5,
                },
                0.0006,
            ),
            (SampleTime::Integer(500), 0.0),
            (SampleTime::Integer(u64::MAX), 0.0),
        ] {
            let (actual, _) =
                effects_at(base.clone(), std::slice::from_ref(&effect), 1.0, None, time).unwrap();
            // Independent IEC transfer numeric constant for encoded .5; no runtime conversion helper.
            let expected = [
                0.1 + (0.4 - 0.1) * 0.21404114048223255 * strength,
                0.2 + (0.4 - 0.2) * strength,
                0.0,
                0.4,
            ];
            for (actual, expected) in actual.pixels[0].iter().zip(expected) {
                assert!((f64::from(*actual) - expected).abs() < 1e-6);
            }
        }
        let transparent = Raster {
            width: 1,
            height: 1,
            pixels: vec![[0.; 4]],
        };
        assert_eq!(
            effects_at(
                transparent.clone(),
                &[effect],
                1.,
                None,
                SampleTime::Integer(100)
            )
            .unwrap()
            .0,
            transparent
        );
    }
    #[test]
    fn particles_keep_high_integer_modulo_exact_and_zero_domains_safe() {
        let effect = VisualEffect::ParticleOverlay {
            id: "p".into(),
            count: 8,
            seed: 1,
            radius_px: 1.7,
            speed_px_per_second: 12.,
            lifetime_ms: 997,
            color: color(),
        };
        let empty = Raster::empty(32, 24).unwrap();
        let high = effects_at(
            empty.clone(),
            std::slice::from_ref(&effect),
            1.,
            None,
            SampleTime::Integer(u64::MAX),
        )
        .unwrap();
        let reduced = effects_at(
            empty.clone(),
            std::slice::from_ref(&effect),
            1.,
            None,
            SampleTime::Integer(u64::MAX % 997),
        )
        .unwrap();
        assert_eq!(high, reduced);
        let split = effects_at(
            empty.clone(),
            std::slice::from_ref(&effect),
            1.,
            None,
            SampleTime::Split {
                whole: u64::MAX,
                fraction: 0.25,
            },
        )
        .unwrap();
        let reduced_split = effects_at(
            empty.clone(),
            std::slice::from_ref(&effect),
            1.,
            None,
            SampleTime::Split {
                whole: u64::MAX % 997,
                fraction: 0.25,
            },
        )
        .unwrap();
        assert_eq!(split, reduced_split);
        let negative_split = effects_at(
            empty.clone(),
            std::slice::from_ref(&effect),
            1.,
            None,
            SampleTime::Split {
                whole: 997,
                fraction: -0.25,
            },
        )
        .unwrap();
        let euclidean = effects_at(
            empty.clone(),
            std::slice::from_ref(&effect),
            1.,
            None,
            SampleTime::Fractional(996.75),
        )
        .unwrap();
        assert_eq!(negative_split, euclidean);
        let zero = effects_at(
            empty.clone(),
            &[effect],
            2.,
            Some([0., 0., 0., 24.]),
            SampleTime::Integer(u64::MAX),
        )
        .unwrap();
        assert!(zero.0.pixels.iter().all(|pixel| *pixel == [0.; 4]));
    }
}

#[cfg(test)]
pub(super) fn assert_text_particle_domain(
    a: &crate::render_plan::PreparedText,
    ashaped: &crate::fonts::shaping::ShapedText,
    b: &crate::render_plan::PreparedText,
    bshaped: &crate::fonts::shaping::ShapedText,
    text: &crate::evaluated_scene::EvaluatedText,
) {
    let effect = VisualEffect::ParticleOverlay {
        id: "text-domain".into(),
        count: 64,
        seed: 173,
        radius_px: 1.25,
        speed_px_per_second: 16.,
        lifetime_ms: 997,
        color: crate::VectorColor {
            r: 1.,
            g: 0.,
            b: 0.,
            a: 0.6,
        },
    };
    let render = |p: &crate::render_plan::PreparedText,
                  shaped: &crate::fonts::shaping::ShapedText| {
        let mut text = text.clone();
        text.shaped = Some(shaped.clone());
        let source = EvaluatedVisualSource::Text(Box::new(text));
        let domain =
            emission_bounds(std::slice::from_ref(&effect), &source, Some(p), None).unwrap();
        effects_in_domain(
            Raster::empty(p.layer_width as usize, p.layer_height as usize).unwrap(),
            std::slice::from_ref(&effect),
            1.,
            None,
            domain,
            extended_visual::SampleTime::Integer(193),
        )
        .unwrap()
    };
    let (ra, pad) = render(a, ashaped);
    let (rb, _) = render(b, bshaped);
    assert!(ra.pixels.iter().any(|p| p[3] > 0.));
    for y in 0..ashaped.height.ceil() as usize {
        for x in 0..ashaped.width.ceil() as usize {
            assert_eq!(
                ra.pixels[(y + a.text_y as usize + pad) * ra.width + x + a.text_x as usize + pad],
                rb.pixels[(y + b.text_y as usize + pad) * rb.width + x + b.text_x as usize + pad]
            );
        }
    }
}
