use cairo::Context;
use std::collections::HashMap;

use n18hex::{Hex, HexColour, HexFace, PI};
use n18tile::{Label, Tile, TokenSpace};
use n18token::{Token, Tokens};

/// A grid of hexes, each of which may contain a `Tile`.
#[derive(Debug, PartialEq)]
pub struct Map {
    /// The tokens that might be placed on the map.
    tokens: Tokens,
    /// Barriers across which track cannot be built, or for which there is an
    /// additional cost (e.g., rivers).
    barriers: Vec<(HexAddress, HexFace)>,
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

// TODO: map::for_1867() -> Map
// - need support for read-only tiles
// - extra route logic (e.g., Timmins): F: Fn(&Route, usize) -> usize ???
//   (https://stackoverflow.com/a/54182204)

impl Map {
    pub fn tiles(&self) -> &[Tile] {
        self.tiles.as_slice()
    }

    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn barriers(&self) -> &[(HexAddress, HexFace)] {
        self.barriers.as_slice()
    }

    pub fn add_barrier(&mut self, addr: HexAddress, face: HexFace) {
        self.barriers.push((addr, face))
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

    // TODO: replace with methods that retrieve specific details?
    // Tokens, Rotation, Tile name, Replaceable ...

    pub fn get_hex(&self, addr: HexAddress) -> Option<&MapHex> {
        self.state.get(&addr)
    }

    // TODO: replace with methods that replace/update specific details?
    // Tokens, Rotation, Tile name, Replaceable ...

    pub fn get_hex_mut(&mut self, addr: HexAddress) -> Option<&mut MapHex> {
        self.state.get_mut(&addr)
    }

    /// Returns the map locations where a matching token has been placed.
    pub fn find_placed_tokens(
        &self,
        t: &Token,
    ) -> Vec<(&HexAddress, &TokenSpace)> {
        // NOTE: a tile may have multiple token spaces that are not connected
        // to each other, so we need to check each of these spaces for a
        // matching token.
        let mut placed: Vec<(&HexAddress, &TokenSpace)> = vec![];
        self.state.iter().for_each(|(addr, state)| {
            state.tokens.iter().for_each(|(token_space, token)| {
                if t == token {
                    placed.push((addr, token_space))
                }
            })
        });
        placed
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

    pub fn new(
        tiles: Vec<Tile>,
        tokens: Tokens,
        hexes: Vec<HexAddress>,
    ) -> Self {
        if hexes.is_empty() {
            panic!("Can not create map with no hexes")
        }

        let barriers = vec![];
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
            tokens,
            tiles,
            barriers,
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
        // TODO: these need to be drawn!?!
        // Or will there always be a tile on this hex?
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
    ) -> Option<(f64, f64)> {
        if row < self.min_row || row > self.max_row {
            return None;
        }
        if col < self.min_col || col > self.max_col {
            return None;
        }
        let row = row - self.min_row;
        let col = col - self.min_col;

        if self.flat_top {
            let x = x0 + (col as f64) * hex.max_d * 0.75;
            let y = if (col + self.min_col) % 2 == 1 {
                y0 + (row as f64 + 0.5) * hex.min_d
            } else {
                y0 + (row as f64) * hex.min_d
            };
            Some((x, y))
        } else {
            let x = if (row + self.min_row) % 2 == 1 {
                x0 + (col as f64 + 0.5) * hex.min_d
            } else {
                x0 + (col as f64) * hex.min_d
            };
            let y = y0 + (row as f64) * hex.max_d * 0.75;
            Some((x, y))
        }
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

        let (x, y) = self
            .hex_centre(addr.row, addr.col, x0, y0, hex)
            .expect(&format!("Invalid hex: {}", addr));

        let m = ctx.get_matrix();
        ctx.translate(x, y);

        if let Some(hex_state) = self.state.get(&addr) {
            ctx.rotate(angle + hex_state.rotation.radians());
        }

        m
    }

    /// Iterates over all map hexes.
    ///
    /// At each iteration, the transformation matrix will be updated to
    /// account for the current hex's location and orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use n18hex::*;
    /// # use n18map::*;
    /// # use n18token::*;
    /// # use n18catalogue::tile_catalogue;
    /// # let hex = Hex::new(125.0);
    /// # let tiles = tile_catalogue(&hex);
    /// # let tokens = vec![].into();
    /// # let hexes: Vec<HexAddress> = (0 as usize..4)
    /// #     .map(|r| (0 as usize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles, tokens, hexes);
    /// // Draw a thick black border around each hex.
    /// ctx.set_source_rgb(0.0, 0.0, 0.0);
    /// ctx.set_line_width(hex.max_d * 0.05);
    /// for h_state in map.hex_iter(&hex, ctx) {
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

    pub fn hex_subset_iter<'a, P: FnMut(&HexAddress) -> bool>(
        &'a self,
        hex: &'a Hex,
        ctx: &'a Context,
        mut include: P,
    ) -> HexIter<'a> {
        let incl: Vec<bool> =
            self.hexes.iter().map(|addr| include(addr)).collect();
        HexIter::new_subset(hex, ctx, self, incl)
    }

