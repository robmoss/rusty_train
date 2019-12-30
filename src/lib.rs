//! # Overview
//!
//! A crate for working with 18xx [tiles](http://www.fwtwr.com/18xx/tiles/)
//! and maps, and searching for train routes with optimal revenue.
//!

pub mod city;
pub mod coord;
pub mod draw;
pub mod hex;
pub mod label;
pub mod tile;
pub mod track;

pub mod catalogue;

pub mod de;

/// Exports commonly-used elements of other modules.
pub mod prelude;

// TODO:
//   grid module for tile arrangement
//   map module for building on top of grid and having off-grid content?
//   ui module for commands and actions (e.g., manipulating map)
//   route module for finding optimal routes and revenue
//   token module for helping define possible routes
//   (may need some modifications to City)
//   a (de)serialisation module for Tile/Map definitions?
//
//   more general handling of labels and revenue circles, able to position
//   them relative to centre / face / corner, with optional nudge ...
//
//   TEST CASES!
//   https://doc.rust-lang.org/1.30.0/book/second-edition/ch11-03-test-organization.html
//   -> no track should escape its hex
//   -> all coords should lie on the track
//   -> check start/end
//   -> check that coords approach the start/end
//   -> check coords are clipped
//   -> check x0/x1 limits are correct
//      -> e.g., that limited tracks connect, but don't cross
//   -> draw every possible segment from every possible face
//      -> iterate over specific x0/x1 and c0/c1 combinations
//   -> ensure topology of track/track and track/city connections are correct
//      for many (most? all?) tiles in catalogue
