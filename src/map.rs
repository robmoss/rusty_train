use cairo::{Context, FontSlant, FontWeight};
use std::collections::HashMap;

use crate::hex::{Hex, HexColour, HexFace};
use crate::label::Label;
use crate::prelude::PI;
use crate::tile::{Tile, TokenSpace};

pub mod descr;

/// A grid of hexes, each of which may contain a `Tile`.
#[derive(Debug, PartialEq)]
pub struct Map {
    /// All tiles that might be placed on the map.
    tiles: Vec<Tile>,
    /// All tiles, indexed by name.
    catalogue: HashMap<String, usize>,
    /// The map state: which tiles are placed on which hexes.
    state: HashMap<HexAddress, MapHex>,
    /// All hexes on which a tile might be placed.
    hexes: Vec<HexAddress>,
    /// All hexes, stored by key to simplify lookup.
    hexes_tbl: HashMap<HexAddress, ()>,
    /// City labels that apply to map hexes.
    labels_tbl: HashMap<HexAddress, Vec<Label>>,
    pub min_row: usize,
    pub max_row: usize,
    pub min_col: usize,
    pub max_col: usize,
    flat_top: bool,
}

impl Map {
    pub fn tiles(&self) -> &[Tile] {
        self.tiles.as_slice()
    }

    pub fn hexes(&self) -> &[HexAddress] {
        self.hexes.as_slice()
    }

    pub fn default_hex(&self) -> Option<HexAddress> {
        if self.hexes.is_empty() {
            None
        } else {
            Some(self.hexes[0])
        }
    }

    pub fn get_hex(&self, addr: HexAddress) -> Option<&MapHex> {
        self.state.get(&addr)
    }

    pub fn get_hex_mut(&mut self, addr: HexAddress) -> Option<&mut MapHex> {
        self.state.get_mut(&addr)
    }

