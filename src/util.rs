use std::{ops::{AddAssign, Div, SubAssign}};

use macroquad::miniquad::conf::Icon;

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
        crate::util::icon_from(include_bytes!($bigfile), include_bytes!($medfile), include_bytes!($smallfile))
    };
}

pub struct RecursiveVals
{
    pub lists: Vec<Vec<u128>>,
    pub progressions: Vec<usize>,
    pub sums: Vec<u128>,
    pub num_els: Vec<usize>,
    pub depth: usize,
    pub virtual_size: u128,
}

impl RecursiveVals
{
    pub fn new(num_els: Vec<usize>) -> Self {
        let depth = num_els.len();
        let mut virtual_size: u128 = 1;
        for n in &num_els {
            virtual_size *= *n as u128;
        }
        RecursiveVals {
            lists: vec![Vec::new(); depth],
            progressions: vec![0; depth],
            sums: vec![0; depth],
            num_els,
            depth,
            virtual_size,
        }
    }

    pub fn add(&mut self, value: u128, depth: usize) {
        if depth >= self.depth {
            return;
        }
        self.sums[depth] -= self.lists[depth][self.progressions[depth]];
        self.lists[depth][self.progressions[depth]] = value;
        self.sums[depth] += value;
        self.progressions[depth] += 1;
        if self.progressions[depth] >= self.num_els[depth] {
            self.progressions[depth] = 0;
            self.add(self.sums[depth], depth + 1);
        }
    }

    pub fn get(&self) -> u128 {
        return self.sums[self.depth - 1] / self.virtual_size;
    }
}
