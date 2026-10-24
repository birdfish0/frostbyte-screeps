//! Contains the [RecursiveAvg] struct and its implementation.

use anyhow::ensure;
use log::{error, info};

/// Uses multiple lists to keep track of the rolling average over a very large window size without storing the entire dataset.
///
/// Note: The following explanations assume a depth of greater than 1.
/// This struct should not be used in cases where `depth = 1` and should be replaced with a normal rolling average struct.
///
/// If the number of data points added is less than the size of the rolling window,
/// the average (with imprecision, see [`get`][Self::get]) of all the data points is taken.
///
/// ## Principle
///
/// Consider two lists l0 and l1 of some maximum size.
///
/// Each element of a dataset is pushed\* to l0 until l0 is filled.
/// Then, the average of l0 is taken and pushed\* to l1, and l0 is cleared.
///
/// Repeat until l1 is filled. Then, the average\*\* of l1 is equal to all the data points within the rolling window.
/// The process can then continue by overwriting the oldest values of l1, as in a rolling average.
///
/// (\*In the actual implementation, the lists are kept at a fixed size and their values replaced one by one until the entire list has been replaced.)
///
/// (\*\*Average values affected by integer division logic.)
///
/// This can be repeated with more lists l0...ln to further expand the window of the average average.
///
/// The smaller the size of the preceding lists, the more frequently the final average value will update.
///
/// NOTE: Depending on the [`sum_mode`][RecursiveAvg::sum_mode] value, l1 may contain a sum of the values of l0 instead of the average, to preserve precision.
/// See the docs for [`sum_mode`][RecursiveAvg::sum_mode] for more information.
///
/// ## Vocabulary
///
/// "Depth" refers to `num_els.len()` and `lists.len()` (they are equal). It is the number of lists used to track the data.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RecursiveAvg {
    /// The lists that keep track of the datasets. `lists[0]` contains the input values,
    /// `lists[1]` contains the averages\* of `lists[0]`, and so on. (\*see note above)
    pub lists: Vec<Vec<u128>>,
    /// Keeps track of the current index to overwrite in each list. Private.
    progressions: Vec<usize>,
    /// Keeps track of the current sum of every list, in order to make taking the average of each list faster. Private.
    sums: Vec<u128>,
    /// Stores the number of elements each list should hold, provided at creation. `num_els[n] == lists[n].len()` for all `0 <= n < lists.len()`.
    /// Private. Please re-create the struct if you wish to modify this value.
    num_els: Vec<usize>,
    /// The size of the rolling average window. In `sum_mode = true`, the final sum is divided by this value to get the average.
    /// Private. Please re-create the struct if you wish to modify this value.
    /// It is calculated using `virtual_size = num_els[0] * num_els[1] * ... * num_els[num_els.len() - 1]`.
    virtual_size: u128,
    /// Keeps track of the number of data points added before enough data points are added to fill the rolling window.
    ///
    /// Used in place of [`virtual_size`][Self::virtual_size] in the calculation of the average value when `sum_mode = true`.
    current_window_size: u128,
    /// The index (`lists[lowest_filled_list]`) of the list with the lowest index that has had an element added to it.
    ///
    /// This is the list that will be used in average calculations, and should be 0 once the rolling window has been filled
    /// enough to place a data point into `lists[0]`.
    lowest_filled_list: usize,
    /// The degree to which `lists[lowest_filled_list]` has been filled.
    ///
    /// When calculating an average with `sum_mode = false`, the first `lowest_filled_list_amount`
    /// elements of `lists[lowest_filled_list]` are averaged.
    ///
    /// Not used when `sum_mode = true`.
    lowest_filled_list_amount: usize,
    /// Determines how the average is calculated, and the values added to `lists[1]` and on.
    /// When `sum_mode` is true, `lists[1]` contains the sum of `lists[0]` values.
    /// This improves the accuracy of the final result, but means that
    /// the sum of all data points in the rolling window will be held within one [`u128`] (`sums[depth - 1]`).\
    /// When `sum_mode` is false, `lists[1]` holds the average values of `lists[0]`, as described in Principle above.\
    /// `sum_mode` should be disabled for datasets where the sum of all datapoints in the rolling window might exceed [`u128::MAX`].\
    /// Note that `sums[depth]` still holds the sum of all values in `lists[depth]`, as it is required in order to calculate the average of `lists[depth]`.
    /// If this would exceed [`u128::MAX`], consider decreasing the number provided in `num_els` and increasing the depth.
    pub sum_mode: bool,
}

