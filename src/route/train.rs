//! Train types and revenue earned for operating routes.
//!
//! # Example
//!
//! ```rust
//! # use rusty_train::prelude::*;
//! // Create a map; this one has 6 rows and 14 columns.
//! let hex = Hex::new(125.0);
//! let tiles = tile_catalogue(&hex);
//! let num_rows: usize = 6;
//! let num_cols: usize = 14;
//! let addrs: Vec<(usize, usize)> = (0..num_rows)
//!     .map(|r| (0..num_cols).map(move |c| (r, c)))
//!     .flatten()
//!     .collect();
//! let hexes: Vec<HexAddress> =
//!     addrs.iter().map(|coords| coords.into()).collect();
//! let mut game_map = Map::new(tiles, hexes);
//! // NOTE: place tiles and tokens, or load an existing map configuration.
//!
//! // Define the collection of trains owned by a company.
//! let trains = vec![
//!     Train::new_8_train(),
//!     Train::new_8_train(),
//!     Train::new_5p5e_train(),
//! ];
//! let mut trains = Trains::new(trains);
//!
//! // Determine the search criteria for this collection of trains.
//! let path_limit = trains.path_limit();
//! let criteria = Criteria {
//!     token: Token::LP,
//!     path_limit: path_limit,
//!     conflict_rule: ConflictRule::TrackOrCityHex,
//!     route_conflict_rule: ConflictRule::TrackOnly,
//! };
//!
//! // Find all paths for which at least one of the company's trains can run.
//! let paths = paths_for_token(&game_map, &criteria);
//!
//! // Find the pairing of trains to paths that earns the most revenue.
//! let best_routes = trains.select_routes(paths);
//! if let Some(pairing) = &best_routes {
//!     println!("Net revenue is ${}", pairing.net_revenue);
//! }
//! # // NOTE: the map is empty, so there will be no paths.
//! # assert!(best_routes.is_none());
//! ```

use super::search::PathLimit;
use super::Path;
use log::info;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/// The types of trains that can operate routes to earn revenue.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Train {
    max_stops: Option<usize>,
    can_skip_dits: bool,
    can_skip_cities: bool,
    revenue_multiplier: usize,
}

impl Default for Train {
    fn default() -> Self {
        Train {
            max_stops: Some(2),
            can_skip_dits: true,
            can_skip_cities: false,
            revenue_multiplier: 1,
        }
    }
}

impl Train {
    pub fn describe(&self) -> String {
        if self.can_skip_dits
            && !self.can_skip_cities
            && self.revenue_multiplier == 1
        {
            format!("{}-train", self.max_stops.unwrap())
        } else if self.can_skip_dits
            && self.can_skip_cities
            && self.revenue_multiplier == 2
            && self.max_stops.is_some()
        {
            format!(
                "{}+{}E-train",
                self.max_stops.as_ref().unwrap(),
                self.max_stops.as_ref().unwrap()
            )
        } else {
            "".to_string()
        }
    }

    fn new_n_train(n: usize) -> Self {
        Train {
            max_stops: Some(n),
            ..Default::default()
        }
    }

    pub fn new_2_train() -> Self {
        Self::new_n_train(2)
    }

    pub fn new_3_train() -> Self {
        Self::new_n_train(3)
    }

    pub fn new_4_train() -> Self {
        Self::new_n_train(4)
    }

    pub fn new_5_train() -> Self {
        Self::new_n_train(5)
    }

    pub fn new_6_train() -> Self {
        Self::new_n_train(6)
    }

    pub fn new_7_train() -> Self {
        Self::new_n_train(7)
    }

    pub fn new_8_train() -> Self {
        Self::new_n_train(8)
    }

    pub fn new_2p2_train() -> Self {
        Train {
            max_stops: Some(2),
            can_skip_dits: true,
            can_skip_cities: false,
            revenue_multiplier: 2,
        }
    }

    pub fn new_5p5e_train() -> Self {
        Train {
            max_stops: Some(5),
            can_skip_dits: true,
            can_skip_cities: true,
            revenue_multiplier: 2,
        }
    }

