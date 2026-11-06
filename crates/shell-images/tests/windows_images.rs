// SPDX-License-Identifier: MIT

//! The shell's icons and thumbnails, read on Windows.

#![cfg(windows)]

use std::path::PathBuf;

use shell_images::{Kind, image, initialize_thread};

fn scratch(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("fahras-shell-images-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("scratch folder");
    folder.join(name)
}

/// A 64-pixel red square, as a PNG.
fn red_png() -> Vec<u8> {
    let mut raw = Vec::new();
    for _ in 0..64 {
        raw.push(0);
        for _ in 0..64 {
            raw.extend_from_slice(&[230, 20, 20]);
        }
    }
    png_bytes(64, 64, &raw)
}

fn crc(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);
    for byte in bytes {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// An uncompressed PNG: zlib "stored" blocks need no encoder.
fn png_bytes(width: u32, height: u32, raw: &[u8]) -> Vec<u8> {
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (index, block) in blocks.iter().enumerate() {
        zlib.push(u8::from(index + 1 == blocks.len()));
        let length = block.len() as u16;
        zlib.extend_from_slice(&length.to_le_bytes());
        zlib.extend_from_slice(&(!length).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler(raw).to_be_bytes());
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut chunk = |kind: &[u8], data: &[u8]| {
        png.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        png.extend_from_slice(&body);
        png.extend_from_slice(&crc(&body).to_be_bytes());
    };
    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &header);
    chunk(b"IDAT", &zlib);
    chunk(b"IEND", &[]);
    png
}

fn opaque_pixels(image: &shell_images::Image) -> usize {
    image
        .bgra
        .chunks_exact(4)
        .filter(|pixel| pixel[3] > 0)
        .count()
}

#[test]
fn a_text_file_gets_its_windows_icon() {
    initialize_thread();
    let path = scratch("notes.txt");
    std::fs::write(&path, b"hello").expect("file");
    let icon = image(&path, 32, Kind::Icon).expect("icon");
    assert_eq!((icon.width, icon.height), (32, 32));
    assert!(opaque_pixels(&icon) > 32, "the icon draws something");
}

#[test]
fn a_folder_gets_its_windows_icon() {
    initialize_thread();
    let path = scratch("album");
    std::fs::create_dir_all(&path).expect("folder");
    let icon = image(&path, 32, Kind::Icon).expect("icon");
    assert!(opaque_pixels(&icon) > 32);
}

#[test]
fn a_picture_shows_its_contents_as_a_thumbnail() {
    initialize_thread();
    let path = scratch("red.png");
    std::fs::write(&path, red_png()).expect("picture");
    let thumbnail = image(&path, 96, Kind::ThumbnailOrIcon).expect("thumbnail");
    let (width, height) = (thumbnail.width as usize, thumbnail.height as usize);
    let center = ((height / 2) * width + width / 2) * 4;
    let [blue, green, red, alpha] = thumbnail.bgra[center..center + 4] else {
        unreachable!()
    };
    assert!(
        red > 180 && green < 80 && blue < 80 && alpha == 255,
        "the middle is the picture's red, not an icon: {:?}",
        [blue, green, red, alpha]
    );
}
