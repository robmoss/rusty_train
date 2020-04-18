/// Constant values used to define, e.g., angles in radians.
pub mod consts;

/// Cartesian coordinates for use with hex tiles.
pub mod coord;

/// Hexagonal tiles, and attributes such as faces, corners, and colours.
pub mod hex;

pub use consts::*;
pub use coord::Coord;
pub use hex::{
    Delta, Direction, Hex, HexColour, HexCorner, HexFace, HexPosition,
};
