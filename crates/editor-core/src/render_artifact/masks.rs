//! Pure scalar application of immutable, certified owner-local mask facts.
use crate::evaluated_scene::masks::{
    EvaluatedMask, EvaluatedMaskStack, MaskGrid, fill_transient_bytes,
};
use crate::{CoreError, ErrorCode, MaskChannel, MaskOperation, Paint, VectorPoint};
use std::{
    cell::Cell,
    mem::size_of,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}
fn count(size: (u32, u32)) -> Result<usize, CoreError> {
    let n = (size.0 as usize)
        .checked_mul(size.1 as usize)
        .filter(|n| *n > 0 && *n <= 16_777_216 && size.0 <= 16384 && size.1 <= 16384)
        .ok_or_else(|| invalid("mask raster exceeds certified dimensions"))?;
    Ok(n)
}
fn unit(value: f32) -> Result<f32, CoreError> {
    if !value.is_finite() || !(-1e-6..=1.0 + 1e-6).contains(&value) {
        return Err(invalid("mask coverage outside finite unit interval"));
    }
    Ok(value.clamp(0.0, 1.0))
}

/// Payload capacities are measured, including overlap of live temporaries. Vec
/// descriptors live on the call stack; retained fact/paint descriptors belong to
/// the evaluated-scene certificate, not to this current-grid scratch arena.
struct Scratch {
    limit: u64,
    live: Cell<u64>,
    peak: Cell<u64>,
    allocations: Cell<usize>,
}
impl Scratch {
    fn new(limit: u64) -> Self {
        Self {
            limit,
            live: Cell::new(0),
            peak: Cell::new(0),
            allocations: Cell::new(0),
        }
    }
    fn charge(&self, bytes: u64) -> Result<(), CoreError> {
        let live = self
            .live
            .get()
            .checked_add(bytes)
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| invalid("mask scratch exceeds certified reservation"))?;
        self.live.set(live);
        self.peak.set(self.peak.get().max(live));
        Ok(())
    }
    fn release(&self, bytes: u64) {
        self.live.set(self.live.get() - bytes);
    }
    fn buffer<T: Clone>(&self, len: usize, value: T) -> Result<Buffer<'_, T>, CoreError> {
        let bytes = len
            .checked_mul(size_of::<T>())
            .ok_or_else(|| invalid("mask allocation overflow"))? as u64;
        self.charge(bytes)?;
        let mut data = Vec::new();
        if data.try_reserve_exact(len).is_err() {
            self.release(bytes);
            return Err(invalid("mask allocation failed"));
        }
        let actual = data
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or_else(|| invalid("mask capacity overflow"))? as u64;
        if actual > bytes
            && let Err(error) = self.charge(actual - bytes)
        {
            self.release(bytes);
            return Err(error);
        }
        data.resize(len, value);
        self.allocations.set(self.allocations.get() + 1);
        Ok(Buffer {
            data,
            scratch: self,
            bytes: actual,
        })
    }
    fn adopt<T>(&self, data: Vec<T>) -> Result<Buffer<'_, T>, CoreError> {
        let bytes = data
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or_else(|| invalid("mask capacity overflow"))? as u64;
        self.charge(bytes)?;
        self.allocations.set(self.allocations.get() + 1);
        Ok(Buffer {
            data,
            scratch: self,
            bytes,
        })
    }
}
struct Buffer<'a, T> {
    data: Vec<T>,
    scratch: &'a Scratch,
    bytes: u64,
}
impl<T> Deref for Buffer<'_, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data
    }
}
impl<T> DerefMut for Buffer<'_, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}
impl<T> Drop for Buffer<'_, T> {
    fn drop(&mut self) {
        self.scratch.release(self.bytes);
    }
}

#[derive(Clone, Copy, Default)]
struct Site {
    index: usize,
    start: usize,
}
#[derive(Default)]
struct DistanceWork {
    crossovers: usize,
    insertions: usize,
    pops: usize,
    advances: usize,
    outputs: usize,
}
type Distance = Option<NonZeroU64>;
fn finite_distance(distance: u64) -> Result<NonZeroU64, CoreError> {
    distance
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| invalid("mask distance overflow"))
}