    /// Iterates over all map hexes that do not contain a tile.
    ///
    /// At each iteration, the transformation matrix will be updated to
    /// account for the current hex's location and orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use n18hex::*;
    /// # use n18map::*;
    /// # use n18token::*;
    /// # use n18catalogue::tile_catalogue;
    /// # let hex = Hex::new(125.0);
    /// # let tiles = tile_catalogue(&hex);
    /// # let tokens = vec![].into();
    /// # let hexes: Vec<HexAddress> = (0 as usize..4)
    /// #     .map(|r| (0 as usize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles, tokens, hexes);
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
    /// # use n18hex::*;
    /// # use n18tile::*;
    /// # use n18map::*;
    /// # use n18token::*;
    /// # use n18catalogue::tile_catalogue;
    /// # let hex = Hex::new(125.0);
    /// # let tiles = tile_catalogue(&hex);
    /// # let tokens = vec![].into();
    /// # let hexes: Vec<HexAddress> = (0 as usize..4)
    /// #     .map(|r| (0 as usize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles, tokens, hexes);
    /// // Draw a red border around each token space.
    /// ctx.set_source_rgb(0.8, 0.2, 0.2);
    /// ctx.set_line_width(hex.max_d * 0.015);
    /// for th in map.tile_hex_iter(&hex, ctx) {
    ///     for token_space in th.tile.token_spaces() {
    ///         th.tile.define_token_space(&token_space, &hex, ctx);
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
            // TODO: keep searching (i.e., jump over holes)?
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
            // TODO: keep searching (i.e., jump over holes)?
            addr.col += 1;
        }
        addr
    }

    pub fn next_row(&self, mut addr: HexAddress) -> HexAddress {
        addr.row += 1;
        if !self.hexes_tbl.contains_key(&addr) {
            // TODO: keep searching (i.e., jump over holes)?
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
            // TODO: keep searching (i.e., jump over holes)?
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
    include: Vec<bool>,
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
        let include = vec![true; map.hexes.len()];
        Self::new_subset(hex, ctx, map, include)
    }

    fn new_subset(
        hex: &'a Hex,
        ctx: &'a Context,
        map: &'a Map,
        include: Vec<bool>,
    ) -> Self {
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
            include,
        }
    }

    fn hex_centre(&self, addr: HexAddress) -> Option<(f64, f64)> {
        self.map
            .hex_centre(addr.row, addr.col, self.x0, self.y0, self.hex)
    }
}

/// The state of a map hex that may, or may not, contain a tile.
pub struct HexState<'a> {
    pub addr: HexAddress,
    pub tile_state: Option<(&'a Tile, &'a TokensTable)>,
    pub available_tokens: &'a Tokens,
    pub tile_rotation: f64,
}

