//! Describe and create map configurations.

use std::collections::BTreeMap;

use crate::map::{HexAddress, Map, MapTile};
use n18hex::RotateCW;
use n18tile::Tile;
use n18token::Tokens;

/// A description of a tile's configuration on a map hex.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileDescr {
    /// The map row in which the tile is located.
    pub row: usize,
    /// The map column in which the tile is located.
    pub col: usize,
    /// The tile name.
    pub tile: String,
    /// The tile rotation.
    pub rotation: RotateCW,
    /// Token spaces are identified by index into `Tile::token_spaces()`.
    pub tokens: Vec<(usize, String)>,
}

/// A description of each tile's configuration on a map.
pub struct Descr {
    tiles: BTreeMap<HexAddress, Option<TileDescr>>,
}

impl<'a> From<&'a Descr> for &'a BTreeMap<HexAddress, Option<TileDescr>> {
    fn from(src: &'a Descr) -> Self {
        &src.tiles
    }
}

impl<'a> From<BTreeMap<HexAddress, Option<TileDescr>>> for Descr {
    fn from(src: BTreeMap<HexAddress, Option<TileDescr>>) -> Self {
        Self { tiles: src }
    }
}

/// Constructs a map configuration from a vector of tile descriptions.
impl From<Vec<TileDescr>> for Descr {
    fn from(src: Vec<TileDescr>) -> Descr {
        Descr {
            tiles: src
                .into_iter()
                .map(|td| ((td.row, td.col).into(), Some(td)))
                .collect(),
        }
    }
}

impl From<(&Map, HexAddress, &MapTile)> for TileDescr {
    fn from(src: (&Map, HexAddress, &MapTile)) -> TileDescr {
        let map = src.0;
        let addr = src.1;
        let map_hex = src.2;
        let tile = map_hex.tile(map);
        let token_spaces = tile.token_spaces();
        let token_table = map_hex.tokens();
        let tokens: Vec<_> = token_table
            .iter()
            .map(|(token_space, token)| {
                let name = map.token_name(token);
                let ix = token_spaces
                    .iter()
                    .position(|ts| ts == token_space)
                    .unwrap();
                (ix, name.to_string())
            })
            .collect();
        TileDescr {
            row: addr.row,
            col: addr.col,
            tile: tile.name.clone(),
            rotation: *map_hex.rotation(),
            tokens,
        }
    }
}

/// Describes the current state of an existing map.
impl From<&Map> for Descr {
    fn from(map: &Map) -> Descr {
        let tile_descrs = map
            .hex_address_iter()
            .map(|addr| (*addr, map.hex_state(*addr)))
            .map(|(addr, map_hex)| {
                let tile_opt: Option<TileDescr> =
                    map_hex.map(|mh| (map, addr, mh).into());
                (addr, tile_opt)
            })
            .collect();
        Descr { tiles: tile_descrs }
    }
}

impl Descr {
    /// Constructs a map whose state reflects the tile configurations.
    pub fn build_map(&self, tiles: Vec<Tile>, tokens: Tokens) -> Map {
        let addrs = self.tiles.keys().copied().collect::<Vec<_>>();
        let mut map = Map::new(tiles.into(), tokens, addrs);
        self.update_map(&mut map);
        map
    }

