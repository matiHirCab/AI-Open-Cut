//! Pure execution of immutable certified matte tasks; no authored graph lookup.
use crate::CoreError;
use crate::evaluated_scene::mattes::MatteFrameSchedule;

#[derive(Debug)]
pub(super) struct LinearPlane {
    pub left: usize,
    pub top: usize,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[f32; 4]>,
}
#[derive(Debug)]
pub(super) struct LeafSamplePlane {
    pub plane: LinearPlane,
    pub gain: f32,
}
fn invalid(message: &str) -> CoreError {
    CoreError::new(crate::ErrorCode::InvalidArgument, message)
}
fn add(a: u64, b: u64) -> Result<u64, CoreError> {
    a.checked_add(b)
        .ok_or_else(|| invalid("matte byte/work overflow"))
}
fn mul(a: u64, b: u64) -> Result<u64, CoreError> {
    a.checked_mul(b)
        .ok_or_else(|| invalid("matte byte/work overflow"))
}
fn bytes<T>(capacity: usize) -> Result<u64, CoreError> {
    mul(capacity as u64, std::mem::size_of::<T>() as u64)
}
fn pixel_count(size: (u32, u32)) -> Result<usize, CoreError> {
    let n = (size.0 as usize)
        .checked_mul(size.1 as usize)
        .filter(|n| *n > 0 && *n <= 16_777_216 && size.0 <= 16384 && size.1 <= 16384)
        .ok_or_else(|| invalid("matte canvas exceeds certified surface bounds"))?;
    Ok(n)
}
fn validate_pixel(pixel: &[f32; 4]) -> Result<(), CoreError> {
    let alpha = pixel[3];
    if !alpha.is_finite()
        || !(-1e-6..=1.0 + 1e-6).contains(&alpha)
        || pixel[..3]
            .iter()
            .any(|v| !v.is_finite() || *v < -1e-6 || *v > alpha + 1e-6)
    {
        return Err(invalid("invalid premultiplied matte pixel"));
    }
    Ok(())
}
fn clamp_pixel(pixel: &mut [f32; 4]) {
    pixel[3] = pixel[3].clamp(0.0, 1.0);
    for c in 0..3 {
        pixel[c] = pixel[c].clamp(0.0, pixel[3]);
    }
}
impl LinearPlane {
    fn empty() -> Self {
        Self {
            left: 0,
            top: 0,
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    }
    fn payload_bytes(&self) -> Result<u64, CoreError> {
        bytes::<[f32; 4]>(self.pixels.capacity())
    }
    fn validate(&mut self, canvas: (u32, u32)) -> Result<(), CoreError> {
        if self.width == 0 || self.height == 0 {
            if self.left != 0
                || self.top != 0
                || self.width != 0
                || self.height != 0
                || !self.pixels.is_empty()
            {
                return Err(invalid("noncanonical empty matte plane"));
            }
        } else if self.width.checked_mul(self.height) != Some(self.pixels.len())
            || self
                .left
                .checked_add(self.width)
                .is_none_or(|r| r > canvas.0 as usize)
            || self
                .top
                .checked_add(self.height)
                .is_none_or(|b| b > canvas.1 as usize)
        {
            return Err(invalid("matte plane support differs from certified canvas"));
        }
        // Check every original f32 boundary before clamping. Tiny positive alpha
        // remains positive; no unassociation or epsilon alpha cutoff is used.
        for pixel in &self.pixels {
            validate_pixel(pixel)?;
        }
        for pixel in &mut self.pixels {
            clamp_pixel(pixel);
        }
        Ok(())
    }
    fn at(&self, x: usize, y: usize) -> [f32; 4] {
        if x >= self.left
            && x - self.left < self.width
            && y >= self.top
            && y - self.top < self.height
        {
            self.pixels[(y - self.top) * self.width + x - self.left]
        } else {
            [0.; 4]
        }
    }
}
#[derive(Default)]
struct Slot {
    plane: Option<LinearPlane>,
    payload_bytes: u64,
    last_use: usize,
    direct_seen: bool,
    aggregate_seen: Option<usize>,
    release_heads: [Option<std::num::NonZeroUsize>; 2],
    release_next: Option<std::num::NonZeroUsize>,
}
fn descriptor_bytes(schedule: &MatteFrameSchedule, slot_capacity: usize) -> Result<u64, CoreError> {
    use crate::evaluated_scene::mattes::{MatteTask, MatteTaskId};
    let mut total = std::mem::size_of::<MatteFrameSchedule>() as u64;
    total = add(total, bytes::<MatteTask>(schedule.tasks.capacity())?)?;
    total = add(total, bytes::<usize>(schedule.last_uses.capacity())?)?;
    total = add(
        total,
        bytes::<crate::evaluated_scene::mattes::DirectDraw>(schedule.direct_draw.capacity())?,
    )?;
    total = add(
        total,
        bytes::<crate::BlendMode>(schedule.owner_modes.capacity())?,
    )?;
    total = add(total, std::mem::size_of::<Vec<Slot>>() as u64)?;
    total = add(total, bytes::<Slot>(slot_capacity)?)?;
    for task in &schedule.tasks {
        match task {
            MatteTask::AverageCopy { samples, .. }
            | MatteTask::AggregateProvider { copies: samples } => {
                total = add(total, bytes::<MatteTaskId>(samples.capacity())?)?
            }
            _ => {}
        }
    }
    Ok(total)
}
fn validate_descriptor(
    schedule: &MatteFrameSchedule,
    capacity: usize,
    pixels: usize,
) -> Result<(), CoreError> {
    use crate::evaluated_scene::mattes::{MATTE_CACHE_RESERVATION, MAX_MATTE_LIVE_BYTES};
    let cert = &schedule.certificate;
    if cert.descriptor_bytes < descriptor_bytes(schedule, capacity)? {
        return Err(invalid("matte descriptor capacity exceeds certificate"));
    }
    // descriptor_bytes is INCLUDED once in fixed_live_bytes, never added to the
    // live ledger a second time. This lower bound cannot be forged away.
    let minimum = add(
        add(MATTE_CACHE_RESERVATION, mul(pixels as u64, 16)?)?,
        cert.descriptor_bytes,
    )?;
    if cert.fixed_live_bytes < minimum
        || cert.peak_live_bytes < cert.fixed_live_bytes
        || cert.peak_live_bytes > MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("matte fixed/peak live certificate is insufficient"));
    }
    Ok(())
}
fn preflight(schedule: &MatteFrameSchedule, pixels: usize) -> Result<Vec<Slot>, CoreError> {
    use crate::evaluated_scene::mattes::{MAX_MATTE_REQUESTS, MAX_MATTE_WORK, MatteTask};
    if schedule.last_uses.len() != schedule.tasks.len() {
        return Err(invalid("matte last-use count differs"));
    }
    validate_descriptor(schedule, schedule.tasks.len(), pixels)?;
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(schedule.tasks.len())
        .map_err(|_| invalid("matte task slot allocation failed"))?;
    slots.resize_with(schedule.tasks.len(), Slot::default);
    validate_descriptor(schedule, slots.capacity(), pixels)?;
    let mut blend_work = 0u64;
    let mut requests = 0u64;
    let mut work = 0u64;
    for (i, task) in schedule.tasks.iter().enumerate() {
        if let MatteTask::LeafSample { layer_index, .. }
        | MatteTask::AverageCopy { layer_index, .. } = task
            && *layer_index >= schedule.owner_modes.len()
        {
            return Err(invalid("composition task owner is absent"));
        }
        slots[i].last_use = i;
        let mut admit = |dependency: usize| -> Result<(), CoreError> {
            if dependency >= i {
                return Err(invalid("matte dependency is not provider-first"));
            }
            slots[dependency].last_use = i;
            Ok(())
        };
        match task {
            MatteTask::LeafSample {
                provider: Some((id, _)),
                ..
            } => {
                admit(id.0)?;
                if !matches!(schedule.tasks[id.0], MatteTask::AggregateProvider { .. }) {
                    return Err(invalid("matte provider task is not an aggregate"));
                }
                work = add(work, mul(pixels as u64, 5)?)?;
            }
            MatteTask::AverageCopy {
                samples,
                layer_index,
            } => {
                if samples.is_empty() || samples.len() > crate::MotionBlur::MAX_SAMPLES as usize {
                    return Err(invalid("matte copy sample count exceeds shutter bounds"));
                }
                for id in samples {
                    admit(id.0)?;
                    if !matches!(schedule.tasks[id.0], MatteTask::LeafSample { layer_index: owner, .. } if owner == *layer_index)
                    {
                        return Err(invalid("matte copy sample is not a leaf sample"));
                    }
                }
            }
            MatteTask::AggregateProvider { copies } => {
                requests = add(requests, 1)?;
                for id in copies {
                    admit(id.0)?;
                }
                for id in copies {
                    if matches!(schedule.tasks[id.0], MatteTask::AggregateProvider { .. })
                        || slots[id.0].aggregate_seen == Some(i)
                    {
                        return Err(invalid(
                            "matte provider must aggregate individual copies once",
                        ));
                    }
                    slots[id.0].aggregate_seen = Some(i);
                }
                work = add(work, mul(mul(pixels as u64, 4)?, copies.len() as u64)?)?;
            }
            _ => {}
        }
    }
    let mut previous_owner = None;
    for (i, draw) in schedule.direct_draw.iter().enumerate() {
        let id = draw.task;
        if schedule.owner_modes.get(draw.layer_index) != Some(&draw.blend_mode)
            || previous_owner.is_some_and(|owner| owner >= draw.layer_index)
            || draw.destination_visits > pixels as u64
        {
            return Err(invalid("direct draw owner/mode/order/support differs"));
        }
        previous_owner = Some(draw.layer_index);
        if !draw.blend_mode.is_normal() {
            blend_work = add(blend_work, mul(draw.destination_visits, 32)?)?;
        }
        if !matches!(schedule.tasks.get(id.0), Some(MatteTask::LeafSample {layer_index,..} | MatteTask::AverageCopy {layer_index,..}) if *layer_index == draw.layer_index)
        {
            return Err(invalid("direct draw task does not belong to owning layer"));
        }
        if id.0 >= slots.len()
            || slots[id.0].direct_seen
            || matches!(schedule.tasks[id.0], MatteTask::AggregateProvider { .. })
        {
            return Err(invalid(
                "matte direct drawing must use individual copies once",
            ));
        }
        slots[id.0].direct_seen = true;
        slots[id.0].last_use = schedule
            .tasks
            .len()
            .checked_add(i)
            .ok_or_else(|| invalid("matte last-use overflow"))?;
    }
    if slots
        .iter()
        .zip(&schedule.last_uses)
        .any(|(s, c)| s.last_use != *c)
    {
        return Err(invalid(
            "matte last-use certificate differs from actual references",
        ));
    }
    if blend_work != schedule.certificate.blend_work_units
        || blend_work > MAX_MATTE_WORK
        || requests != schedule.certificate.provider_requests
        || requests > MAX_MATTE_REQUESTS
        || work > schedule.certificate.matte_work_units
        || schedule.certificate.matte_work_units > MAX_MATTE_WORK
    {
        return Err(invalid("matte request/work certificate is insufficient"));
    }
    // Two stage buckets per slot cover N tasks followed by <=N unique direct
    // draws. Intrusive release links retain O(tasks+references) lifetime work
    // without an extra allocation or repeated scanning of all result slots.
    for i in 0..slots.len() {
        let stage = slots[i].last_use;
        let bucket = usize::from(stage >= slots.len());
        let head_index = if bucket == 0 {
            stage
        } else {
            stage - slots.len()
        };
        slots[i].release_next = slots[head_index].release_heads[bucket];
        slots[head_index].release_heads[bucket] = std::num::NonZeroUsize::new(i + 1);
    }
    Ok(slots)
}
struct Live {
    bytes: u64,
    peak: u64,
    limit: u64,
    work: u64,
    work_limit: u64,
    requests: u64,
}
impl Live {
    fn new(schedule: &MatteFrameSchedule) -> Self {
        let c = &schedule.certificate;
        Self {
            bytes: c.fixed_live_bytes,
            peak: c.fixed_live_bytes,
            limit: c.peak_live_bytes,
            work: 0,
            work_limit: c.matte_work_units,
            requests: 0,
        }
    }
    fn charge(&mut self, bytes: u64) -> Result<(), CoreError> {
        let next = add(self.bytes, bytes)?;
        if next > self.limit {
            return Err(invalid("matte live allocation exceeds certificate"));
        }
        self.bytes = next;
        self.peak = self.peak.max(next);
        Ok(())
    }
    fn release(&mut self, bytes: u64) {
        self.bytes -= bytes;
    }
    fn work(&mut self, units: u64) -> Result<(), CoreError> {
        let next = add(self.work, units)?;
        if next > self.work_limit {
            return Err(invalid("matte touched work exceeds certificate"));
        }
        self.work = next;
        Ok(())
    }
}
fn result(
    slots: &[Slot],
    id: crate::evaluated_scene::mattes::MatteTaskId,
) -> Result<&LinearPlane, CoreError> {
    slots
        .get(id.0)
        .and_then(|s| s.plane.as_ref())
        .ok_or_else(|| invalid("matte dependency released before its last use"))
}
fn union_bounds(
    ids: &[crate::evaluated_scene::mattes::MatteTaskId],
    slots: &[Slot],
) -> Result<(usize, usize, usize, usize), CoreError> {
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for &id in ids {
        let p = result(slots, id)?;
        if p.width == 0 {
            continue;
        }
        let right = p
            .left
            .checked_add(p.width)
            .ok_or_else(|| invalid("matte union overflow"))?;
        let bottom = p
            .top
            .checked_add(p.height)
            .ok_or_else(|| invalid("matte union overflow"))?;
        bounds = Some(match bounds {
            None => (p.left, p.top, right, bottom),
            Some((l, t, r, b)) => (l.min(p.left), t.min(p.top), r.max(right), b.max(bottom)),
        });
    }
    Ok(bounds.map_or((0, 0, 0, 0), |(l, t, r, b)| (l, t, r - l, b - t)))
}
fn allocate_plane(
    bounds: (usize, usize, usize, usize),
    live: &mut Live,
) -> Result<LinearPlane, CoreError> {
    let (left, top, width, height) = bounds;
    if width == 0 {
        return Ok(LinearPlane::empty());
    }
    let n = width
        .checked_mul(height)
        .ok_or_else(|| invalid("matte plane size overflow"))?;
    let requested = bytes::<[f32; 4]>(n)?;
    live.charge(requested)?;
    let mut pixels = Vec::new();
    if pixels.try_reserve_exact(n).is_err() {
        live.release(requested);
        return Err(invalid("matte plane allocation failed"));
    }
    let actual = bytes::<[f32; 4]>(pixels.capacity())?;
    if actual > requested
        && let Err(e) = live.charge(actual - requested)
    {
        live.release(requested);
        return Err(e);
    }
    pixels.resize(n, [0.; 4]);
    Ok(LinearPlane {
        left,
        top,
        width,
        height,
        pixels,
    })
}
fn source_over(destination: &mut [f32; 4], source: [f32; 4]) {
    // Inputs have already passed canonical f32 validation/clamping. Bounded
    // source-over has at most ordinary f32 roundoff, corrected exactly as the
    // existing compositor; no fallible operation remains after destination commit.
    for c in 0..4 {
        destination[c] = source[c] + destination[c] * (1. - source[3]);
    }
    clamp_pixel(destination);
}
fn average(
    ids: &[crate::evaluated_scene::mattes::MatteTaskId],
    slots: &[Slot],
    live: &mut Live,
) -> Result<LinearPlane, CoreError> {
    let mut output = allocate_plane(union_bounds(ids, slots)?, live)?;
    for y in 0..output.height {
        for x in 0..output.width {
            let mut sum = [0f64; 4];
            for &id in ids {
                let p = result(slots, id)?.at(output.left + x, output.top + y);
                for c in 0..4 {
                    sum[c] += f64::from(p[c]);
                }
            }
            let pixel = &mut output.pixels[y * output.width + x];
            for c in 0..4 {
                pixel[c] = (sum[c] / ids.len() as f64) as f32;
            }
            validate_pixel(pixel)?;
            clamp_pixel(pixel);
        }
    }
    Ok(output)
}
fn aggregate(
    ids: &[crate::evaluated_scene::mattes::MatteTaskId],
    slots: &[Slot],
    live: &mut Live,
) -> Result<LinearPlane, CoreError> {
    let mut output = allocate_plane(union_bounds(ids, slots)?, live)?;
    for &id in ids {
        let source = result(slots, id)?;
        live.work(mul(mul(source.width as u64, source.height as u64)?, 4)?)?;
        for y in 0..source.height {
            for x in 0..source.width {
                let index =
                    (source.top + y - output.top) * output.width + source.left + x - output.left;
                source_over(
                    &mut output.pixels[index],
                    source.pixels[y * source.width + x],
                );
            }
        }
    }
    Ok(output)
}
fn apply_provider(
    plane: &mut LinearPlane,
    provider: &LinearPlane,
    channel: crate::MatteChannel,
) -> Result<(), CoreError> {
    for y in 0..plane.height {
        for x in 0..plane.width {
            let source = provider.at(plane.left + x, plane.top + y);
            let coverage = match channel {
                crate::MatteChannel::Alpha => source[3],
                crate::MatteChannel::Luma => {
                    (0.2126 * f64::from(source[0])
                        + 0.7152 * f64::from(source[1])
                        + 0.0722 * f64::from(source[2])) as f32
                }
            };
            let pixel = &mut plane.pixels[y * plane.width + x];
            for c in pixel.iter_mut() {
                *c *= coverage;
            }
            validate_pixel(pixel)?;
            clamp_pixel(pixel);
        }
    }
    Ok(())
}
fn release_finished(slots: &mut [Slot], stage: usize, live: &mut Live) {
    if slots.is_empty() {
        return;
    }
    let bucket = usize::from(stage >= slots.len());
    let head_index = if bucket == 0 {
        stage
    } else {
        stage - slots.len()
    };
    let mut next = slots[head_index].release_heads[bucket].take();
    while let Some(index) = next {
        let slot = &mut slots[index.get() - 1];
        next = slot.release_next;
        if slot.plane.take().is_some() {
            live.release(slot.payload_bytes);
            slot.payload_bytes = 0;
        }
    }
}

