//! Generate combinations of, e.g., paths for trains to operate.
use log::info;

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
///
/// Note that filtering is done on a pairwise-comparison basis, to check for
/// pairs of values that should be excluded.
/// This means that all *1*-combinations (0..n-1, inclusive) will be returned.
/// By providing the filtering function as part of this iterator, instead of
/// calling `iter.filter()`, this prunes *k*-combinations where, e.g., the
/// first two values should be excluded.
pub struct CombinationsFilter<F: Fn(usize, usize) -> bool> {
    item_count: usize,
    max_len: usize,
    items: Vec<usize>,
    current_ix: usize,
    item_filter: F,
    filter_calls: usize,
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
            filter_calls: 0,
        }
    }
}

/// Implement a parallel iterator for filtered subsets of *k*-combinations.
///
/// While this is conceptually simple — divide the interval `[0, n-1]` for
/// the first value in each *k*-combination into multiple sub-intervals, with
/// a maximal split of `[0]`, `[1]`, ..., `[n-1]` — the implementation is
/// somewhat complicated by the need to implement `ParallelIterator`,
/// `UnindexedProducer`, and `Iterator`, and by the fact that the
/// `UnindexedProducer` values need a reference to the filtering function.
///
/// We define `CombFilt` to own the filtering function and implement
/// `ParallelIterator`.
/// In the `drive_unindexed()` method it creates a `CombFiltProducer` and
/// feeds this producer to the consumer.
///
/// `CombFiltProducer` implements `UnindexedProducer`, which allows it to
/// split into multiple producers, and also implements `Iterator`, so that it
/// can yield each valid *k*-combination and feed them to the folder.
pub mod par {
    pub struct CombFilt<F: Fn(usize, usize) -> bool + Send + Sync> {
        item_count: usize,
        max_len: usize,
        current_ix: usize,
        ix0_max: usize,
        item_filter: F,
    }

    impl<F: Fn(usize, usize) -> bool + Send + Sync>
        rayon::iter::IntoParallelIterator for super::CombinationsFilter<F>
    {
        type Iter = CombFilt<F>;
        type Item = Vec<usize>;

        fn into_par_iter(self) -> Self::Iter {
            if self.current_ix != 0 {
                panic!("CombinationsFilter.current_ix != 0")
            } else if !self.items.is_empty() {
                panic!("CombinationsFilter.items is not empty")
            }
            CombFilt {
                item_count: self.item_count,
                max_len: self.max_len,
                current_ix: self.current_ix,
                item_filter: self.item_filter,
                ix0_max: self.item_count,
            }
        }
    }

