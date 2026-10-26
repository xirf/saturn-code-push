use super::dto::{AssetFile, AssetInspection, AssetPreview, PreviewKind};
use crate::platform::error::{Result, bad, internal, missing};
use base64::{Engine, engine::general_purpose::STANDARD};
use image::{ImageFormat, ImageReader, Limits};
use std::{
    collections::HashSet,
    fs::File,
    io::{Cursor, Read},
    path::Path,
};
use zip::ZipArchive;

#[cfg(test)]
mod tests;

const MAX_ENTRIES: usize = 4096;
const MAX_EXPANDED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TEXT_BYTES: u64 = 256 * 1024;
const MAX_IMAGE_BYTES: u64 = 4 * 1024 * 1024;

/// Inspection never extracts archive paths to disk or executes uploaded content.
pub(super) fn list(path: &Path) -> Result<AssetInspection> {
    let mut archive = open(path)?;
    let files = entries(&mut archive)?;
    let expanded_size = files.iter().map(|entry| entry.size).sum();
    Ok(AssetInspection {
        files,
        expanded_size,
    })
}

pub(super) fn preview(path: &Path, index: usize) -> Result<AssetPreview> {
    let mut archive = open(path)?;
    let entries = entries(&mut archive)?;
    let entry = entries
        .iter()
        .find(|entry| entry.index == index)
        .ok_or_else(missing)?;
    let limit = match entry.preview {
        PreviewKind::Text => MAX_TEXT_BYTES,
        PreviewKind::Image => MAX_IMAGE_BYTES,
        PreviewKind::None => return Err(bad("Preview is unavailable for this file type or size")),
    };
    let mut file = archive
        .by_index(index)
        .map_err(|_| bad("Cannot read this ZIP entry"))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| bad("ZIP entry is corrupted or cannot be decompressed"))?;
    if bytes.len() as u64 > limit || bytes.len() as u64 != entry.size {
        return Err(bad("ZIP entry exceeds its declared size or preview limit"));
    }
    let content = match entry.preview {
        PreviewKind::Text => {
            let text = String::from_utf8(bytes).map_err(|_| bad("File is not UTF-8 text"))?;
            if text
                .chars()
                .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
            {
                return Err(bad(
                    "File contains binary data and cannot be previewed as text",
                ));
            }
            text
        }
        PreviewKind::Image => thumbnail(bytes)?,
        PreviewKind::None => unreachable!(),
    };
    Ok(AssetPreview {
        kind: entry.preview,
        content,
    })
}

fn open(path: &Path) -> Result<ZipArchive<File>> {
    let file = File::open(path).map_err(internal)?;
    if file.metadata().map_err(internal)?.len() > 32 * 1024 * 1024 {
        return Err(bad("Asset bundle exceeds the inspection limit of 32 MiB"));
    }
    ZipArchive::new(file).map_err(|_| bad("Cannot inspect bundle: invalid ZIP archive"))
}

fn entries(archive: &mut ZipArchive<File>) -> Result<Vec<AssetFile>> {
    if archive.len() > MAX_ENTRIES {
        return Err(bad(
            "Asset bundle exceeds the inspection limit of 4096 entries",
        ));
    }
    let mut files = Vec::new();
    let mut names = HashSet::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let file = archive
            .by_index_raw(index)
            .map_err(|_| bad("Invalid ZIP entry"))?;
        let name = file.name().map_err(|_| bad("Invalid ZIP entry name"))?;
        let is_directory = file.is_dir();
        let normalized = if is_directory {
            name.trim_end_matches('/')
        } else {
            name.as_ref()
        };
        if !safe_path(normalized)
            || file
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
            || !names.insert(name.to_string())
        {
            return Err(bad("Asset bundle contains an unsafe or duplicate path"));
        }
        total = total
            .checked_add(file.size())
            .ok_or_else(|| bad("Invalid ZIP entry size"))?;
        if total > MAX_EXPANDED_BYTES {
            return Err(bad(
                "Asset bundle exceeds the expanded size limit of 64 MiB",
            ));
        }
        if !is_directory {
            files.push(AssetFile {
                index,
                path: name.to_string(),
                size: file.size(),
                compressed_size: file.compressed_size(),
                preview: preview_kind(&name, file.size()),
            });
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn preview_kind(path: &str, size: u64) -> PreviewKind {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" if size <= MAX_IMAGE_BYTES => PreviewKind::Image,
        "txt" | "json" | "yaml" | "yml" | "xml" | "csv" | "md" | "svg" | "css"
            if size <= MAX_TEXT_BYTES =>
        {
            PreviewKind::Text
        }
        _ => PreviewKind::None,
    }
}

fn thumbnail(bytes: Vec<u8>) -> Result<String> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| bad("Cannot identify image format"))?;
    if !matches!(
        reader.format(),
        Some(ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP)
    ) {
        return Err(bad("Image format is not supported for preview"));
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| bad("Image is invalid or exceeds the 2048 × 2048 preview limit"))?;
    let thumbnail = if image.width() > 768 || image.height() > 768 {
        image.thumbnail(768, 768)
    } else {
        image
    };
    let mut output = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut output, ImageFormat::Png)
        .map_err(internal)?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(output.into_inner())
    ))
}
