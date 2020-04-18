/// Maps with spaces for tiles.
pub mod map;

pub mod descr;

pub use descr::{Descr, TileDescr};
pub use map::{
    EmptyHexIter, HexAddress, HexIter, Map, RotateCW, TileHexIter, Token,
    TokensTable,
};
