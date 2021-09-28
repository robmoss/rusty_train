/// Maps with spaces for tiles.
pub mod map;

pub mod address;

pub mod descr;

#[doc(inline)]
pub use descr::{Descr, TileDescr};

#[doc(inline)]
pub use address::*;

#[doc(inline)]
pub use map::{EmptyHexIter, HexIter, Map, TileHexIter, TokensTable};
