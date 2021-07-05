//! Train types and revenue earned for operating routes.
//!
//! # Example
//!
//! ```rust
//! # use n18hex::*;
//! # use n18tile::*;
//! # use n18map::*;
//! # use n18catalogue::tile_catalogue;
//! # use n18route::*;
//! # use n18token::{Token, Tokens, TokenStyle};
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
//!
//! // Define the token colours and appearance for an example company.
//! let fg = (63, 153, 153).into();
//! let bg = (255, 127, 127).into();
//! let text = (0, 0, 0).into();
//! let company_token = Token::new(TokenStyle::SideArcs {fg, bg, text});
//! let tokens: Tokens = vec![("AB".to_string(), company_token)].into();
//!
//! // Create the game map.
//! let mut game_map = Map::new(tiles, tokens, hexes);
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
//!     token: company_token,
//!     path_limit: path_limit,
//!     conflict_rule: ConflictRule::TrackOrCityHex,
//!     route_conflict_rule: ConflictRule::TrackOnly,
//! };
//!
//! // Find all paths for which at least one of the company's trains can run.
//! let paths = paths_for_token(&game_map, &criteria);
//!
//! // Assume there are no relevant route bonuses.
//! let bonuses = vec![];
//!
//! // Find the pairing of trains to paths that earns the most revenue.
//! let best_routes = trains.select_routes(paths, bonuses);
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
use super::{Path, Step, Visit};
use log::info;
use n18map::HexAddress;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::FromIterator;

/// The types of trains that can operate routes to earn revenue.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Train {
    /// The constraints on the routes that the train can operate.
    pub train_type: TrainType,
    /// The maximum number of stops the train can make, if any.
    pub max_stops: Option<usize>,
    /// The multiplier that is applied to the base revenue for each stop.
    pub revenue_multiplier: usize,
}

/// The types of trains that can operate routes to earn revenue.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrainType {
    /// Cannot skip towns or cities.
    MustStop,
    /// Can skip towns but cannot skip cities.
    SkipTowns,
    /// Can skip towns or cities.
    SkipAny,
}

impl Default for Train {
    fn default() -> Self {
        Train {
            max_stops: Some(2),
            train_type: TrainType::SkipTowns,
            revenue_multiplier: 1,
        }
    }
}

/// Identify visits along a path where a train stops and earns revenue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrainStop {
    /// The index of the visit in the path.
    pub visit_ix: usize,
    /// The revenue earned by stopping at this location, including bonus
    /// revenue where relevant, but not including the effect of any revenue
    /// multiplier associated with the train itself.
    pub revenue: usize,
}

impl Train {
    fn new_n_train(n: usize) -> Self {
        Train {
            max_stops: Some(n),
            ..Default::default()
        }
    }

    /// Return true if this train can operate a route of arbitrary length, as
    /// a result of being able to (a) make an unlimited number of stops; or
    /// (b) skip any number of towns and cities.
    pub fn is_express(&self) -> bool {
        self.max_stops.is_none() || self.train_type == TrainType::SkipAny
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
            train_type: TrainType::SkipTowns,
            revenue_multiplier: 2,
        }
    }

    pub fn new_5p5e_train() -> Self {
        Train {
            max_stops: Some(5),
            train_type: TrainType::SkipAny,
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
        visit_bonuses: &HashMap<HexAddress, usize>,
        conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
    ) -> Option<(usize, Vec<TrainStop>)> {
        let (revenue, stops): (usize, Vec<TrainStop>) = match self.max_stops {
            // With no limit on stops, we can stop at every visit, and this
            // should earn more revenue than skipping any of the visits (if
            // possible).
            None => {
                let stop_ixs: Vec<usize> = (0..(path.visits.len())).collect();
                revenue_for_stops(
                    path,
                    self,
                    &stop_ixs,
                    visit_bonuses,
                    conn_bonuses,
                )
            }
            Some(max_stops) => {
                if path.num_visits <= max_stops {
                    // Can stop at every visit, and this should earn more
                    // revenue than skipping any of the visits (if possible).
                    let stop_ixs: Vec<usize> =
                        (0..(path.visits.len())).collect();
                    revenue_for_stops(
                        path,
                        self,
                        &stop_ixs,
                        visit_bonuses,
                        conn_bonuses,
                    )
                } else {
                    // Must be able to skip some of the visits.
                    let final_ix = path.visits.len() - 1;
                    let can_skip: Vec<bool> = match self.train_type {
                        TrainType::MustStop => {
                            return None;
                        }
                        TrainType::SkipTowns => path
                            .visits
                            .iter()
                            .enumerate()
                            .map(|(ix, visit)| {
                                {
                                    ix > 0
                                        && ix < final_ix
                                        && visit.visits.is_dit()
                                }
                            })
                            .collect(),
                        TrainType::SkipAny => path
                            .visits
                            .iter()
                            .enumerate()
                            .map(|(ix, _visit)| ix > 0 && ix < final_ix)
                            .collect(),
                    };

                    // Check that enough visits can be skipped that the train
                    // is capable of operating this route.
                    let num_skip: usize =
                        can_skip.iter().map(|b| *b as usize).sum();
                    if path.visits.len() > (max_stops + num_skip) {
                        return None;
                    }

                    // Return the stops that earn the most revenue.
                    best_stop_ixs(
                        path,
                        self,
                        visit_bonuses,
                        conn_bonuses,
                        can_skip,
                        max_stops,
                    )
                }
            }
        };
        return Some((revenue, stops));
    }
}