    /// Returns the map locations where a matching token has been placed.
    pub fn find_placed_tokens(
        &self,
        t: &Token,
    ) -> Vec<(&HexAddress, &TokenSpace)> {
        self.state
            .iter()
            .filter_map(|(addr, state)| {
                state.tokens.iter().find_map(|(token_space, token)| {
                    if t == token {
                        Some((addr, token_space))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Returns the hex face **relative to the map** that corresponds to the
    /// the specified hex face **relative to the tile's orientation**.
    fn map_face_from_tile_face(
        &self,
        addr: HexAddress,
        tile_face: HexFace,
    ) -> Option<HexFace> {
        self.state
            .get(&addr)
            .map(|hs| hs.rotation.count_turns())
            .map(|turns| {
                let mut hex_face = tile_face;
                for _ in 0..turns {
                    // NOTE: turn clockwise
                    hex_face = hex_face.clockwise()
                }
                hex_face
            })
    }

    /// Returns the hex face **relative to the tile's orientation** that
    /// corresponds to the specified hex face **relative to the map**.
    fn tile_face_from_map_face(
        &self,
        addr: HexAddress,
        tile_face: HexFace,
    ) -> Option<HexFace> {
        self.state
            .get(&addr)
            .map(|hs| hs.rotation.count_turns())
            .map(|turns| {
                let mut hex_face = tile_face;
                for _ in 0..turns {
                    // NOTE: turn anti-clockwise
                    hex_face = hex_face.anti_clockwise()
                }
                hex_face
            })
    }

    /// Returns the address of the hex that is adjacent to the specified face
    /// (in terms of map orientation, not tile orientation) of the given tile.
    fn adjacent_address(
        &self,
        addr: HexAddress,
        map_face: HexFace,
    ) -> Option<HexAddress> {
        // NOTE: if the column is even, its upper-left and upper-right sides
        // are adjacent to the row above; if the column is odd then its
        // upper-left and upper-right sides are adjacent to its own row.
        // Similar conditions apply for the lower-right and lower-right sides.
        let is_upper = addr.col % 2 == 0;

        match map_face {
            HexFace::Top => {
                if addr.row > 0 {
                    Some((addr.row - 1, addr.col).into())
                } else {
                    None
                }
            }
            HexFace::UpperRight => {
                if is_upper {
                    if addr.row > 0 {
                        Some((addr.row - 1, addr.col + 1).into())
                    } else {
                        None
                    }
                } else {
                    Some((addr.row, addr.col + 1).into())
                }
            }
            HexFace::LowerRight => {
                if is_upper {
                    Some((addr.row, addr.col + 1).into())
                } else {
                    Some((addr.row + 1, addr.col + 1).into())
                }
            }
            HexFace::Bottom => Some((addr.row + 1, addr.col).into()),
            HexFace::LowerLeft => {
                if addr.col > 0 {
                    if is_upper {
                        Some((addr.row, addr.col - 1).into())
                    } else {
                        Some((addr.row + 1, addr.col - 1).into())
                    }
                } else {
                    None
                }
            }
            HexFace::UpperLeft => {
                if addr.col > 0 {
                    if is_upper {
                        if addr.row > 0 {
                            Some((addr.row - 1, addr.col - 1).into())
                        } else {
                            None
                        }
                    } else {
                        Some((addr.row, addr.col - 1).into())
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Returns details of the tile that is adjacent to the specified face:
    ///
    /// - The address of the adjacent tile;
    /// - The face **relative to this tile's orientation** that is adjacent;
    ///   and
    /// - The tile itself.
    pub fn adjacent_face(
        &self,
        addr: HexAddress,
        tile_face: HexFace,
    ) -> Option<(HexAddress, HexFace, &Tile)> {
        // Determine the actual face (i.e., accounting for tile rotation).
        let map_face = self.map_face_from_tile_face(addr, tile_face);

        if let Some(map_face) = map_face {
            // Determine the address of the adjacent hex.
            let adj_addr = self.adjacent_address(addr, map_face);
            // This will be adjacent to the opposite face on the adjacent hex.
            let adj_tile_face = adj_addr.and_then(|addr| {
                self.tile_face_from_map_face(addr, map_face.opposite())
            });
            let adj_tile = adj_addr.and_then(|addr| self.tile_at(addr));
            // Combine the three option values into a single option tuple.
            adj_addr.and_then(|addr| {
                adj_tile_face
                    .and_then(|face| adj_tile.map(|tile| (addr, face, tile)))
            })
        } else {
            None
        }
    }

    pub fn new(tiles: Vec<Tile>, hexes: Vec<HexAddress>) -> Self {
        if hexes.is_empty() {
            panic!("Can not create map with no hexes")
        }

        let catalogue = tiles
            .iter()
            .enumerate()
            .map(|(ix, t)| (t.name.clone(), ix))
            .collect();
        let state = HashMap::new();
        let hexes_tbl = hexes.iter().map(|addr| (*addr, ())).collect();
        let labels_tbl = HashMap::new();
        let min_col = hexes.iter().map(|hc| hc.col).min().unwrap();
        let max_col = hexes.iter().map(|hc| hc.col).max().unwrap();
        let min_row = hexes.iter().map(|hc| hc.row).min().unwrap();
        let max_row = hexes.iter().map(|hc| hc.row).max().unwrap();
        let flat_top = true;

        Map {
            tiles,
            catalogue,
            state,
            hexes,
            hexes_tbl,
            labels_tbl,
            min_col,
            max_col,
            min_row,
            max_row,
            flat_top,
        }
    }

    pub fn tile_at(&self, hex: HexAddress) -> Option<&Tile> {
        self.state.get(&hex).map(|hs| &self.tiles[hs.tile_ix])
    }

    pub fn place_tile(
        &mut self,
        hex: HexAddress,
        tile: &str,
        rot: RotateCW,
    ) -> bool {
        let tile_ix = if let Some(ix) = self.catalogue.get(tile) {
            *ix
        } else {
            return false;
        };
        if let Some(hex_state) = self.state.get_mut(&hex) {
            if !hex_state.replaceable {
                // This tile cannot be replaced.
                return false;
            }
            // NOTE: leave the tokens as-is!
            // TODO: presumably this is correct behaviour?
            hex_state.tile_ix = tile_ix;
            hex_state.rotation = rot;
        } else {
            self.state.insert(
                hex,
                MapHex {
                    tile_ix: tile_ix,
                    rotation: rot,
                    tokens: HashMap::new(),
                    replaceable: true,
                },
            );
        }
        true
    }

    pub fn remove_tile(&mut self, addr: HexAddress) {
        self.state.remove(&addr);
    }

    pub fn add_label_at(&mut self, addr: HexAddress, label: Label) {
        self.labels_tbl.entry(addr).or_insert(vec![]).push(label)
    }

    pub fn labels_at(&self, addr: HexAddress) -> Option<&Vec<Label>> {
        self.labels_tbl.get(&addr)
    }

    /// Check whether a tile can be placed on an empty hex.
    pub fn can_place_on_empty(&self, addr: HexAddress, tile: &Tile) -> bool {
        // Only first-phase tiles can be placed on an empty hex.
        if Some(tile.colour) != HexColour::Empty.next_phase() {
            return false;
        }
        // An empty hex contains no dits.
        if tile.dit_count() > 0 {
            return false;
        }
        // An empty hex contains no cities or token spaces.
        if tile.token_space_count() > 0 {
            return false;
        }
        // Check that the tile labels are consistent with those of the hex.
        self.can_upgrade_to(addr, tile)
    }

    /// Check whether a tile can be upgraded to another tile.
    pub fn can_upgrade_to(&self, addr: HexAddress, tile: &Tile) -> bool {
        if let Some(hex_labels) = self.labels_tbl.get(&addr) {
            // Check that the tile has one label in common with this hex.
            tile.labels()
                .iter()
                .filter(|(label, _posn)| match label {
                    Label::City(_) => true,
                    Label::Y => true,
                    _ => false,
                })
                .any(|(label, _posn)| hex_labels.contains(label))
        } else {
            // Check that this tile has no City or Y labels.
            tile.labels()
                .iter()
                .filter(|(label, _posn)| match label {
                    Label::City(_) => true,
                    Label::Y => true,
                    _ => false,
                })
                .count()
                == 0
        }
    }

    fn hex_centre(
        &self,
        row: usize,
        col: usize,
        x0: f64,
        y0: f64,
        hex: &Hex,
    ) -> (f64, f64) {
        if row < self.min_row || row > self.max_row {
            panic!("Invalid hex row")
        }
        if col < self.min_col || col > self.max_col {
            panic!("Invalid hex column")
        }
        let row = row - self.min_row;
        let col = col - self.min_col;

        if self.flat_top {
            let x = x0 + (col as f64) * hex.max_d * 0.75;
            let y = if col % 2 == 1 {
                y0 + (row as f64 + 0.5) * hex.min_d
            } else {
                y0 + (row as f64) * hex.min_d
            };
            (x, y)
        } else {
            let x = if row % 2 == 1 {
                x0 + (col as f64 + 0.5) * hex.min_d
            } else {
                x0 + (col as f64) * hex.min_d
            };
            let y = y0 + (row as f64) * hex.max_d * 0.75;
            (x, y)
        }
    }

    fn draw_hex_border(&self, hex: &Hex, ctx: &Context) {
        hex.define_boundary(ctx);
        ctx.set_line_width(hex.max_d * 0.01);
        ctx.stroke();
    }

    pub fn prepare_to_draw(
        &self,
        addr: HexAddress,
        hex: &Hex,
        ctx: &Context,
    ) -> cairo::Matrix {
        let angle = if self.flat_top { 0.0 } else { PI / 6.0 };
        let x0 = if self.flat_top {
            0.5 * hex.max_d + 10.0
        } else {
            0.5 * hex.min_d + 10.0
        };
        let y0 = if self.flat_top {
            0.5 * hex.min_d + 10.0
        } else {
            0.5 * hex.max_d + 10.0
        };

        let (x, y) = self.hex_centre(addr.row, addr.col, x0, y0, hex);

        let m = ctx.get_matrix();
        ctx.translate(x, y);

        if let Some(hex_state) = self.state.get(&addr) {
            ctx.rotate(angle + hex_state.rotation.radians());
        }

        m
    }

    pub fn draw_tiles(&self, hex: &Hex, ctx: &Context) {
        // TODO! This should probably be implemented for a separate structure,
        // since it will involve drawing backgrounds and other details ...
        // Or should this simply draw hexes and let the UI fill in other stuff?
        // If Map doesn't do this, the UI needs to interrogate each HexCoord ...

        let angle = if self.flat_top { 0.0 } else { PI / 6.0 };
        let x0 = if self.flat_top {
            0.5 * hex.max_d + 10.0
        } else {
            0.5 * hex.min_d + 10.0
        };
        let y0 = if self.flat_top {
            0.5 * hex.min_d + 10.0
        } else {
            0.5 * hex.max_d + 10.0
        };

        let m = ctx.get_matrix();

        for r in 0..(self.max_row + 1 - self.min_row) {
            for c in 0..(self.max_col + 1 - self.min_col) {
                let (x, y) = self.hex_centre(r, c, x0, y0, hex);
                ctx.translate(x, y);

                let hex_locn = HexAddress::new(r, c);
                if let Some(hex_state) = self.state.get(&hex_locn) {
                    // Draw this tile.
                    ctx.rotate(angle + hex_state.rotation.radians());
                    let tile = &self.tiles[hex_state.tile_ix];
                    tile.draw(ctx, &hex);
                    // Draw any tokens.
                    for (token_space, map_token) in hex_state.tokens.iter() {
                        tile.define_token_space(&token_space, &hex, ctx);
                        map_token.draw_token(&hex, ctx);
                    }
                } else {
                    // Draw the hex border.
                    ctx.set_source_rgb(0.7, 0.7, 0.7);
                    self.draw_hex_border(&hex, ctx);
                }

                ctx.set_matrix(m);
            }
        }
    }

    /// Iterates over all map hexes.
    ///
    /// At each iteration, the transformation matrix will be updated to
    /// account for the current hex's location and orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rusty_train::prelude::*;
    /// # let hex = Hex::new(125.0);
    /// # let tiles = tile_catalogue(&hex);
    /// # let hexes: Vec<HexAddress> = (0 as usize..4)
    /// #     .map(|r| (0 as usize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles, hexes);
    /// // Draw a thick black border around each hex.
    /// ctx.set_source_rgb(0.0, 0.0, 0.0);
    /// ctx.set_line_width(hex.max_d * 0.05);
    /// for (_addr, _tile_opt) in map.hex_iter(&hex, ctx) {
    ///     hex.define_boundary(ctx);
    ///     ctx.stroke();
    /// }
    /// ```
    pub fn hex_iter<'a>(
        &'a self,
        hex: &'a Hex,
        ctx: &'a Context,
    ) -> HexIter<'a> {
        HexIter::new(hex, ctx, self)
    }

    /// Iterates over all map hexes that do not contain a tile.
    ///
    /// At each iteration, the transformation matrix will be updated to
    /// account for the current hex's location and orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rusty_train::prelude::*;
    /// # let hex = Hex::new(125.0);
    /// # let tiles = tile_catalogue(&hex);
    /// # let hexes: Vec<HexAddress> = (0 as usize..4)
    /// #     .map(|r| (0 as usize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles, hexes);
    /// // Fill each empty tile with a dark grey.
    /// ctx.set_source_rgb(0.4, 0.4, 0.4);
    /// for _addr in map.empty_hex_iter(&hex, ctx) {
    ///     hex.define_boundary(ctx);
    ///     ctx.fill();
    /// }
    /// ```
    pub fn empty_hex_iter<'a>(
        &'a self,
        hex: &'a Hex,
        ctx: &'a Context,
    ) -> EmptyHexIter<'a> {
        HexIter::new(hex, ctx, self).into()
    }

    /// Iterates over all map hexes that contain a tile.
    ///
    /// At each iteration, the transformation matrix will be updated to
    /// account for the current hex's location and orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rusty_train::prelude::*;
    /// # let hex = Hex::new(125.0);
    /// # let tiles = tile_catalogue(&hex);
    /// # let hexes: Vec<HexAddress> = (0 as usize..4)
    /// #     .map(|r| (0 as usize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles, hexes);
    /// // Draw a red border around each token space.
    /// ctx.set_source_rgb(0.8, 0.2, 0.2);
    /// ctx.set_line_width(hex.max_d * 0.015);
    /// for (_addr, (tile, _tokens)) in map.tile_hex_iter(&hex, ctx) {
    ///     for token_space in tile.token_spaces() {
    ///         tile.define_token_space(&token_space, &hex, ctx);
    ///         ctx.stroke();
    ///     }
    /// }
    /// ```
    pub fn tile_hex_iter<'a>(
        &'a self,
        hex: &'a Hex,
        ctx: &'a Context,
    ) -> TileHexIter<'a> {
        HexIter::new(hex, ctx, self).into()
    }

    pub fn next_col(&self, mut addr: HexAddress) -> HexAddress {
        addr.col += 1;
        if !self.hexes_tbl.contains_key(&addr) {
            addr.col -= 1;
        }
        addr
    }

    pub fn prev_col(&self, mut addr: HexAddress) -> HexAddress {
        if addr.col == 0 {
            return addr;
        }
        addr.col -= 1;
        if !self.hexes_tbl.contains_key(&addr) {
            addr.col += 1;
        }
        addr
    }

    pub fn next_row(&self, mut addr: HexAddress) -> HexAddress {
        addr.row += 1;
        if !self.hexes_tbl.contains_key(&addr) {
            addr.row -= 1;
        }
        addr
    }

    pub fn prev_row(&self, mut addr: HexAddress) -> HexAddress {
        if addr.row == 0 {
            return addr;
        }
        addr.row -= 1;
        if !self.hexes_tbl.contains_key(&addr) {
            addr.row += 1;
        }
        addr
    }

    // TODO: define methods so that we can replace Map in main.rs.
    // TODO: rotate_tile_{cw|anti_cw}
    // TODO: upgrade_candidates()
    // TODO: get_tokens(), set_tokens(), get_token(), set_token()
    // TODO: translate_to_hex()
}

/// The state of each token space on a tile.
pub type TokensTable = HashMap<TokenSpace, Token>;

/// The state of a tile on the map.
pub type TileState<'a> = (&'a Tile, &'a TokensTable);

/// An iterator over each hex in a `Map`.
pub struct HexIter<'a> {
    hex: &'a Hex,
    ctx: &'a Context,
    map: &'a Map,
    x0: f64,
    y0: f64,
    angle: f64,
    ix: usize,
    m: cairo::Matrix,
}

impl<'a> HexIter<'a> {
    pub fn reset_matrix(&self) {
        self.ctx.set_matrix(self.m)
    }

    pub fn restart(&mut self) {
        self.ctx.set_matrix(self.m);
        self.ix = 0;
    }

    fn new(hex: &'a Hex, ctx: &'a Context, map: &'a Map) -> Self {
        let angle = if map.flat_top { 0.0 } else { PI / 6.0 };
        let x0 = if map.flat_top {
            0.5 * hex.max_d + 10.0
        } else {
            0.5 * hex.min_d + 10.0
        };
        let y0 = if map.flat_top {
            0.5 * hex.min_d + 10.0
        } else {
            0.5 * hex.max_d + 10.0
        };

        Self {
            hex,
            ctx,
            map,
            x0,
            y0,
            angle,
            ix: 0,
            m: ctx.get_matrix(),
        }
    }

    fn hex_centre(&self, addr: HexAddress) -> Option<(f64, f64)> {
        let row = addr.row;
        let col = addr.col;

        // NOTE: currently assuming that 0 marks the first row/column.
        // if row < self.min_row || row > self.max_row {
        //     return None;
        // }
        // if col < self.min_col || col > self.max_col {
        //     return None;
        // }
        // let row = row - self.min_row;
        // let col = col - self.min_col;

        if self.map.flat_top {
            let x = self.x0 + (col as f64) * self.hex.max_d * 0.75;
            let y = if col % 2 == 1 {
                self.y0 + (row as f64 + 0.5) * self.hex.min_d
            } else {
                self.y0 + (row as f64) * self.hex.min_d
            };
            Some((x, y))
        } else {
            let x = if row % 2 == 1 {
                self.x0 + (col as f64 + 0.5) * self.hex.min_d
            } else {
                self.x0 + (col as f64) * self.hex.min_d
            };
            let y = self.y0 + (row as f64) * self.hex.max_d * 0.75;
            Some((x, y))
        }
    }
}

impl<'a> Iterator for HexIter<'a> {
    type Item = (HexAddress, Option<TileState<'a>>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ix >= self.map.hexes.len() {
            // NOTE: restore the original matrix.
            self.ctx.set_matrix(self.m);
            return None;
        }
        let addr = self.map.hexes[self.ix];
        self.ix += 1;

        let (x, y) = if let Some((x, y)) = self.hex_centre(addr) {
            (x, y)
        } else {
            // NOTE: restore the original matrix.
            self.ctx.set_matrix(self.m);
            return None;
        };

        self.ctx.set_matrix(self.m);
        self.ctx.translate(x, y);

        if let Some(hex_state) = self.map.state.get(&addr) {
            self.ctx.rotate(self.angle + hex_state.rotation.radians());
            let tile =
                Some((&self.map.tiles[hex_state.tile_ix], &hex_state.tokens));
            Some((addr, tile))
        } else {
            Some((addr, None))
        }
    }
}

/// An iterator over each hex in a `Map` that does not contain a `Tile`.
pub struct EmptyHexIter<'a> {
    iter: HexIter<'a>,
}

impl<'a> EmptyHexIter<'a> {
    fn new(iter: HexIter<'a>) -> Self {
        EmptyHexIter { iter }
    }

    pub fn reset_matrix(&self) {
        self.iter.reset_matrix()
    }

    pub fn restart(&mut self) {
        self.iter.restart()
    }
}

impl<'a> Iterator for EmptyHexIter<'a> {
    type Item = HexAddress;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.iter.next();
        while let Some((addr, tile)) = item {
            if tile == None {
                return Some(addr);
            }
            item = self.iter.next();
        }
        None
    }
}

impl<'a> From<HexIter<'a>> for EmptyHexIter<'a> {
    fn from(src: HexIter<'a>) -> Self {
        Self::new(src)
    }
}