/// The later parabola first wins strictly after the exact rational crossover.
/// Floor division matters for negative numerators; ties select the earlier site.
fn first_winning_sample(p: usize, fp: u64, q: usize, fq: u64) -> Result<i128, CoreError> {
    let p = p as i128;
    let q = q as i128;
    let square_q = q
        .checked_mul(q)
        .ok_or_else(|| invalid("mask envelope overflow"))?;
    let square_p = p
        .checked_mul(p)
        .ok_or_else(|| invalid("mask envelope overflow"))?;
    let numerator = i128::from(fq)
        .checked_add(square_q)
        .and_then(|n| n.checked_sub(i128::from(fp)))
        .and_then(|n| n.checked_sub(square_p))
        .ok_or_else(|| invalid("mask envelope overflow"))?;
    let denominator = q
        .checked_sub(p)
        .and_then(|v| v.checked_mul(2))
        .ok_or_else(|| invalid("mask envelope overflow"))?;
    if denominator <= 0 {
        return Err(invalid("mask envelope sites out of order"));
    }
    numerator
        .div_euclid(denominator)
        .checked_add(1)
        .ok_or_else(|| invalid("mask envelope overflow"))
}
fn distance_line(
    len: usize,
    input: impl Fn(usize) -> Distance,
    mut output: impl FnMut(usize, Distance),
    sites: &mut [Site],
    work: &mut DistanceWork,
) -> Result<(), CoreError> {
    let mut used = 0;
    for q in 0..len {
        let Some(fq) = input(q) else { continue };
        let fq = fq.get() - 1;
        let start = loop {
            if used == 0 {
                break 0;
            }
            let last = sites[used - 1];
            let fp = input(last.index).unwrap().get() - 1;
            work.crossovers += 1;
            let start = first_winning_sample(last.index, fp, q, fq)?;
            if start <= last.start as i128 {
                used -= 1;
                work.pops += 1;
                continue;
            }
            break usize::try_from(start).unwrap_or(usize::MAX);
        };
        if start < len {
            sites[used] = Site { index: q, start };
            used += 1;
            work.insertions += 1;
        }
    }
    if used == 0 {
        for x in 0..len {
            output(x, None);
        }
        return Ok(());
    }
    let mut current = 0;
    for x in 0..len {
        while current + 1 < used && sites[current + 1].start <= x {
            current += 1;
            work.advances += 1;
        }
        let q = sites[current].index;
        let delta = x.abs_diff(q) as u64;
        let distance = delta
            .checked_mul(delta)
            .and_then(|d| d.checked_add(input(q).unwrap().get() - 1))
            .ok_or_else(|| invalid("mask squared distance overflow"))?;
        output(x, Some(finite_distance(distance)?));
        work.outputs += 1;
    }
    Ok(())
}
fn distances<'a>(
    coverage: &[f32],
    size: (u32, u32),
    inside: bool,
    scratch: &'a Scratch,
    work: &mut DistanceWork,
) -> Result<Buffer<'a, Distance>, CoreError> {
    let n = count(size)?;
    let (w, h) = (size.0 as usize, size.1 as usize);
    if coverage.len() != n {
        return Err(invalid("mask coverage size mismatch"));
    }
    let mut rows = scratch.buffer(n, None)?;
    let mut result = scratch.buffer(n, None)?;
    let mut sites = scratch.buffer(w.max(h), Site::default())?;
    for y in 0..h {
        distance_line(
            w,
            |x| {
                let c = coverage[y * w + x];
                if if inside { c > 0.0 } else { c < 1.0 } {
                    NonZeroU64::new(1)
                } else {
                    None
                }
            },
            |x, d| rows[y * w + x] = d,
            &mut sites,
            work,
        )?;
    }
    for x in 0..w {
        distance_line(
            h,
            |y| rows[y * w + x],
            |y, d| result[y * w + x] = d,
            &mut sites,
            work,
        )?;
    }
    Ok(result)
}
fn expand(
    coverage: &mut [f32],
    size: (u32, u32),
    amount: f64,
    scratch: &Scratch,
    work: &mut DistanceWork,
) -> Result<(), CoreError> {
    if !amount.is_finite() {
        return Err(invalid("nonfinite mask expansion"));
    }
    if amount == 0.0 {
        return Ok(());
    }
    if !coverage.iter().any(|c| *c > 0.0) {
        return Ok(());
    }
    let inside = distances(coverage, size, true, scratch, work)?;
    let outside = distances(coverage, size, false, scratch, work)?;
    for (i, c) in coverage.iter_mut().enumerate() {
        let d = if *c > 0.0 && *c < 1.0 {
            f64::from(*c) - 0.5
        } else if *c == 1.0 {
            (outside[i]
                .ok_or_else(|| invalid("mask grid lacks transparent exterior"))?
                .get()
                - 1) as f64
        } else {
            (inside[i]
                .ok_or_else(|| invalid("mask grid lacks inside support"))?
                .get()
                - 1) as f64
        };
        let signed = if *c == 1.0 {
            d.sqrt() - 0.5
        } else if *c == 0.0 {
            0.5 - d.sqrt()
        } else {
            d
        };
        *c = unit((0.5 + signed + amount).clamp(0.0, 1.0) as f32)?;
    }
    Ok(())
}
fn radius(sigma: f64) -> Result<usize, CoreError> {
    let r = (3.0 * sigma).ceil();
    if !sigma.is_finite() || sigma < 0.0 || r > 16384.0 {
        return Err(invalid("mask kernel exceeds certified support"));
    }
    Ok(r as usize)
}
fn feather(
    coverage: &mut [f32],
    size: (u32, u32),
    sigma: f64,
    scratch: &Scratch,
) -> Result<(), CoreError> {
    if sigma == 0.0 {
        return Ok(());
    }
    let r = radius(sigma)?;
    let k = r
        .checked_mul(2)
        .and_then(|v| v.checked_add(1))
        .ok_or_else(|| invalid("mask kernel overflow"))?;
    let mut kernel = scratch.buffer(k, 0.0f32)?;
    let sum: f64 = (0..k)
        .map(|i| {
            let d = (i as f64 - r as f64) / sigma;
            (-0.5 * d * d).exp()
        })
        .sum();
    if !sum.is_finite() || sum <= 0.0 {
        return Err(invalid("invalid mask kernel normalization"));
    }
    for (i, tap) in kernel.iter_mut().enumerate() {
        let d = (i as f64 - r as f64) / sigma;
        *tap = ((-0.5 * d * d).exp() / sum) as f32;
    }
    let (w, h) = (size.0 as usize, size.1 as usize);
    if count(size)? != coverage.len() {
        return Err(invalid("mask feather size mismatch"));
    }
    let mut horizontal = scratch.buffer(coverage.len(), 0.0f32)?;
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f64;
            for (i, tap) in kernel.iter().enumerate() {
                let sx = x as i64 + i as i64 - r as i64;
                if sx >= 0 && sx < w as i64 {
                    sum += f64::from(coverage[y * w + sx as usize]) * f64::from(*tap);
                }
            }
            horizontal[y * w + x] = unit(sum as f32)?;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f64;
            for (i, tap) in kernel.iter().enumerate() {
                let sy = y as i64 + i as i64 - r as i64;
                if sy >= 0 && sy < h as i64 {
                    sum += f64::from(horizontal[sy as usize * w + x]) * f64::from(*tap);
                }
            }
            coverage[y * w + x] = unit(sum as f32)?;
        }
    }
    Ok(())
}
fn bilinear(coverage: &[f32], size: (u32, u32), x: f64, y: f64) -> Result<f32, CoreError> {
    if !x.is_finite() || !y.is_finite() {
        return Err(invalid("nonfinite mask sampling coordinate"));
    }
    // Entirely outside support can be rejected without saturating float→integer casts.
    if x <= -0.5 || y <= -0.5 || x >= f64::from(size.0) + 0.5 || y >= f64::from(size.1) + 0.5 {
        return Ok(0.0);
    }
    let (x, y) = (x - 0.5, y - 0.5);
    let (ix, iy) = (x.floor() as i64, y.floor() as i64);
    let (fx, fy) = ((x - ix as f64) as f32, (y - iy as f64) as f32);
    let mut result = 0.0;
    for (dx, dy, weight) in [
        (0, 0, (1.0 - fx) * (1.0 - fy)),
        (1, 0, fx * (1.0 - fy)),
        (0, 1, (1.0 - fx) * fy),
        (1, 1, fx * fy),
    ] {
        let (sx, sy) = (ix + dx, iy + dy);
        if sx >= 0 && sy >= 0 && sx < i64::from(size.0) && sy < i64::from(size.1) {
            result += coverage[sy as usize * size.0 as usize + sx as usize] * weight;
        }
    }
    unit(result)
}
fn combine(a: f32, b: f32, operation: MaskOperation) -> Result<f32, CoreError> {
    unit(match operation {
        MaskOperation::Add => a + b - a * b,
        MaskOperation::Subtract => a * (1.0 - b),
        MaskOperation::Intersect => a * b,
        MaskOperation::Exclude => a + b - 2.0 * a * b,
    })
}
/// The shared line-only fill emits consecutive edges and implicit closure.
/// Reject a forged segment count before using it to bound opaque allocations.
fn opaque_fill_reservation(grid: &MaskGrid) -> Result<u64, CoreError> {
    let contours = grid.contours.len() as u64;
    let mut lines = 0u64;
    for contour in &grid.contours {
        if contour.points.len() < 3 {
            continue;
        }
        let closing = u64::from(contour.points.first() != contour.points.last());
        let emitted = (contour.points.len() as u64 - 1)
            .checked_add(closing)
            .ok_or_else(|| invalid("mask emitted line count overflow"))?;
        lines = lines
            .checked_add(emitted)
            .ok_or_else(|| invalid("mask emitted line count overflow"))?;
    }
    let allowed = grid
        .segments
        .checked_add(contours)
        .ok_or_else(|| invalid("mask segment certificate overflow"))?;
    if lines > allowed {
        return Err(invalid(
            "mask segment certificate underdeclares emitted lines",
        ));
    }
    fill_transient_bytes(grid.segments, contours, grid.size)
}

