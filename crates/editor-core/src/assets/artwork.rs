//! Bounded, renderer-independent PNG artwork preparation.
//!
//! Implements the exact cleanup, alpha dilation and review planes specified by
//! artwork-preparation. Source bytes are immutable; publication belongs to store.
use std::{collections::VecDeque, io::Cursor};

use crate::{ArtworkParameters, CoreError, ErrorCode};

const SOURCE_BYTES: usize = 4 * 1024 * 1024;
const SOURCE_SIDE: u32 = 512;
const SOURCE_PIXELS: u32 = 262_144;
const REVIEW_BYTES: usize = 8 * 1024 * 1024;
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub(crate) struct ArtworkPngFiles {
    pub(crate) result: Vec<u8>,
    pub(crate) mask: Vec<u8>,
    pub(crate) light: Vec<u8>,
    pub(crate) dark: Vec<u8>,
}

pub(crate) struct PreparedArtwork {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) source_width: u32,
    pub(crate) source_height: u32,
    pub(crate) content_offset: [u32; 2],
    pub(crate) files: ArtworkPngFiles,
}

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}

fn unsupported() -> CoreError {
    CoreError::new(
        ErrorCode::UnsupportedMedia,
        "artwork preparation supports static PNG images only",
    )
}

// This bounded chunk walk also detects APNG control chunks after the first IDAT.
// Pixel decoding and checksums remain the pinned PNG decoder's responsibility.
fn check_container(bytes: &[u8]) -> Result<(), CoreError> {
    if bytes.len() > SOURCE_BYTES {
        return Err(invalid("artwork PNG exceeds the 4 MiB source limit"));
    }
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Err(unsupported());
    }
    let mut position = PNG_SIGNATURE.len();
    while position < bytes.len() {
        let header = bytes
            .get(position..position + 8)
            .ok_or_else(|| invalid("artwork PNG chunk header is truncated"))?;
        let size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let end = position
            .checked_add(size)
            .and_then(|value| value.checked_add(12))
            .ok_or_else(|| invalid("artwork PNG chunk length is invalid"))?;
        if end > bytes.len() {
            return Err(invalid("artwork PNG chunk is truncated"));
        }
        let kind = &header[4..8];
        if kind == b"acTL" || kind == b"fcTL" || kind == b"fdAT" {
            return Err(unsupported());
        }
        if kind == b"IEND" {
            return if size == 0 && end == bytes.len() {
                Ok(())
            } else {
                Err(invalid("artwork PNG has an invalid ending"))
            };
        }
        position = end;
    }
    Err(invalid("artwork PNG is missing its ending"))
}

