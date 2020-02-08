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
    pub revenue: usize,
}
