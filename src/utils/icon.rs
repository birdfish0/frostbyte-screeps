//! Includes functions and macros for handling [Icon]s.

use macroquad::miniquad::conf::Icon;

/// Converts byte arrays containing info from `.bmp` files into a byte array for use in the [Icon] struct.
///
/// In the implementation, it strips the first 70 bytes and reorders the data structure from BGRA to RGBA.
fn fill_argb_array_from_bitmap(arr: &mut [u8], bitmap: &[u8]) {
    let mut bigiter = bitmap.split_at(70).1.iter();
    let mut i = 0;
    while let Some(v) = bigiter.next() {
        arr[i + 2] = *v;
        arr[i + 1] = *bigiter.next().unwrap_or(&0);
        arr[i] = *bigiter.next().unwrap_or(&0);
        arr[i + 3] = *bigiter.next().unwrap_or(&0);
        i += 4;
    }
}

/// Create an [Icon] object from three byte arrays containing info from `.bmp` files, 64x64, 32x32, and 16x16 in size respectively.
///
/// Color data is expected to begin on byte 71 exactly, in the format of B, G, R, A, 4 bytes per pixel.
pub fn icon_from(bigfile: &[u8], medfile: &[u8], smallfile: &[u8]) -> Icon {
    let mut big = [0; 16384];
    fill_argb_array_from_bitmap(&mut big, bigfile);

    let mut medium = [0; 4096];
    fill_argb_array_from_bitmap(&mut medium, medfile);

    let mut small = [0; 1024];
    fill_argb_array_from_bitmap(&mut small, smallfile);

    macroquad::miniquad::conf::Icon { big, medium, small }
}

#[macro_export]
/// Create the [Icon] object from the paths to three `.bmp` files, 64x64, 32x32, and 16x16 in size respectively.
///
/// Color data is expected to begin on byte 71 exactly, in the format of B, G, R, A, 4 bytes per pixel.
///
/// Equivalent to a call to [`icon_from`] using `include_bytes!` around the provided filenames.
macro_rules! icon_from {
    ($bigfile:literal, $medfile: literal, $smallfile: literal) => {
        $crate::utils::icon::icon_from(
            include_bytes!($bigfile),
            include_bytes!($medfile),
            include_bytes!($smallfile),
        )
    };
}
