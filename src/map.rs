use cairo::{Context, FontSlant, FontWeight};
use std::collections::HashMap;

use crate::hex::Hex;
use crate::prelude::PI;
use crate::tile::{Tile, Tok};

#[derive(Debug, PartialEq)]
pub struct Map {
    /// All tiles that might be placed on the map.
    tiles: Vec<Tile>,
    /// All tiles, indexed by name.
    catalogue: HashMap<String, usize>,
    /// The map state: which tiles are placed on which hexes.
    state: HashMap<HexAddress, HexState>,
    /// All hexes on which a tile might be placed.
    hexes: Vec<HexAddress>,
    min_row: usize,
    max_row: usize,
    min_col: usize,
    max_col: usize,
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

    pub fn get(&self, addr: HexAddress) -> Option<&HexState> {
        self.state.get(&addr)
    }

    pub fn get_mut(&mut self, addr: HexAddress) -> Option<&mut HexState> {
        self.state.get_mut(&addr)
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
            min_col,
            max_col,
            min_row,
            max_row,
            flat_top,
        }
    }

    pub fn tile_at(&self, hex: HexAddress) -> Option<&Tile> {
        self.state.get(&hex).map(|hs| &hs.name).and_then(|name| {
            self.catalogue.get(name).map(|ix| &self.tiles[*ix])
        })
    }

    pub fn place_tile(
        &mut self,
        hex: HexAddress,
        tile: &String,
        rot: RotateCW,
    ) -> bool {
        if !self.catalogue.contains_key(tile) {
            return false;
        }
        if let Some(hex_state) = self.state.get_mut(&hex) {
            // NOTE: leave the tokens as-is!
            // TODO: presumably this is correct behaviour?
            hex_state.name = tile.clone();
            hex_state.rotation = rot;
        } else {
            self.state.insert(
                hex,
                HexState {
                    name: tile.clone(),
                    rotation: rot,
                    tokens: HashMap::new(),
                },
            );
        }
        true
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
                    let tile = self
                        .catalogue
                        .get(&hex_state.name)
                        .map(|ix| &self.tiles[*ix])
                        .expect("Invalid tile name");
                    tile.draw(ctx, &hex);
                    // Draw any tokens.
                    for (tok, map_token) in hex_state.tokens.iter() {
                        tile.define_tok_path(&tok, &hex, ctx);
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
    ///     for token_space in tile.toks() {
    ///         tile.define_tok_path(&token_space, &hex, ctx);
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
        if !self.state.contains_key(&addr) {
            addr.col -= 1;
        }
        addr
    }

    pub fn prev_col(&self, mut addr: HexAddress) -> HexAddress {
        if addr.col == 0 {
            return addr;
        }
        addr.col -= 1;
        if !self.state.contains_key(&addr) {
            addr.col += 1;
        }
        addr
    }

    pub fn next_row(&self, mut addr: HexAddress) -> HexAddress {
        addr.row += 1;
        if !self.state.contains_key(&addr) {
            addr.row -= 1;
        }
        addr
    }

    pub fn prev_row(&self, mut addr: HexAddress) -> HexAddress {
        if addr.row == 0 {
            return addr;
        }
        addr.row -= 1;
        if !self.state.contains_key(&addr) {
            addr.row += 1;
        }
        addr
    }

    // TODO: define methods so that we can replace Map in main.rs.
    // TODO: rotate_tile_{cw|anti_cw}
    // TODO: upgrade_candidates()
    // TODO: get_tokens(), set_tokens(), get_token(), set_token()
    // TODO: translate_to_hex()
    // TODO: define_hex_boundary(), define_tok_path()
}

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
    type Item = (HexAddress, Option<(&'a Tile, &'a HashMap<Tok, Token>)>);

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
            let tile = self
                .map
                .catalogue
                .get(&hex_state.name)
                .map(|ix| (&self.map.tiles[*ix], &hex_state.tokens));
            Some((addr, tile))
        } else {
            Some((addr, None))
        }
    }
}

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
}

impl<'a> Iterator for TileHexIter<'a> {
    type Item = (HexAddress, (&'a Tile, &'a HashMap<Tok, Token>));

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexState {
    name: String,
    rotation: RotateCW,
    tokens: HashMap<Tok, Token>,
}

impl HexState {
    pub fn rotate_anti_cw(&mut self) {
        self.rotation = self.rotation.rotate_anti_cw()
    }

    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.rotate_cw()
    }

    pub fn radians(&self) -> f64 {
        self.rotation.radians()
    }

    pub fn get_token_at(&self, tok: &Tok) -> Option<&Token> {
        self.tokens.get(tok)
    }

    pub fn set_token_at(&mut self, tok: &Tok, token: Token) {
        self.tokens.insert(*tok, token);
    }

    pub fn remove_token_at(&mut self, tok: &Tok) {
        self.tokens.remove(tok);
    }

    pub fn get_tokens(&self) -> &HashMap<Tok, Token> {
        &self.tokens
    }

    pub fn set_tokens(&mut self, tokens: HashMap<Tok, Token>) {
        self.tokens = tokens
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    LP,
    PO,
    MK,
    N,
}

impl Token {
    fn text(&self) -> &str {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexAddress {
    row: usize,
    col: usize,
}

impl HexAddress {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
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