/// An iterator over each hex in a `Map` that contains a `Tile`.
pub struct TileHexIter<'a> {
    iter: HexIter<'a>,
}

impl<'a> TileHexIter<'a> {
    fn new(iter: HexIter<'a>) -> Self {
        TileHexIter { iter }
    }

    pub fn reset_matrix(&self) {
        self.iter.reset_matrix()
    }

    pub fn restart(&mut self) {
        self.iter.restart()
    }
}

impl<'a> Iterator for TileHexIter<'a> {
    type Item = (HexAddress, TileState<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.iter.next();
        while let Some((addr, tile_opt)) = item {
            if let Some(tile) = tile_opt {
                return Some((addr, tile));
            }
            item = self.iter.next();
        }
        None
    }
}

impl<'a> From<HexIter<'a>> for TileHexIter<'a> {
    fn from(src: HexIter<'a>) -> Self {
        Self::new(src)
    }
}

/// The rotation of a `Tile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotateCW {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
}

impl RotateCW {
    pub fn radians(&self) -> f64 {
        use RotateCW::*;

        match self {
            Zero => 0.0,
            One => crate::prelude::PI_2_6,
            Two => crate::prelude::PI_4_6,
            Three => crate::prelude::PI,
            Four => -crate::prelude::PI_4_6,
            Five => -crate::prelude::PI_2_6,
        }
    }

