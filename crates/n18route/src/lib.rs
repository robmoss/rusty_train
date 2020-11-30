//! # Overview
//!
//! This module solves the problem of finding the set of routes that can be
//! run by a company to yield the highest possible revenue.
//!
//! For example, the following function can find the best routes on a map for
//! a specific company (identified here by their `Token`) that owns one or
//! more trains, given (game-specific) [rules](ConflictRule) about which
//! elements may be reused by a single route (`conflict_rule`) and which
//! elements may be shared by multiple routes (`route_conflict_rule`):
//!
//! ```rust
//! use n18route::{paths_for_token, Bonus, Criteria, ConflictRule, Trains, Routes};
//! use n18map::Map;
//! use n18token::Token;
//!
//! fn find_best_routes(map: &Map, token: Token, trains: Trains,
//!                     bonuses: Vec<Bonus>) -> Routes {
//!     // Find all of the paths that the trains could operate.
//!     let criteria = Criteria {
//!         token,
//!         path_limit: trains.path_limit(),
//!         // NOTE: game-specific rule.
//!         conflict_rule: ConflictRule::TrackOrCityHex,
//!         // NOTE: game-specific rule.
//!         route_conflict_rule: ConflictRule::TrackOnly,
//!     };
//!     let paths = paths_for_token(&map, &criteria);
//!
//!     // Return the best routes out of the available paths.
//!     trains
//!         .select_routes(paths, bonuses)
//!         .expect("Could not find an optimal set of routes")
//! }
//! ```
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
