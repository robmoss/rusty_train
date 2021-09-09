use cairo::Context;
use std::collections::{BTreeMap, BTreeSet};

use n18catalogue::{Availability, Catalogue};
use n18hex::{Hex, HexColour, HexFace, RotateCW, PI};
use n18tile::{Label, Tile, TokenSpace};
use n18token::{Token, Tokens};

/// Supported orientations for the map's hexagonal grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Orientation {
    /// Arrange hexagons in vertical columns; the top and bottom of each
    /// hexagon is flat.
    VerticalColumns,
    /// Arrange hexagons in horizontal rows; the top and bottom of each
    /// hexagon is pointed.
    HorizontalRows,
}

/// A grid of hexes, each of which may contain a [Tile].
#[derive(Debug, PartialEq, Clone)]
pub struct Map {
    /// The tokens that can be placed on the map.
    tokens: Tokens,
    /// Barriers across which track cannot be built, or for which there is an
    /// additional cost (e.g., rivers).
    barriers: Vec<(HexAddress, HexFace)>,
    /// All tiles that might be placed on the map.
    tiles: Catalogue,
    /// The map state: the tile (if any) placed on each map hex, and other
    /// tile-related details, such as the tile's rotation and placed tokens.
    hexes: BTreeMap<HexAddress, Option<MapTile>>,
    /// City labels that apply to map hexes.
    labels_tbl: BTreeMap<HexAddress, Vec<Label>>,
    /// The minimum row number for which there is a hex.
    min_row: isize,
    /// The minimum column number for which there is a hex.
    min_col: isize,
    /// The orientation of the hexagonal grid.
    orientation: Orientation,
}