/// Calculate the revenue bonus for stopping at a location.
fn visit_bonus(
    addr: &HexAddress,
    visit_bonuses: &HashMap<HexAddress, usize>,
) -> usize {
    visit_bonuses.get(addr).map(|b| *b).unwrap_or(0)
}

/// Return true if the selected path stops include at least one of the
/// provided locations.
fn stops_at_any(
    path: &Path,
    stop_ixs: &[usize],
    dests: &Vec<HexAddress>,
) -> bool {
    dests.iter().any(|addr| {
        // NOTE: the train must stop at one of the connecting locations.
        path.visits
            .iter()
            .enumerate()
            .any(|(ix, v)| stop_ixs.contains(&ix) && v.addr == *addr)
    })
}

/// Calculate the revenue bonus for connecting one location to another.
fn connection_bonus(
    addr: &HexAddress,
    path: &Path,
    stop_ixs: &[usize],
    conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
) -> usize {
    conn_bonuses
        .get(addr)
        .map(|(dests, bonus)| {
            if stops_at_any(path, stop_ixs, dests) {
                *bonus
            } else {
                0
            }
        })
        .unwrap_or(0)
}

fn revenue_for_stop(
    path: &Path,
    stop_ixs: &[usize],
    ix: usize,
    visit_bonuses: &HashMap<HexAddress, usize>,
    conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
) -> usize {
    let base_revenue: usize = path.visits[ix].revenue;
    let addr = path.visits[ix].addr;
    let visit = visit_bonus(&addr, visit_bonuses);
    let connect = connection_bonus(&addr, path, stop_ixs, conn_bonuses);
    base_revenue + visit + connect
}

fn addr_ix_and_base_revenue(
    path: &Path,
    addr: &HexAddress,
    visit_bonuses: &HashMap<HexAddress, usize>,
    conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
) -> (usize, usize) {
    let ix = path
        .visits
        .iter()
        .enumerate()
        .find_map(
            |(ix, visit)| {
                if visit.addr == *addr {
                    Some(ix)
                } else {
                    None
                }
            },
        )
        .unwrap();
    let revenue =
        revenue_for_stop(path, &vec![], ix, visit_bonuses, conn_bonuses);
    (ix, revenue)
}

fn best_ix_and_base_revenue(
    path: &Path,
    addrs: &[HexAddress],
    visit_bonuses: &HashMap<HexAddress, usize>,
    conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
) -> (usize, usize) {
    addrs
        .iter()
        .map(|addr| {
            addr_ix_and_base_revenue(path, addr, visit_bonuses, conn_bonuses)
        })
        .max_by_key(|&(_ix, revenue)| revenue)
        .unwrap()
}

