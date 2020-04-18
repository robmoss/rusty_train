//! Generate permutations of, e.g., trains to allocate to paths.

use std::collections::HashSet;

/// Iterate over *k*-permutations of a set of size *n*, for a fixed *k*.
///
/// This is an implementation of the "Simple, Efficient P(n, k) Algorithm"
/// described by
/// [Alistair Israel](https://alistairisrael.wordpress.com/2009/09/22/simple-efficient-pnk-algorithm/)
/// and published in his
/// [JCombinatorics](https://github.com/aisrael/jcombinatorics) Java library
/// under the MIT License.
pub struct KPermutations {
    n: usize,
    k: usize,
    a: Vec<usize>,
    edge: usize,
    first: bool,
}

impl KPermutations {
    /// Create an iterator over *k*-permutations of a set.
    pub fn new(n: usize, k: usize) -> Self {
        let a: Vec<_> = (0..n).collect();
        KPermutations {
            n,
            k,
            a,
            edge: k - 1,
            first: true,
        }
    }
}

impl Iterator for KPermutations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        // NOTE: all 1-permutations can be obtained by returning each element
        // of a[] in turn.
        if self.k == 1 {
            return self.a.pop().map(|ix| vec![ix]);
        }

        // The first permutation should be returned before making any swaps.
        if self.first {
            self.first = false;
            return Some(self.a[0..self.k].to_vec());
        }

        let mut j = self.k;
        while j < self.n && self.a[self.edge] >= self.a[j] {
            j += 1;
        }
        if j < self.n {
            self.a.swap(self.edge, j);
        } else {
            // Reverse a[k] to a[n-1].
            if self.k < (self.n + 2) {
                let num_items = (self.n - self.k) / 2;
                for ix in 0..num_items {
                    self.a.swap(self.k + ix, self.n - ix - 1)
                }
            }

            // Find the right-most ascent to the left of edge.
            let mut i = self.edge - 1;
            while self.a[i] >= self.a[i + 1] {
                if i == 0 {
                    return None;
                }
                i -= 1;
            }

            // Find j in (n-1 ... i+1) where a[j] > a[i].
            j = self.n - 1;
            while j > i && self.a[i] >= self.a[j] {
                j -= 1;
            }

            self.a.swap(i, j);

            // Reverse a[i+1] to a[n-1].
            if (i + 1) < (self.n + 2) {
                let num_items = (self.n - i - 1) / 2;
                for ix in 0..num_items {
                    self.a.swap(i + 1 + ix, self.n - ix - 1)
                }
            }
        }

        Some(self.a[0..self.k].to_vec())
    }
}

/// Iterate over *k*-permutations of a set of size *n* where each element is
/// associated with a "class", returning only those permutations that produce
/// unique orderings of these classes.
pub struct KPermutationsFilter {
    /// The mapping of indices to classes.
    classes: Vec<usize>,
    /// The class permutations that have been yielded already.
    yielded: HashSet<Vec<usize>>,
    /// The underlying permutations iterator.
    perms: KPermutations,
}

impl KPermutationsFilter {
    /// Create an iterator over *k*-permutations of a set where each element
    /// is associated with a "class".
    pub fn new(classes: Vec<usize>, k: usize) -> Self {
        let n = classes.len();
        KPermutationsFilter {
            classes: classes,
            yielded: HashSet::new(),
            perms: KPermutations::new(n, k),
        }
    }

    /// Check whether a permutation duplicates the class ordering of a
    /// previous permutation.
    fn unique_class_permutation(&mut self, item: &Vec<usize>) -> bool {
        let class_perm: Vec<_> =
            item.iter().map(|ix| self.classes[*ix]).collect();
        // NOTE: insert() returns true if the set *did* *not* already contain
        // the inserted value.
        self.yielded.insert(class_perm)
    }
}

impl Iterator for KPermutationsFilter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.perms.next();
            match item {
                None => return None,
                Some(ref ixs) => {
                    if self.unique_class_permutation(ixs) {
                        return item;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KPermutations, KPermutationsFilter};
    use log::info;

    fn init() {
        let _ = env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("info"),
        )
        .is_test(true)
        .try_init();
    }

    #[test]
    /// Check that there is 1 *1*-permutation for a set of size 1.
    fn test_kpermutations_1_1() {
        init();
        let perm = KPermutations::new(1, 1);
        let perms: Vec<_> = perm.collect();
        let expected_count = 1;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }

    #[test]
    /// Check that there are 5 *1*-permutations for a set of size 5.
    fn test_kpermutations_5_1() {
        init();
        let perm = KPermutations::new(5, 1);
        let perms: Vec<_> = perm.collect();
        let expected_count = 5;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }

    #[test]
    /// Check that there are 20 *2*-permutations for a set of size 5.
    fn test_kpermutations_5_2() {
        init();
        let perm = KPermutations::new(5, 2);
        let perms: Vec<_> = perm.collect();
        let expected_count = 20;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }

    #[test]
    /// Check that there are 2 *2*-permutations for a set of size 2.
    fn test_kpermutations_2_2() {
        init();
        let perm = KPermutations::new(2, 2);
        let perms: Vec<_> = perm.collect();
        let expected_count = 2;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }

    #[test]
    /// Check that there are 4 *2*-permutations for a set of size 5 with only
    /// 2 distinct classes.
    fn test_kpermutationsfilter_5_2_2() {
        init();
        let classes = vec![0, 0, 1, 1, 1];
        let perm = KPermutationsFilter::new(classes, 2);
        let perms: Vec<_> = perm.collect();
        let expected_count = 4;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }

    #[test]
    /// Check that there is 1 *2*-permutations for a set of size 2 with only
    /// 1 distinct class.
    fn test_kpermutationsfilter_2_1_2() {
        init();
        let classes = vec![0, 0];
        let perm = KPermutationsFilter::new(classes, 2);
        let perms: Vec<_> = perm.collect();
        let expected_count = 1;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }

    #[test]
    /// Check that there is 2 *2*-permutations for a set of size 2 with
    /// 2 distinct classes.
    fn test_kpermutationsfilter_2_2_2() {
        init();
        let classes = vec![0, 1];
        let perm = KPermutationsFilter::new(classes, 2);
        let perms: Vec<_> = perm.collect();
        let expected_count = 2;
        assert_eq!(expected_count, perms.len());
        for p in &perms {
            info!("Received {:?}", p);
        }
    }
}
