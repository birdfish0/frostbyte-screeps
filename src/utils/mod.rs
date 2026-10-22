pub mod recursive_avg;

use macroquad::miniquad::conf::Icon;
use rand::{Rng, RngExt};

fn fill_argb_array_from_bitmap(arr: &mut [u8], bitmap: &[u8]) {
    let mut bigiter = bitmap.split_at(70).1.into_iter();
    let mut i = 0;
    while let Some(v) = bigiter.next() {
        arr[i+2] = *v;
        arr[i+1] = *bigiter.next().unwrap_or(&0);
        arr[i] = *bigiter.next().unwrap_or(&0);
        arr[i+3] = *bigiter.next().unwrap_or(&0);
        i += 4;
    }
}

pub fn icon_from(bigfile: &[u8], medfile: &[u8], smallfile: &[u8]) -> Icon {
    let mut big = [0; 16384];
    fill_argb_array_from_bitmap(&mut big, &bigfile);

    let mut medium = [0; 4096];
    fill_argb_array_from_bitmap(&mut medium, &medfile);

    let mut small = [0; 1024];
    fill_argb_array_from_bitmap(&mut small, &smallfile);

    macroquad::miniquad::conf::Icon {
        big,
        medium,
        small
    }
}

#[macro_export]
macro_rules! icon_from {
    ($bigfile:literal, $medfile: literal, $smallfile: literal) => {
        crate::utils::icon_from(include_bytes!($bigfile), include_bytes!($medfile), include_bytes!($smallfile))
    };
}

/// Choose a random value from `vals` and return it.
/// The random choice is weighted by the corresponding number in `weights`.
/// Weights can be any number, so long as the sum of all weights is not 0.
/// 
/// # Panics
/// 
/// Panics if sum of all weights is 0.
/// 
/// Panics if the quantity of weights is not equal to the quantity of values provided.
pub fn random_choice<T, R>(random:&mut R, vals: &[T], weights: &[u32]) -> T where R: Rng, T: Copy {
    if vals.len() != weights.len() {
        panic!("Number of values should equal number of weights.");
    }
    let total_weight: u32 = weights.iter().sum();
    if total_weight == 0 {
        panic!("Sum of all weights must not equal zero.");
    }
    let mut now_weight = 0;
    let mut iterweight = weights.iter();
    let mut iterval = vals.iter();
    while let Some(w) = iterweight.next() {
        if let Some(v) = iterval.next() {
            now_weight += w;
            if random.random_ratio(now_weight, total_weight) {
                return *v;
            }
        }
    }
    unreachable!("random_ratio should always return true when numerator and denominator are equal.");
}
