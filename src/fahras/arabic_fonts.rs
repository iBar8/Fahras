// SPDX-License-Identifier: MIT

use std::{
    io,
    path::{Path, PathBuf},
};

pub(crate) struct ArabicFont {
    pub(crate) family: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

macro_rules! font_files {
    ($($name:literal),+) => {
        &[$(($name, include_bytes!(concat!("../../data/fahras/fonts/arabic/", $name)))),+]
    };
}

pub(crate) const DEFAULT_FAMILY: &str = "IBM Plex Sans Arabic";

/// Text drawn in each family by the font picker; it is sample glyphs, not UI copy.
pub(crate) const SAMPLE: &str = "فهرس — ملفاتك في مكان واحد";

pub(crate) static FONTS: &[ArabicFont] = &[
    ArabicFont {
        family: "IBM Plex Sans Arabic",
        files: font_files!(
            "IBMPlexSansArabic-Regular.ttf",
            "IBMPlexSansArabic-Bold.ttf"
        ),
    },
    ArabicFont {
        family: "Noto Sans Arabic",
        files: font_files!("NotoSansArabic[wdth,wght].ttf"),
    },
    ArabicFont {
        family: "Noto Naskh Arabic",
        files: font_files!("NotoNaskhArabic[wght].ttf"),
    },
    ArabicFont {
        family: "Noto Kufi Arabic",
        files: font_files!("NotoKufiArabic[wght].ttf"),
    },
    ArabicFont {
        family: "Cairo",
        files: font_files!("Cairo[slnt,wght].ttf"),
    },
    ArabicFont {
        family: "Tajawal",
        files: font_files!("Tajawal-Regular.ttf", "Tajawal-Bold.ttf"),
    },
    ArabicFont {
        family: "Almarai",
        files: font_files!("Almarai-Regular.ttf", "Almarai-Bold.ttf"),
    },
    ArabicFont {
        family: "Readex Pro",
        files: font_files!("ReadexPro[HEXP,wght].ttf"),
    },
    ArabicFont {
        family: "Vazirmatn",
        files: font_files!("Vazirmatn[wght].ttf"),
    },
    ArabicFont {
        family: "Rubik",
        files: font_files!("Rubik[wght].ttf"),
    },
    ArabicFont {
        family: "Alexandria",
        files: font_files!("Alexandria[wght].ttf"),
    },
    ArabicFont {
        family: "Mada",
        files: font_files!("Mada[wght].ttf"),
    },
    ArabicFont {
        family: "Changa",
        files: font_files!("Changa[wght].ttf"),
    },
    ArabicFont {
        family: "El Messiri",
        files: font_files!("ElMessiri[wght].ttf"),
    },
    ArabicFont {
        family: "Amiri",
        files: font_files!("Amiri-Regular.ttf", "Amiri-Bold.ttf"),
    },
    ArabicFont {
        family: "Markazi Text",
        files: font_files!("MarkaziText[wght].ttf"),
    },
    ArabicFont {
        family: "Reem Kufi",
        files: font_files!("ReemKufi[wght].ttf"),
    },
    ArabicFont {
        family: "Baloo Bhaijaan 2",
        files: font_files!("BalooBhaijaan2[wght].ttf"),
    },
    ArabicFont {
        family: "Zain",
        files: font_files!("Zain-Regular.ttf", "Zain-Bold.ttf"),
    },
];

/// A saved family that is no longer bundled falls back to the default.
pub(crate) fn resolved_family(choice: Option<&str>) -> &'static str {
    choice
        .and_then(|choice| FONTS.iter().find(|font| font.family == choice))
        .map_or(DEFAULT_FAMILY, |font| font.family)
}

/// Writes every bundled Arabic font into `directory` and returns their paths for registration.
pub(crate) fn materialize(
    directory: &Path,
    write: impl Fn(&Path, &[u8]) -> io::Result<()>,
) -> io::Result<Vec<PathBuf>> {
    let directory = directory.join("arabic");
    std::fs::create_dir_all(&directory)?;
    FONTS
        .iter()
        .flat_map(|font| font.files)
        .map(|(name, contents)| {
            let path = directory.join(name);
            write(&path, contents).map(|()| path)
        })
        .collect()
}

#[cfg(test)]
mod tests;
