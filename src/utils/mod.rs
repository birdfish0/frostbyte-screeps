//! Provides general utility functions for use in the engine. See the documentation of specific submodules for more information.

pub mod event_queue;
pub mod icon;
pub mod recursive_avg;

use rand::{Rng, RngExt};

/// Choose a random value from `vals` and return it.
/// The random choice is weighted by the corresponding number in `weights`.
/// Weights can be any number, so long as the sum of all weights is not 0.
///
/// # Panics
///
/// Panics if sum of all weights is 0.
///
/// Panics if the quantity of weights is not equal to the quantity of values provided.
pub fn random_choice<T, R>(random: &mut R, vals: &[T], weights: &[u32]) -> T
where
    R: Rng,
    T: Copy,
{
    if vals.len() != weights.len() {
        panic!("Number of values should equal number of weights.");
    }
    let total_weight: u32 = weights.iter().sum();
    if total_weight == 0 {
        panic!("Sum of all weights must not equal zero.");
    }
    let mut now_weight = 0;
    let iterweight = weights.iter();
    let mut iterval = vals.iter();
    for w in iterweight {
        let v = iterval.next().unwrap(); // Safe unwrap, length confirmed before
        now_weight += w;
        if random.random_ratio(now_weight, total_weight) {
            return *v;
        }
    }
    unreachable!(
        "random_ratio should always return true when numerator and denominator are equal."
    );
}