/// Calculate the revenue, including bonuses, for stopping at a subset of
/// visits along a path; this includes the train's revenue multiplier, if any.
fn revenue_for_stops(
    path: &Path,
    train: &Train,
    stop_ixs: &Vec<usize>,
    visit_bonuses: &HashMap<HexAddress, usize>,
    conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
) -> (usize, Vec<TrainStop>) {
    let stops: Vec<TrainStop> = stop_ixs
        .iter()
        .map(|ix| {
            let rev = revenue_for_stop(
                path,
                stop_ixs,
                *ix,
                visit_bonuses,
                conn_bonuses,
            );
            // NOTE: apply the train's revenue multiplier here.
            TrainStop {
                visit_ix: *ix,
                revenue: rev * train.revenue_multiplier,
            }
        })
        .collect();
    let net_revenue = stops.iter().map(|stop| stop.revenue).sum();
    (net_revenue, stops)
}

/// Calculate the best visits at which to stop, given possible restrictions on
/// which visits may be skipped.
fn best_stop_ixs(
    path: &Path,
    train: &Train,
    visit_bonuses: &HashMap<HexAddress, usize>,
    conn_bonuses: &HashMap<HexAddress, (Vec<HexAddress>, usize)>,
    can_skip: Vec<bool>,
    max_stops: usize,
) -> (usize, Vec<TrainStop>) {
    // Categorise each visit as must-stop or can-skip.
    let must_stop: Vec<bool> = can_skip.iter().map(|b| !b).collect();
    let must_stop_ixs: Vec<usize> = must_stop
        .iter()
        .enumerate()
        .filter_map(|(ix, stop)| if *stop { Some(ix) } else { None })
        .collect();

    if must_stop_ixs.len() > max_stops {
        panic!("Train cannot operate this path")
    }

    // For the can-skip visits, calculate their revenue when only stops are
    // the must-stop visits.
    let mut skip_revenues: Vec<(usize, usize)> = can_skip
        .iter()
        .enumerate()
        .filter_map(|(ix, skip)| {
            if *skip {
                let rev = revenue_for_stop(
                    path,
                    &must_stop_ixs,
                    ix,
                    visit_bonuses,
                    conn_bonuses,
                );
                Some((ix, rev))
            } else {
                None
            }
        })
        .collect();

    // Sort the can-skip visits from most revenue to least revenue.
    skip_revenues.sort_by_key(|(_ix, v)| *v);
    skip_revenues.reverse();

    // Stop at the can-skip visits that earn the most revenue.
    let num_to_keep = max_stops - must_stop_ixs.len();
    let extra_stop_ixs: Vec<_> = skip_revenues
        .iter()
        .take(num_to_keep)
        .map(|(ix, _rev)| *ix)
        .collect();
    let default_skip_ixs: Vec<_> = skip_revenues
        .iter()
        .skip(num_to_keep)
        .map(|(ix, _rev)| *ix)
        .collect();
    let default_skip_addrs: HashSet<HexAddress> = default_skip_ixs
        .iter()
        .map(|ix| path.visits[*ix].addr)
        .collect();

    // Combine the must-stop visits and the can-skip visits that earn the most
    // revenue. These are the optimal stops, with the possible exception of
    // connection bonuses.
    let default_ixs: Vec<usize> = must_stop_ixs
        .iter()
        .chain(extra_stop_ixs.iter())
        .map(|ix| *ix)
        .collect();
    let (default_revenue, default_stops) = revenue_for_stops(
        path,
        train,
        &default_ixs,
        visit_bonuses,
        conn_bonuses,
    );

    let visit_addrs: HashSet<HexAddress> =
        path.visits.iter().map(|v| v.addr).collect();
    // Find connection bonuses that could be satisfied, but are not satisfied
    // by the default approach of stopping at visits with the most revenue.
    let maybe_conn: HashMap<_, _> = conn_bonuses
        .iter()
        .filter(|(addr, (conns, _bonus))| {
            visit_addrs.contains(addr)
                && conns.iter().any(|conn| visit_addrs.contains(conn))
                && (default_skip_addrs.contains(addr)
                    || conns
                        .iter()
                        .any(|conn| default_skip_addrs.contains(conn)))
        })
        .collect();
    if maybe_conn.len() == 1 {
        let (src, (dests, _bonus)) = maybe_conn.iter().next().unwrap();
        let skipped_src = default_skip_addrs.contains(src);
        // NOTE: not all dests may belong to the path!!!
        let candidate_dests: Vec<HexAddress> = dests
            .iter()
            .filter(|addr| visit_addrs.contains(addr))
            .map(|addr| *addr)
            .collect();
        let skipped_dests = candidate_dests
            .iter()
            .all(|dest| default_skip_addrs.contains(dest));
        let (src_ix, _revenue) =
            addr_ix_and_base_revenue(path, src, visit_bonuses, conn_bonuses);
        let (dest_ix, _revenue) = best_ix_and_base_revenue(
            path,
            &candidate_dests,
            visit_bonuses,
            conn_bonuses,
        );
        // Determine the new stops that need to be made.
        let must_not_skip_ixs: Vec<usize> = vec![src_ix, dest_ix];
        let mut new_stop_ixs: Vec<usize> = vec![];
        if skipped_src {
            new_stop_ixs.push(src_ix)
        }
        if skipped_dests {
            new_stop_ixs.push(dest_ix)
        }
        let num_to_skip = new_stop_ixs.len();
        if num_to_skip > num_to_keep {
            // NOTE: cannot skip enough visits to satisfy this bonus.
            info!(
                "num_to_skip = {} > num_to_keep = {}",
                num_to_skip, num_to_keep
            );
            return (default_revenue, default_stops);
        }
        let new_num_to_keep = num_to_keep - num_to_skip;
        // NOTE: it's important here that we don't skip any visit that
        // currently contributes towards satisfying the connection bonus.
        let new_extra_stop_ixs: Vec<usize> = skip_revenues
            .iter()
            .filter(|(ix, _revenue)| !must_not_skip_ixs.contains(ix))
            .take(new_num_to_keep)
            .map(|(ix, _rev)| *ix)
            .chain(new_stop_ixs.into_iter())
            .collect();
        let new_ixs: Vec<usize> = must_stop_ixs
            .iter()
            .chain(new_extra_stop_ixs.iter())
            .map(|ix| *ix)
            .collect();
        let (new_revenue, new_stops) = revenue_for_stops(
            path,
            train,
            &new_ixs,
            visit_bonuses,
            conn_bonuses,
        );
        info!("Without the connection bonus: {}", default_revenue);
        info!("With the connection bonus: {}", new_revenue);
        info!("Without the connection bonus: {} stops", default_ixs.len());
        info!("With the connection bonus: {} stops", new_ixs.len());
        if new_revenue > default_revenue {
            return (new_revenue, new_stops);
        }
    } else if maybe_conn.len() > 0 {
        info!(
            "Found {} relevant connection bonuses, ignoring",
            maybe_conn.len()
        )
    }

    // NOTE: also return the revenue (excluding any revenue multiplier).
    (default_revenue, default_stops)
}

