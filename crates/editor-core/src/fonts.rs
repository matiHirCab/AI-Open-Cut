//! Pure font validation and versioned layout over supplied immutable bytes.
pub(crate) mod shaping;
use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::{
    CoreError, ErrorCode, FontBinding, FontRecord, MAX_FONT_BYTES, MAX_FONT_FILES,
    MAX_TOTAL_FONT_BYTES, TEXT_LAYOUT_PROFILE,
};

pub(crate) const DEFAULT_FACES: [&[u8]; 4] = [
    include_bytes!("../resources/fonts/DejaVuSans.ttf"),
    include_bytes!("../resources/fonts/DejaVuSans-Bold.ttf"),
    include_bytes!("../resources/fonts/DejaVuSans-Oblique.ttf"),
    include_bytes!("../resources/fonts/DejaVuSans-BoldOblique.ttf"),
];

pub(crate) fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

pub(crate) fn content_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

pub(crate) fn validate_face(bytes: &[u8]) -> Result<ttf_parser::Face<'_>, CoreError> {
    if bytes.is_empty() || bytes.len() > MAX_FONT_BYTES || bytes.starts_with(b"ttcf") {
        return Err(invalid(
            "font must be a bounded standalone static outline face",
        ));
    }
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|_| invalid("malformed font face"))?;
    if face.is_variable()
        || face
            .raw_face()
            .table_records
            .into_iter()
            .any(|record| record.tag == ttf_parser::Tag::from_bytes(b"fvar"))
        || (face.tables().glyf.is_none() && face.tables().cff.is_none())
    {
        return Err(invalid("font must have static TrueType or CFF outlines"));
    }
    Ok(face)
}

pub(crate) fn record(bytes: &[u8]) -> Result<FontRecord, CoreError> {
    validate_face(bytes)?;
    let sha256 = content_hash(bytes);
    Ok(FontRecord {
        relative_path: format!("fonts/{sha256}.font"),
        sha256,
        size_bytes: bytes.len() as u64,
        face_index: 0,
    })
}

pub(crate) fn validate_catalog(fonts: &BTreeMap<String, FontRecord>) -> Result<(), CoreError> {
    if fonts.len() > MAX_FONT_FILES {
        return Err(invalid("font catalog exceeds 256 files"));
    }
    let mut total = 0u64;
    for (hash, face) in fonts {
        if !valid_hash(hash)
            || face.sha256 != *hash
            || face.face_index != 0
            || face.size_bytes == 0
            || face.size_bytes > MAX_FONT_BYTES as u64
        {
            return Err(invalid("invalid managed font identity or bounds"));
        }
        if face.relative_path != format!("fonts/{hash}.font") {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "font path must match its managed hash",
            ));
        }
        total = total
            .checked_add(face.size_bytes)
            .ok_or_else(|| invalid("font byte total overflow"))?;
    }
    if total > MAX_TOTAL_FONT_BYTES {
        return Err(invalid("font catalog exceeds 256 MiB"));
    }
    Ok(())
}

pub(crate) fn validate_binding(
    binding: &FontBinding,
    fonts: &BTreeMap<String, FontRecord>,
) -> Result<(), CoreError> {
    if binding.profile != TEXT_LAYOUT_PROFILE
        || binding.warnings.len() > 2
        || binding.warnings.iter().any(|w| w.len() > 512)
    {
        return Err(invalid("invalid text layout profile or font warnings"));
    }
    for hash in binding.hashes() {
        if !valid_hash(hash) {
            return Err(invalid("font binding requires lowercase SHA-256"));
        }
        if !fonts.contains_key(hash) {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "text font reference is missing from its catalog",
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_bytes(face: &FontRecord, bytes: &[u8]) -> Result<(), CoreError> {
    if bytes.len() as u64 != face.size_bytes || content_hash(bytes) != face.sha256 {
        return Err(CoreError::new(
            ErrorCode::AssetIntegrityFailed,
            "managed font bytes do not match their recorded hash",
        ));
    }
    validate_face(bytes).map_err(|_| {
        CoreError::new(
            ErrorCode::AssetIntegrityFailed,
            "managed font face is malformed",
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_faces_match_canonical_contract() {
        let contract: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/text-layout-v2.json")).unwrap();
        for (bytes, style) in DEFAULT_FACES
            .iter()
            .zip(["regular", "bold", "italic", "boldItalic"])
        {
            let face = record(bytes).unwrap();
            assert_eq!(face.sha256, contract["defaultFamily"][style]);
            verify_bytes(&face, bytes).unwrap();
        }
    }

    #[test]
    fn font_bytes_fail_closed() {
        for (old, new) in [(b"glyf", b"XXXX"), (b"name", b"fvar")] {
            let mut malformed = DEFAULT_FACES[0].to_vec();
            let count = u16::from_be_bytes([malformed[4], malformed[5]]) as usize;
            let table = (0..count)
                .map(|i| 12 + i * 16)
                .find(|offset| &malformed[*offset..*offset + 4] == old)
                .unwrap();
            malformed[table..table + 4].copy_from_slice(new);
            assert_eq!(
                validate_face(&malformed).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        let mut exact = DEFAULT_FACES[0].to_vec();
        exact.resize(MAX_FONT_BYTES, 0);
        validate_face(&exact).unwrap();
        exact.push(0);
        assert_eq!(
            validate_face(&exact).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        for bytes in [
            vec![],
            vec![0; MAX_FONT_BYTES + 1],
            b"ttcf".to_vec(),
            b"malformed".to_vec(),
        ] {
            assert_eq!(
                validate_face(&bytes).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        let face = record(DEFAULT_FACES[0]).unwrap();
        assert_eq!(
            verify_bytes(&face, DEFAULT_FACES[1]).unwrap_err().code,
            ErrorCode::AssetIntegrityFailed
        );
    }

    #[test]
    fn catalog_limits_and_paths_are_canonical() {
        let mut fonts = BTreeMap::new();
        for i in 0..256 {
            let hash = format!("{i:064x}");
            fonts.insert(
                hash.clone(),
                FontRecord {
                    relative_path: format!("fonts/{hash}.font"),
                    sha256: hash,
                    size_bytes: 1024 * 1024,
                    face_index: 0,
                },
            );
        }
        validate_catalog(&fonts).unwrap();
        let mut too_many = fonts.clone();
        let hash = format!("{:064x}", 256);
        too_many.insert(
            hash.clone(),
            FontRecord {
                sha256: hash.clone(),
                relative_path: format!("fonts/{hash}.font"),
                size_bytes: 1,
                face_index: 0,
            },
        );
        assert_eq!(
            validate_catalog(&too_many).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        fonts.values_mut().next().unwrap().size_bytes += 1;
        assert_eq!(
            validate_catalog(&fonts).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        fonts.values_mut().next().unwrap().size_bytes -= 1;
        fonts.values_mut().next().unwrap().relative_path = "../font.ttf".into();
        assert_eq!(
            validate_catalog(&fonts).unwrap_err().code,
            ErrorCode::PathNotAllowed
        );
    }
}
