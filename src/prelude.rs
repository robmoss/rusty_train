pub use std::f64::consts::PI;

use std::f64::consts::FRAC_PI_4;
use std::f64::consts::FRAC_PI_6;

/// π/6
pub const PI_1_6: f64 = FRAC_PI_6;

/// 2π/6
pub const PI_2_6: f64 = 2.0 * FRAC_PI_6;

/// 3π/6 = 2π/4 = π/2
pub const PI_3_6: f64 = 3.0 * FRAC_PI_6;

/// 4π/6
pub const PI_4_6: f64 = 4.0 * FRAC_PI_6;

/// 5π/6
pub const PI_5_6: f64 = 5.0 * FRAC_PI_6;

/// π/4
pub const PI_1_4: f64 = FRAC_PI_4;

/// 3π/4
pub const PI_3_4: f64 = 3.0 * FRAC_PI_4;

#[doc(inline)]
pub use crate::hex::Hex;

#[doc(inline)]
pub use crate::hex::HexColour;

#[doc(inline)]
pub use crate::hex::HexCorner;

#[doc(inline)]
pub use crate::hex::HexFace;

#[doc(inline)]
pub use crate::hex::HexPosition;

#[doc(inline)]
pub use crate::hex::Delta;

#[doc(inline)]
pub use crate::hex::Direction;

#[doc(inline)]
pub use crate::city::City;

#[doc(inline)]
pub use crate::city::Rotation;

#[doc(inline)]
pub use crate::track::Track;

#[doc(inline)]
pub use crate::track::TrackCurve;

#[doc(inline)]
pub use crate::track::TrackEnd;

#[doc(inline)]
pub use crate::label::Label;

#[doc(inline)]
pub use crate::tile::Tile;

#[doc(inline)]
pub use crate::tile::Tiles;

#[doc(inline)]
pub use crate::connection::Connection;

#[doc(inline)]
pub use crate::catalogue::tile_catalogue;

#[doc(inline)]
pub use crate::de::read_tile;

#[doc(inline)]
pub use crate::de::write_tile;

#[doc(inline)]
pub use crate::de::read_tiles;

#[doc(inline)]
pub use crate::de::write_tiles;

#[doc(inline)]
pub use crate::map::Map;

#[doc(inline)]
pub use crate::map::HexAddress;

#[doc(inline)]
pub use crate::map::RotateCW;

#[doc(inline)]
pub use crate::map::Token;

#[doc(inline)]
pub use crate::map::TokensTable;