    /// Returns the number of single clock-wise rotations that are equivalent
    /// to this rotation value.
    pub fn count_turns(&self) -> usize {
        use RotateCW::*;

        match self {
            Zero => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
        }
    }

    pub fn rotate_cw(&self) -> Self {
        use RotateCW::*;

        match self {
            Zero => One,
            One => Two,
            Two => Three,
            Three => Four,
            Four => Five,
            Five => Zero,
        }
    }

    pub fn rotate_anti_cw(&self) -> Self {
        use RotateCW::*;

        match self {
            Zero => Five,
            One => Zero,
            Two => One,
            Three => Two,
            Four => Three,
            Five => Four,
        }
    }
}

/// The state of a hex in a `Map`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapHex {
    tile_ix: usize,
    rotation: RotateCW,
    tokens: TokensTable,
    /// Whether this tile can be replaced by another tile; set to false for
    /// hexes such as the red off-board areas.
    replaceable: bool,
}

impl MapHex {
    pub fn tile<'a>(&self, map: &'a Map) -> &'a Tile {
        &map.tiles[self.tile_ix]
    }

    pub fn rotate_anti_cw(&mut self) {
        self.rotation = self.rotation.rotate_anti_cw()
    }

    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.rotate_cw()
    }

    pub fn radians(&self) -> f64 {
        self.rotation.radians()
    }

    pub fn rotation(&self) -> &RotateCW {
        &self.rotation
    }

    pub fn get_token_at(&self, space: &TokenSpace) -> Option<&Token> {
        self.tokens.get(space)
    }

    pub fn set_token_at(&mut self, space: &TokenSpace, token: Token) {
        self.tokens.insert(*space, token);
    }

    pub fn remove_token_at(&mut self, space: &TokenSpace) {
        self.tokens.remove(space);
    }

    pub fn get_tokens(&self) -> &TokensTable {
        &self.tokens
    }

    pub fn set_tokens(&mut self, tokens: TokensTable) {
        self.tokens = tokens
    }
}