    pub struct CombFiltProducer<'a, F: Fn(usize, usize) -> bool + Send + Sync> {
        item_count: usize,
        max_len: usize,
        items: Vec<usize>,
        current_ix: usize,
        ix0_max: usize,
        item_filter: &'a F,
    }

    impl<F: Fn(usize, usize) -> bool + Send + Sync>
        rayon::iter::ParallelIterator for CombFilt<F>
    {
        type Item = Vec<usize>;

        fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where
            C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
        {
            let producer = CombFiltProducer {
                item_count: self.item_count,
                max_len: self.max_len,
                items: vec![],
                current_ix: self.current_ix,
                ix0_max: self.ix0_max,
                item_filter: &self.item_filter,
            };
            rayon::iter::plumbing::bridge_unindexed(producer, consumer)
        }
    }

    impl<'a, F: Fn(usize, usize) -> bool + Send + Sync> CombFiltProducer<'a, F> {
        /// Determine where to split the interval `[a, b]` of values for the
        /// first element of each *k*-combination, such that each partition
        /// will contain about the same number of *k*-combinations.
        /// This helps ensure that the parallel iterator can distribute the
        /// work as evenly as possible.
        ///
        /// We need to consider the distribution of *k*-combinations, which is
        /// `n!/[k! (n - k)!]`, for all `1 <= k <= k_max`.
        ///
        /// For `k_max = 1`, we can simply split at `n/2`.
        ///
        /// For `k_max = 2`, we have `(n + n^2) / 2` combinations.
        /// I think the split should then be at `sqrt([n^2 + n] / 2)`?
        /// But plotting (see R code below) suggests `n/sqrt(2)` is better?
        ///
        /// ```R
        /// xs <- 10:100
        /// f <- function(n) { n + n * (n - 1) / 2 }
        /// g <- function(n) { sqrt((n^2 + n) / 2) }
        /// library(ggplot2)
        /// qplot(f(xs), f(g(xs)) / f(xs))
        /// g <- function(n) { n / sqrt(2) }
        /// f3 <- function(n) { n + n * (n-1) / 2 + n * (n-1) * (n-2) / 6 }
        /// qplot(f3(xs), f3(round(xs / 2^(1/3))) / f3(xs))
        /// ```
        ///
        pub fn split_at(&self) -> Option<usize> {
            // We cannot split this interval if there's only a single valid
            // value for the first element.
            let too_narrow = self.ix0_max - self.current_ix <= 1;
            // Perhaps we can split this if we clone self.items?
            let not_empty = !self.items.is_empty();
            if too_narrow || not_empty {
                None
            } else {
                // NOTE: the second half starts at the value split_at.
                let split_at =
                    self.current_ix + (self.ix0_max - self.current_ix) / 2;
                Some(split_at)
            }
        }
    }

    impl<'a, F: Fn(usize, usize) -> bool + Send + Sync> Iterator
        for CombFiltProducer<'a, F>
    {
        type Item = Vec<usize>;

        fn next(&mut self) -> Option<Self::Item> {
            let ix_ub = if self.items.is_empty() {
                // Use a different upper limit for the first element.
                self.ix0_max
            } else {
                self.item_count
            };
            while self.current_ix < ix_ub {
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
                None
            }
        }
    }

    impl<'a, F: Fn(usize, usize) -> bool + Send + Sync>
        rayon::iter::plumbing::UnindexedProducer for CombFiltProducer<'a, F>
    {
        type Item = Vec<usize>;

        fn split(self) -> (Self, Option<Self>) {
            if let Some(split_at) = self.split_at() {
                let low = Self {
                    item_count: self.item_count,
                    max_len: self.max_len,
                    items: vec![],
                    current_ix: self.current_ix,
                    item_filter: self.item_filter,
                    ix0_max: split_at,
                };
                let high = Self {
                    item_count: self.item_count,
                    max_len: self.max_len,
                    items: vec![],
                    current_ix: split_at,
                    item_filter: self.item_filter,
                    ix0_max: self.ix0_max,
                };
                (low, Some(high))
            } else {
                (self, None)
            }
        }

        fn fold_with<G>(mut self, folder: G) -> G
        where
            G: rayon::iter::plumbing::Folder<Self::Item>,
        {
            let mut folder = folder;
            for val in &mut self {
                folder = folder.consume(val);
            }
            folder
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::CombinationsFilter;
        use super::CombFilt;
        use log::info;
        use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
        /// - 10 x *2*-combinations: 5! / (2! * 3!) = 20 / 2 = 10; and;
        /// - 10 x *3*-combinations: 5! / (3! * 2!) = 10.
        fn test_par_comb_filter_1() {
            init();
            let comb = CombinationsFilter::new(5, 3, |_x, _y| false);
            let combs: Vec<_> = comb.collect();
            let expected_count = 5 + 10 + 10;
            assert_eq!(expected_count, combs.len());
            info!("CombinationsFilter returned:");
            for c in &combs {
                info!("    {:?}", c);
            }
            info!("");
            let pcomb: CombFilt<_> =
                CombinationsFilter::new(5, 3, |_x, _y| false).into_par_iter();
            let pcombs: Vec<_> = pcomb.collect();
            info!("Parallel returned:");
            for c in &pcombs {
                info!("    {:?}", c);
            }
            info!("");
            assert_eq!(expected_count, pcombs.len());
            assert_eq!(combs, pcombs)
        }

        #[test]
        /// Check that filtering the 25 *{1,2,3}*-combinations for a set of
        /// size 5 returns the same results when using the serial and parallel
        /// iterators.
        fn test_par_comb_filter_2() {
            init();
            let comb = CombinationsFilter::new(5, 3, |x, y| x == 2 || y == 2);
            let combs: Vec<_> = comb.collect();
            let expected_count = 15;
            assert_eq!(expected_count, combs.len());
            info!("CombinationsFilter returned:");
            for c in &combs {
                info!("    {:?}", c);
            }
            info!("");
            let pcomb: CombFilt<_> =
                CombinationsFilter::new(5, 3, |x, y| x == 2 || y == 2)
                    .into_par_iter();
            let pcombs: Vec<_> = pcomb.collect();
            info!("Parallel returned:");
            for c in &pcombs {
                info!("    {:?}", c);
            }
            info!("");
            assert_eq!(expected_count, pcombs.len());
            assert_eq!(combs, pcombs)
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
            self.filter_calls += 1;
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
            info!("Made {} calls to filter fn", self.filter_calls);
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
            } else if *k == 2 || *k == 3 {
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
