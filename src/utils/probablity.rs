#![allow(dead_code)]

use super::random::Random;

/// This struct will be used to perform all the probablity related operations
pub struct Probablity(pub Random);

impl Probablity {

    pub fn new(rng: Random) -> Self {
        Self(rng)
    }

    /// Return a random probablity (no. between 0 and 1)
    pub fn prob(&mut self) -> f64 {
        self.0.rand_in_range(0, 11) as f64 / 10.0
    }

    /// Returns true if the given probablity is satisfied and false otherwise
    pub fn probablity(&mut self, prob: f64) -> bool {
        if prob == 1.0 {
            return true;
        }

        if self.prob() < prob {
            true
        } else {
            false
        }
    }

    /// If the provided probablity is satisfied, then call the `true_func` else
    /// call `false_func` if it is passed
    pub fn with_probablity<F: FnMut(), F1: FnMut()>(&mut self, prob: f64,
                           mut true_func: F, false_func: Option<F1> ) {
        if self.probablity(prob) {
            true_func();
        } else if let Some(mut false_func) = false_func {
            false_func();
        }

    }

    /// Select and call a function from the provided list giving equal
    /// probablity to all the provided functions
    pub fn with_equal_probablity(&mut self, funcs: &[fn()]) {

        let idx = self.0.rand_in_range(0, funcs.len() as isize);
        funcs[idx as usize]();
    }

    /// Select an element from the input (Array/Vec) with the probablity of
    /// selecting each consecutive element being `factor` times the probablity
    /// of the previous. Note that if the value of factor is between 0 and 1
    /// then the priority will be given to the elements in the start else it
    /// will be given to the ones in the end.
    pub fn choose_biased<'a, T, U>(&mut self, array: &'a T, factor: f64) -> &'a U
        where T: AsRef<[U]> {

        let array = array.as_ref();
        let len = array.len();
        if len == 0 || len == 1 {
            return &array[0];
        }

        let mut x = 0.0;
        for i in 0..len {
            x += factor.powi(i as i32);
        }

        for i in (0..len).rev() {

            let weight = factor.powi(i as i32) as f64;
            let prob   = weight * (1.0/x);

            if self.probablity(prob) {
                return &array[i];
            } else {
                x -= weight;
            }
        }

        &array[0]
    }

    /// Select an element based on the weigths that are provided. The inputs is
    /// an array of tuples where the first tuple member is the element and the
    /// second one is the corresponding weight of the element
    pub fn choose_weighted_baised<'a, T>(&mut self,
                                         d: &'a [(T, u16)]) -> &'a T {

        let mut total: u32 = 0;

        for (_, w) in d {
            total += *w as u32;
        }

        for pair in d {
            let prob = pair.1 as f64 * (1.0/total as f64);
            if self.probablity(prob) {
                return &pair.0;
            } else {
                total -= pair.1 as u32;
            };
        }

        assert!(total == 0, "Unbalanced total");

        let idx = self.0.rand_idx(d.len());

        &d[idx].0

    }
}