fn validate_fact(mask: &EvaluatedMask, peak: u64) -> Result<(), CoreError> {
    if mask.inverse.iter().any(|v| !v.is_finite())
        || !mask.opacity.is_finite()
        || !(0.0..=1.0).contains(&mask.opacity)
        || !mask.expansion_grid.is_finite()
        || !mask.feather_grid.is_finite()
        || mask.feather_grid < 0.0
    {
        return Err(invalid("invalid certified mask scalar facts"));
    }
    let det = mask.inverse[0] * mask.inverse[3] - mask.inverse[1] * mask.inverse[2];
    if !det.is_finite() || det == 0.0 {
        return Err(invalid("invalid certified mask inverse"));
    }
    if let Paint::LinearGradient { stops, .. } | Paint::RadialGradient { stops, .. } = &mask.paint
        && stops.len() < 2
    {
        return Err(invalid("invalid certified mask paint shape"));
    }
    if let Some(grid) = &mask.grid {
        let n = count(grid.size)? as u64;
        if !grid.density.is_finite()
            || grid.density < 1.0
            || !grid.origin.0.is_finite()
            || !grid.origin.1.is_finite()
            || grid.analytic_bounds.iter().any(|v| !v.is_finite())
        {
            return Err(invalid("invalid certified mask grid"));
        }
        let kernel = if mask.feather_grid == 0.0 {
            0
        } else {
            4 * (2 * radius(mask.feather_grid)? as u64 + 1)
        };
        let required = (64 * n + 16 * (u64::from(grid.size.0) + u64::from(grid.size.1)) + kernel)
            .checked_add(opaque_fill_reservation(grid)?)
            .ok_or_else(|| invalid("mask scratch certificate overflow"))?;
        if mask.certified_scratch_bytes < required || peak < mask.certified_scratch_bytes {
            return Err(invalid("insufficient certified mask scratch"));
        }
    }
    Ok(())
}
fn apply_paint(
    coverage: &mut [f32],
    grid: &MaskGrid,
    mask: &EvaluatedMask,
) -> Result<(), CoreError> {
    let w = grid.size.0 as usize;
    for (i, c) in coverage.iter_mut().enumerate() {
        let point = VectorPoint {
            x: grid.origin.0 + ((i % w) as f64 + 0.5) / grid.density,
            y: grid.origin.1 + ((i / w) as f64 + 0.5) / grid.density,
        };
        let color = super::shapes::paint_at(&mask.paint, point);
        if color.iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite mask paint coverage"));
        }
        let channel = match mask.channel {
            MaskChannel::Alpha => color[3],
            MaskChannel::Luma => 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2],
        };
        *c = unit((f64::from(*c) * channel) as f32)?;
    }
    Ok(())
}
fn local_coverage<'a>(
    grid: &MaskGrid,
    mask: &EvaluatedMask,
    scratch: &'a Scratch,
) -> Result<Buffer<'a, f32>, CoreError> {
    let n = count(grid.size)?;
    // Opaque path/scanner allocation is conservatively reserved, separately
    // from Pixmap/result pixels. The allocator witness measures the whole fill;
    // this ledger records its bound, not an observation of internal capacities.
    let fill_bytes = opaque_fill_reservation(grid)?
        .checked_add(8 * n as u64)
        .ok_or_else(|| invalid("mask fill reservation overflow"))?;
    scratch.charge(fill_bytes)?;
    let filled = super::shapes::fill_contours(
        &grid.contours,
        grid.fill_rule,
        grid.origin,
        grid.density,
        grid.size,
    );
    scratch.release(fill_bytes);
    let mut coverage = scratch.adopt(filled?)?;
    if coverage.len() != n {
        return Err(invalid("mask fill size mismatch"));
    }
    for c in coverage.iter_mut() {
        *c = unit(*c)?;
    }
    expand(
        &mut coverage,
        grid.size,
        mask.expansion_grid,
        scratch,
        &mut DistanceWork::default(),
    )?;
    apply_paint(&mut coverage, grid, mask)?;
    feather(&mut coverage, grid.size, mask.feather_grid, scratch)?;
    Ok(coverage)
}