impl RecursiveAvg {
    /// Constructor.
    ///
    /// ## Arguments
    ///
    /// - `num_els`: See the docs of the struct on [`num_els`][RecursiveAvg::num_els] for more information.
    /// - `sum_mode`: See the docs of the struct on [`sum_mode`][RecursiveAvg::sum_mode] for more information.
    ///
    /// ## Errors
    ///
    /// Returns error if `num_els.len() == 0`.
    ///
    /// Returns error if `num_els` contains a 0.
    pub fn new(num_els: Vec<usize>, sum_mode: bool) -> Result<Self, anyhow::Error> {
        ensure!(!num_els.is_empty(), "num_els must not be empty.");
        ensure!(
            !num_els.contains(&0),
            "num_els must not contain a 0. (num_els: {:?})",
            num_els
        );

        let depth = num_els.len();
        let mut virtual_size: u128 = 1;
        for n in &num_els {
            virtual_size *= *n as u128;
        }

        let mut lists = Vec::with_capacity(depth);
        for item in num_els.iter() {
            lists.push(vec![0; *item]);
        }

        Ok(RecursiveAvg {
            lists,
            progressions: vec![0; depth],
            current_window_size: 0,
            lowest_filled_list: depth - 1,
            lowest_filled_list_amount: 0,
            sums: vec![0; depth],
            num_els,
            virtual_size,
            sum_mode,
        })
    }

    /// Add a value to the dataset and increment [`Self::current_window_size`] if it is not equal to [`Self::virtual_size`].
    pub fn add(&mut self, value: u128) {
        if self.current_window_size < self.virtual_size {
            self.current_window_size += 1;
        }
        self.add_at_depth(value, self.lists.len() - 1);
    }

    /// Add a value to the list at depth `depth` and increment the corresponding index, as well as update
    /// [`Self::lowest_filled_list`] and [`Self::lowest_filled_list_amount`] accordingly.
    ///
    /// This should only be used with `depth = 0`, except internally to add the average (or sum) of each list to the next list.
    ///
    /// Returns when `depth` is greater than the depth of the struct.
    fn add_at_depth(&mut self, value: u128, depth: usize) {
        if depth >= self.lists.len() {
            return;
        }
        self.sums[depth] -= self.lists[depth][self.progressions[depth]];
        self.lists[depth][self.progressions[depth]] = value;
        self.sums[depth] += value;
        self.progressions[depth] += 1;

        if self.lowest_filled_list > depth {
            self.lowest_filled_list = depth;
            self.lowest_filled_list_amount = 0;
        }
        if self.lowest_filled_list == depth && self.lowest_filled_list_amount < self.num_els[depth]
        {
            self.lowest_filled_list_amount += 1;
        }

        if self.progressions[depth] >= self.num_els[depth] {
            self.progressions[depth] = 0;
            if depth != 0 {
                if self.sum_mode {
                    self.add_at_depth(self.sums[depth], depth - 1);
                } else {
                    self.add_at_depth(self.sums[depth] / self.num_els[depth] as u128, depth - 1);
                }
            }
        }
    }

    /// Get the average of the values of the dataset.
    ///
    /// This value is affected by integer division precision,
    /// and may be less precise than excpected in `sum_mode = false` (see docs on `sum_mode` for the struct).
    ///
    /// Returns 0 if no data points have been added.
    pub fn get(&self) -> u128 {
        if self.sum_mode {
            self.sums[self.lowest_filled_list]
                .checked_div(self.current_window_size)
                .unwrap_or(0)
        } else {
            self.sums[self.lowest_filled_list]
                .checked_div(self.lowest_filled_list_amount as u128)
                .unwrap_or(0)
        }
    }
}

impl Default for RecursiveAvg {
    fn default() -> Self {
        Self::new(vec![10, 10], false).unwrap() // Safe unwrap
    }
}