/// A token that may occupy a token space on a `Tile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    LP,
    PO,
    MK,
    N,
}

impl Token {
    pub fn text(&self) -> &str {
        use Token::*;

        match self {
            LP => "LP",
            PO => "PO",
            MK => "MK",
            N => "N",
        }
    }

    fn set_bg(&self, ctx: &Context) {
        use Token::*;

        match self {
            LP => ctx.set_source_rgb(1.0, 0.5, 0.5),
            PO => ctx.set_source_rgb(0.5, 1.0, 0.5),
            MK => ctx.set_source_rgb(0.5, 1.0, 1.0),
            N => ctx.set_source_rgb(1.0, 0.5, 1.0),
        }
    }

    pub fn draw_token(&self, hex: &Hex, ctx: &Context) {
        let text = self.text();
        self.set_bg(ctx);

        let (x0, y0, x1, y1) = ctx.fill_extents();
        let x = 0.5 * (x0 + x1);
        let y = 0.5 * (y0 + y1);
        ctx.fill_preserve();

        // Draw background elements.
        let stroke_path = ctx.copy_path();
        ctx.save();
        ctx.clip_preserve();
        let radius = hex.max_d * 0.125;
        ctx.set_source_rgb(0.25, 0.6, 0.6);
        ctx.new_path();
        ctx.arc(x - 1.5 * radius, y, 1.0 * radius, 0.0, 2.0 * PI);
        ctx.arc(x + 1.5 * radius, y, 1.0 * radius, 0.0, 2.0 * PI);
        ctx.fill();
        ctx.restore();

        // Redraw the outer black circle.
        ctx.new_path();
        ctx.append_path(&stroke_path);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.01);
        ctx.stroke_preserve();

