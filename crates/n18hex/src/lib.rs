/// Constant values used to define, e.g., angles in radians.
pub mod consts;

/// Cartesian coordinates for use with hex tiles.
pub mod coord;

/// Hexagonal tiles, and attributes such as faces, corners, and colours.
pub mod hex;

/// Define colours, line styles, and other drawing properties.
pub mod theme;

#[doc(inline)]
pub use consts::*;

#[doc(inline)]
pub use coord::Coord;

#[doc(inline)]
pub use hex::{
    Delta, Direction, Hex, HexColour, HexCorner, HexFace, HexPosition,
    Orientation, RotateCW,
};

#[doc(inline)]
pub use theme::{Colour, Theme};
