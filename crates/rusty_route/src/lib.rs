//! # Overview
//!
//! This module solves the problem of finding the set of routes that can be
//! run by a company to yield the highest possible revenue.
//!
//! See the [route-finding documentation](doc/index.html) for details.

pub mod conflict;

pub mod path;

pub mod search;

pub mod perm;

pub mod comb;

pub mod train;

pub mod doc;

pub use conflict::{Conflict, ConflictRule};
pub use path::{Path, Step, Stop, StopLocation, Visit};
pub use search::{paths_for_token, Criteria, PathLimit, Query};
pub use train::{Pair, Pairing, Train, Trains};
