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

pub mod upgrade;

pub mod ekmf;

#[doc(inline)]
pub use city::{City, Rotation, Tokens};

#[doc(inline)]
pub use connection::{Connection, Connections, Dit};

#[doc(inline)]
pub use draw::Draw;

#[doc(inline)]
pub use label::Label;

#[doc(inline)]
pub use tile::{LabelAndPos, Tile, TokenSpace};

#[doc(inline)]
pub use track::{DitShape, Track, TrackCurve, TrackEnd};
