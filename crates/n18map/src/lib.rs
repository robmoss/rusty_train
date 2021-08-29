/// Maps with spaces for tiles.
pub mod map;

pub mod descr;

#[doc(inline)]
pub use descr::{Descr, TileDescr};

#[doc(inline)]
pub use map::{
    EmptyHexIter, HexAddress, HexIter, Map, TileHexIter, TokensTable,
};