impl Map {
    /// Returns an iterator over all tiles in the map catalogue.
    ///
    /// This includes tiles that are not available to the player.
    pub fn tile_iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.tile_iter()
    }

    /// Returns an iterator over all tiles available to the player.
    ///
    /// Note that this returns all tiles that indicated as being available to
    /// the player.
    /// To obtain only those tiles that are currently available to be placed
    /// on the map in its current state, see
    /// [available_tiles](Map::available_tiles).
    pub fn available_tiles_iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().filter_map(|(tile, avail)| {
            if *avail == Availability::Unavailable {
                None
            } else {
                Some(tile)
            }
        })
    }

    /// Returns an iterator over all tiles in the map catalogue, and their
    /// availability.
    ///
    /// This includes tiles that are not available to the player.
    pub fn tile_avail_iter(
        &self,
    ) -> impl Iterator<Item = &(Tile, Availability)> {
        self.tiles.iter()
    }

    /// Returns how many copies of the specified tile are currently placed on
    /// the map.
    fn number_placed(&self, tile_ix: usize) -> usize {
        self.hexes
            .values()
            .filter_map(|map_hex| {
                map_hex.as_ref().and_then(|map_hex| {
                    if map_hex.tile_ix == tile_ix {
                        Some(map_hex)
                    } else {
                        None
                    }
                })
            })
            .count()
    }

    /// Returns `true` if the tile is available to be placed on the map in its
    /// current state, respecting any limits on tile availability.
    fn tile_ix_available(&self, tile_ix: usize) -> bool {
        use n18catalogue::Availability::*;
        match self.tiles[tile_ix].1 {
            Unlimited => true,
            Unavailable => false,
            Limited(count) => self.number_placed(tile_ix) < count,
        }
    }

    /// Returns `true` if the tile is available to be placed on the map in its
    /// current state, respecting any limits on tile availability.
    pub fn tile_is_available(&self, tile_name: &str) -> bool {
        if let Some(ix) = self.tiles.index_of(tile_name) {
            self.tile_ix_available(ix)
        } else {
            false
        }
    }

    /// Returns all of the tiles in the map catalogue that can be placed on
    /// the map in its current state, respecting any limits on tile
    /// availability.
    ///
    /// Note that this method returns a `Vec`, rather than an `Iterator`,
    /// because it must collect the available tiles.
    /// In order to return an `Iterator`, the closure that filters the map
    /// catalogue would need to take ownership of the map.
    pub fn available_tiles(&self) -> Vec<(usize, &Tile)> {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(|(ix, (tile, _avail))| {
                if self.tile_ix_available(ix) {
                    Some((ix, tile))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the nth tile in the map catalogue.
    pub fn nth_tile(&self, ix: usize) -> &Tile {
        &self.tiles[ix].0
    }

    /// Returns the number of distinct tiles in the map catalogue.
    pub fn num_tiles(&self) -> usize {
        self.tiles.len()
    }

    /// Returns the abbreviated names associated with each token.
    pub fn token_names(&self) -> &[String] {
        self.tokens.names()
    }

    /// Returns the token with the given abbreviated name, if it exists.
    pub fn try_token(&self, name: &str) -> Option<Token> {
        self.tokens.token(name).copied()
    }

    /// Returns the token with the given abbreviated name.
    ///
    /// # Panics
    ///
    /// Panics if there is no token with the given name.
    pub fn token(&self, name: &str) -> Token {
        self.tokens.token(name).copied().unwrap_or_else(|| {
            let valid_names = self.tokens.names();
            if valid_names.is_empty() {
                panic!(
                    "Invalid token name '{}'; this map has no tokens",
                    name
                )
            } else {
                panic!(
                    "No token with name '{}'; valid names are: '{}'",
                    name,
                    self.tokens.names().join("', '")
                )
            }
        })
    }

    /// Returns the abbreviated name of the given token, if it exists.
    pub fn try_token_name(&self, token: &Token) -> Option<&str> {
        self.tokens.name(token)
    }

    /// Returns the abbreviated name of the given token.
    ///
    /// # Panics
    ///
    /// Panics if there is no such token.
    pub fn token_name(&self, token: &Token) -> &str {
        self.tokens
            .name(token)
            .unwrap_or_else(|| panic!("Unknown token {:?}", token))
    }

    /// Returns the barriers across which track cannot be built, or for which
    /// there is an additional cost (e.g., rivers).
    ///
    /// # Limitations
    ///
    /// Note that these barriers are currently cosmetic --- they are drawn but
    /// do not prevent a route from being laid across them.
    pub fn barriers(&self) -> &[(HexAddress, HexFace)] {
        self.barriers.as_slice()
    }

    /// Adds a new barrier to a single face of a specific map hex.
    pub fn add_barrier(&mut self, addr: HexAddress, face: HexFace) {
        self.barriers.push((addr, face))
    }

    /// Returns an iterator over the valid hex addresses for this map.
    pub fn hex_address_iter(&self) -> impl Iterator<Item = &HexAddress> {
        self.hexes.keys()
    }

    /// Returns a valid hex address, if the map contains at least one hex.
    pub fn default_hex(&self) -> Option<HexAddress> {
        self.hexes.keys().next().copied()
    }

    // TODO: replace with methods that retrieve specific details?
    // Tokens, Rotation, Tile name, Replaceable ...

    /// Returns a reference to the state of a map hex.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is not a valid hex address for this map.
    pub fn hex_state(&self, addr: HexAddress) -> Option<&MapTile> {
        self.hexes
            .get(&addr)
            .unwrap_or_else(|| panic!("Invalid address {:#?}", addr))
            .as_ref()
    }

    // TODO: replace with methods that replace/update specific details?
    // Tokens, Rotation, Tile name, Replaceable ...

    /// Returns a mutable reference to the state of a map hex.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is not a valid hex address for this map.
    pub fn hex_state_mut(
        &mut self,
        addr: HexAddress,
    ) -> Option<&mut MapTile> {
        self.hexes
            .get_mut(&addr)
            .unwrap_or_else(|| panic!("Invalid address {:#?}", addr))
            .as_mut()
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
        self.hexes.iter().for_each(|(addr, state_opt)| {
            if let Some(state) = state_opt {
                state.tokens.iter().for_each(|(token_space, token)| {
                    if t == token {
                        placed.push((addr, token_space))
                    }
                })
            }
        });
        placed
    }

    /// Returns the set of unique tokens that are currently placed on the map.
    pub fn unique_placed_tokens(&self) -> BTreeSet<&Token> {
        let mut placed: BTreeSet<&Token> = BTreeSet::new();
        self.hexes.iter().for_each(|(_addr, state_opt)| {
            if let Some(state) = state_opt {
                state.tokens.iter().for_each(|(_space, token)| {
                    let _ = placed.insert(token);
                })
            }
        });
        placed
    }

    /// Returns the hex face **relative to the map** that corresponds to the
    /// the specified hex face **relative to the tile's orientation**.
    pub fn map_face_from_tile_face(
        &self,
        addr: HexAddress,
        tile_face: HexFace,
    ) -> Option<HexFace> {
        self.hex_state(addr).map(|hs| tile_face + hs.rotation)
    }

    /// Returns the hex face **relative to the tile's orientation** that
    /// corresponds to the specified hex face **relative to the map**.
    pub fn tile_face_from_map_face(
        &self,
        addr: HexAddress,
        tile_face: HexFace,
    ) -> Option<HexFace> {
        self.hex_state(addr).map(|hs| tile_face - hs.rotation)
    }

    /// Returns the address of the hex that is adjacent to the specified face
    /// (in terms of map orientation, not tile orientation) of the given tile.
    ///
    /// If there is no such hex on this map, returns `None`.
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

        let addr = match map_face {
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
        };

        // NOTE: ensure that this address is valid.
        addr.filter(|addr| self.hexes.contains_key(addr))
    }

    /// Returns details of the tile that is adjacent to the specified face:
    ///
    /// - The address of the adjacent tile;
    /// - The face **relative to this tile's orientation** that is adjacent;
    ///   and
    /// - The tile itself.
    ///
    /// If there is no adjacent tile, returns `None`.
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

    /// Creates a new map.
    pub fn new<T>(tiles: Catalogue, tokens: Tokens, hexes: T) -> Self
    where
        T: IntoIterator<Item = HexAddress>,
    {
        let hexes: BTreeMap<HexAddress, Option<MapTile>> =
            hexes.into_iter().map(|addr| (addr, None)).collect();

        if hexes.is_empty() {
            panic!("Can not create map with no hexes")
        }

        let barriers = vec![];
        let labels_tbl = BTreeMap::new();
        let min_col = hexes.keys().map(|hc| hc.col).min().unwrap();
        let min_row = hexes.keys().map(|hc| hc.row).min().unwrap();
        // Note: we currently only support vertical columns.
        let orientation = Orientation::VerticalColumns;

        Map {
            tokens,
            barriers,
            tiles,
            hexes,
            labels_tbl,
            min_row,
            min_col,
            orientation,
        }
    }

    /// Returns the current tile, if any, placed at the specified hex address.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is not a valid hex address for this map.
    pub fn tile_at(&self, addr: HexAddress) -> Option<&Tile> {
        self.hex_state(addr).map(|hs| &self.tiles[hs.tile_ix].0)
    }

    /// Replaces the existing tile `old` at the specified map hex `addr` with
    /// the tile `new`, placed with rotation `rot`.
    ///
    /// Returns `Some(false)` if the tile `old` was not placed at the map hex,
    /// `Some(true)` if the tile `old` was replaced by the tile `new`, and
    /// `None` if `addr` or `new` were invalid.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is not a valid hex address for this map.
    pub fn replace_tile(
        &mut self,
        addr: HexAddress,
        old: &str,
        new: &str,
        rot: RotateCW,
    ) -> Option<bool> {
        let do_replace = if let Some(hs) = self.hex_state(addr) {
            if !hs.replaceable {
                // This tile cannot be replaced.
                return Some(false);
            }
            let curr_tile_name = self.tiles[hs.tile_ix].0.name.as_str();
            curr_tile_name == old
        } else {
            return None;
        };
        if do_replace {
            Some(self.place_tile(addr, new, rot))
        } else {
            Some(false)
        }
    }

    /// Places `tile` at the specified map hex `addr` with rotation `rot` and returns `true`.
    /// If the tile cannot be placed, returns `false`.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is not a valid hex address for this map.
    pub fn place_tile(
        &mut self,
        hex: HexAddress,
        tile: &str,
        rot: RotateCW,
    ) -> bool {
        let tile_ix = if let Some(ix) = self.tiles.index_of(tile) {
            ix
        } else {
            return false;
        };

        let new_tokens = if let Some(hex_state) = self.hex_state(hex) {
            if !hex_state.replaceable {
                // This tile cannot be replaced.
                return false;
            }
            // See if we can place each token from the original tile on the
            // new tile in such a way so as to preserve their connectivity
            // with adjacent hexes.
            let orig_tile = &self.tiles[hex_state.tile_ix].0;
            let orig_rotn = &hex_state.rotation;
            let tokens = &hex_state.tokens;
            let new_tile = &self.tiles[tile_ix].0;
            let new_rotn = &rot;
            try_placing_tokens(
                orig_tile, orig_rotn, tokens, new_tile, new_rotn,
            )
            .unwrap_or_else(BTreeMap::new)
        } else {
            BTreeMap::new()
        };

        // NOTE: hex_mut() panics if `hex` is an invalid address.
        if let Some(hex_state) = self.hex_state_mut(hex) {
            hex_state.tile_ix = tile_ix;
            hex_state.rotation = rot;
            hex_state.tokens = new_tokens
        } else {
            self.hexes.insert(
                hex,
                Some(MapTile {
                    tile_ix,
                    rotation: rot,
                    tokens: BTreeMap::new(),
                    replaceable: true,
                }),
            );
        }
        true
    }

    /// Removes the current tile, if any, from the specified map hex.
    pub fn remove_tile(&mut self, addr: HexAddress) {
        // NOTE: must ensure that this is a valid hex address.
        // Otherwise, this would add a new hex to the map.
        if self.hexes.contains_key(&addr) {
            self.hexes.insert(addr, None);
        }
    }

    /// Defines labels for a specific map hex, such as [Label::CityKind] and
    /// [Label::City], which are used to identify valid upgrade tiles.
    ///
    /// # Multiple labels
    ///
    /// If a hex has multiple labels, a tile is considered valid if it has at
    /// least one label in common with the hex.
    /// This allows, e.g., for both "O" tiles and "Y" tiles to be placed on
    /// the Ottawa tile in 1867, regardless of the labels on the current tile
    /// (i.e., you can place an "O" tile on top of a "Y" tile, if the hex has
    /// both "O" and "Y" labels).
    pub fn add_label_at(&mut self, addr: HexAddress, label: Label) {
        self.labels_tbl
            .entry(addr)
            .or_insert_with(Vec::new)
            .push(label)
    }

    /// Returns the labels associated with the specified map hex.
    pub fn labels_at(&self, addr: HexAddress) -> &[Label] {
        self.labels_tbl
            .get(&addr)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check whether a tile can be placed on an empty hex, given the current
    /// map state and respecting any limits on tile availability.
    pub fn can_place_on_empty(&self, addr: HexAddress, tile: &Tile) -> bool {
        // Check if the tile is available.
        if !self.tile_is_available(&tile.name) {
            return false;
        }

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

    /// Check whether a tile can be upgraded to another tile, given the
    /// current map state and respecting any limits on tile availability.
    pub fn can_upgrade_to(&self, addr: HexAddress, tile: &Tile) -> bool {
        // Check if the tile is available.
        if !self.tile_is_available(&tile.name) {
            return false;
        }

        if let Some(hex_labels) = self.labels_tbl.get(&addr) {
            // Check that the tile has at least one tile-restriction label in
            // common with this hex.
            tile.labels()
                .iter()
                .filter(|(label, _posn)| label.is_tile_restriction())
                .any(|(label, _posn)| hex_labels.contains(label))
        } else {
            // Check that this tile has no tile-restriction labels.
            tile.labels()
                .iter()
                .filter(|(label, _posn)| label.is_tile_restriction())
                .count()
                == 0
        }
    }

    /// Returns the coordinates for the centre of a map hex.
    ///
    /// Note that this accepts any valid row and column, so it can be used to
    /// locate the centre of hexes that are not part of the map itself.
    fn hex_centre(
        &self,
        row: isize,
        col: isize,
        x0: f64,
        y0: f64,
        hex: &Hex,
    ) -> (f64, f64) {
        let row = row - self.min_row;
        let col = col - self.min_col;

        match self.orientation {
            Orientation::VerticalColumns => {
                let x = x0 + (col as f64) * hex.max_d * 0.75;
                let y = if (col + self.min_col) % 2 == 1 {
                    y0 + (row as f64 + 0.5) * hex.min_d
                } else {
                    y0 + (row as f64) * hex.min_d
                };
                (x, y)
            }
            Orientation::HorizontalRows => {
                let x = if (row + self.min_row) % 2 == 1 {
                    x0 + (col as f64 + 0.5) * hex.min_d
                } else {
                    x0 + (col as f64) * hex.min_d
                };
                let y = y0 + (row as f64) * hex.max_d * 0.75;
                (x, y)
            }
        }
    }

    /// Returns the hexagon rotation for the map.
    fn hex_angle(&self) -> f64 {
        use Orientation::*;
        match self.orientation {
            VerticalColumns => 0.0,
            HorizontalRows => PI / 6.0,
        }
    }

    /// Returns the hexagon x-origin coordinate for the map.
    fn hex_x0(&self, hex: &Hex) -> f64 {
        use Orientation::*;
        let margin = hex.theme.map_margin.absolute(hex);
        match self.orientation {
            VerticalColumns => 0.5 * hex.max_d + margin,
            HorizontalRows => 0.5 * hex.min_d + margin,
        }
    }

    /// Returns the hexagon y-origin coordinate for the map.
    fn hex_y0(&self, hex: &Hex) -> f64 {
        use Orientation::*;
        let margin = hex.theme.map_margin.absolute(hex);
        match self.orientation {
            VerticalColumns => 0.5 * hex.min_d + margin,
            HorizontalRows => 0.5 * hex.max_d + margin,
        }
    }

    /// Translates and rotates the provided context `ctx` in preparation for
    /// drawing on the map hex `addr`, and returns the context's original
    /// matrix so that it can be restored afterwards.
    ///
    /// Note that this function accepts any hex address `addr` with a valid row
    /// and column.
    /// This means that it can be used for drawing on hexes that are not part
    /// of the map itself.
    ///
    /// Also note that it does not define or update the current point.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cairo::Context;
    /// use n18hex::Hex;
    /// use n18map::{HexAddress, Map};
    ///
    /// // Draw a thick black border around the specified map hex.
    /// fn outline_hex(map: &Map, addr: HexAddress, hex:&Hex, ctx: &Context) {
    ///     let m = map.prepare_to_draw(addr, hex, ctx);
    ///     ctx.set_source_rgb(0.0, 0.0, 0.0);
    ///     ctx.set_line_width(hex.max_d * 0.05);
    ///     hex.define_boundary(ctx);
    ///     ctx.stroke().unwrap();
    ///     ctx.set_matrix(m);
    /// }
    /// ```
    pub fn prepare_to_draw(
        &self,
        addr: HexAddress,
        hex: &Hex,
        ctx: &Context,
    ) -> cairo::Matrix {
        let angle = self.hex_angle();
        let x0 = self.hex_x0(hex);
        let y0 = self.hex_y0(hex);

        let (x, y) = self.hex_centre(addr.row, addr.col, x0, y0, hex);
        let m = ctx.matrix();
        ctx.translate(x, y);

        // NOTE: check whether this hex address is a valid map hex before
        // attempting to retrieve the current tile rotation, so that we allow
        // the caller to draw in hexes that are not part of the map itself.
        let tile_angle = if self.hexes.contains_key(&addr) {
            if let Some(hex_state) = self.hex_state(addr) {
                hex_state.rotation.radians()
            } else {
                0.0
            }
        } else {
            0.0
        };
        ctx.rotate(angle + tile_angle);

        m
    }

    /// Returns an iterator over all map hexes.
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
    /// # let tiles = tile_catalogue();
    /// # let tokens = vec![].into();
    /// # let hexes: Vec<HexAddress> = (0 as isize..4)
    /// #     .map(|r| (0 as isize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles.into(), tokens, hexes);
    /// // Draw a thick black border around each hex.
    /// ctx.set_source_rgb(0.0, 0.0, 0.0);
    /// ctx.set_line_width(hex.max_d * 0.05);
    /// for h_state in map.hex_iter(&hex, ctx) {
    ///     hex.define_boundary(ctx);
    ///     ctx.stroke().unwrap();
    /// }
    /// ```
    pub fn hex_iter<'a>(
        &'a self,
        hex: &'a Hex,
        ctx: &'a Context,
    ) -> HexIter<'a> {
        HexIter::new(hex, ctx, self)
    }

    /// Returns an iterator over all map hexes for which the provided closure
    /// returns `true`.
    ///
    /// At each iteration, the transformation matrix will be updated to
    /// account for the current hex's location and orientation.
    pub fn hex_subset_iter<'a, P: FnMut(&HexAddress) -> bool>(
        &'a self,
        hex: &'a Hex,
        ctx: &'a Context,
        mut include: P,
    ) -> HexIter<'a> {
        let include: BTreeSet<HexAddress> = self
            .hexes
            .keys()
            .filter_map(
                |&addr| if include(&addr) { Some(addr) } else { None },
            )
            .collect();
        HexIter::new_subset(hex, ctx, self, include)
    }

    /// Returns an iterator over all map hexes that do not contain a tile.
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
    /// # let tiles = tile_catalogue();
    /// # let tokens = vec![].into();
    /// # let hexes: Vec<HexAddress> = (0 as isize..4)
    /// #     .map(|r| (0 as isize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles.into(), tokens, hexes);
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

    /// Returns an iterator over all map hexes that contain a tile.
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
    /// # let tiles = tile_catalogue();
    /// # let tokens = vec![].into();
    /// # let hexes: Vec<HexAddress> = (0 as isize..4)
    /// #     .map(|r| (0 as isize..4).map(move |c| (r, c)))
    /// #     .flatten()
    /// #     .map(|coords| coords.into())
    /// #     .collect();
    /// # let ctx = hex.context();
    /// # let map = Map::new(tiles.into(), tokens, hexes);
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
        if !self.hexes.contains_key(&addr) {
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
        if !self.hexes.contains_key(&addr) {
            // TODO: keep searching (i.e., jump over holes)?
            addr.col += 1;
        }
        addr
    }

    pub fn next_row(&self, mut addr: HexAddress) -> HexAddress {
        addr.row += 1;
        if !self.hexes.contains_key(&addr) {
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
        if !self.hexes.contains_key(&addr) {
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
pub type TokensTable = BTreeMap<TokenSpace, Token>;

/// Attempts to place each token from `old_tile` on `new_tile` in such a way
/// so as to preserve each token's connectivity with adjacent hexes.
///
/// This is a wrapper around [n18tile::upgrade::try_placing_tokens] that
/// accepts and returns [TokensTable] values, which contain [Token] values
/// rather than token indices (`usize`).
pub fn try_placing_tokens(
    orig_tile: &Tile,
    orig_rotn: &RotateCW,
    tokens: &TokensTable,
    new_tile: &Tile,
    new_rotn: &RotateCW,
) -> Option<TokensTable> {
    let token_list: Vec<Token> = tokens.values().copied().collect();
    // Create a new table that maps token spaces to token indices.
    let tokens: BTreeMap<TokenSpace, usize> = tokens
        .keys()
        .enumerate()
        .map(|(ix, &token_space)| (token_space, ix))
        .collect();
    n18tile::upgrade::try_placing_tokens(
        orig_tile, orig_rotn, &tokens, new_tile, new_rotn,
    )
    .map(|token_index| {
        // Convert token indices back into Token values.
        token_index
            .into_iter()
            .map(|(token_space, ix)| (token_space, token_list[ix]))
            .collect()
    })
}

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
    iter: std::collections::btree_map::Iter<'a, HexAddress, Option<MapTile>>,
    m: cairo::Matrix,
    include: Option<BTreeSet<HexAddress>>,
}

impl<'a> HexIter<'a> {
    pub fn restart(&mut self) {
        self.ctx.set_matrix(self.m);
        self.iter = self.map.hexes.iter();
    }

    pub fn map(&self) -> &Map {
        self.map
    }

    fn new(hex: &'a Hex, ctx: &'a Context, map: &'a Map) -> Self {
        let angle = map.hex_angle();
        let x0 = map.hex_x0(hex);
        let y0 = map.hex_y0(hex);
        let iter = map.hexes.iter();

        Self {
            hex,
            ctx,
            map,
            x0,
            y0,
            angle,
            iter,
            m: ctx.matrix(),
            include: None,
        }
    }

    fn new_subset(
        hex: &'a Hex,
        ctx: &'a Context,
        map: &'a Map,
        include: BTreeSet<HexAddress>,
    ) -> Self {
        let angle = map.hex_angle();
        let x0 = map.hex_x0(hex);
        let y0 = map.hex_y0(hex);
        let iter = map.hexes.iter();

        Self {
            hex,
            ctx,
            map,
            x0,
            y0,
            angle,
            iter,
            m: ctx.matrix(),
            include: Some(include),
        }
    }

    fn hex_centre(&self, addr: HexAddress) -> (f64, f64) {
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
        let mut entry_opt = self.iter.next();
        while let Some((addr, _hs_opt)) = entry_opt {
            if let Some(ref addrs) = self.include {
                if addrs.contains(addr) {
                    break;
                }
            } else {
                break;
            }
            entry_opt = self.iter.next();
        }
        let (&addr, hex_state_opt) = if let Some(entry) = entry_opt {
            (entry.0, entry.1)
        } else {
            // NOTE: restore the original matrix.
            self.ctx.set_matrix(self.m);
            return None;
        };

        let (x, y) = self.hex_centre(addr);

        self.ctx.set_matrix(self.m);
        self.ctx.translate(x, y);

        if let Some(hex_state) = hex_state_opt {
            self.ctx.rotate(self.angle + hex_state.rotation.radians());
            let tile_state = Some((
                &self.map.tiles[hex_state.tile_ix].0,
                &hex_state.tokens,
            ));
            Some(HexState {
                addr,
                tile_state,
                available_tokens: &self.map.tokens,
                tile_rotation: hex_state.rotation.radians(),
            })
        } else {
            Some(HexState {
                addr,
                tile_state: None,
                available_tokens: &self.map.tokens,
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

/// Describes the placement of a specific tile on a map hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTile {
    tile_ix: usize,
    rotation: RotateCW,
    tokens: TokensTable,
    /// Whether this tile can be replaced by another tile; set to false for
    /// hexes such as the red off-board areas.
    replaceable: bool,
}

impl MapTile {
    pub fn tile<'a>(&self, map: &'a Map) -> &'a Tile {
        &map.tiles[self.tile_ix].0
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

    pub fn token_at(&self, space: &TokenSpace) -> Option<&Token> {
        self.tokens.get(space)
    }

    pub fn set_token_at(&mut self, space: &TokenSpace, token: Token) {
        self.tokens.insert(*space, token);
    }

    pub fn remove_token_at(&mut self, space: &TokenSpace) {
        self.tokens.remove(space);
    }

    pub fn tokens(&self) -> &TokensTable {
        &self.tokens
    }

    pub fn set_tokens(&mut self, tokens: TokensTable) {
        self.tokens = tokens
    }
}

/// A hex location on a `Map`, identified by row and column.
/// The row and column may be defined in terms of several different coordinate
/// systems, as described below.
///
/// # Logical coordinates
///
/// Logical coordinates are defined by a row number and a column number, where
/// the column number may be any `isize` value and the valid row numbers are
/// determined by column number:
///
/// - For even-numbered columns, the row number **must be even**; and
/// - For odd-numbered columns, the row number **must be odd**.
///
/// Valid `(row, column)` pairs include `(0, 0)`, `(1, 7)`, and `(-2, -4)`.
///
/// Invalid `(row, column)` pairs include `(0, 1)`, `(1, 8)`, and `(-2, -3)`.
///
/// ```rust
/// # use n18map::HexAddress;
/// let valid_addr = HexAddress::logical(1, 3);
/// assert!(valid_addr.is_some());
/// let invalid_addr = HexAddress::logical(1, 4);
/// assert!(invalid_addr.is_none());
/// ```
///
/// # String coordinates
///
/// When the column number is between `0` and `25` (inclusive), and the row
/// number is positive, these coordinates can be defined by a string of the
/// form "[A-Z][0-9]+" and parsed with `str::parse()`.
///
/// **Important:** when using string coordinates, the row number encoded in
/// the string is **one larger** than the logical row number.
/// For example:
/// - "A1" corresponds to the logical coordinates `(0, 0)`;
/// - "A3" corresponds to the logical coordinates `(0, 2)`;
/// - "B2" corresponds to the logical coordinates `(1, 1)`;
/// - "B4" corresponds to the logical coordinates `(1, 3)`;
/// - "A2" is **invalid**.
///
/// ```
/// # use n18map::HexAddress;
/// assert!("A0".parse::<HexAddress>().is_err());
/// assert!("A1".parse::<HexAddress>().is_ok());
/// assert!("A2".parse::<HexAddress>().is_err());
/// assert!("B0".parse::<HexAddress>().is_err());
/// assert!("B1".parse::<HexAddress>().is_err());
/// assert!("B2".parse::<HexAddress>().is_ok());
/// let a1 = "A1".parse::<HexAddress>().unwrap();
/// let logical = HexAddress::logical(0, 0).unwrap();
/// assert_eq!(a1, logical);
/// ```
///
/// # Alternating-row coordinates
///
/// When constructing [HexAddress] values with [HexAddress::new], converting
/// `(row, column)` tuples into to [HexAddress] values, and converting
/// [HexAddress] values into `(row, column)` tuples, rows and columns are
/// defined using alternating-row coordinates.
///
/// The column number is the same as when using logical coordinates:
/// - Column `0` corresponds to the sequence "A1", "A3," "A5", "A7", etc,
/// of map hexes.
/// - Column `1` corresponds to the sequence "B2", "B4", "B6", "B8", etc.
/// - Column `2` corresponds to the sequence "C1", "C3", "C5", "C7", etc.
///
/// In contrast, alternating row numbers are defined such that:
/// - Row `0` corresponds to the sequence "A1", "B2", "C1", "D2", etc, of map hexes.
/// - Row `1` corresponds to the sequence "A3", "B4", "C3", "D4", etc.
/// - Row `2` corresponds to the sequence "A5", "B6", "C5", "D6", etc.
///
/// This means that **any combination** of internal row and column numbers
/// defines a valid [HexAddress] value.
///
/// ```
/// # use n18map::HexAddress;
/// assert_eq!(HexAddress::new(0, 0), "A1".parse::<HexAddress>().unwrap());
/// assert_eq!(HexAddress::new(0, 1), "B2".parse::<HexAddress>().unwrap());
/// assert_eq!(HexAddress::new(0, 2), "C1".parse::<HexAddress>().unwrap());
/// assert_eq!(HexAddress::new(1, 0), "A3".parse::<HexAddress>().unwrap());
/// assert_eq!(HexAddress::new(1, 1), "B4".parse::<HexAddress>().unwrap());
/// assert_eq!(HexAddress::new(1, 2), "C3".parse::<HexAddress>().unwrap());
/// ```
///
/// Note that negative row and column numbers are permitted.
/// This allows maps to include tiles (such as off-board locations)
/// adjacent to any regular map hex.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HexAddress {
    /// The row number.
    pub(crate) row: isize,
    /// The column number.
    pub(crate) col: isize,
}

impl HexAddress {
    /// Returns a new `HexAddress` with the specified `row` and `column`
    /// (alternating-row coordinates).
    pub fn new(row: isize, column: isize) -> Self {
        Self { row, col: column }
    }

    /// Returns a new `HexAddress` with the specified `row` and `column`
    /// (logical coordinates) if the coordinates are valid.
    pub fn logical(row: isize, column: isize) -> Option<Self> {
        if row % 2 != column % 2 {
            None
        } else {
            Some(Self { row, col: column })
        }
    }

    /// Returns the character, if any, that corresponds to the logical column
    /// number.
    pub fn column_char(&self) -> Option<char> {
        let ix = if self.col >= 0 && self.col <= 25 {
            self.col as usize
        } else {
            return None;
        };
        let alphabet: Vec<_> = (b'A'..=b'Z').map(|b| b as char).collect();
        Some(alphabet[ix])
    }

    /// Returns the logical column number.
    pub fn logical_column(&self) -> isize {
        self.col
    }

    /// Returns the logical row number.
    pub fn logical_row(&self) -> isize {
        if self.col % 2 == 0 {
            2 * self.row
        } else {
            2 * self.row + 1
        }
    }

    /// Returns the address of the hex that is adjacent to the specified face
    /// (in terms of map orientation, not tile orientation) of this hex.
    pub fn adjacent(&self, face: HexFace) -> HexAddress {
        let is_upper = self.col % 2 == 0;

        match face {
            HexFace::Top => (self.row - 1, self.col).into(),
            HexFace::UpperRight => {
                if is_upper {
                    (self.row - 1, self.col + 1).into()
                } else {
                    (self.row, self.col + 1).into()
                }
            }
            HexFace::LowerRight => {
                if is_upper {
                    (self.row, self.col + 1).into()
                } else {
                    (self.row + 1, self.col + 1).into()
                }
            }
            HexFace::Bottom => (self.row + 1, self.col).into(),
            HexFace::LowerLeft => {
                if is_upper {
                    (self.row, self.col - 1).into()
                } else {
                    (self.row + 1, self.col - 1).into()
                }
            }
            HexFace::UpperLeft => {
                if is_upper {
                    (self.row - 1, self.col - 1).into()
                } else {
                    (self.row, self.col - 1).into()
                }
            }
        }
    }

    /// Calls a closure on this hex address, and returns the hex address.
    pub fn do_here<F>(&self, mut f: F) -> &HexAddress
    where
        F: FnMut(&Self),
    {
        f(self);
        self
    }

    /// Calls a closure on the address of the hex that is adjacent to the
    /// specified face (in terms of map orientation, not tile orientation) of
    /// this hex, without doing bounds checking, and returns the new hex
    /// address.
    ///
    /// This is short-hand for calling adjacent](HexAddress::adjacent) and
    /// [do_here](HexAddress::do_here), then returning the new hex address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use n18hex::{HexFace, RotateCW};
    /// use n18map::{HexAddress, Map};
    ///
    /// fn place_connected_track(map: &mut Map, starting_city: HexAddress) {
    ///     starting_city
    ///         // Move to the hex below and place tile 8.
    ///         .move_and_do(HexFace::Bottom, |&addr| {
    ///             let _ = map.place_tile(addr, "8", RotateCW::Five);
    ///         })
    ///         // Move to the hex on the lower right and place tile 9.
    ///         .move_and_do(HexFace::LowerRight, |&addr| {
    ///             let _ = map.place_tile(addr, "9", RotateCW::Two);
    ///         })
    ///         // Move to the hex on the lower right and place tile 9.
    ///         .move_and_do(HexFace::LowerRight, |&addr| {
    ///             let _ = map.place_tile(addr, "9", RotateCW::Two);
    ///         });
    /// }
    /// ```
    pub fn move_and_do<F>(&self, face: HexFace, f: F) -> HexAddress
    where
        F: FnMut(&Self),
    {
        let addr = self.adjacent(face);
        addr.do_here(f);
        addr
    }
}

/// Formats [HexAddress] values using string coordinates.
///
/// Returns `std::fmt::Error` if the column number is negative or exceeds 25.
impl std::fmt::Display for HexAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alphabet: Vec<_> = (b'A'..=b'Z').map(|b| b as char).collect();
        // NOTE: this is consistent with the 1861/67 maps.
        let ix = if self.col >= 0 && self.col <= 25 {
            self.col as usize
        } else {
            return Err(std::fmt::Error);
        };
        let col_letter = alphabet[ix];
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
        let col: isize = (b'A'..=b'Z')
            .enumerate()
            .find_map(|(ix, byte)| {
                s.as_bytes().get(0).and_then(|b| {
                    if b == &byte {
                        Some(ix as isize)
                    } else {
                        None
                    }
                })
            })
            .ok_or(ParseHexAddressError {})?;
        // Parse the remaining bytes as a positive integer.
        let row = s[1..].parse::<isize>().or(Err(ParseHexAddressError {}))?;
        // NOTE: depending on whether the column is odd or even, the row must
        // be even or odd, respectively.
        let row = if col % 2 == 0 {
            if row < 1 {
                return Err(ParseHexAddressError {});
            }
            if row % 2 == 1 {
                (row - 1) / 2
            } else {
                return Err(ParseHexAddressError {});
            }
        } else {
            if row < 2 {
                return Err(ParseHexAddressError {});
            }
            if row % 2 == 0 {
                (row - 2) / 2
            } else {
                return Err(ParseHexAddressError {});
            }
        };
        Ok(HexAddress { row, col })
    }
}

/// Converts `(row, column)` tuples into a [HexAddress] value.
///
/// The `row` and `column` values are defined in terms of alternating-row
/// coordinates, as per [HexAddress::new].
impl From<(isize, isize)> for HexAddress {
    fn from(src: (isize, isize)) -> Self {
        let (row, col) = src;
        Self { row, col }
    }
}

/// Converts [HexAddress] references into alternating-row coordinates.
impl From<&HexAddress> for (isize, isize) {
    fn from(src: &HexAddress) -> Self {
        (src.row, src.col)
    }
}

/// Converts `(row, column)` tuples into a [HexAddress] value.
///
/// The `row` and `column` values are defined in terms of alternating-row
/// coordinates, as per [HexAddress::new].
impl From<&(isize, isize)> for HexAddress {
    fn from(src: &(isize, isize)) -> Self {
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
    use n18tile::{Connection, TrackEnd};

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

    /// Tests that logical row numbers are consistent when the input row
    /// and/or column number is negative.
    #[test]
    fn test_logical_row_numbers() {
        // NOTE: this corresponds to "A1".
        let origin = HexAddress::from((0, 0));
        let orig_row = origin.logical_row();
        let orig_col = origin.logical_column();
        assert_eq!(orig_row, 0);
        assert_eq!(orig_col, 0);

        // Returns the column number, relative to the origin.
        let dcol = |addr: &HexAddress| addr.logical_column() - orig_col;

        // Returns the row number, relative to the origin, accounting for the
        // alternating up/down sequence along each row.
        let drow = |addr: &HexAddress, odd_column: bool| {
            let dr = addr.logical_row() - orig_row;
            if odd_column {
                // Odd column: hexes are one row below hexes in even rows.
                // Subtract one to shift this value up one row, to match the
                // row number for even columns.
                dr - 1
            } else {
                dr
            }
        };

        // Check that negating the column and/or row number used to construct
        // a HexAddress does not change the distance from the origin, after
        // accounting for the effect of the column number on the row number.
        let compare_addrs =
            |row: isize, col: isize, neg_row: bool, neg_col: bool| {
                let row_2 = if neg_row { -row } else { row };
                let col_2 = if neg_col { -col } else { col };
                let addr_1 = HexAddress::from((row, col));
                let addr_2 = HexAddress::from((row_2, col_2));
                let odd_column = col % 2 != 0;
                let dc_1 = dcol(&addr_1);
                let dr_1 = drow(&addr_1, odd_column);
                let dc_2 = dcol(&addr_2);
                let dr_2 = drow(&addr_2, odd_column);
                if neg_row {
                    assert_eq!(dr_1, -dr_2)
                } else {
                    assert_eq!(dr_1, dr_2)
                }
                if neg_col {
                    assert_eq!(dc_1, -dc_2)
                } else {
                    assert_eq!(dc_1, dc_2)
                }
            };

        // Check hexes in the 7x7 grid centred at the origin.
        let vals: Vec<isize> = vec![0, 1, 2, 3];
        for &row in &vals {
            for &col in &vals {
                // Compare the row and column numbers of the corresponding
                // HexAddress in each quadrant of this grid.
                compare_addrs(row, col, false, false);
                compare_addrs(row, col, true, false);
                compare_addrs(row, col, false, true);
                compare_addrs(row, col, true, true);
            }
        }
    }

    #[test]
    fn test_simple_two_by_two() {
        let map = crate::descr::tests::map_2x2_tiles_5_6_58_63();

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
            if ix == 0 || ix == 1 {
                assert_eq!(conns.len(), 2);
            } else if ix == 2 {
                assert_eq!(conns.len(), 6);
            }

            // Check that each city is connected to the end of a different
            // track segment.
            for j in 0..conns.len() {
                assert!(conns.iter().any(|&c| c
                    == Connection::Track {
                        ix: j,
                        end: TrackEnd::End
                    }));
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
