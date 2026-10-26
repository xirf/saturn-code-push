use super::*;
use std::io::Write;
use zip::{ZipWriter, write::SimpleFileOptions};

fn archive(entries: &[(&str, &[u8])]) -> tempfile::NamedTempFile {
    let mut output = tempfile::NamedTempFile::new().unwrap();
    let mut zip = ZipWriter::new(output.as_file_mut());
    for (name, bytes) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    output
}

#[test]
fn lists_files_and_previews_text_without_interpreting_markup() {
    let content = b"<script>alert('plain text')</script>";
    let zip = archive(&[
        ("assets/readme.txt", content),
        ("AssetManifest.bin", b"manifest"),
    ]);
    let listing = list(zip.path()).unwrap();
    assert_eq!(listing.files.len(), 2);
    assert_eq!(listing.files[0].path, "AssetManifest.bin");
    assert_eq!(listing.expanded_size, content.len() as u64 + 8);
    let text = listing
        .files
        .iter()
        .find(|file| file.path.ends_with(".txt"))
        .unwrap();
    let result = preview(zip.path(), text.index).unwrap();
    assert_eq!(result.content.as_bytes(), content);
    assert!(preview(zip.path(), listing.files[0].index).is_err());
    assert!(preview(zip.path(), 999).is_err());
}

#[test]
fn rejects_unsafe_paths_and_malformed_archives() {
    for path in [
        "../secret.txt",
        "/absolute.txt",
        "assets/../secret.txt",
        "assets\\secret.txt",
        "C:secret.txt",
    ] {
        let zip = archive(&[(path, b"unsafe")]);
        assert!(list(zip.path()).is_err(), "Accepted unsafe path: {path}");
    }
    let mut invalid = tempfile::NamedTempFile::new().unwrap();
    invalid.write_all(b"not a zip").unwrap();
    assert!(list(invalid.path()).is_err());
}

#[test]
fn rejects_binary_text_and_large_previews() {
    let large = vec![b'x'; MAX_TEXT_BYTES as usize + 1];
    let zip = archive(&[("large.txt", &large), ("binary.txt", b"a\0b")]);
    let listing = list(zip.path()).unwrap();
    for file in listing.files {
        assert!(preview(zip.path(), file.index).is_err());
    }
}

#[test]
fn bounds_archive_entry_count_and_expanded_size() {
    let mut output = tempfile::NamedTempFile::new().unwrap();
    let mut zip = ZipWriter::new(output.as_file_mut());
    for index in 0..=MAX_ENTRIES {
        zip.start_file(format!("{index}.txt"), SimpleFileOptions::default())
            .unwrap();
    }
    zip.finish().unwrap();
    assert!(list(output.path()).is_err());

    // Highly compressible input must still respect the expanded size limit.
    let mut output = tempfile::NamedTempFile::new().unwrap();
    let mut zip = ZipWriter::new(output.as_file_mut());
    zip.start_file(
        "large.bin",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    let chunk = [0_u8; 65536];
    for _ in 0..=MAX_EXPANDED_BYTES / chunk.len() as u64 {
        zip.write_all(&chunk).unwrap();
    }
    zip.finish().unwrap();
    assert!(list(output.path()).is_err());
}

#[test]
fn images_are_decoded_and_reencoded_as_bounded_png_thumbnails() {
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(12, 8)
        .write_to(&mut png, ImageFormat::Png)
        .unwrap();
    let zip = archive(&[("assets/image.png", png.get_ref())]);
    let result = preview(zip.path(), 0).unwrap();
    let decoded = STANDARD
        .decode(
            result
                .content
                .strip_prefix("data:image/png;base64,")
                .unwrap(),
        )
        .unwrap();
    let image = image::load_from_memory(&decoded).unwrap();
    assert_eq!((image.width(), image.height()), (12, 8));

    let zip = archive(&[("assets/fake.png", b"<svg onload='alert(1)' />")]);
    assert!(preview(zip.path(), 0).is_err());

    let mut large = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(2049, 1)
        .write_to(&mut large, ImageFormat::Png)
        .unwrap();
    let zip = archive(&[("assets/large.png", large.get_ref())]);
    assert!(preview(zip.path(), 0).is_err());
}
