//! Describe and create map configurations.

use std::collections::HashMap;

use crate::map::{HexAddress, Map, RotateCW, Token};
use crate::tile::Tile;

/// A description of a tile's configuration on a map hex.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TileDescr {
    /// The map row in which the tile is located.
    pub row: usize,
    /// The map column in which the tile is located.
    pub col: usize,
    /// The tile name.
    pub tile: &'static str,
    /// The tile rotation.
    pub rotation: RotateCW,
    /// Token spaces are identified by index into `Tile::token_spaces()`.
    pub tokens: Vec<(usize, Token)>,
}

/// A description of each tile's configuration on a map.
pub struct Descr {
    tiles: HashMap<HexAddress, TileDescr>,
}

impl From<Vec<TileDescr>> for Descr {
    fn from(src: Vec<TileDescr>) -> Descr {
        Descr {
            tiles: src
                .into_iter()
                .map(|td| ((td.row, td.col).into(), td))
                .collect(),
        }
    }
}

impl Descr {
    /// Constructs a map whose state reflects the tile configurations.
    pub fn build_map(&self, tiles: Vec<Tile>) -> Map {
        let addrs = self
            .tiles
            .keys()
            .map(|addr| addr.clone())
            .collect::<Vec<_>>();
        let mut map = Map::new(tiles, addrs);

        for (addr, tile_descr) in self.tiles.iter() {
            map.place_tile(*addr, tile_descr.tile, tile_descr.rotation);
            let spaces = {
                let tile = map.tile_at(*addr).expect("No tile");
                tile.token_spaces()
            };
            let hex_state = map.get_hex_mut(*addr).expect("No hex state");
            for (space_ix, token) in &tile_descr.tokens {
                hex_state.set_token_at(&spaces[*space_ix], *token);
            }
        }

        map
    }
}

#[cfg(test)]
pub mod tests {
    use cairo::{Context, Format, ImageSurface};

    use super::*;

    use crate::hex::Hex;

    static HEX_DIAMETER: f64 = 150.0;

    fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
        let surface = ImageSurface::create(Format::ARgb32, width, height)
            .expect("Can't create surface");
        (Context::new(&surface), surface)
    }

    /// Return a 2x2 map that contains the following tiles:
    ///
    /// - Tile 5 at (0, 0);
    /// - Tile 6 at (0, 1) (rotated clockwise twice);
    /// - Tile 58 at (1, 0) (rotated anti-clockwise once);
    /// - Tile 63 at (1, 1);
    ///
    /// "LP" tokens are placed on tiles 5 and 63; and "PO" tokens are placed
    /// on tiles 6 and 63.
    ///
    /// Note that this map may be used by test cases in other modules.
    pub fn map_2x2_tiles_5_6_58_63(hex: &Hex) -> Map {
        let tiles = crate::catalogue::tile_catalogue(hex);
        let descr = descr_2x2_tiles_5_6_58_63();
        let map = descr.build_map(tiles);
        map
    }

    /// Defines the map that should be created by `map_2x2_tiles_5_6_58_63`.
    fn descr_2x2_tiles_5_6_58_63() -> Descr {
        vec![
            TileDescr {
                row: 0,
                col: 0,
                tile: "5",
                rotation: RotateCW::Zero,
                tokens: vec![(0, Token::LP)],
            },
            TileDescr {
                row: 0,
                col: 1,
                tile: "6",
                rotation: RotateCW::Two,
                tokens: vec![(0, Token::PO)],
            },
            TileDescr {
                row: 1,
                col: 0,
                tile: "58",
                rotation: RotateCW::Five,
                tokens: vec![],
            },
            TileDescr {
                row: 1,
                col: 1,
                tile: "63",
                rotation: RotateCW::Zero,
                tokens: vec![(0, Token::PO), (1, Token::LP)],
            },
        ]
        .into()
    }

    #[test]
    fn simple_two_by_two() {
        let hex = Hex::new(HEX_DIAMETER);
        let map = map_2x2_tiles_5_6_58_63(&hex);

        // NOTE: check the three hex iterators to ensure they all yield the
        // expected map configuration.

        // Check that there are no empty hexes.
        let empty_iter = map.empty_hex_iter(&hex, hex.context());
        assert_eq!(empty_iter.count(), 0);

        // Check that there are four hexes.
        let hexes: Vec<_> = map.hex_iter(&hex, hex.context()).collect();
        assert_eq!(hexes.len(), 4);
        // Check that all four hexes contain tiles.
        assert!(hexes.iter().all(|(_addr, ts_opt)| ts_opt.is_some()));

        // Check (again) that there are four tiles.
        let tile_hexes: Vec<_> =
            map.tile_hex_iter(&hex, hex.context()).collect();
        assert_eq!(tile_hexes.len(), 4);

        // Check that the same tiles are reported to be at the same locations
        // according to Map::hex_iter() and Map::tile_hex_iter().
        for (addr, ts_opt) in hexes.into_iter() {
            let h = (addr, ts_opt.unwrap());
            assert!(tile_hexes.iter().find(|&&th| th == h).is_some())
        }

        // Check the hex location, rotation, and tokens for each tile.
        let descr = descr_2x2_tiles_5_6_58_63();
        for (addr, tile_descr) in descr.tiles.iter() {
            // Check that the map contains a tile at this hex location.
            let th = tile_hexes.iter().find(|&(th_addr, _)| addr == th_addr);
            assert!(th.is_some());

            // Check that tile names match.
            let (_addr, (tile, tokens_tbl)) = th.unwrap();
            assert_eq!(tile_descr.tile, tile.name);

            // Check that all of the tokens are placed correctly, and that no
            // additional tokens have been placed.
            assert_eq!(tokens_tbl.len(), tile_descr.tokens.len());
            let token_spaces = tile.token_spaces();
            for (ix, token) in &tile_descr.tokens {
                let token_space = token_spaces[*ix];
                assert_eq!(tokens_tbl.get(&token_space), Some(token));
            }

            // Check that the tile rotations match.
            let hex_state = map.get_hex(*addr);
            assert!(hex_state.is_some());
            let rot = hex_state.unwrap().rotation();
            assert_eq!(rot, &tile_descr.rotation);
        }

        // Also save the map to disk.
        let dx = HEX_DIAMETER * 2.1;
        let dy = HEX_DIAMETER * 2.3;
        let (ctx, surf) = new_context(dx as i32, dy as i32);
        map.draw_tiles(&hex, &ctx);
        let filename = "test-map-descr-simple-2x2.png";
        let mut file = std::fs::File::create(filename)
            .expect("Couldn't create output PNG file");
        surf.write_to_png(&mut file)
            .expect("Couldn't write to output PNG file");
    }
}
