//! # Overview
//!
//! This module solves the problem of finding the set of routes that can be
//! run by a company to yield the highest possible revenue.

use std::collections::HashSet;

use crate::connection::Connection;
use crate::map::HexAddress;

use conflict::Conflict;

pub mod conflict;
pub mod search;

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

/// A path that a train may travel along.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub steps: Vec<Step>,
    pub conflicts: HashSet<Conflict>,
    pub stops: Vec<Stop>,
    pub num_visits: usize,
    pub revenue: usize,
}

impl Path {
    /// Join two paths, which should both start from the same location.
    fn append(&self, other: &Path) -> Path {
        let mut steps = self.steps.clone();
        let mut other_steps: Vec<_> =
            other.steps[1..].iter().map(|s| *s).collect();
        steps.append(&mut other_steps);
        let mut stops = self.stops.clone();
        let mut other_stops: Vec<_> =
            other.stops[1..].iter().map(|s| *s).collect();
        stops.append(&mut other_stops);
        let conflicts: HashSet<_> =
            self.conflicts.union(&other.conflicts).map(|c| *c).collect();
        let start_revenue = self.stops[0].revenue.unwrap();
        let num_visits = stops.iter().filter(|s| s.revenue.is_some()).count();
        Path {
            steps: steps,
            stops: stops,
            num_visits: num_visits,
            conflicts: conflicts,
            revenue: self.revenue + other.revenue - start_revenue,
        }
    }

    /// Join two paths, which should both start from the same location, and
    /// skip over the starting location.
    fn append_with_skip(&self, other: &Path) -> Path {
        let mut steps = self.steps.clone();
        let mut other_steps: Vec<_> =
            other.steps[1..].iter().map(|s| *s).collect();
        steps.append(&mut other_steps);
        let mut stops = self.stops.clone();
        stops[0].revenue = None;
        let mut other_stops: Vec<_> =
            other.stops[1..].iter().map(|s| *s).collect();
        stops.append(&mut other_stops);
        let num_visits = stops.iter().filter(|s| s.revenue.is_some()).count();
        let conflicts: HashSet<_> =
            self.conflicts.union(&other.conflicts).map(|c| *c).collect();
        let start_revenue = self.stops[0].revenue.unwrap();
        Path {
            steps: steps,
            stops: stops,
            num_visits: num_visits,
            conflicts: conflicts,
            revenue: self.revenue + other.revenue - 2 * start_revenue,
        }
    }
}
