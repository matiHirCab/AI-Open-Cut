//! Disposable raster bytes. Callers must complete ordinary render preflight first.
use std::{
    collections::VecDeque,
    io::Write,
    sync::{Arc, Mutex},
};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    CoreError, ErrorCode,
    evaluated_scene::{EvaluatedSceneResult, EvaluatedText, shapes::EvaluatedShape},
    fonts::shaping::ShapedText,
    render_plan::PreparedText,
};

type Key = [u8; 32];
const VERSION: &str = "opencut-raster-v1";
const MAX_ENTRIES: usize = 128;
const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug)]
struct Entry {
    key: Key,
    bytes: Arc<[u8]>,
}

#[derive(Debug, Default)]
struct State {
    entries: VecDeque<Entry>,
    bytes: usize,
}

pub(crate) struct RasterCache {
    state: Mutex<State>,
    max_entries: usize,
    max_bytes: usize,
    #[cfg(any(test, feature = "raster-cache-test-hooks"))]
    pub(crate) hits: std::sync::atomic::AtomicUsize,
    #[cfg(any(test, feature = "raster-cache-test-hooks"))]
    pub(crate) misses: std::sync::atomic::AtomicUsize,
}

impl std::fmt::Debug for RasterCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Renderer diagnostics must not expose retained user text or pixel bytes.
        f.debug_struct("RasterCache")
            .field("max_entries", &self.max_entries)
            .field("max_bytes", &self.max_bytes)
            .finish_non_exhaustive()
    }
}

impl Default for RasterCache {
    fn default() -> Self {
        Self {
            state: Mutex::default(),
            max_entries: MAX_ENTRIES,
            max_bytes: MAX_BYTES,
            #[cfg(any(test, feature = "raster-cache-test-hooks"))]
            hits: Default::default(),
            #[cfg(any(test, feature = "raster-cache-test-hooks"))]
            misses: Default::default(),
        }
    }
}

impl State {
    fn get(&mut self, key: Key) -> Option<Arc<[u8]>> {
        let index = self.entries.iter().position(|e| e.key == key)?;
        let entry = self.entries.remove(index)?;
        let bytes = entry.bytes.clone();
        self.entries.push_back(entry);
        Some(bytes)
    }
}

impl RasterCache {
    pub(super) fn raster(
        &self,
        key: Key,
        render: impl FnOnce() -> Result<Vec<u8>, CoreError>,
    ) -> Result<Arc<[u8]>, CoreError> {
        if let Ok(mut state) = self.state.lock()
            && let Some(bytes) = state.get(key)
        {
            #[cfg(any(test, feature = "raster-cache-test-hooks"))]
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(bytes);
        }
        #[cfg(any(test, feature = "raster-cache-test-hooks"))]
        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Never hold the lock during rasterization and never cache errors.
        let bytes: Arc<[u8]> = render()?.into();
        let Some(cost) = bytes.len().checked_add(size_of::<Key>()) else {
            return Ok(bytes);
        };
        if cost > self.max_bytes || self.max_entries == 0 {
            return Ok(bytes);
        }
        let Ok(mut state) = self.state.lock() else {
            return Ok(bytes);
        };
        if let Some(existing) = state.get(key) {
            return Ok(existing);
        }
        while state.entries.len() >= self.max_entries || state.bytes > self.max_bytes - cost {
            if let Some(entry) = state.entries.pop_front() {
                state.bytes -= size_of::<Key>() + entry.bytes.len();
            } else {
                break;
            }
        }
        state.bytes += cost; // bounded by max_bytes - cost above
        state.entries.push_back(Entry {
            key,
            bytes: bytes.clone(),
        });
        Ok(bytes)
    }
}