    pub fn revenue_for(&self, path: &Path) -> Option<usize> {
        if let Some(max_stops) = self.max_stops {
            if self.can_skip_cities && self.can_skip_dits {
                // NOTE: must include first and last stops.
                let rev_first = path.visits.first().unwrap().revenue;
                let rev_last = path.visits.last().unwrap().revenue;
                let mut rev_rest = path
                    .visits
                    .iter()
                    .skip(1)
                    .take(path.visits.len() - 2)
                    .map(|v| v.revenue)
                    .collect::<Vec<usize>>();
                rev_rest.sort();
                rev_rest.reverse();
                let rev_rest: usize =
                    rev_rest.iter().take(max_stops - 2).sum();
                let path_revenue = rev_first + rev_last + rev_rest;
                return Some(path_revenue * self.revenue_multiplier);
            } else if self.can_skip_dits {
                if path.num_cities <= max_stops {
                    let cities =
                        path.visits.iter().filter(|v| v.visits.is_city());
                    let dits =
                        path.visits.iter().filter(|v| v.visits.is_dit());
                    let city_revenue: usize = cities.map(|v| v.revenue).sum();
                    let mut dit_revenues =
                        dits.map(|v| v.revenue).collect::<Vec<usize>>();
                    dit_revenues.sort();
                    dit_revenues.reverse();
                    let dit_revenue: usize = dit_revenues
                        .iter()
                        .take(max_stops - path.num_cities)
                        .sum();
                    let path_revenue = city_revenue + dit_revenue;
                    return Some(path_revenue * self.revenue_multiplier);
                } else {
                    // NOTE: too many cities, cannot stop at them all.
                    return None;
                }
            } else if self.can_skip_cities {
                if path.num_dits <= max_stops {
                    let cities =
                        path.visits.iter().filter(|v| v.visits.is_city());
                    let dits =
                        path.visits.iter().filter(|v| v.visits.is_dit());
                    let dit_revenue: usize = dits.map(|v| v.revenue).sum();
                    let mut city_revenues =
                        cities.map(|v| v.revenue).collect::<Vec<usize>>();
                    city_revenues.sort();
                    city_revenues.reverse();
                    let city_revenue: usize = city_revenues
                        .iter()
                        .take(max_stops - path.num_dits)
                        .sum();
                    let path_revenue = city_revenue + dit_revenue;
                    return Some(path_revenue * self.revenue_multiplier);
                } else {
                    // NOTE: too many dits, cannot stop at them all.
                    return None;
                }
            } else {
                // NOTE: cannot skip dits or cities, must be able to stop at
                // every visit along the path.
                if path.num_visits <= max_stops {
                    return Some(path.revenue * self.revenue_multiplier);
                } else {
                    return None;
                }
            }
        } else {
            // NOTE: no limit on stops, so we can stop at every visit.
            return Some(path.revenue * self.revenue_multiplier);
        }
    }
}

/// Pairings of trains to routes.
pub struct Pairing {
    /// The total revenue earned from this pairing.
    pub net_revenue: usize,
    /// The routes that were operated and earned revenue.
    pub pairs: Vec<Pair>,
}

/// A train that operates a path to earn revenue.
///
/// Note that the train may not earn revenue from every location along the
/// path.
pub struct Pair {
    /// The train.
    pub train: Train,
    /// The path.
    pub path: Path,
    /// The revenue earned by having the train run the path.
    pub revenue: usize,
}

/// The trains owned by a single company, which may operate routes.
pub struct Trains {
    trains: HashMap<Train, usize>,
    train_vec: Vec<Train>,
}

impl From<Vec<Train>> for Trains {
    fn from(src: Vec<Train>) -> Self {
        let mut trains = HashMap::new();
        for train in &src {
            let count = trains.entry(*train).or_insert(0);
            *count += 1;
        }
        Trains {
            trains,
            train_vec: src,
        }
    }
}

impl FromIterator<Train> for Trains {
    fn from_iter<I: IntoIterator<Item = Train>>(iter: I) -> Self {
        let train_vec: Vec<Train> = iter.into_iter().collect();
        train_vec.into()
    }
}

impl Trains {
    /// Creates a new collection of trains.
    pub fn new(trains: Vec<Train>) -> Self {
        trains.into()
    }

    /// Returns the number of trains in this collection.
    pub fn train_count(&self) -> usize {
        self.trains.values().sum()
    }

    /// Returns the most restrictive path limit that respects the abilities of
    /// each train in this collection.
    pub fn path_limit(&self) -> Option<PathLimit> {
        let express = self.trains.keys().any(|t| {
            t.max_stops.is_none() || (t.can_skip_cities && t.can_skip_dits)
        });
        if express {
            return None;
        }

        // NOTE: so there is a maximum number of stops, and no train can skip
        // cities and dits. For now, ignore the possibility of trains that can
        // skip cities but cannot skip dits.
        let skip_dits = self.trains.keys().any(|t| t.can_skip_dits);
        let max_stops = self
            .trains
            .keys()
            .map(|t| t.max_stops.unwrap())
            .max()
            .unwrap();
        if skip_dits {
            Some(PathLimit::Cities { count: max_stops })
        } else {
            Some(PathLimit::CitiesAndTowns { count: max_stops })
        }
    }