fn decode(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), CoreError> {
    check_container(bytes)?;
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(bytes),
        png::Limits {
            bytes: REVIEW_BYTES,
        },
    );
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let header = decoder
        .read_header_info()
        .map_err(|_| invalid("artwork PNG header cannot be decoded"))?;
    let (width, height) = (header.width, header.height);
    if width == 0
        || height == 0
        || width > SOURCE_SIDE
        || height > SOURCE_SIDE
        || width
            .checked_mul(height)
            .is_none_or(|size| size > SOURCE_PIXELS)
    {
        return Err(invalid("artwork PNG exceeds the 512 by 512 pixel limit"));
    }
    let mut reader = decoder
        .read_info()
        .map_err(|_| invalid("artwork PNG metadata cannot be decoded"))?;
    if reader.info().animation_control.is_some() {
        return Err(unsupported());
    }
    let buffer_size = reader
        .output_buffer_size()
        .filter(|size| *size <= SOURCE_PIXELS as usize * 4)
        .ok_or_else(|| invalid("artwork PNG decoded buffer exceeds its limit"))?;
    let mut decoded = vec![0; buffer_size];
    let frame = reader
        .next_frame(&mut decoded)
        .map_err(|_| invalid("artwork PNG pixels cannot be decoded"))?;
    reader
        .finish()
        .map_err(|_| invalid("artwork PNG ending cannot be decoded"))?;
    if frame.width != width || frame.height != height || frame.bit_depth != png::BitDepth::Eight {
        return Err(unsupported());
    }
    let decoded = &decoded[..frame.buffer_size()];
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    match frame.color_type {
        png::ColorType::Rgba => pixels.extend_from_slice(decoded),
        png::ColorType::Rgb => {
            for value in decoded.as_chunks::<3>().0 {
                pixels.extend_from_slice(&[value[0], value[1], value[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for value in decoded {
                pixels.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for value in decoded.as_chunks::<2>().0 {
                pixels.extend_from_slice(&[value[0], value[0], value[0], value[1]]);
            }
        }
        png::ColorType::Indexed => return Err(unsupported()),
    }
    if pixels.len() != width as usize * height as usize * 4 {
        return Err(invalid("artwork PNG decoded pixel count is inconsistent"));
    }
    Ok((width, height, pixels))
}

fn cleanup(pixels: &mut [u8], width: usize, height: usize, color: [u8; 3], tolerance: u8) {
    let mut seen = vec![false; width * height];
    let mut queue = VecDeque::new();
    let enqueue = |index: usize, seen: &mut [bool], queue: &mut VecDeque<usize>| {
        if seen[index] {
            return;
        }
        let pixel = &pixels[index * 4..index * 4 + 4];
        if pixel[3] == 0
            || (0..3).all(|channel| pixel[channel].abs_diff(color[channel]) <= tolerance)
        {
            seen[index] = true;
            queue.push_back(index);
        }
    };
    for x in 0..width {
        enqueue(x, &mut seen, &mut queue);
        enqueue((height - 1) * width + x, &mut seen, &mut queue);
    }
    for y in 0..height {
        enqueue(y * width, &mut seen, &mut queue);
        enqueue(y * width + width - 1, &mut seen, &mut queue);
    }
    while let Some(index) = queue.pop_front() {
        let (x, y) = (index % width, index / width);
        if x > 0 {
            enqueue(index - 1, &mut seen, &mut queue);
        }
        if x + 1 < width {
            enqueue(index + 1, &mut seen, &mut queue);
        }
        if y > 0 {
            enqueue(index - width, &mut seen, &mut queue);
        }
        if y + 1 < height {
            enqueue(index + width, &mut seen, &mut queue);
        }
    }
    for (pixel, clear) in pixels.as_chunks_mut::<4>().0.iter_mut().zip(seen) {
        if clear {
            pixel[3] = 0;
        }
    }
}

fn dilate(alpha: &[u8], width: usize, height: usize, radius: usize) -> Vec<u8> {
    let mut horizontal = vec![0; alpha.len()];
    for y in 0..height {
        for x in 0..width {
            horizontal[y * width + x] = alpha
                [y * width + x.saturating_sub(radius)..y * width + (x + radius + 1).min(width)]
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
        }
    }
    let mut result = vec![0; alpha.len()];
    for y in 0..height {
        for x in 0..width {
            result[y * width + x] = (y.saturating_sub(radius)..(y + radius + 1).min(height))
                .map(|sample| horizontal[sample * width + x])
                .max()
                .unwrap_or(0);
        }
    }
    result
}

fn rounded(numerator: u32, denominator: u32) -> u8 {
    ((numerator + denominator / 2) / denominator) as u8
}

fn source_over(source: &[u8], outline_alpha: u8, color: [u8; 4]) -> [u8; 4] {
    let source_alpha = u32::from(source[3]);
    let below_alpha = u32::from(outline_alpha);
    let denominator = source_alpha * 255 + below_alpha * (255 - source_alpha);
    if denominator == 0 {
        return [0; 4];
    }
    let mut result = [0, 0, 0, rounded(denominator, 255)];
    for channel in 0..3 {
        result[channel] = rounded(
            u32::from(source[channel]) * source_alpha * 255
                + u32::from(color[channel]) * below_alpha * (255 - source_alpha),
            denominator,
        );
    }
    result
}

fn encode(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, CoreError> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder
            .write_header()
            .map_err(|_| CoreError::new(ErrorCode::InternalError, "artwork PNG encoding failed"))?;
        writer
            .write_image_data(rgba)
            .map_err(|_| CoreError::new(ErrorCode::InternalError, "artwork PNG encoding failed"))?;
        writer
            .finish()
            .map_err(|_| CoreError::new(ErrorCode::InternalError, "artwork PNG encoding failed"))?;
    }
    Ok(bytes)
}

pub(crate) fn prepare_png(
    bytes: &[u8],
    parameters: &ArtworkParameters,
) -> Result<PreparedArtwork, CoreError> {
    parameters.validate()?;
    let (source_width, source_height, mut source) = decode(bytes)?;
    if let Some(selected) = &parameters.cleanup {
        cleanup(
            &mut source,
            source_width as usize,
            source_height as usize,
            selected.color,
            selected.tolerance,
        );
    }
    let radius = parameters.outline.radius as usize;
    let width = source_width as usize + 2 * radius;
    let height = source_height as usize + 2 * radius;
    let mut padded = vec![0; width * height * 4];
    for y in 0..source_height as usize {
        let start = ((y + radius) * width + radius) * 4;
        let source_start = y * source_width as usize * 4;
        padded[start..start + source_width as usize * 4]
            .copy_from_slice(&source[source_start..source_start + source_width as usize * 4]);
    }
    let alpha: Vec<u8> = padded
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[3])
        .collect();
    let outline = if radius == 0 {
        vec![0; alpha.len()]
    } else {
        dilate(&alpha, width, height, radius)
    };
    let mut result = Vec::with_capacity(padded.len());
    let mut mask = Vec::with_capacity(padded.len());
    let mut light = Vec::with_capacity(padded.len());
    let mut dark = Vec::with_capacity(padded.len());
    for (index, source_pixel) in padded.as_chunks::<4>().0.iter().enumerate() {
        let outline_alpha = rounded(
            u32::from(outline[index]) * u32::from(parameters.outline.color[3]),
            255,
        );
        let pixel = source_over(source_pixel, outline_alpha, parameters.outline.color);
        result.extend_from_slice(&pixel);
        mask.extend_from_slice(&[alpha[index], alpha[index], alpha[index], 255]);
        for (background, plane) in [(255u32, &mut light), (32u32, &mut dark)] {
            for channel in pixel.iter().take(3) {
                plane.push(rounded(
                    u32::from(*channel) * u32::from(pixel[3])
                        + background * (255 - u32::from(pixel[3])),
                    255,
                ));
            }
            plane.push(255);
        }
    }
    let width = width as u32;
    let height = height as u32;
    let files = ArtworkPngFiles {
        result: encode(width, height, &result)?,
        mask: encode(width, height, &mask)?,
        light: encode(width, height, &light)?,
        dark: encode(width, height, &dark)?,
    };
    if [&files.result, &files.mask, &files.light, &files.dark]
        .iter()
        .map(|file| file.len())
        .sum::<usize>()
        > REVIEW_BYTES
    {
        return Err(invalid("artwork review PNGs exceed their 8 MiB limit"));
    }
    Ok(PreparedArtwork {
        width,
        height,
        source_width,
        source_height,
        content_offset: [radius as u32; 2],
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArtworkCleanup, ArtworkOutline};

    fn params(cleanup: Option<ArtworkCleanup>, radius: u32, color: [u8; 4]) -> ArtworkParameters {
        ArtworkParameters {
            cleanup,
            outline: ArtworkOutline { radius, color },
        }
    }

    fn clean(color: [u8; 3], tolerance: u8) -> Option<ArtworkCleanup> {
        Some(ArtworkCleanup { color, tolerance })
    }

    fn pixels(png: &[u8]) -> (u32, u32, Vec<u8>) {
        let decoder = png::Decoder::new(Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        (info.width, info.height, pixels)
    }

    fn prepare_pixels(
        width: u32,
        height: u32,
        values: &[u8],
        controls: &ArtworkParameters,
    ) -> PreparedArtwork {
        prepare_png(&encode(width, height, values).unwrap(), controls).unwrap()
    }

    #[test]
    fn radius_zero_preserves_partial_alpha_and_has_visible_mask_and_backgrounds() {
        let prepared = prepare_pixels(
            2,
            1,
            &[10, 20, 30, 128, 99, 80, 9, 0],
            &params(None, 0, [255, 0, 255, 255]),
        );
        assert_eq!((prepared.width, prepared.height), (2, 1));
        assert_eq!(prepared.content_offset, [0, 0]);
        assert_eq!(
            pixels(&prepared.files.result).2,
            [10, 20, 30, 128, 0, 0, 0, 0]
        );
        assert_eq!(
            pixels(&prepared.files.mask).2,
            [128, 128, 128, 255, 0, 0, 0, 255]
        );
        assert_eq!(
            pixels(&prepared.files.light).2,
            [132, 137, 142, 255, 255, 255, 255, 255]
        );
        assert_eq!(
            pixels(&prepared.files.dark).2,
            [21, 26, 31, 255, 32, 32, 32, 255]
        );
    }

    #[test]
    fn opaque_edge_subject_gets_explicit_padding_instead_of_cropped_outline() {
        let prepared = prepare_pixels(1, 1, &[200, 10, 20, 255], &params(None, 1, [3, 4, 5, 255]));
        assert_eq!((prepared.source_width, prepared.source_height), (1, 1));
        assert_eq!((prepared.width, prepared.height), (3, 3));
        assert_eq!(prepared.content_offset, [1, 1]);
        let mut expected = [3, 4, 5, 255].repeat(9);
        expected[16..20].copy_from_slice(&[200, 10, 20, 255]);
        assert_eq!(pixels(&prepared.files.result).2, expected);
        let mut mask = [0, 0, 0, 255].repeat(9);
        mask[16..20].copy_from_slice(&[255; 4]);
        assert_eq!(pixels(&prepared.files.mask).2, mask);
    }

    #[test]
    fn partial_alpha_outline_uses_exact_straight_source_over_without_halo_guessing() {
        let prepared = prepare_pixels(1, 1, &[200, 0, 0, 128], &params(None, 1, [0, 0, 200, 128]));
        // Independent 1x1 oracle: outline A=round(128*128/255)=64;
        // center D=128*255+64*127=40768, hence A160/R160/B40.
        let mut expected = [0, 0, 200, 64].repeat(9);
        expected[16..20].copy_from_slice(&[160, 0, 40, 160]);
        assert_eq!(pixels(&prepared.files.result).2, expected);
        assert_eq!(
            &pixels(&prepared.files.light).2[16..20],
            &[195, 95, 120, 255]
        );
        assert_eq!(&pixels(&prepared.files.dark).2[16..20], &[112, 12, 37, 255]);
    }

    #[test]
    fn cleanup_uses_four_connectivity_and_preserves_interior_opaque_white() {
        // Only diagonal white corner pixels reach the border; the central opaque
        // white subject is separated by four red neighbors and remains intact.
        let white = [255; 4];
        let red = [180, 0, 0, 255];
        let source = [white, red, white, red, white, red, white, red, white].concat();
        let prepared = prepare_pixels(3, 3, &source, &params(clean([255; 3], 0), 0, [0; 4]));
        let clear = [0; 4];
        let expected = [clear, red, clear, red, white, red, clear, red, clear].concat();
        assert_eq!(pixels(&prepared.files.result).2, expected);
    }

    #[test]
    fn transparent_border_routes_cleanup_without_matching_its_hidden_rgb() {
        let transparent = [6, 7, 8, 0];
        let red = [200, 0, 0, 255];
        let white = [255; 4];
        let source = [red, transparent, red, red, white, red, red, red, red].concat();
        let prepared = prepare_pixels(3, 3, &source, &params(clean([255; 3], 0), 0, [0; 4]));
        let expected = [red, [0; 4], red, red, [0; 4], red, red, red, red].concat();
        assert_eq!(pixels(&prepared.files.result).2, expected);
    }

    #[test]
    fn tolerance_is_inclusive_per_channel_and_disabled_is_distinct() {
        let source = [11, 20, 30, 255];
        for (controls, expected) in [
            (params(None, 0, [0; 4]), source),
            (params(clean([10, 20, 30], 0), 0, [0; 4]), source),
            (params(clean([10, 20, 30], 1), 0, [0; 4]), [0; 4]),
            (params(clean([255, 255, 255], 255), 0, [0; 4]), [0; 4]),
        ] {
            assert_eq!(
                pixels(&prepare_pixels(1, 1, &source, &controls).files.result).2,
                expected
            );
        }
    }

    #[test]
    fn square_dilation_preserves_a_hole_at_zero_radius_and_covers_it_only_when_selected() {
        let ink = [20, 30, 40, 255];
        let mut source = ink.repeat(9);
        source[16..20].copy_from_slice(&[0; 4]);
        let unchanged = prepare_pixels(3, 3, &source, &params(None, 0, [0; 4]));
        assert_eq!(pixels(&unchanged.files.result).2, source);
        let outlined = prepare_pixels(3, 3, &source, &params(None, 1, [9, 8, 7, 255]));
        let result = pixels(&outlined.files.result).2;
        assert_eq!(
            &result[(2 * 5 + 2) * 4..(2 * 5 + 2) * 4 + 4],
            &[9, 8, 7, 255]
        );
        let mask = pixels(&outlined.files.mask).2;
        assert_eq!(&mask[(2 * 5 + 2) * 4..(2 * 5 + 2) * 4 + 4], &[0, 0, 0, 255]);
        assert_eq!(&result[4..8], &[9, 8, 7, 255]);
    }

    fn custom_png(
        color: png::ColorType,
        depth: png::BitDepth,
        data: &[u8],
        palette: Option<Vec<u8>>,
        transparency: Option<Vec<u8>>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
            encoder.set_color(color);
            encoder.set_depth(depth);
            if let Some(palette) = palette {
                encoder.set_palette(palette);
            }
            if let Some(transparency) = transparency {
                encoder.set_trns(transparency);
            }
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(data).unwrap();
        }
        bytes
    }

    #[test]
    fn png_normalization_expands_palette_transparency_gray_and_strips_low_16_bits() {
        let cases = [
            (
                custom_png(
                    png::ColorType::Rgb,
                    png::BitDepth::Eight,
                    &[4, 5, 6],
                    None,
                    None,
                ),
                [4, 5, 6, 255],
            ),
            (
                custom_png(
                    png::ColorType::Grayscale,
                    png::BitDepth::One,
                    &[128],
                    None,
                    None,
                ),
                [255; 4],
            ),
            (
                custom_png(
                    png::ColorType::GrayscaleAlpha,
                    png::BitDepth::Eight,
                    &[80, 128],
                    None,
                    None,
                ),
                [80, 80, 80, 128],
            ),
            (
                custom_png(
                    png::ColorType::Indexed,
                    png::BitDepth::One,
                    &[0],
                    Some(vec![5, 10, 15]),
                    Some(vec![64]),
                ),
                [5, 10, 15, 64],
            ),
            (
                custom_png(
                    png::ColorType::Rgba,
                    png::BitDepth::Sixteen,
                    &[10, 254, 20, 253, 30, 252, 40, 251],
                    None,
                    None,
                ),
                [10, 20, 30, 40],
            ),
            (
                custom_png(
                    png::ColorType::Grayscale,
                    png::BitDepth::Eight,
                    &[80],
                    None,
                    Some(vec![0, 80]),
                ),
                [0; 4],
            ),
        ];
        for (bytes, expected) in cases {
            assert_eq!(
                pixels(
                    &prepare_png(&bytes, &params(None, 0, [0; 4]))
                        .unwrap()
                        .files
                        .result
                )
                .2,
                expected
            );
        }
    }

    #[test]
    fn malformed_oversized_animated_and_unsupported_sources_are_typed_failures() {
        let controls = params(None, 0, [0; 4]);
        assert_eq!(
            prepare_png(b"JPEG", &controls).err().unwrap().code,
            ErrorCode::UnsupportedMedia
        );
        assert_eq!(
            prepare_png(PNG_SIGNATURE, &controls).err().unwrap().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(
            prepare_png(&vec![0; SOURCE_BYTES + 1], &controls)
                .err()
                .unwrap()
                .code,
            ErrorCode::ValidationFailed
        );
        let large = encode(513, 1, &vec![0; 513 * 4]).unwrap();
        assert_eq!(
            prepare_png(&large, &controls).err().unwrap().code,
            ErrorCode::ValidationFailed
        );
        let valid = encode(1, 1, &[10, 20, 30, 255]).unwrap();
        let mut corrupt = valid.clone();
        corrupt[29] ^= 1; // IHDR checksum.
        assert_eq!(
            prepare_png(&corrupt, &controls).err().unwrap().code,
            ErrorCode::ValidationFailed
        );
        let mut truncated = valid.clone();
        truncated.pop();
        assert_eq!(
            prepare_png(&truncated, &controls).err().unwrap().code,
            ErrorCode::ValidationFailed
        );
        let mut animated = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut animated, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_animated(1, 0).unwrap();
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[10, 20, 30, 255]).unwrap();
        }
        assert_eq!(
            prepare_png(&animated, &controls).err().unwrap().code,
            ErrorCode::UnsupportedMedia
        );
        assert_eq!(
            prepare_png(&valid, &params(None, 17, [0; 4]))
                .err()
                .unwrap()
                .code,
            ErrorCode::ValidationFailed
        );
    }

    #[test]
    fn preparation_is_deterministic_from_original_and_maximum_bounds_are_admitted() {
        let source = encode(512, 512, &vec![255; 512 * 512 * 4]).unwrap();
        let controls = params(None, 16, [4, 5, 6, 255]);
        let first = prepare_png(&source, &controls).unwrap();
        let second = prepare_png(&source, &controls).unwrap();
        assert_eq!((first.width, first.height), (544, 544));
        assert_eq!(first.files.result, second.files.result);
        assert_eq!(first.files.mask, second.files.mask);
        assert_eq!(first.files.light, second.files.light);
        assert_eq!(first.files.dark, second.files.dark);
        assert_eq!(&pixels(&first.files.result).2[..4], &[4, 5, 6, 255]);
    }

    #[test]
    fn a_single_pixel_line_retains_exact_detail_when_outline_is_transparent() {
        let mut source = vec![0; 5 * 3 * 4];
        for y in 0..3 {
            source[(y * 5 + 2) * 4..(y * 5 + 2) * 4 + 4].copy_from_slice(&[12, 34, 56, 192]);
        }
        let prepared = prepare_pixels(5, 3, &source, &params(None, 1, [200, 0, 0, 0]));
        let mut expected = vec![0; 7 * 5 * 4];
        for y in 1..4 {
            expected[(y * 7 + 3) * 4..(y * 7 + 3) * 4 + 4].copy_from_slice(&[12, 34, 56, 192]);
        }
        assert_eq!(pixels(&prepared.files.result).2, expected);
    }

    #[test]
    fn gamma_metadata_does_not_change_sample_values_or_result_identity() {
        let mut source = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut source, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_source_gamma(png::ScaledFloat::from_scaled(100_000));
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[64, 128, 192, 255]).unwrap();
        }
        let plain = encode(1, 1, &[64, 128, 192, 255]).unwrap();
        let controls = params(None, 0, [0; 4]);
        assert_eq!(
            prepare_png(&source, &controls).unwrap().files.result,
            prepare_png(&plain, &controls).unwrap().files.result
        );
    }
}