fn execute(
    schedule: &MatteFrameSchedule,
    destination: &mut [[f32; 4]],
    sample: &mut dyn FnMut(usize, u64) -> Result<LeafSamplePlane, CoreError>,
) -> Result<Live, CoreError> {
    use crate::evaluated_scene::mattes::MatteTask;
    let n = pixel_count(schedule.canvas)?;
    if destination.len() != n {
        return Err(invalid("matte destination dimensions differ"));
    }
    for pixel in destination.iter() {
        validate_pixel(pixel)?;
    }
    let mut slots = preflight(schedule, n)?;
    let mut live = Live::new(schedule);
    for (i, task) in schedule.tasks.iter().enumerate() {
        let plane = match task {
            MatteTask::LeafSample {
                layer_index,
                at_ms,
                provider,
                source_live_bytes,
            } => {
                live.charge(*source_live_bytes)?;
                let sampled = sample(*layer_index, *at_ms);
                let mut sampled = match sampled {
                    Ok(p) => p,
                    Err(e) => {
                        live.release(*source_live_bytes);
                        return Err(e);
                    }
                };
                let actual = sampled.plane.payload_bytes()?;
                if actual > *source_live_bytes {
                    return Err(invalid("matte callback capacity exceeds source reserve"));
                }
                sampled.plane.validate(schedule.canvas)?;
                if !sampled.gain.is_finite() || !(0.0..=1.0).contains(&sampled.gain) {
                    return Err(invalid("invalid matte inherited gain"));
                }
                if let Some((id, channel)) = provider {
                    live.work(mul(
                        mul(sampled.plane.width as u64, sampled.plane.height as u64)?,
                        5,
                    )?)?;
                    apply_provider(&mut sampled.plane, result(&slots, *id)?, *channel)?;
                }
                for pixel in &mut sampled.plane.pixels {
                    for c in pixel.iter_mut() {
                        *c *= sampled.gain;
                    }
                    validate_pixel(pixel)?;
                    clamp_pixel(pixel);
                }
                // Keep returned capacity continuously charged: release ONLY the
                // unused transient portion, rather than release/adopt with a gap.
                live.release(*source_live_bytes - actual);
                sampled.plane
            }
            MatteTask::AverageCopy { samples, .. } => average(samples, &slots, &mut live)?,
            MatteTask::AggregateProvider { copies } => {
                live.requests = add(live.requests, 1)?;
                aggregate(copies, &slots, &mut live)?
            }
        };
        slots[i].payload_bytes = plane.payload_bytes()?;
        slots[i].plane = Some(plane);
        release_finished(&mut slots, i, &mut live);
    }
    // All callback, numeric, support, index, capacity and work failures precede
    // destination mutation. Direct results are checked together before commit.
    let mut actual_blend_work = 0;
    for draw in &schedule.direct_draw {
        let plane = result(&slots, draw.task)?;
        let visits = mul(plane.width as u64, plane.height as u64)?;
        if visits > draw.destination_visits {
            return Err(invalid("direct result exceeds certified footprint"));
        }
        if !draw.blend_mode.is_normal() {
            actual_blend_work = add(actual_blend_work, mul(visits, 32)?)?;
        }
    }
    if actual_blend_work > schedule.certificate.blend_work_units {
        return Err(invalid("blend touched work exceeds certificate"));
    }
    for pixel in destination.iter_mut() {
        clamp_pixel(pixel);
    }
    let width = schedule.canvas.0 as usize;
    for (i, draw) in schedule.direct_draw.iter().enumerate() {
        let plane = slots[draw.task.0]
            .plane
            .as_ref()
            .expect("preflighted live direct plane");
        for y in 0..plane.height {
            for x in 0..plane.width {
                let destination = &mut destination[(plane.top + y) * width + plane.left + x];
                let source = plane.pixels[y * plane.width + x];
                if draw.blend_mode.is_normal() {
                    source_over(destination, source);
                } else {
                    super::blend::composite(destination, source, draw.blend_mode);
                }
            }
        }
        release_finished(&mut slots, schedule.tasks.len() + i, &mut live);
    }
    debug_assert_eq!(live.bytes, schedule.certificate.fixed_live_bytes);
    Ok(live)
}
pub(super) fn compose_frame(
    schedule: &MatteFrameSchedule,
    destination: &mut [[f32; 4]],
    sample: &mut dyn FnMut(usize, u64) -> Result<LeafSamplePlane, CoreError>,
) -> Result<(), CoreError> {
    execute(schedule, destination, sample).map(|_| ())
}
#[cfg(test)]
mod tests;
