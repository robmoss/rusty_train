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
//! - Defining and manipulating 18xx game maps.
//! - Selecting trains to operate routes for a company.
//! - Selecting route bonuses for a company.
//! - Searching maps for optimal pairings of trains to routes.
//!
//! ## Supported games
//!
//! Maps, tiles, and trains for the following games are implemented:
//!
//! - 1830: Railways and Robber Barons
//! - 1861: The Railways of the Russian Empire
//! - 1867: The Railways of Canada
//! - 1889: History of Shikoku Railways (Shikoku 1889)
//!
//! ## Defining tiles
//!
//! Use the [`Tile`](tile::Tile) data
//! structure.
//! This uses the [Cairo bindings](https://gtk-rs.org/docs/cairo/) provided by
//! the [Gtk-rs](https://gtk-rs.org/) project.
//!
//! ```rust
//! use navig18xx::prelude::*;
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
//!             .with_dit(TrackEnd::End, 10, DitShape::Bar),
//!         Track::hard_l(HexFace::Bottom).with_span(0.5, 1.0),
//!     ],
//!     vec![],
//!     &hex,
//!     );
//!
//! // Save this tile to a JSON file.
//! let pretty_json = true;
//! # std::env::set_current_dir("../../tests/output").unwrap();
//! write_tile("tile_3.json", &tile, pretty_json);
//! tile.save_png(&hex, "tile_3.png")
//!     .expect("Could not save tile as a PNG");
//! ```
//!
//! More complex tiles, with multiple token spaces and overlapping tracks, can
//! be defined in the same way. For example, here are definitions of tiles
//! [45](http://www.fwtwr.com/18xx/tiles/tf/0045_1.gif) and
//! [X5](http://www.fwtwr.com/18xx/tiles/tf/X5_1.gif):
//!
//! ```rust
//! # use navig18xx::prelude::*;
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
//!         Track::straight(HexFace::Top).with_span(0.0, 0.1),
//!         Track::straight(HexFace::Top)
//!             .with_span(0.1, 1.0)
//!             .with_clip(0.3625, 0.75),
//!         Track::mid(HexFace::UpperLeft),
//!         Track::mid(HexFace::LowerLeft),
//!         Track::mid(HexFace::LowerRight),
//!         Track::mid(HexFace::UpperRight),
//!     ],
//!     vec![
//!         City::single_at_face(70, &HexFace::Top),
//!         City::double(70).in_dir(Direction::S, 0.1),
//!     ],
//!     &hex,
//! )
//! .label(Label::City("M".to_string()), HexCorner::BottomLeft)
//! .label(Label::Revenue(0), HexCorner::Left.to_centre(0.1));
//!
//! # std::env::set_current_dir("../../tests/output").unwrap();
//! tile_x5.save_png(&hex, "tile_x5.png")
//!     .expect("Could not save tile X5 as a PNG");
//! tile_x5.save_svg(&hex, "tile_x5.svg")
//!     .expect("Could not save tile X5 as an SVG");
//! tile_x5.save_pdf(&hex, "tile_x5.pdf")
//!     .expect("Could not save tile X5 as a PDF");
//! ```
//!

/// Exports commonly-used elements of other modules.
pub mod prelude;

pub use n18brush as brush;
pub use n18catalogue as catalogue;
pub use n18example as example;
pub use n18game as game;
pub use n18hex as hex;
pub use n18io as io;
pub use n18map as map;
pub use n18route as route;
pub use n18tile as tile;
pub use n18token as token;
#[cfg(feature = "ui")]
pub use n18ui as ui;
