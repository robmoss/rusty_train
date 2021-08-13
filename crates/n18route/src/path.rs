//! # Overview
//!
//! This module solves the problem of finding the set of routes that can be
//! run by a company to yield the highest possible revenue.
//!
//! See the [route-finding documentation](crate::doc) for details.

use std::collections::BTreeSet;

use n18map::HexAddress;
use n18tile::Connection;

use crate::conflict::RouteConflicts;
use crate::Conflict;

/// A single step in a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    pub addr: HexAddress,
    pub conn: Connection,
}

/// The different locations at which a train may stop and earn revenue.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StopLocation {
    City { ix: usize },
    Dit { ix: usize },
}

impl StopLocation {
    pub fn is_city(&self) -> bool {
        matches!(self, StopLocation::City { .. })
    }

    pub fn is_dit(&self) -> bool {
        matches!(self, StopLocation::Dit { .. })
    }
}

/// A location on a path that, if the train stops here, may earn revenue.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    /// The tile on which this stop occurs.
    pub addr: HexAddress,
    /// The base revenue for this location.
    pub revenue: usize,
    /// The city or dit associated with this visit.
    pub visits: StopLocation,
}

/// A path that a train may travel along.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub steps: Vec<Step>,
    pub conflicts: BTreeSet<Conflict>,
    pub route_conflicts: RouteConflicts,
    pub visits: Vec<Visit>,
    pub num_visits: usize,
    pub num_cities: usize,
    pub num_dits: usize,
    pub num_hexes: usize,
    pub revenue: usize,
}

impl Path {
    /// Returns the location at which this path starts.
    pub fn start(&self) -> &Visit {
        self.visits.first().unwrap()
    }

    /// Returns the location at which this path ends.
    pub fn end(&self) -> &Visit {
        self.visits.last().unwrap()
    }

    /// Joins two paths, which must start from the same location.
    pub(crate) fn append(&self, other: &Path) -> Path {
        // NOTE: ensure that the first step of both paths is the same.
        if self.steps[0] != other.steps[0] {
            panic!(
                "Paths don't start from the same location: {:?} and {:?}",
                self.steps[0], other.steps[0]
            );
        }
        let mut steps = self.steps.clone();
        let mut other_steps: Vec<_> =
            other.steps[1..].iter().copied().collect();
        steps.append(&mut other_steps);
        let mut visits = self.visits.clone();
        let mut other_visits: Vec<_> =
            other.visits[1..].iter().copied().collect();
        // NOTE: ensure the visits are in order, so start from the end of
        // self's path and travel to the self's start, which is also other's
        // start, and continue on to other's end.
        visits.reverse();
        visits.append(&mut other_visits);
        let conflicts: BTreeSet<_> =
            self.conflicts.union(&other.conflicts).copied().collect();
        let route_conflicts =
            self.route_conflicts.merge(&other.route_conflicts);
        let start_revenue = self.visits[0].revenue;
        let revenue = self.revenue + other.revenue - start_revenue;
        let num_visits = visits.len();
        let num_cities = self.num_cities + other.num_cities - 1;
        let num_dits = self.num_dits + other.num_dits;
        let num_hexes = self.num_hexes + other.num_hexes - 1;
        Path {
            steps,
            conflicts,
            route_conflicts,
            visits,
            num_visits,
            num_cities,
            num_dits,
            num_hexes,
            revenue,
        }
    }
}
