//! Generate combinations of, e.g., paths for trains to operate.

/// Iterate over *k*-combinations of a set of size *n*, for all *k* up to some
/// limit *k_max*.
pub struct Combinations {
    item_count: usize,
    max_len: usize,
    items: Vec<usize>,
    current_ix: usize,
}

impl Combinations {
    /// Create an iterator over *k*-combinations of a set of size *n*, for all
    /// *k* up to the limit *k_max*.
    pub fn new(n: usize, k_max: usize) -> Self {
        Combinations {
            item_count: n,
            max_len: k_max,
            items: Vec::with_capacity(k_max),
            current_ix: 0,
        }
    }
}

impl Iterator for Combinations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ix >= self.item_count {
            // Reached the end of nesting, pop.
            if let Some(prev_ix) = self.items.pop() {
                // Move to the next sibling, and rely on recursive searching.
                self.current_ix = prev_ix + 1;
                self.next()
            } else {
                // Have iterated over all possible combinations.
                None
            }
        } else {
            self.items.push(self.current_ix);
            let item = Some(self.items.clone());
            if self.items.len() < self.max_len {
                // Prepare to descend, starting at the smallest value that
                // hasn't already been included in the current combination.
                self.current_ix = self.items.iter().max().unwrap() + 1;
            } else {
                // Prepare to move to the next sibling.
                self.items.pop();
                self.current_ix += 1;
            }
            item
        }
    }
}

/// Iterate over *k*-combinations of a set of size *n*, for all *k* up to some
/// limit *k_max*, filtering out combinations that meet some criteria.
pub struct CombinationsFilter<F: Fn(usize, usize) -> bool> {
    item_count: usize,
    max_len: usize,
    items: Vec<usize>,
    current_ix: usize,
    item_filter: F,
}

impl<F: Fn(usize, usize) -> bool> CombinationsFilter<F> {
    /// Create an iterator over *k*-combinations of a set of size *n*, for all
    /// *k* up to the limit *k_max*, filtering out combinations for which
    /// `ignore` returns `true` for any pair of elements.
    pub fn new(n: usize, k_max: usize, ignore: F) -> Self {
        CombinationsFilter {
            item_count: n,
            max_len: k_max,
            items: Vec::with_capacity(k_max),
            current_ix: 0,
            item_filter: ignore,
        }
    }
}

impl<F: Fn(usize, usize) -> bool> Iterator for CombinationsFilter<F> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_ix < self.item_count {
            // NOTE: if we pass self.items in one go, the filter can also
            // prune combinations that no combination of trains is capable of
            // operating.
            if self
                .items
                .iter()
                .any(|x| (self.item_filter)(*x, self.current_ix))
            {
                // NOTE: this efficiently prunes all sub-branches of the
                // depth-first search.
                self.current_ix += 1;
                continue;
            }
            self.items.push(self.current_ix);
            let item = Some(self.items.clone());
            if self.items.len() < self.max_len {
                // Prepare to descend, starting at the smallest value that
                // hasn't already been included in the current combination.
                self.current_ix = self.items.iter().max().unwrap() + 1;
            } else {
                // Prepare to move to the next sibling.
                self.items.pop();
                self.current_ix += 1;
            }
            return item;
        }

        // Reached the end of nesting, pop.
        if let Some(prev_ix) = self.items.pop() {
            // Move to the next sibling, and rely on recursive searching.
            self.current_ix = prev_ix + 1;
            self.next()
        } else {
            // Have iterated over all possible combinations.
            None
        }
    }
}

/// Iterate over *k*-combinations of a set of size *n*.
pub struct KCombinations {
    item_count: usize,
    want_len: usize,
    items: Vec<usize>,
    current_ix: usize,
}

impl KCombinations {
    /// Create an iterator over *k*-combinations of a set of size *n*.
    pub fn new(n: usize, k: usize) -> Self {
        KCombinations {
            item_count: n,
            want_len: k,
            items: Vec::with_capacity(k),
            current_ix: 0,
        }
    }
}

impl Iterator for KCombinations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ix >= self.item_count {
            // Reached the end of nesting, pop.
            if let Some(prev_ix) = self.items.pop() {
                // Move to the next sibling, and rely on recursive searching.
                self.current_ix = prev_ix + 1;
                self.next()
            } else {
                // Have iterated over all possible combinations.
                None
            }
        } else {
            self.items.push(self.current_ix);
            if self.items.len() < self.want_len {
                // Prepare to descend, starting at the smallest value that
                // hasn't already been included in the current combination.
                self.current_ix = self.items.iter().max().unwrap() + 1;
                self.next()
            } else {
                let item = Some(self.items.clone());
                // Prepare to move to the next sibling.
                self.items.pop();
                self.current_ix += 1;
                item
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Combinations, CombinationsFilter, KCombinations};
    use log::info;

    fn init() {
        let _ = env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("info"),
        )
        .is_test(true)
        .try_init();
    }

    #[test]
    /// Check that there are 25 *{1,2,3}*-combinations for a set of size 5:
    ///
    /// - 5 x *1*-combinations (0..4);
    /// - 10 x *2*-combinations: 5! / (2! * 3!) = 20 / 2 = 10; and
    /// - 10 x *3*-combinations: 5! / (3! * 2!) = 10.
    fn test_combinations_1() {
        init();
        let comb = Combinations::new(5, 3);
        let combs: Vec<_> = comb.collect();
        let expected_count = 5 + 10 + 10;
        assert_eq!(expected_count, combs.len());
        for c in &combs {
            info!("{:?}", c);
        }
    }

    #[test]
    /// Check that there are 18 *{1,2,3}*-combinations for a set of size 5
    /// where no element *i* in a combination is double the value of any
    /// element *j*.
    ///
    /// Of the 25 *{1,2,3}*-combinations, 7 should be ignored:
    ///
    /// - *2*-combinations ``[1 2]``, ``[2 4]``.
    /// - *3*-combinations ``[0 1 2]``, ``[0 2 4]``, ``[1 2 3]``, ``[1 2 4]``,
    ///   ``[2 3 4]``.
    fn test_combinations_filter_1() {
        init();
        let filter = Box::new(|i, j| j == (2 * i));
        let comb = CombinationsFilter::new(5, 3, filter);
        let combs: Vec<_> = comb.collect();
        let expected_count = 5 + 10 + 10 - 7;
        assert_eq!(expected_count, combs.len());
        for c in &combs {
            info!("{:?}", c);
        }
    }

    #[test]
    /// Check that for a set of size 5 there are:
    ///
    /// - 5 x *1*-combinations (0..4);
    /// - 10 x *2*-combinations: 5! / (2! * 3!) = 20 / 2 = 10; and
    /// - 10 x *3*-combinations: 5! / (3! * 2!) = 10.
    fn test_kcombinations_1() {
        init();
        let ks: [usize; 3] = [1, 2, 3];
        for k in &ks {
            let comb = KCombinations::new(5, *k);
            let combs: Vec<_> = comb.collect();
            if *k == 1 {
                assert_eq!(5, combs.len())
            } else if *k == 2 {
                assert_eq!(10, combs.len())
            } else if *k == 3 {
                assert_eq!(10, combs.len())
            } else {
                unreachable!("k should be in [1, 2, 2]")
            }
            for comb in &combs {
                assert_eq!(comb.len(), *k)
            }
        }
    }
}
