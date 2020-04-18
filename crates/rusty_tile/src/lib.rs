/// Generic trait for tiles and tile elements that draw themselves.
pub mod draw;

/// Cities and token spaces.
pub mod city;

/// Tile labels, such as tile names, city names, and revenue.
pub mod label;

/// Track segments.
pub mod track;

/// Connections between track segments, cities, and tile edges.
pub mod connection;

/// Tiles that can contain track segments, cities, and token spaces.
pub mod tile;

pub use city::{City, Rotation, Tokens};
pub use connection::{Connection, Connections, Dit};
pub use draw::Draw;
pub use label::Label;
pub use tile::{LabelAndPos, Tile, TokenSpace};
pub use track::{DitShape, Track, TrackCurve, TrackEnd};