pub(super) fn apply_stack(
    pixels: &mut [[f32; 4]],
    stack: &EvaluatedMaskStack,
) -> Result<(), CoreError> {
    // Preserve the exact original no-mask path, even its validation behavior.
    if stack.masks.is_empty() {
        return Ok(());
    }
    let n = count(stack.owner.size)?;
    if pixels.len() != n
        || !stack.owner.density.is_finite()
        || stack.owner.density <= 0.0
        || stack.accumulated_bytes < 4 * n as u64
    {
        return Err(invalid("invalid certified mask owner"));
    }
    for mask in &stack.masks {
        validate_fact(mask, stack.peak_scratch_bytes)?;
    }
    // Validate before the final mutation; late mask failures cannot alter source.
    for pixel in pixels.iter() {
        let alpha = pixel[3];
        if !alpha.is_finite()
            || !(-1e-6..=1.0 + 1e-6).contains(&alpha)
            || pixel[..3]
                .iter()
                .any(|c| !c.is_finite() || *c < -1e-6 || *c > alpha + 1e-6)
        {
            return Err(invalid("invalid premultiplied mask source pixel"));
        }
    }
    let owner_scratch = Scratch::new(stack.accumulated_bytes);
    let seed = if matches!(
        stack.masks[0].operation,
        MaskOperation::Add | MaskOperation::Exclude
    ) {
        0.0
    } else {
        1.0
    };
    let mut accumulated = owner_scratch.buffer(n, seed)?;
    let w = stack.owner.size.0 as usize;
    for mask in &stack.masks {
        let scratch = Scratch::new(mask.certified_scratch_bytes);
        let coverage = mask
            .grid
            .as_ref()
            .map(|grid| local_coverage(grid, mask, &scratch))
            .transpose()?;
        for (i, a) in accumulated.iter_mut().enumerate() {
            let mut b = if let (Some(grid), Some(coverage)) = (&mask.grid, &coverage) {
                let (x, y) = (
                    ((i % w) as f64 + 0.5) / stack.owner.density,
                    ((i / w) as f64 + 0.5) / stack.owner.density,
                );
                let m = mask.inverse;
                let (qx, qy) = (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]);
                bilinear(
                    coverage,
                    grid.size,
                    (qx - grid.origin.0) * grid.density,
                    (qy - grid.origin.1) * grid.density,
                )?
            } else {
                0.0
            };
            b = unit((f64::from(b) * mask.opacity) as f32)?;
            if mask.inverted {
                b = unit(1.0 - b)?;
            }
            *a = combine(*a, b, mask.operation)?;
        }
    }
    for (pixel, a) in pixels.iter_mut().zip(accumulated.iter()) {
        // Canonical f32 source-over acceptance/clamping, after all fallible stages.
        let alpha = pixel[3].clamp(0.0, 1.0);
        pixel[3] = alpha;
        for c in &mut pixel[..3] {
            *c = c.clamp(0.0, alpha);
        }
        for c in pixel {
            *c *= *a;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "masks/tests.rs"]
mod tests;
