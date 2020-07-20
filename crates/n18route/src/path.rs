//! # Overview
//!
//! This module solves the problem of finding the set of routes that can be
//! run by a company to yield the highest possible revenue.
//!
//! See the [route-finding documentation](doc/index.html) for details.

use std::collections::HashSet;

use n18map::HexAddress;
use n18tile::Connection;

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
        if let StopLocation::City { ix: _ } = self {
            true
        } else {
            false
        }
    }

    pub fn is_dit(&self) -> bool {
        if let StopLocation::Dit { ix: _ } = self {
            true
        } else {
            false
        }
    }
}

/// A location at which a train may stop and, optionally, earn revenue.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stop {
    /// The tile on which this stop occurs.
    pub addr: HexAddress,
    /// The revenue earned for this stop, if visited.
    pub revenue: Option<usize>,
    /// The city or dit associated with this stop.
    pub stop_at: StopLocation,
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
    pub conflicts: HashSet<Conflict>,
    pub route_conflicts: HashSet<Conflict>,
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
            other.steps[1..].iter().map(|s| *s).collect();
        steps.append(&mut other_steps);
        let mut visits = self.visits.clone();
        let mut other_visits: Vec<_> =
            other.visits[1..].iter().map(|s| *s).collect();
        // NOTE: ensure the visits are in order, so start from the end of
        // self's path and travel to the self's start, which is also other's
        // start, and continue on to other's end.
        visits.reverse();
        visits.append(&mut other_visits);
        let conflicts: HashSet<_> =
            self.conflicts.union(&other.conflicts).map(|c| *c).collect();
        let route_conflicts: HashSet<_> = self
            .route_conflicts
            .union(&other.route_conflicts)
            .map(|c| *c)
            .collect();
        let start_revenue = self.visits[0].revenue;
        let revenue = self.revenue + other.revenue - start_revenue;
        let num_visits = visits.len();
        let num_cities = self.num_cities + other.num_cities - 1;
        let num_dits = self.num_dits + other.num_dits;
        let num_hexes = self.num_hexes + other.num_hexes - 1;
        Path {
            steps: steps,
            conflicts: conflicts,
            route_conflicts: route_conflicts,
            visits: visits,
            num_visits: num_visits,
            num_cities: num_cities,
            num_dits: num_dits,
            num_hexes: num_hexes,
            revenue: revenue,
        }
    }
}
