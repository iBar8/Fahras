// SPDX-License-Identifier: MIT

#![allow(unsafe_code)]

use std::path::Path;

use windows::{
    Win32::{
        Foundation::SIZE,
        Graphics::Gdi::{
            BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS,
            DeleteDC, DeleteObject, GetDIBits, GetObjectW, HBITMAP, HGDIOBJ,
        },
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx},
        UI::Shell::{
            IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF, SIIGBF_ICONONLY,
            SIIGBF_RESIZETOFIT,
        },
    },
    core::HSTRING,
};

use super::{Image, Kind, opaque_if_alpha_missing};

/// Sets up COM for the calling thread; call it once on each thread that reads images.
pub fn initialize_thread() {
    // A thread that already set it up only has its count raised.
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
}

/// The icon or thumbnail Explorer shows for `path`, `size` pixels on its
/// longer side.
pub fn image(path: &Path, size: u32, kind: Kind) -> Result<Image, String> {
    let factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None) }
            .map_err(|error| error.to_string())?;
    let flags: SIIGBF = match kind {
        Kind::Icon => SIIGBF_ICONONLY,
        Kind::ThumbnailOrIcon => SIIGBF_RESIZETOFIT,
    };
    let side = size as i32;
    let bitmap = unsafe { factory.GetImage(SIZE { cx: side, cy: side }, flags) }
        .map_err(|error| error.to_string())?;
    let pixels = read_bitmap(bitmap);
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
    pixels
}

fn read_bitmap(bitmap: HBITMAP) -> Result<Image, String> {
    let mut header = BITMAP::default();
    let read = unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0),
            size_of::<BITMAP>() as i32,
            Some(std::ptr::from_mut(&mut header).cast()),
        )
    };
    if read == 0 || header.bmWidth <= 0 || header.bmHeight == 0 {
        return Err("the shell returned no bitmap".to_owned());
    }
    let (width, height) = (header.bmWidth as u32, header.bmHeight.unsigned_abs());
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            // Negative: rows from the top.
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0_u8; (width * height * 4) as usize];
    let context = unsafe { CreateCompatibleDC(None) };
    let lines = unsafe {
        GetDIBits(
            context,
            bitmap,
            0,
            height,
            Some(bgra.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    let _ = unsafe { DeleteDC(context) };
    if lines != height as i32 {
        return Err("could not read the shell's bitmap".to_owned());
    }
    opaque_if_alpha_missing(&mut bgra);
    Ok(Image {
        width,
        height,
        bgra,
    })
}
