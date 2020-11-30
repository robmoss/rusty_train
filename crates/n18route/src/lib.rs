//! # Overview
//!
//! This module solves the problem of finding the set of routes that can be
//! run by a company to yield the highest possible revenue.
//!
//! See the [route-finding documentation](crate::doc) for details.

pub mod conflict;

pub mod path;

pub mod search;

pub mod bonus;

pub mod perm;

pub mod comb;

pub mod train;

pub mod builder;

pub mod doc;

#[doc(inline)]
pub use conflict::{Conflict, ConflictRule};

#[doc(inline)]
pub use path::{Path, Step, StopLocation, Visit};

#[doc(inline)]
pub use search::{paths_for_token, Criteria, PathLimit, Query};

#[doc(inline)]
pub use train::{Route, Routes, Train, TrainRoute, TrainType, Trains};

#[doc(inline)]
pub use bonus::Bonus;
