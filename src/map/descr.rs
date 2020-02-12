//! Describe and create map configurations.

use std::collections::HashMap;

use crate::map::{HexAddress, Map, MapHex, RotateCW, Token};
use crate::tile::Tile;

/// A description of a tile's configuration on a map hex.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TileDescr<'a> {
    /// The map row in which the tile is located.
    pub row: usize,
    /// The map column in which the tile is located.
    pub col: usize,
    /// The tile name.
    pub tile: &'a str,
    /// The tile rotation.
    pub rotation: RotateCW,
    /// Token spaces are identified by index into `Tile::token_spaces()`.
    pub tokens: Vec<(usize, Token)>,
}

/// A description of each tile's configuration on a map.
pub struct Descr<'a> {
    tiles: HashMap<HexAddress, Option<TileDescr<'a>>>,
}

/// Constructs a map configuration from a vector of tile descriptions.
impl<'a> From<Vec<TileDescr<'a>>> for Descr<'a> {
    fn from(src: Vec<TileDescr<'a>>) -> Descr<'a> {
        Descr {
            tiles: src
                .into_iter()
                .map(|td| ((td.row, td.col).into(), Some(td)))
                .collect(),
        }
    }
}

impl<'a> From<(&'a Map, HexAddress, &'a MapHex)> for TileDescr<'a> {
    fn from(src: (&'a Map, HexAddress, &'a MapHex)) -> TileDescr<'a> {
        let map = src.0;
        let addr = src.1;
        let map_hex = src.2;
        let tile = map_hex.tile(map);
        let token_spaces = tile.token_spaces();
        let token_table = map_hex.get_tokens();
        let tokens: Vec<_> = token_table
            .iter()
            .map(|(token_space, token)| {
                let ix = token_spaces
                    .iter()
                    .position(|ts| ts == token_space)
                    .unwrap();
                (ix, *token)
            })
            .collect();
        TileDescr {
            row: addr.row,
            col: addr.col,
            tile: &tile.name,
            rotation: *map_hex.rotation(),
            tokens: tokens,
        }
    }
}

/// Describes the current state of an existing map.
impl<'a> From<&'a Map> for Descr<'a> {
    fn from(map: &'a Map) -> Descr<'a> {
        let tile_hexes =
            map.hexes().iter().map(|addr| (*addr, map.get_hex(*addr)));
        let tile_descrs = tile_hexes
            .map(|(addr, map_hex)| {
                let tile_opt: Option<TileDescr<'a>> =
                    map_hex.map(|mh| (map, addr, mh).into());
                (addr, tile_opt)
            })
            .collect();
        Descr { tiles: tile_descrs }
    }
}

impl<'a> Descr<'a> {
    /// Constructs a map whose state reflects the tile configurations.
    pub fn build_map(&self, tiles: Vec<Tile>) -> Map {
        let addrs = self
            .tiles
            .keys()
            .map(|addr| addr.clone())
            .collect::<Vec<_>>();
        let mut map = Map::new(tiles, addrs);
        self.update_map(&mut map);
        map
    }

    /// Updates the state of an existing map.
    pub fn update_map(&self, map: &mut Map) {
        for (addr, tile_descr) in self.tiles.iter() {
            if let Some(tile_descr) = tile_descr {
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
        }
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
    fn descr_2x2_tiles_5_6_58_63() -> Descr<'static> {
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
            if let Some(tile_descr) = tile_descr {
                // Check that the map contains a tile at this hex location.
                let th =
                    tile_hexes.iter().find(|&(th_addr, _)| addr == th_addr);
                assert!(th.is_some());

                // Check that tile names match.
                let (_addr, (tile, tokens_tbl)) = th.unwrap();
                assert_eq!(tile_descr.tile, tile.name);

                // Check that all of the tokens are placed correctly, and that
                // no additional tokens have been placed.
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
            } else {
                // Check that the map contains no tile at this hex location.
                let th =
                    tile_hexes.iter().find(|&(th_addr, _)| addr == th_addr);
                assert!(th.is_none());
            }
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

    #[test]
    fn simple_two_by_two_with_empty_hexes() {
        let hex = Hex::new(HEX_DIAMETER);
        let tiles = crate::catalogue::tile_catalogue(&hex);
        let mut descr = descr_2x2_tiles_5_6_58_63();
        // Remove two of the tiles.
        descr.tiles.insert((0, 1).into(), None);
        descr.tiles.insert((1, 1).into(), None);
        let map = descr.build_map(tiles);

        // Check that there are two empty hexes.
        let empty_iter = map.empty_hex_iter(&hex, hex.context());
        assert_eq!(empty_iter.count(), 2);

        // Check that there are four hexes.
        let hexes: Vec<_> = map.hex_iter(&hex, hex.context()).collect();
        assert_eq!(hexes.len(), 4);

        // Check that there are only two tiles.
        let tile_hexes: Vec<_> =
            map.tile_hex_iter(&hex, hex.context()).collect();
        assert_eq!(tile_hexes.len(), 2);

        // Check that the tiles are at the correct locations.
        for (addr, ts_opt) in hexes.into_iter() {
            if addr.col == 0 {
                assert!(ts_opt.is_some())
            } else {
                assert!(ts_opt.is_none())
            }
        }

        // Also save the map to disk.
        let dx = HEX_DIAMETER * 2.1;
        let dy = HEX_DIAMETER * 2.3;
        let (ctx, surf) = new_context(dx as i32, dy as i32);
        map.draw_tiles(&hex, &ctx);
        let filename = "test-map-descr-simple-2x2_with_empty_hexes.png";
        let mut file = std::fs::File::create(filename)
            .expect("Couldn't create output PNG file");
        surf.write_to_png(&mut file)
            .expect("Couldn't write to output PNG file");
    }
}
