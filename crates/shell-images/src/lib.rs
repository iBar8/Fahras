// SPDX-License-Identifier: MIT

//! Windows' own file icons and thumbnails, read through the shell the way
//! File Explorer reads them.

/// Pixels in rows from the top, four bytes each: blue, green, red and alpha,
/// with the colors premultiplied by alpha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Whether a picture or video may show its contents or only its type's icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The icon Explorer shows in Details view.
    Icon,
    /// A thumbnail where the file has one, as in Large icons view, else its icon.
    ThumbnailOrIcon,
}

#[cfg(windows)]
mod windows_shell;
#[cfg(windows)]
pub use windows_shell::{image, initialize_thread};

/// Fills in alpha for bitmaps that carry none, which Windows returns for
/// photos: every pixel then reads as fully transparent.
pub fn opaque_if_alpha_missing(bgra: &mut [u8]) {
    if bgra.chunks_exact(4).all(|pixel| pixel[3] == 0) {
        for pixel in bgra.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photos_without_alpha_become_opaque_and_icons_keep_theirs() {
        let mut photo = vec![10, 20, 30, 0, 40, 50, 60, 0];
        opaque_if_alpha_missing(&mut photo);
        assert_eq!(photo, [10, 20, 30, 255, 40, 50, 60, 255]);
        let mut icon = vec![0, 0, 0, 0, 40, 50, 60, 128];
        opaque_if_alpha_missing(&mut icon);
        assert_eq!(icon, [0, 0, 0, 0, 40, 50, 60, 128]);
    }
}