    /// Updates the state of an existing map.
    pub fn update_map(&self, map: &mut Map) {
        for (addr, tile_descr) in self.tiles.iter() {
            if let Some(tile_descr) = tile_descr {
                map.place_tile(
                    *addr,
                    tile_descr.tile.as_str(),
                    tile_descr.rotation,
                );
                let spaces = {
                    let tile = map.tile_at(*addr).expect("No tile");
                    tile.token_spaces()
                };
                // NOTE: we need to retrieve each token by name before we get
                // a mutable reference to the hex state, because looking up
                // tokens requires us to borrow map as immutable.
                let tile_tokens: Vec<_> = tile_descr
                    .tokens
                    .iter()
                    .map(|(space_ix, token_name)| {
                        let token_opt = map.try_token(token_name);
                        if token_opt.is_none() {
                            eprintln!("No token for '{}'", token_name);
                            eprintln!("Token names: {:?}", map.token_names());
                        }
                        let token = token_opt.unwrap_or_else(|| {
                            panic!("No token for '{}'", token_name)
                        });
                        (space_ix, token)
                    })
                    .collect();
                let hex_state =
                    map.hex_state_mut(*addr).expect("No hex state");
                for (space_ix, token) in tile_tokens {
                    hex_state.set_token_at(&spaces[*space_ix], token);
                }
            } else {
                // Ensure that no tiles occupy empty hexes.
                map.remove_tile(*addr);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use cairo::{Context, Format, ImageSurface};

    use super::*;
    use crate::map::TileHexState;

    use n18hex::Hex;
    use n18token::Token;

    static HEX_DIAMETER: f64 = 150.0;

    static OUT_DIR: &str = "../../tests/output";

    fn output_path(file: &'static str) -> std::path::PathBuf {
        std::path::Path::new(OUT_DIR).join(file)
    }

    fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
        let surface = ImageSurface::create(Format::ARgb32, width, height)
            .expect("Can't create surface");
        let context =
            Context::new(&surface).expect("Can't create cairo::Surface");
        (context, surface)
    }

    fn draw_tiles(map: &Map, hex: &Hex, ctx: &Context) {
        for hex_state in map.hex_iter(hex, ctx) {
            match hex_state.tile_state {
                Some((tile, tokens_table)) => {
                    tile.draw(ctx, hex);
                    for (token_space, token) in tokens_table.iter() {
                        tile.define_token_space(token_space, hex, ctx);
                        let rotn = hex_state.tile_rotation;
                        let token_name = map.token_name(token);
                        token.draw(hex, ctx, token_name, rotn);
                    }
                }
                None => {
                    // Draw a border around this hex.
                    hex.define_boundary(ctx);
                    hex.theme.hex_border.apply_line_and_stroke(ctx, hex);
                    ctx.stroke().unwrap();
                }
            }
        }
    }

    /// Define the tokens used in the following test cases.
    fn define_tokens() -> Tokens {
        use n18token::TokenStyle;

        vec![
            (
                "LP".to_string(),
                Token::new(TokenStyle::SideArcs {
                    fg: (63, 153, 153).into(),
                    bg: (255, 127, 127).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
            (
                "PO".to_string(),
                Token::new(TokenStyle::SideArcs {
                    fg: (63, 153, 153).into(),
                    bg: (127, 255, 127).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
        ]
        .into()
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
    pub fn map_2x2_tiles_5_6_58_63() -> Map {
        let tiles = n18catalogue::tile_catalogue();
        let tokens = define_tokens();
        let descr = descr_2x2_tiles_5_6_58_63();
        descr.build_map(tiles, tokens)
    }

    /// Defines the map that should be created by `map_2x2_tiles_5_6_58_63`.
    fn descr_2x2_tiles_5_6_58_63() -> Descr {
        vec![
            TileDescr {
                row: 0,
                col: 0,
                tile: "5".to_string(),
                rotation: RotateCW::Zero,
                tokens: vec![(0, "LP".to_string())],
            },
            TileDescr {
                row: 0,
                col: 1,
                tile: "6".to_string(),
                rotation: RotateCW::Two,
                tokens: vec![(0, "PO".to_string())],
            },
            TileDescr {
                row: 1,
                col: 0,
                tile: "58".to_string(),
                rotation: RotateCW::Five,
                tokens: vec![],
            },
            TileDescr {
                row: 1,
                col: 1,
                tile: "63".to_string(),
                rotation: RotateCW::Zero,
                tokens: vec![(0, "PO".to_string()), (1, "LP".to_string())],
            },
        ]
        .into()
    }

    #[test]
    fn simple_two_by_two() {
        let hex = Hex::new(HEX_DIAMETER);
        let map = map_2x2_tiles_5_6_58_63();
        let tokens = define_tokens();

        // NOTE: check the three hex iterators to ensure they all yield the
        // expected map configuration.

        // Check that there are no empty hexes.
        let empty_iter = map.empty_hex_iter(&hex, hex.context());
        assert_eq!(empty_iter.count(), 0);

        // Check that there are four hexes.
        let hexes: Vec<_> = map.hex_iter(&hex, hex.context()).collect();
        assert_eq!(hexes.len(), 4);
        // Check that all four hexes contain tiles.
        assert!(hexes.iter().all(|hex_state| hex_state.tile_state.is_some()));

        // Check (again) that there are four tiles.
        let tile_hexes: Vec<_> =
            map.tile_hex_iter(&hex, hex.context()).collect();
        assert_eq!(tile_hexes.len(), 4);

        // Check that the same tiles are reported to be at the same locations
        // according to Map::hex_iter() and Map::tile_hex_iter().
        for hex_state in hexes.into_iter() {
            let (tile, tok_mgr) = hex_state.tile_state.unwrap();
            let tile_hex_state = TileHexState {
                addr: hex_state.addr,
                tile,
                tile_tokens: tok_mgr,
                available_tokens: hex_state.available_tokens,
                tile_rotation: hex_state.tile_rotation,
            };
            assert!(tile_hexes.iter().any(|th| th == &tile_hex_state))
        }

        // Check the hex location, rotation, and tokens for each tile.
        let descr = descr_2x2_tiles_5_6_58_63();
        for (addr, tile_descr) in descr.tiles.iter() {
            if let Some(tile_descr) = tile_descr {
                // Check that the map contains a tile at this hex location.
                let th = tile_hexes
                    .iter()
                    .find(|tile_hex_state| addr == &tile_hex_state.addr);
                assert!(th.is_some());

                // Check that tile names match.
                let tile_hex_state = th.unwrap();
                assert_eq!(tile_descr.tile, tile_hex_state.tile.name);

                // Check that all of the tokens are placed correctly, and that
                // no additional tokens have been placed.
                assert_eq!(
                    tile_hex_state.tile_tokens.len(),
                    tile_descr.tokens.len()
                );
                let token_spaces = tile_hex_state.tile.token_spaces();
                for (ix, token_name) in &tile_descr.tokens {
                    let token_opt = tokens.token(token_name);
                    let token_space = token_spaces[*ix];
                    assert_eq!(
                        tile_hex_state.tile_tokens.get(&token_space),
                        token_opt,
                    );
                }

                // Check that the tile rotations match.
                let hex_state = map.hex_state(*addr);
                assert!(hex_state.is_some());
                let rot = hex_state.unwrap().rotation();
                assert_eq!(rot, &tile_descr.rotation);
            } else {
                // Check that the map contains no tile at this hex location.
                let th = tile_hexes
                    .iter()
                    .find(|tile_hex_state| addr == &tile_hex_state.addr);
                assert!(th.is_none());
            }
        }

        // Also save the map to disk.
        let dx = HEX_DIAMETER * 2.1;
        let dy = HEX_DIAMETER * 2.3;
        let (ctx, surf) = new_context(dx as i32, dy as i32);
        draw_tiles(&map, &hex, &ctx);
        let filename = output_path("test-map-descr-simple-2x2.png");
        let mut file = std::fs::File::create(filename)
            .expect("Couldn't create output PNG file");
        surf.write_to_png(&mut file)
            .expect("Couldn't write to output PNG file");
    }

    #[test]
    fn simple_two_by_two_with_empty_hexes() {
        let hex = Hex::new(HEX_DIAMETER);
        let tiles = n18catalogue::tile_catalogue();
        let tokens = define_tokens();
        let mut descr = descr_2x2_tiles_5_6_58_63();
        // Remove two of the tiles.
        descr.tiles.insert((0, 1).into(), None);
        descr.tiles.insert((1, 1).into(), None);
        let map = descr.build_map(tiles, tokens);

        // Check that there are two empty hexes.
        let empty_iter = map.empty_hex_iter(&hex, hex.context());
        assert_eq!(empty_iter.count(), 2);

        // Check that there are four hexes.
        let hexes: Vec<_> = map.hex_iter(&hex, hex.context()).collect();
        assert_eq!(hexes.len(), 4);

        // Check that there are only two tiles.
        let tile_count = map.tile_hex_iter(&hex, hex.context()).count();
        assert_eq!(tile_count, 2);

        // Check that the tiles are at the correct locations.
        for hex_state in hexes.into_iter() {
            if hex_state.addr.col == 0 {
                assert!(hex_state.tile_state.is_some())
            } else {
                assert!(hex_state.tile_state.is_none())
            }
        }

        // Also save the map to disk.
        let dx = HEX_DIAMETER * 2.1;
        let dy = HEX_DIAMETER * 2.3;
        let (ctx, surf) = new_context(dx as i32, dy as i32);
        draw_tiles(&map, &hex, &ctx);
        let filename =
            output_path("test-map-descr-simple-2x2_with_empty_hexes.png");
        let mut file = std::fs::File::create(filename)
            .expect("Couldn't create output PNG file");
        surf.write_to_png(&mut file)
            .expect("Couldn't write to output PNG file");
    }
}