/// Pairings of trains to routes.
#[derive(Debug, PartialEq, Eq)]
pub struct Routes {
    /// The total revenue earned from these routes.
    pub net_revenue: usize,
    /// The routes that were operated and earned revenue.
    pub train_routes: Vec<TrainRoute>,
}

impl Routes {
    pub fn routes(&self) -> Vec<&Route> {
        self.train_routes.iter().map(|tr| &tr.route).collect()
    }
}

/// A train that operates a path to earn revenue.
///
/// Note that the train may not earn revenue from every location along the
/// path.
#[derive(Debug, PartialEq, Eq)]
pub struct TrainRoute {
    /// The train.
    pub train: Train,
    /// The revenue earned by having the train operate the route.
    pub revenue: usize,
    /// The route operated by the train.
    pub route: Route,
}

impl AsRef<Route> for TrainRoute {
    fn as_ref(&self) -> &Route {
        &self.route
    }
}

/// A route operated by a train.
#[derive(Debug, PartialEq, Eq)]
pub struct Route {
    /// The steps that form the entire route.
    pub steps: Vec<Step>,
    /// The visits along the route where revenue is earned.
    pub visits: Vec<Visit>,
}

impl AsRef<Route> for Route {
    fn as_ref(&self) -> &Route {
        self
    }
}

impl From<Path> for Route {
    fn from(path: Path) -> Route {
        Route {
            steps: path.steps,
            visits: path.visits,
        }
    }
}

impl From<&Path> for Route {
    fn from(path: &Path) -> Route {
        Route {
            steps: path.steps.clone(),
            visits: path.visits.clone(),
        }
    }
}

/// The trains owned by a single company, which may operate routes.
pub struct Trains {
    trains: BTreeMap<Train, usize>,
    train_vec: Vec<Train>,
    train_classes: Vec<usize>,
}