    /// Returns a pairing of trains to routes that earns the most revenue.
    pub fn select_routes(&mut self, path_tbl: Vec<Path>) -> Option<Pairing> {
        let num_paths = path_tbl.len();
        let num_trains = self.train_count();

        // Build a table that maps each path (identified by index) to a
        // train-revenue table.
        let rev: HashMap<usize, HashMap<Train, usize>> = (0..num_paths)
            .map(|path_ix| {
                (
                    path_ix,
                    self.trains
                        .keys()
                        .filter_map(|train| {
                            train
                                .revenue_for(&path_tbl[path_ix])
                                .map(|revenue| (*train, revenue))
                        })
                        .collect(),
                )
            })
            .collect();

        // Record all pairs of paths that conflict, and which therefore cannot
        // be operated at the same time.
        // NOTE: paths are referred to by index into `path_tbl`. We record
        // conflicts for paths with indices `a` and `b` where `a` < `b`.
        let now = std::time::Instant::now();
        let path_tbl_ref = &path_tbl;
        let conflict_tbl: HashSet<(usize, usize)> = (0..num_paths)
            .flat_map(|a| {
                ((a + 1)..num_paths)
                    .filter(move |b| {
                        !path_tbl_ref[a]
                            .route_conflicts
                            .is_disjoint(&path_tbl_ref[*b].route_conflicts)
                    })
                    .map(move |b| (a, b))
            })
            .collect();
        info!(
            "There are {} conflicting pairs out of {} paths ({} pairs)",
            conflict_tbl.len(),
            num_paths,
            num_paths * (num_paths - 1) / 2,
        );
        info!("This took {}", now.elapsed().as_secs_f64());

        // Identify all non-conflicting path combinations, from a single path
        // to one path for each train.
        let now = std::time::Instant::now();
        let filter =
            CombinationsFilter::new(num_paths, num_trains, |a, b| {
                conflict_tbl.contains(&(a, b))
            });
        let combs: Vec<Vec<usize>> = filter.collect();
        info!(
            "Found {} {}-combinations in {}",
            combs.len(),
            num_trains,
            now.elapsed().as_secs_f64()
        );

        // Filter out path combinations that cannot be operated and find the
        // train-path pairing that yields the greatest revenue.
        let best_pairing: Option<(usize, Vec<(Train, usize, usize)>)> = combs
            .into_iter()
            .filter_map(|path_ixs| self.best_pairing_for(&rev, &path_ixs))
            .max_by_key(|&(revenue, _)| revenue);

        // Remove the paths from `path_tbl` and replace the path index in each
        // pairing with the corresponding path itself.
        let best_pairing = best_pairing.map(|(net_revenue, pairings)| {
            // Build a table that maps path indices to paths, retaining only
            // those paths that are paired with a train.
            let ixs: Vec<usize> = pairings.iter().map(|p| p.1).collect();
            let mut path_map: HashMap<usize, Path> = path_tbl
                .into_iter()
                .enumerate()
                .filter_map(|(ix, path)| {
                    if ixs.contains(&ix) {
                        Some((ix, path))
                    } else {
                        None
                    }
                })
                .collect();

            // Replace the path indices with the actual paths.
            let pairs = pairings
                .into_iter()
                .map(|(train, path_ix, revenue)| Pair {
                    train: train,
                    path: path_map.remove(&path_ix).unwrap(),
                    revenue: revenue,
                })
                .collect();

            Pairing { net_revenue, pairs }
        });

        info!("Found a best pairing? {}", best_pairing.is_some());

        best_pairing
    }

    fn best_pairing_for(
        &self,
        revenue: &HashMap<usize, HashMap<Train, usize>>,
        path_ixs: &Vec<usize>,
    ) -> Option<(usize, Vec<(Train, usize, usize)>)> {
        let num_paths = path_ixs.len();
        let num_trains = self.train_count();
        let train_combinations = Combinations::new(num_trains, num_paths);

        train_combinations
            .filter_map(|train_ixs| {
                let revenues: Vec<usize> = train_ixs
                    .iter()
                    .enumerate()
                    .filter_map(|(path_ixs_ix, train_ix)| {
                        revenue.get(&path_ixs[path_ixs_ix]).and_then(|tbl| {
                            tbl.get(&self.train_vec[*train_ix])
                                .map(|revenue| *revenue)
                        })
                    })
                    .collect();
                let net_revenue: usize = revenues.iter().sum();
                if revenues.len() < train_ixs.len() {
                    // Some trains could not operate the corresponding path.
                    None
                } else {
                    Some((
                        net_revenue,
                        train_ixs
                            .iter()
                            .enumerate()
                            .map(|(path_ixs_ix, train_ix)| {
                                (
                                    self.train_vec[*train_ix],
                                    path_ixs[path_ixs_ix],
                                    revenues[path_ixs_ix],
                                )
                            })
                            .collect(),
                    ))
                }
            })
            .max_by_key(|(rev, _)| *rev)
    }
}

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

#[cfg(test)]
mod tests {
    use super::{Combinations, CombinationsFilter};
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
}
