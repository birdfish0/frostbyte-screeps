//! Provides functions related to random number generation.

use anyhow::{Error, bail, ensure};
use rand::{Rng, RngExt};

/// Choose a random value from `vals` and return it.
/// The random choice is weighted by the corresponding number in `weights`.
/// Weights can be any number, so long as the sum of all weights is not 0.
///
/// # Errors
///
/// Returns error if sum of all weights is 0.
///
/// Returns error if the quantity of weights is not equal to the quantity of values provided.
///
/// Returns error if all choices are exhausted and no element has been chosen.
/// (This should be impossble, but is covered anyway just in case.)
pub fn random_choice<T, R>(random: &mut R, vals: &[T], weights: &[u32]) -> Result<T, Error>
where
    R: Rng,
    T: Copy,
{
    ensure!(
        vals.len() == weights.len(),
        "Number of values should equal number of weights. (currently {} and {})",
        vals.len(),
        weights.len()
    );
    let total_weight: u32 = weights.iter().sum();
    ensure!(
        total_weight != 0,
        "Sum of all weights must not equal zero. (was provided with {} weights)",
        weights.len()
    );
    let mut now_weight = 0;
    let iterweight = weights.iter();
    let mut iterval = vals.iter();
    for w in iterweight {
        let v = iterval.next().unwrap(); // Safe unwrap, length confirmed before
        now_weight += w;
        if random.random_ratio(now_weight, total_weight) {
            return Ok(*v);
        }
    }
    bail!("random_ratio should always return true when numerator and denominator are equal.");
}
