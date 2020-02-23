//! # Overview
//!
//! A crate for working with 18xx [tiles](http://www.fwtwr.com/18xx/tiles/)
//! and maps, and searching for train routes with optimal revenue.
//!
//! ## Current status
//!
//! The following features are implemented:
//!
//! - Ability to define **most** 18xx tiles.
//! - Drawing tiles on-screen and saving images to disk.
//! - (De)serialising tile descriptions.
//! - Placing tokens in token spaces.
//!
//! The following features are not yet provided:
//!
//! - Defining and manipulating 18xx game maps.
//! - Searching maps for optimal routes.
//!
//! Note: there **is** support for finding a single route that yields the
//! highest possible revenue (ignoring bonuses for, e.g., visiting specific
//! locations).
//! See the [route-finding documentation](route/doc/index.html) for details.
//!
//! ## Defining tiles
//!
//! Use the [`rusty_train::tile::Tile`](tile/struct.Tile.html) data structure.
//! This uses the [Cairo bindings](https://gtk-rs.org/docs/cairo/) provided by
//! the [Gtk-rs](https://gtk-rs.org/) project.
//!
//! ```rust
//! use rusty_train::prelude::*;
//!
//! // Define the basic tile geometry.
//! let hex_max_diameter = 125.0;
//! let hex = Hex::new(hex_max_diameter);
//!
//! // Create a tile that contains two track segments and a dit.
//! let tile = Tile::new(
//!     HexColour::Yellow,
//!     "3",
//!     vec![
//!         Track::hard_l(HexFace::Bottom)
//!             .with_span(0.0, 0.5)
//!             .with_dit(TrackEnd::End, 10),
//!         Track::hard_l(HexFace::Bottom).with_span(0.5, 1.0),
//!     ],
//!     vec![],
//!     &hex,
//!     );
//!
//! // Save this tile to a JSON file.
//! let pretty_json = true;
//! write_tile("tile_3.json", &tile, pretty_json);
//! ```
//!
//! More complex tiles, with multiple token spaces and overlapping tracks, can
//! be defined in the same way. For example, here are definitions of tiles
//! [45](http://www.fwtwr.com/18xx/tiles/tf/0045_1.gif) and
//! [X5](http://www.fwtwr.com/18xx/tiles/tf/X5_1.gif):
//!
//! ```rust,no_run
//! # use rusty_train::prelude::*;
//! #
//! # // Define the basic tile geometry.
//! # let hex_max_diameter = 125.0;
//! # let hex = Hex::new(hex_max_diameter);
//! #
//! let tile_45 = Tile::new(
//!     HexColour::Brown,
//!     "45",
//!     vec![
//!         Track::gentle_l(HexFace::UpperLeft),
//!         Track::hard_r(HexFace::Top),
//!         Track::gentle_r(HexFace::Bottom),
//!         Track::straight(HexFace::Bottom),
//!     ],
//!     vec![],
//!     &hex,
//! );
//! let tile_x5 = Tile::new(
//!     HexColour::Brown,
//!     "X5",
//!     vec![
//!         Track::straight(HexFace::Top).with_clip(0.3625, 0.75),
//!         Track::mid(HexFace::UpperLeft),
//!         Track::mid(HexFace::LowerLeft),
//!         Track::mid(HexFace::LowerRight),
//!         Track::mid(HexFace::UpperRight),
//!     ],
//!     vec![
//!         City::single_at_face(70, &HexFace::Top),
//!         City::double(70).nudge(Direction::S, 0.1),
//!     ],
//!     &hex,
//! )
//! .label(Label::City("M".to_string()), HexCorner::BottomLeft)
//! .label(Label::Revenue(0), HexCorner::Left.to_centre(0.1));
//!
//! tile_x5.save_png(&hex, "tile_x5.png")
//!     .expect("Could not save tile X5 as a PNG");
//! tile_x5.save_svg(&hex, "tile_x5.svg")
//!     .expect("Could not save tile X5 as an SVG");
//! tile_x5.save_pdf(&hex, "tile_x5.pdf")
//!     .expect("Could not save tile X5 as a PDF");
//! ```
//!

/// Cities and token spaces.
pub mod city;

/// Cartesian coordinates for use with hex tiles.
pub mod coord;

/// Generic trait for tiles and tile elements that draw themselves.
pub mod draw;

/// Hexagonal tiles, and attributes such as faces, corners, and colours.
pub mod hex;

/// Tile labels, such as tile names, city names, and revenue.
pub mod label;

/// Tiles that can contain track segments, cities, and token spaces.
pub mod tile;

/// Track segments.
pub mod track;

/// Connections between track segments, cities, and tile edges.
pub mod connection;

/// Game-specific tile catalogues.
pub mod catalogue;

/// Support for tile (de)serialisation.
pub mod de;

/// Maps with spaces for tiles.
pub mod map;

/// Find train routes with the greatest revenue.
pub mod route;

/// A user interface for creating map states and calculating route revenue.
pub mod ui;

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