        // Draw the token label.
        ctx.select_font_face("Serif", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(10.0);
        let exts = ctx.text_extents(text);
        let x = x - 0.5 * exts.width;
        let y = y + 0.5 * exts.height;
        ctx.move_to(x, y);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(text);
    }

    pub fn next(&self) -> Self {
        use Token::*;

        match self {
            LP => PO,
            PO => MK,
            MK => N,
            N => LP,
        }
    }

    pub fn prev(&self) -> Self {
        use Token::*;

        match self {
            LP => N,
            PO => LP,
            MK => PO,
            N => MK,
        }
    }

    pub fn first() -> Self {
        Token::LP
    }

    pub fn last() -> Self {
        Token::N
    }
}

/// A hex location on a `Map`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HexAddress {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

impl HexAddress {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl std::fmt::Display for HexAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alphabet: Vec<_> = (b'A'..=b'Z').map(|b| b as char).collect();
        let row_letter = alphabet[self.row];
        write!(f, "{}{}", row_letter, self.col)
    }
}

impl From<(usize, usize)> for HexAddress {
    fn from(src: (usize, usize)) -> Self {
        let (row, col) = src;
        Self { row, col }
    }
}

impl From<&(usize, usize)> for HexAddress {
    fn from(src: &(usize, usize)) -> Self {
        let (row, col) = src;
        Self {
            row: *row,
            col: *col,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HexAddress;
    use crate::connection::Connection;
    use crate::hex::Hex;
    use crate::track::TrackEnd;

    static HEX_DIAMETER: f64 = 150.0;

    #[test]
    fn test_simple_two_by_two() {
        let hex = Hex::new(HEX_DIAMETER);
        let map = super::descr::tests::map_2x2_tiles_5_6_58_63(&hex);

        // NOTE: iterate over starting connection and, for each, check that it
        // has the expected connections, and only the expected connections.
        let starts = vec![
            (HexAddress::new(0, 0), Connection::City { ix: 0 }),
            (HexAddress::new(0, 1), Connection::City { ix: 0 }),
            (HexAddress::new(1, 1), Connection::City { ix: 0 }),
        ];

        for (ix, (addr, conn)) in starts.iter().enumerate() {
            let tile = map.tile_at(*addr).expect("No tile found");
            let conns = tile.connections(conn);
            assert!(conns.is_some());
            let conns = conns.unwrap();

            // Check that each city has the expected number of connections.
            if ix == 0 {
                assert_eq!(conns.len(), 2);
            } else if ix == 1 {
                assert_eq!(conns.len(), 2);
            } else if ix == 2 {
                assert_eq!(conns.len(), 6);
            }

            // Check that each city is connected to the end of a different
            // track segment.
            for j in 0..conns.len() {
                assert!(conns
                    .iter()
                    .find(|&&c| c
                        == Connection::Track {
                            ix: j,
                            end: TrackEnd::End
                        })
                    .is_some());
            }

            // Check that the other end of each track segment is connected to
            // a hex face, and to nothing else.
            // Also check that each tile has two track segments that are
            // connected to track segments on other tiles.
            let mut track_to_track = 0;
            for track_conn in conns {
                let other_end = track_conn.other_end().expect("No other end");
                let end_conns = tile.connections(&other_end);
                assert!(end_conns.is_some());
                let end_conns = end_conns.unwrap();
                assert_eq!(end_conns.len(), 1);
                match end_conns[0] {
                    Connection::Face { face } => {
                        // Check if this segment is connected to another tile.
                        // If so, it should be connected to a track segment.
                        let adj = map.adjacent_face(*addr, face);
                        if let Some((_addr, face, tile)) = adj {
                            let from = Connection::Face { face };
                            let conns = tile.connections(&from);
                            assert!(conns.is_some());
                            let conns = conns.unwrap();
                            assert_eq!(conns.len(), 1);
                            let conn = conns[0];
                            if let Connection::Track { ix: _, end: _ } = conn
                            {
                                track_to_track += 1;
                            } else {
                                panic!(
                                    "Invalid adjacent connection {:?}",
                                    conn
                                )
                            }
                        }
                    }
                    _ => {
                        panic!("Invalid track connection: {:?}", end_conns[0])
                    }
                }
            }
            // Verify that each tile has two track segments that are connected
            // to track segments on other tiles.
            assert_eq!(track_to_track, 2);
        }
    }
}