impl<'a> Iterator for HexIter<'a> {
    type Item = HexState<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the next address that should be returned.
        while self.ix < self.map.hexes.len() && !self.include[self.ix] {
            self.ix += 1;
        }
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
            let tile_state =
                Some((&self.map.tiles[hex_state.tile_ix], &hex_state.tokens));
            Some(HexState {
                addr,
                tile_state,
                available_tokens: &self.map.tokens(),
                tile_rotation: hex_state.rotation.radians(),
            })
        } else {
            Some(HexState {
                addr,
                tile_state: None,
                available_tokens: &self.map.tokens(),
                tile_rotation: 0.0,
            })
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
        while let Some(hex_state) = item {
            if hex_state.tile_state == None {
                return Some(hex_state.addr);
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

/// The state of a map hex that contains a tile.
#[derive(PartialEq)]
pub struct TileHexState<'a> {
    pub addr: HexAddress,
    pub tile: &'a Tile,
    pub tile_tokens: &'a TokensTable,
    pub available_tokens: &'a Tokens,
    pub tile_rotation: f64,
}

impl<'a> Iterator for TileHexIter<'a> {
    type Item = TileHexState<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.iter.next();
        while let Some(hex_state) = item {
            if let Some(tile_state) = hex_state.tile_state {
                return Some(TileHexState {
                    addr: hex_state.addr,
                    tile: tile_state.0,
                    tile_tokens: tile_state.1,
                    available_tokens: hex_state.available_tokens,
                    tile_rotation: hex_state.tile_rotation,
                });
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
            One => n18hex::PI_2_6,
            Two => n18hex::PI_4_6,
            Three => n18hex::PI,
            Four => -n18hex::PI_4_6,
            Five => -n18hex::PI_2_6,
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
        // NOTE: this is consistent with the 1861/67 maps.
        let col_letter = alphabet[self.col];
        let row_num = if self.col % 2 == 0 {
            2 * self.row + 1
        } else {
            2 * self.row + 2
        };
        write!(f, "{}{}", col_letter, row_num)
    }
}

#[derive(Debug)]
pub struct ParseHexAddressError {}

impl std::fmt::Display for ParseHexAddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not parse hex address")
    }
}

impl std::error::Error for ParseHexAddressError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Parse the result of `format!("{}", hex_address)`.
impl std::str::FromStr for HexAddress {
    type Err = ParseHexAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Examine the first byte, it must be within 'A'..'Z' (inclusive).
        let col: usize = (b'A'..=b'Z')
            .enumerate()
            .find_map(|(ix, byte)| {
                s.bytes().nth(0).and_then(|b| {
                    if b == byte {
                        Some(ix)
                    } else {
                        None
                    }
                })
            })
            .ok_or(ParseHexAddressError {})?;
        // Parse the remaining bytes as a positive integer.
        let row = s[1..].parse::<usize>().or(Err(ParseHexAddressError {}))?;
        // NOTE: depending on whether the column is odd or even, the row must
        // be even or odd, respectively.
        let row = if col % 2 == 0 {
            if row < 1 {
                Err(ParseHexAddressError {})?
            }
            if row % 2 == 1 {
                (row - 1) / 2
            } else {
                Err(ParseHexAddressError {})?
            }
        } else {
            if row < 2 {
                Err(ParseHexAddressError {})?
            }
            if row % 2 == 0 {
                (row - 2) / 2
            } else {
                Err(ParseHexAddressError {})?
            }
        };
        Ok(HexAddress { row, col })
    }
}

impl From<(usize, usize)> for HexAddress {
    fn from(src: (usize, usize)) -> Self {
        let (row, col) = src;
        Self { row, col }
    }
}

impl From<&HexAddress> for (usize, usize) {
    fn from(src: &HexAddress) -> Self {
        (src.row, src.col)
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
    use n18hex::Hex;
    use n18tile::{Connection, TrackEnd};

    static HEX_DIAMETER: f64 = 150.0;

    #[test]
    /// Test various strings of the form "A[0-9]+"; only strings where the
    /// digits form an odd integer should result in valid HexAddress values.
    fn test_parse_hex_address_a() {
        let a0 = "A0".parse::<HexAddress>();
        let a1 = "A1".parse::<HexAddress>();
        let a2 = "A2".parse::<HexAddress>();
        let a3 = "A3".parse::<HexAddress>();
        let a10 = "A10".parse::<HexAddress>();
        let a11 = "A11".parse::<HexAddress>();

        // Check that we obtained the expected Ok or Err value.
        assert!(a0.is_err());
        assert!(a1.is_ok());
        assert!(a2.is_err());
        assert!(a3.is_ok());
        assert!(a10.is_err());
        assert!(a11.is_ok());

        // Check the coordinates of the Ok values.
        let a1 = a1.unwrap();
        assert!(a1.row == 0);
        assert!(a1.col == 0);
        let a3 = a3.unwrap();
        assert!(a3.row == 1);
        assert!(a3.col == 0);
        let a11 = a11.unwrap();
        assert!(a11.row == 5);
        assert!(a11.col == 0);
    }

    #[test]
    /// Test various strings of the form "B[0-9]+"; only strings where the
    /// digits form an even integer should result in valid HexAddress values.
    fn test_parse_hex_address_b() {
        let b0 = "B0".parse::<HexAddress>();
        let b1 = "B1".parse::<HexAddress>();
        let b2 = "B2".parse::<HexAddress>();
        let b3 = "B3".parse::<HexAddress>();
        let b4 = "B4".parse::<HexAddress>();
        let b10 = "B10".parse::<HexAddress>();
        let b11 = "B11".parse::<HexAddress>();

        // Check that we obtained the expected Ok or Err value.
        assert!(b0.is_err());
        assert!(b1.is_err());
        assert!(b2.is_ok());
        assert!(b3.is_err());
        assert!(b4.is_ok());
        assert!(b10.is_ok());
        assert!(b11.is_err());

        // Check the coordinates of the Ok values.
        let b2 = b2.unwrap();
        assert!(b2.row == 0);
        assert!(b2.col == 1);
        let b4 = b4.unwrap();
        assert!(b4.row == 1);
        assert!(b4.col == 1);
        let b10 = b10.unwrap();
        assert!(b10.row == 4);
        assert!(b10.col == 1);
    }

    #[test]
    fn test_simple_two_by_two() {
        let hex = Hex::new(HEX_DIAMETER);
        let map = crate::descr::tests::map_2x2_tiles_5_6_58_63(&hex);

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
