//! Train types and revenue earned for operating routes.
//!
//! # Example
//!
//! ```rust
//! # use rusty_hex::*;
//! # use rusty_tile::*;
//! # use rusty_map::*;
//! # use rusty_catalogue::tile_catalogue;
//! # use rusty_route::*;
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

use super::bonus::Bonus;
use super::comb::CombinationsFilter;
use super::perm::KPermutationsFilter;
use super::search::PathLimit;
use super::Path;
use log::info;
use rusty_map::HexAddress;
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

    /// Determine the revenue earned and stops made when the train operates
    /// the given path, if it can operate this path.
    ///
    /// The train must stop at the first and last visits, and the indices of
    /// the intermediate stops are returned.
    pub fn revenue_for(
        &self,
        path: &Path,
        bonuses: &HashMap<HexAddress, Bonus>,
    ) -> Option<(usize, Vec<usize>)> {
        // TODO: apply route bonuses!!!!
        //
        // Visit bonuses are simple:
        // Bonus::VisitBonus { locn: HexAddress, bonus: usize }
        //
        // Connection bonuses are difficult:
        // Bonus::ConnectionBonus { from: HexAddress, to_any: Vec<HexAddress>,
        //                          bonus: usize }

        if let Some(max_stops) = self.max_stops {
            if self.can_skip_cities && self.can_skip_dits {
                // NOTE: must include first and last stops.
                let rev_first = path.visits.first().unwrap().revenue;
                let rev_last = path.visits.last().unwrap().revenue;
                let mut rev_rest = path
                    .visits
                    .iter()
                    .enumerate()
                    .skip(1)
                    .take(path.visits.len() - 2)
                    .map(|(ix, v)| (ix, v.revenue))
                    .collect::<Vec<(usize, usize)>>();
                rev_rest.sort_by_key(|(_ix, v)| *v);
                rev_rest.reverse();
                let stops: Vec<_> =
                    rev_rest.iter().take(max_stops - 2).collect();
                let rev_rest: usize = stops.iter().map(|(_ix, v)| v).sum();
                let path_revenue = rev_first + rev_last + rev_rest;
                let stop_ixs: Vec<_> =
                    stops.iter().map(|(ix, _v)| *ix).collect();
                return Some((
                    path_revenue * self.revenue_multiplier,
                    stop_ixs,
                ));
            } else if self.can_skip_dits {
                if path.num_cities <= max_stops {
                    let mut num_dit_stops = max_stops - path.num_cities;
                    let first_visit = path.visits.first().unwrap();
                    let last_visit = path.visits.last().unwrap();
                    if first_visit.visits.is_dit() {
                        if num_dit_stops > 0 {
                            num_dit_stops -= 1;
                        } else {
                            return None;
                        }
                    }
                    if last_visit.visits.is_dit() {
                        if num_dit_stops > 0 {
                            num_dit_stops -= 1;
                        } else {
                            return None;
                        }
                    }
                    let rev_first = first_visit.revenue;
                    let rev_last = last_visit.revenue;
                    let cities: Vec<_> = path
                        .visits
                        .iter()
                        .enumerate()
                        .skip(1)
                        .take(path.visits.len() - 2)
                        .filter(|(_ix, v)| v.visits.is_city())
                        .collect();
                    let mut dits: Vec<_> = path
                        .visits
                        .iter()
                        .enumerate()
                        .skip(1)
                        .take(path.visits.len() - 2)
                        .filter(|(_ix, v)| v.visits.is_dit())
                        .map(|(ix, v)| (ix, v.revenue))
                        .collect();
                    let city_revenue: usize =
                        cities.iter().map(|(_ix, v)| v.revenue).sum();
                    dits.sort_by_key(|(_ix, v)| *v);
                    dits.reverse();
                    let dit_stops: Vec<_> =
                        dits.iter().take(num_dit_stops).collect();
                    let dit_revenue: usize =
                        dit_stops.iter().map(|(_ix, v)| *v).sum();
                    let stop_ixs: Vec<_> = dit_stops
                        .iter()
                        .map(|(ix, _)| *ix)
                        .chain(cities.iter().map(|(ix, _)| *ix))
                        .collect();
                    let path_revenue =
                        city_revenue + dit_revenue + rev_first + rev_last;
                    return Some((
                        path_revenue * self.revenue_multiplier,
                        stop_ixs,
                    ));
                } else {
                    // NOTE: too many cities, cannot stop at them all.
                    return None;
                }
            } else if self.can_skip_cities {
                if path.num_dits <= max_stops {
                    let mut num_city_stops = max_stops - path.num_dits;
                    let first_visit = path.visits.first().unwrap();
                    let last_visit = path.visits.last().unwrap();
                    if first_visit.visits.is_city() {
                        if num_city_stops > 0 {
                            num_city_stops -= 1;
                        } else {
                            return None;
                        }
                    }
                    if last_visit.visits.is_city() {
                        if num_city_stops > 0 {
                            num_city_stops -= 1;
                        } else {
                            return None;
                        }
                    }
                    let rev_first = first_visit.revenue;
                    let rev_last = last_visit.revenue;
                    let mut cities: Vec<_> = path
                        .visits
                        .iter()
                        .enumerate()
                        .skip(1)
                        .take(path.visits.len() - 2)
                        .filter(|(_ix, v)| v.visits.is_city())
                        .map(|(ix, v)| (ix, v.revenue))
                        .collect();
                    let dits: Vec<_> = path
                        .visits
                        .iter()
                        .enumerate()
                        .skip(1)
                        .take(path.visits.len() - 2)
                        .filter(|(_ix, v)| v.visits.is_dit())
                        .collect();
                    let dit_revenue: usize =
                        dits.iter().map(|(_ix, v)| v.revenue).sum();
                    cities.sort_by_key(|(_ix, v)| *v);
                    cities.reverse();
                    let city_stops: Vec<_> =
                        cities.iter().take(num_city_stops).collect();
                    let city_revenue: usize =
                        city_stops.iter().map(|(_ix, v)| *v).sum();
                    let stop_ixs: Vec<_> = city_stops
                        .iter()
                        .map(|(ix, _)| *ix)
                        .chain(dits.iter().map(|(ix, _)| *ix))
                        .collect();
                    let path_revenue =
                        city_revenue + dit_revenue + rev_first + rev_last;
                    return Some((
                        path_revenue * self.revenue_multiplier,
                        stop_ixs,
                    ));
                } else {
                    // NOTE: too many dits, cannot stop at them all.
                    return None;
                }
            } else {
                // NOTE: cannot skip dits or cities, must be able to stop at
                // every visit along the path.
                if path.num_visits <= max_stops {
                    let stop_ixs: Vec<_> = (0..(path.visits.len())).collect();
                    return Some((
                        path.revenue * self.revenue_multiplier,
                        stop_ixs,
                    ));
                } else {
                    return None;
                }
            }
        } else {
            // NOTE: no limit on stops, so we can stop at every visit.
            let stop_ixs: Vec<_> = (0..(path.visits.len())).collect();
            return Some((path.revenue * self.revenue_multiplier, stop_ixs));
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
    train_classes: Vec<usize>,
}

impl From<Vec<Train>> for Trains {
    fn from(src: Vec<Train>) -> Self {
        let mut trains = HashMap::new();
        let mut seen_trains = vec![];
        let mut train_classes = Vec::with_capacity(src.len());
        for train in &src {
            let count = trains.entry(*train).or_insert(0);
            *count += 1;
            let mut found = false;
            for ix in 0..seen_trains.len() {
                if seen_trains[ix] == train {
                    train_classes.push(ix);
                    found = true;
                    break;
                }
            }
            if !found {
                seen_trains.push(train);
                train_classes.push(seen_trains.len() - 1);
            }
        }
        Trains {
            trains,
            train_vec: src,
            train_classes,
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
    pub fn select_routes(
        &self,
        path_tbl: Vec<Path>,
        bonuses: Vec<Bonus>,
    ) -> Option<Pairing> {
        let num_paths = path_tbl.len();
        let num_trains = self.train_count();

        // Index each bonus by location.
        let bonuses: HashMap<HexAddress, Bonus> = bonuses
            .into_iter()
            .map(|b| match b {
                Bonus::VisitBonus { locn, .. } => (locn, b),
                Bonus::ConnectionBonus { from, .. } => (from, b),
            })
            .collect();

        // Build a table that maps each path (identified by index) to a
        // train-revenue table.
        let rev: HashMap<usize, HashMap<Train, (usize, Vec<usize>)>> = (0
            ..num_paths)
            .map(|path_ix| {
                (
                    path_ix,
                    self.trains
                        .keys()
                        .filter_map(|train| {
                            train
                                .revenue_for(&path_tbl[path_ix], &bonuses)
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
        let best_pairing: Option<(
            usize,
            Vec<(Train, usize, usize, Vec<usize>)>,
        )> = combs
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
                .map(|(train, path_ix, revenue, stop_ixs)| {
                    let mut path = path_map.remove(&path_ix).unwrap();
                    // Mark visit as a stop or not, by setting revenue to 0
                    // for skipped visits.
                    // NOTE: the first and last visit are always stopped at.
                    for ix in 1..(path.visits.len() - 1) {
                        if !stop_ixs.contains(&ix) {
                            path.visits[ix].revenue = 0;
                        }
                    }
                    Pair {
                        train: train,
                        path: path,
                        revenue: revenue,
                    }
                })
                .collect();

            Pairing { net_revenue, pairs }
        });

        info!("Found a best pairing? {}", best_pairing.is_some());

        best_pairing
    }

    fn best_pairing_for(
        &self,
        revenue: &HashMap<usize, HashMap<Train, (usize, Vec<usize>)>>,
        path_ixs: &Vec<usize>,
    ) -> Option<(usize, Vec<(Train, usize, usize, Vec<usize>)>)> {
        let num_paths = path_ixs.len();
        // NOTE: we only need to consider pairings that allocate a train to
        // each path, we can can ignore smaller combinations.
        // NOTE: we need train *permutations*, rather than combinations,
        // because the ordering matters. But we can also ignore permutations
        // that don't change the ordering of *train types*.
        let train_combinations =
            KPermutationsFilter::new(self.train_classes.clone(), num_paths);

        train_combinations
            .filter_map(|train_ixs| {
                let revenues: Vec<(usize, Vec<usize>)> = train_ixs
                    .iter()
                    .enumerate()
                    .filter_map(|(path_ixs_ix, train_ix)| {
                        revenue.get(&path_ixs[path_ixs_ix]).and_then(|tbl| {
                            tbl.get(&self.train_vec[*train_ix])
                                .map(|revenue| revenue.clone())
                        })
                    })
                    .collect();
                let net_revenue: usize =
                    revenues.iter().map(|(r, _)| r).sum();
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
                                let stop_ixs =
                                    revenues[path_ixs_ix].1.clone();
                                (
                                    self.train_vec[*train_ix],
                                    path_ixs[path_ixs_ix],
                                    revenues[path_ixs_ix].0,
                                    stop_ixs,
                                )
                            })
                            .collect(),
                    ))
                }
            })
            .max_by_key(|(rev, _)| *rev)
    }
}
