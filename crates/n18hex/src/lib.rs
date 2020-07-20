/// Constant values used to define, e.g., angles in radians.
pub mod consts;

/// Cartesian coordinates for use with hex tiles.
pub mod coord;

/// Hexagonal tiles, and attributes such as faces, corners, and colours.
pub mod hex;

#[doc(inline)]
pub use consts::*;

#[doc(inline)]
pub use coord::Coord;

#[doc(inline)]
pub use hex::{
    Delta, Direction, Hex, HexColour, HexCorner, HexFace, HexPosition,
};
