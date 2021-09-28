use cairo::Context;
use std::collections::{BTreeMap, BTreeSet};

use n18catalogue::{Availability, Catalogue};
use n18hex::{Hex, HexColour, HexFace, Orientation, RotateCW};
use n18tile::{Label, Tile, TokenSpace};
use n18token::{Token, Tokens};

use crate::{Adjacency, HexAddress};

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

    /// Returns a valid hex address, since all maps contain at least one hex.
    pub fn default_hex(&self) -> HexAddress {
        self.hexes.keys().next().copied().unwrap()
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
        let adj_addr = self.orientation.adjacent(addr, map_face);
        // NOTE: ensure that this address is valid.
        if self.hexes.contains_key(&adj_addr) {
            Some(adj_addr)
        } else {
            None
        }
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
        let map_face = self.map_face_from_tile_face(addr, tile_face)?;

        // Determine the address of the adjacent hex.
        let adj_addr = self.adjacent_address(addr, map_face)?;
        // This will be adjacent to the opposite face on the adjacent hex.
        let adj_tile_face =
            self.tile_face_from_map_face(adj_addr, map_face.opposite())?;
        let adj_tile = self.tile_at(adj_addr)?;
        Some((adj_addr, adj_tile_face, adj_tile))
    }

    /// Returns the hexagon orientation.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Creates a new map.
    pub fn new<T>(
        tiles: Catalogue,
        tokens: Tokens,
        hexes: T,
        orientation: Orientation,
    ) -> Self
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
            Orientation::FlatTop => {
                let x = x0 + (col as f64) * hex.max_d * 0.75;
                let y = if (col + self.min_col) % 2 == 1 {
                    y0 + (row as f64 + 0.5) * hex.min_d
                } else {
                    y0 + (row as f64) * hex.min_d
                };
                (x, y)
            }
            Orientation::PointedTop => {
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

    /// Returns the hexagon x-origin coordinate for the map.
    fn hex_x0(&self, hex: &Hex) -> f64 {
        use Orientation::*;
        let margin = hex.theme.map_margin.absolute(hex);
        match self.orientation {
            FlatTop => 0.5 * hex.max_d + margin,
            PointedTop => 0.5 * hex.min_d + margin,
        }
    }

    /// Returns the hexagon y-origin coordinate for the map.
    fn hex_y0(&self, hex: &Hex) -> f64 {
        use Orientation::*;
        let margin = hex.theme.map_margin.absolute(hex);
        match self.orientation {
            FlatTop => 0.5 * hex.min_d + margin,
            PointedTop => 0.5 * hex.max_d + margin,
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
        ctx.rotate(tile_angle);

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
    /// # let orientation = Orientation::FlatTop;
    /// # let map = Map::new(tiles.into(), tokens, hexes, orientation);
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
    /// # let orientation = Orientation::FlatTop;
    /// # let map = Map::new(tiles.into(), tokens, hexes, orientation);
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
    /// # let orientation = Orientation::FlatTop;
    /// # let map = Map::new(tiles.into(), tokens, hexes, orientation);
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
    iter: std::collections::btree_map::Iter<'a, HexAddress, Option<MapTile>>,
    m: cairo::Matrix,
    include: Option<BTreeSet<HexAddress>>,
}

impl<'a> HexIter<'a> {
    pub fn restart(&mut self) {
        self.ctx.set_matrix(self.m);
        self.iter = self.map.hexes.iter();
    }

    /// Returns a reference to the map associated with this hex iterator.
    ///
    /// Call this method with `HexIter::map(hex_iter)` in order to distinguish
    /// this from the `Iterator::map()` method.
    pub fn map(&self) -> &Map {
        self.map
    }

    fn new(hex: &'a Hex, ctx: &'a Context, map: &'a Map) -> Self {
        let x0 = map.hex_x0(hex);
        let y0 = map.hex_y0(hex);
        let iter = map.hexes.iter();

        Self {
            hex,
            ctx,
            map,
            x0,
            y0,
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
        let x0 = map.hex_x0(hex);
        let y0 = map.hex_y0(hex);
        let iter = map.hexes.iter();

        Self {
            hex,
            ctx,
            map,
            x0,
            y0,
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
            self.ctx.rotate(hex_state.rotation.radians());
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

#[cfg(test)]
mod tests {
    use super::HexAddress;
    use n18tile::{Connection, TrackEnd};

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