impl From<Vec<Train>> for Trains {
    fn from(src: Vec<Train>) -> Self {
        let mut trains = BTreeMap::new();
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
        let express = self.trains.keys().any(|t| t.is_express());
        if express {
            return None;
        }

        // NOTE: so there is a maximum number of stops, and no train can skip
        // cities and dits. For now, ignore the possibility of trains that can
        // skip cities but cannot skip dits.
        let skip_dits = self
            .trains
            .keys()
            .any(|t| t.train_type == TrainType::SkipTowns);
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
    ) -> Option<Routes> {
        let num_paths = path_tbl.len();
        let num_trains = self.train_count();

        // Index visit bonuses by location.
        let visit_bonuses: HashMap<HexAddress, usize> = bonuses
            .iter()
            .filter_map(|b| match b {
                Bonus::VisitBonus { locn, bonus } => Some((*locn, *bonus)),
                Bonus::ConnectionBonus { .. } => None,
            })
            .collect();

        // Index connection bonuses by location.
        let connect_bonuses: HashMap<HexAddress, (Vec<HexAddress>, usize)> =
            bonuses
                .into_iter()
                .filter_map(|b| match b {
                    Bonus::VisitBonus { .. } => None,
                    Bonus::ConnectionBonus {
                        from,
                        to_any,
                        bonus,
                    } => Some((from, (to_any, bonus))),
                })
                .collect();

        // Build a table that maps each path (identified by index) to a
        // train-revenue table.
        info!("Building path/train revenue table");
        let rev: BTreeMap<_, BTreeMap<Train, (usize, Vec<TrainStop>)>> = (0
            ..num_paths)
            .map(|path_ix| {
                (
                    path_ix,
                    self.trains
                        .keys()
                        .filter_map(|train| {
                            train
                                .revenue_for(
                                    &path_tbl[path_ix],
                                    &visit_bonuses,
                                    &connect_bonuses,
                                )
                                .map(|revenue| (*train, revenue))
                        })
                        .collect(),
                )
            })
            .collect();

        info!("Searching for best path combination");
        let best_pairing: Option<(usize, Vec<_>)> =
            CombinationsFilter::new(num_paths, num_trains, |a, b| {
                !path_tbl[a]
                    .route_conflicts
                    .is_disjoint(&path_tbl[b].route_conflicts)
            })
            .into_par_iter()
            .filter_map(|path_ixs| self.best_pairing_for(&rev, &path_ixs))
            .max_by_key(|&(revenue, _)| revenue);

        // Remove the paths from `path_tbl` and replace the path index in each
        // pairing with the corresponding path itself.
        let best_pairing = best_pairing.map(|(net_revenue, pairings)| {
            // Build a table that maps path indices to paths, retaining only
            // those paths that are paired with a train.
            let ixs: Vec<usize> = pairings.iter().map(|p| p.1).collect();
            let mut path_map: BTreeMap<usize, Path> = path_tbl
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
            let train_routes = pairings
                .into_iter()
                .map(|(train, path_ix, revenue, stops)| {
                    let mut path = path_map.remove(&path_ix).unwrap();
                    // Mark visit as a stop or not, by setting revenue to 0
                    // for skipped visits.
                    // NOTE: the first and last visit are always stopped at,
                    // but we may need to update their revenue due to bonuses.
                    for ix in 0..path.visits.len() {
                        let stop_opt =
                            stops.iter().find(|stop| stop.visit_ix == ix);
                        path.visits[ix].revenue =
                            stop_opt.map(|stop| stop.revenue).unwrap_or(0);
                    }
                    let route: Route = path.into();
                    TrainRoute {
                        train,
                        revenue,
                        route,
                    }
                })
                .collect();

            Routes {
                net_revenue,
                train_routes,
            }
        });

        info!("Found a best pairing? {}", best_pairing.is_some());

        best_pairing
    }

    fn best_pairing_for(
        &self,
        revenue: &BTreeMap<usize, BTreeMap<Train, (usize, Vec<TrainStop>)>>,
        path_ixs: &Vec<usize>,
    ) -> Option<(usize, Vec<(Train, usize, usize, Vec<TrainStop>)>)> {
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
                let revenues: Vec<(usize, Vec<TrainStop>)> = train_ixs
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