// Each typed JSON value is terminated with a byte JSON cannot contain literally.
// Streaming avoids allocating keys proportional to geometry or glyph count.
struct KeyWriter(Sha256);
impl Write for KeyWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl KeyWriter {
    fn new(scope: Key, kind: &str) -> Result<Self, CoreError> {
        let mut result = Self(Sha256::new());
        result.field(&(VERSION, scope, kind))?;
        Ok(result)
    }
    fn field(&mut self, value: &impl Serialize) -> Result<(), CoreError> {
        serde_json::to_writer(&mut *self, value).map_err(|_| {
            CoreError::new(ErrorCode::InternalError, "cannot identify raster input")
        })?;
        self.0.update([0]);
        Ok(())
    }
    fn finish(self) -> Key {
        self.0.finalize().into()
    }
}

pub(crate) fn scope(evaluated: &EvaluatedSceneResult) -> Result<Key, CoreError> {
    let mut key = KeyWriter::new([0; 32], "scope")?;
    key.field(&(
        &evaluated.project_id,
        evaluated.revision,
        evaluated.scene.canvas.width,
        evaluated.scene.canvas.height,
    ))?;
    Ok(key.finish())
}

pub(super) fn text_key(
    scope: Key,
    text: &EvaluatedText,
    shaped: &ShapedText,
    prepared: &PreparedText,
) -> Result<Key, CoreError> {
    let mut key = KeyWriter::new(scope, "text")?;
    // Verified bindings use SHA-256 of exact bytes and face index zero. Warnings,
    // selector paths, resource/occurrence IDs and composition anchors aren't pixels.
    if let Some(binding) = &text.font_binding {
        key.field(&(&binding.profile, binding.hashes(), 0_u32))?;
    }
    key.field(&(
        &text.text,
        &text.rich_runs,
        &text.spans,
        text.font_size,
        &text.color,
    ))?;
    let style = &text.style;
    key.field(&(
        &style.layout,
        &style.paint_layers,
        style.wrap_width_px,
        style.line_spacing_px,
        &style.outline_color,
        style.outline_width_px,
        &style.background_color,
        style.background_opacity,
    ))?;
    key.field(&(
        &style.shadow.color,
        style.shadow.opacity,
        style.shadow.offset_x,
        style.shadow.offset_y,
        style.padding.top,
        style.padding.right,
        style.padding.bottom,
        style.padding.left,
    ))?;
    key.field(&(
        prepared.layer_width,
        prepared.layer_height,
        prepared.canvas_width,
        prepared.canvas_height,
        prepared.text_x,
        prepared.text_y,
    ))?;
    key.field(&(
        shaped.font_size,
        shaped.width,
        shaped.height,
        shaped.line_height,
        &shaped.line_widths,
        &shaped.glyph_lines,
    ))?;
    key.field(&shaped.layout.as_ref().map(|l| {
        (
            l.background,
            l.content_width,
            l.content_height,
            l.overflow_x,
            l.overflow_y,
        )
    }))?;
    key.field(&shaped.glyphs.len())?;
    for g in &shaped.glyphs {
        key.field(&(
            &g.face,
            g.id,
            g.cluster,
            g.x,
            g.y,
            g.advance,
            &g.color,
            &g.paint_layers,
        ))?;
    }
    Ok(key.finish())
}

pub(super) fn shape_key(scope: Key, shape: &EvaluatedShape) -> Result<Key, CoreError> {
    let mut key = KeyWriter::new(scope, "vector")?;
    shape_fields(&mut key, shape)?;
    Ok(key.finish())
}

fn shape_fields(key: &mut KeyWriter, shape: &EvaluatedShape) -> Result<(), CoreError> {
    // Covers coverage(), svg_raster_path/stroke(), local paints and PAM geometry.
    key.field(&(
        &shape.geometry,
        &shape.grid_descriptor,
        &shape.svg_document,
        &shape.fill,
        &shape.stroke,
        shape.fill_rule,
        shape.bounds,
        shape.origin,
        shape.size,
        shape.density,
    ))?;
    key.field(&shape.contours.len())?;
    for contour in &shape.contours {
        key.field(&(&contour.points, contour.closed))?;
    }
    key.field(&shape.svg_children.as_ref().map(Vec::len))?;
    if let Some(children) = &shape.svg_children {
        for child in children {
            shape_fields(key, child)?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
