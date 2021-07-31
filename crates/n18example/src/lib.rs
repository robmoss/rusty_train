#![allow(dead_code)]

use cairo::{Content, Context, Format, RecordingSurface};
use n18brush as brush;
use n18game::Game;
use n18hex::theme::Text;
use n18hex::{Colour, Hex, Theme};
use n18map::{HexAddress, Map, RotateCW};
use n18route::{Path, Route};
use n18tile::Tile;
use n18token::{Token, Tokens};
use std::ops::Deref;

pub struct Example {
    hex: Hex,
    map: Map,
    rec_surf: cairo::RecordingSurface,
    rec_ctx: cairo::Context,
}

impl Example {
    pub fn new<T: ToString, H: Into<Hex>>(
        hex: H,
        tokens: Vec<(T, Token)>,
        tiles: Vec<PlacedTile>,
    ) -> Self {
        let hex = hex.into();
        let all_tiles = n18catalogue::tile_catalogue();
        let tokens = tokens
            .into_iter()
            .map(|(name, style)| (name.to_string(), style))
            .collect();
        let token_mgr = Tokens::new(tokens);
        let hexes: Vec<HexAddress> =
            tiles.iter().map(|t| t.addr.parse().unwrap()).collect();
        let map = Map::new(all_tiles, token_mgr, hexes);
        let rec_surf = RecordingSurface::create(Content::ColorAlpha, None)
            .expect("Can't create recording surface");
        let rec_ctx =
            Context::new(&rec_surf).expect("Can't create cairo::Context");
        let mut example = Example {
            hex,
            map,
            rec_surf,
            rec_ctx,
        };
        example.place_tiles(tiles);
        example
    }

    pub fn new_game<H: Into<Hex>>(game: &dyn Game, hex: H) -> Self {
        let hex = hex.into();
        let map = game.create_map(&hex);
        let rec_surf = RecordingSurface::create(Content::ColorAlpha, None)
            .expect("Can't create recording surface");
        let rec_ctx =
            Context::new(&rec_surf).expect("Can't create cairo::Context");
        Example {
            hex,
            map,
            rec_surf,
            rec_ctx,
        }
    }

    pub fn new_catalogue<T: ToString, H: Into<Hex>>(
        hex: H,
        tokens: Vec<(T, Token)>,
        tiles: Vec<PlacedTile>,
        catalogue: Vec<Tile>,
    ) -> Self {
        let hex = hex.into();
        let tokens = tokens
            .into_iter()
            .map(|(name, style)| (name.to_string(), style))
            .collect();
        let token_mgr = Tokens::new(tokens);
        let hexes: Vec<HexAddress> =
            tiles.iter().map(|t| t.addr.parse().unwrap()).collect();
        let map = Map::new(catalogue, token_mgr, hexes);
        let rec_surf = RecordingSurface::create(Content::ColorAlpha, None)
            .expect("Can't create recording surface");
        let rec_ctx =
            Context::new(&rec_surf).expect("Can't create cairo::Context");
        let mut example = Example {
            hex,
            map,
            rec_surf,
            rec_ctx,
        };
        example.place_tiles(tiles);
        example
    }

    pub fn place_tiles(&mut self, tiles: Vec<PlacedTile>) {
        for tile in tiles {
            let addr = tile.addr.parse().unwrap();
            assert!(self.map.place_tile(addr, tile.name, tile.rotn));
            if !tile.toks.is_empty() {
                let hex_tile = self.map.tile_at(addr).unwrap();
                let tok_spaces = hex_tile.token_spaces();
                let place_toks: Vec<(usize, Token)> = tile
                    .toks
                    .iter()
                    .map(|(ix, name)| (*ix, self.map.token(name)))
                    .collect();
                let map_hex = self.map.hex_mut(addr).unwrap();
                for (ix, token) in &place_toks {
                    map_hex.set_token_at(&tok_spaces[*ix], *token)
                }
            }
        }
    }

    pub fn draw_map(&self) {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        let mut hex_iter = self.map.hex_iter(hex, ctx);
        brush::draw_map(hex, ctx, &mut hex_iter);
    }

    pub fn draw_map_subset<P>(&self, include: P)
    where
        P: FnMut(&HexAddress) -> bool,
    {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        let mut hex_iter = self.map.hex_subset_iter(hex, ctx, include);
        brush::draw_map_subset(hex, ctx, &self.map, &mut hex_iter);
    }

    pub fn draw_path<C>(&self, path: &Path, colour: C)
    where
        C: Into<Colour>,
    {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        colour.into().apply_colour(&self.rec_ctx);
        brush::highlight_path(hex, ctx, &self.map, path);
    }

    pub fn draw_route<C>(&self, route: &Route, colour: C)
    where
        C: Into<Colour>,
    {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        colour.into().apply_colour(&self.rec_ctx);
        brush::highlight_route(hex, ctx, &self.map, route);
    }

    pub fn text_style(&self) -> Text {
        Text::new()
    }

    pub fn context(&self) -> &Context {
        &self.rec_ctx
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn hex(&self) -> &Hex {
        &self.hex
    }

    pub fn theme(&self) -> &Theme {
        &self.hex.theme
    }

    pub fn content_size(&self) -> (f64, f64) {
        let (_x0, _y0, ink_w, ink_h) = self.rec_surf.ink_extents();
        (ink_w, ink_h)
    }

    pub fn erase_all(&mut self) -> Result<(), cairo::Error> {
        let rec_surf = RecordingSurface::create(Content::ColorAlpha, None)?;
        let rec_ctx =
            Context::new(&rec_surf).expect("Can't create cairo::Context");
        self.rec_surf = rec_surf;
        self.rec_ctx = rec_ctx;
        Ok(())
    }

    fn image_size(&self, margin: usize) -> (f64, f64, f64, f64) {
        let (x0, y0, ink_w, ink_h) = self.rec_surf.ink_extents();
        let width = 2.0 * margin as f64 + ink_w + 1.0;
        let height = 2.0 * margin as f64 + ink_h + 1.0;
        let dx = margin as f64 - x0 - 1.0;
        let dy = margin as f64 - y0 - 1.0;
        (width, height, dx, dy)
    }

    fn copy_surface<S, C>(
        &self,
        surf: &S,
        dx: f64,
        dy: f64,
        background: Option<C>,
    ) where
        S: Deref<Target = cairo::Surface>,
        C: Into<Colour>,
    {
        let ctx = Context::new(surf).expect("Can't create cairo::Context");

        if let Some(colour) = background {
            brush::clear_surface(&ctx, colour.into());
        }
        ctx.set_source_surface(&self.rec_surf, dx, dy).unwrap();
        ctx.paint().unwrap();
    }

    pub fn write_png<C, P>(
        &self,
        margin: usize,
        background: Option<C>,
        path: P,
    ) where
        C: Into<Colour>,
        P: AsRef<std::path::Path>,
    {
        let (width, height, dx, dy) = self.image_size(margin);
        let surf = cairo::ImageSurface::create(
            Format::ARgb32,
            width as i32,
            height as i32,
        )
        .expect("Can't create surface");
        self.copy_surface(&surf, dx, dy, background);
        let mut file =
            std::fs::File::create(path).expect("Can't create output file");
        surf.write_to_png(&mut file)
            .expect("Can't write output file")
    }

    pub fn write_pdf<C, P>(
        &self,
        margin: usize,
        background: Option<C>,
        path: P,
    ) where
        C: Into<Colour>,
        P: AsRef<std::path::Path>,
    {
        let (width, height, dx, dy) = self.image_size(margin);
        let surf = cairo::PdfSurface::new(width, height, path)
            .expect("Can't create surface");
        self.copy_surface(&surf, dx, dy, background);
        surf.finish();
    }

    pub fn write_svg<C, P>(
        &self,
        margin: usize,
        background: Option<C>,
        path: P,
    ) where
        C: Into<Colour>,
        P: AsRef<std::path::Path>,
    {
        let (width, height, dx, dy) = self.image_size(margin);
        let surf = cairo::SvgSurface::new(width, height, Some(path))
            .expect("Can't create surface");
        self.copy_surface(&surf, dx, dy, background);
        surf.finish();
    }
}

pub fn tile_at<'a>(name: &'a str, addr: &'a str) -> PlacedTile<'a> {
    PlacedTile::new(name, addr)
}

pub struct PlacedTile<'a> {
    pub name: &'a str,
    pub addr: &'a str,
    pub rotn: RotateCW,
    pub toks: Vec<(usize, &'a str)>,
}

impl<'a> PlacedTile<'a> {
    pub fn new(name: &'a str, addr: &'a str) -> Self {
        PlacedTile {
            name,
            addr,
            rotn: RotateCW::Zero,
            toks: vec![],
        }
    }

    pub fn rotate<R: Into<RotateCW>>(mut self, rotn: R) -> Self {
        self.rotn = rotn.into();
        self
    }

    pub fn rotate_cw(self, turns: usize) -> Self {
        match turns % 6 {
            0 => self.rotate(RotateCW::Zero),
            1 => self.rotate(RotateCW::One),
            2 => self.rotate(RotateCW::Two),
            3 => self.rotate(RotateCW::Three),
            4 => self.rotate(RotateCW::Four),
            5 => self.rotate(RotateCW::Five),
            _ => unreachable!(),
        }
    }

    pub fn rotate_acw(self, turns: usize) -> Self {
        match turns % 6 {
            0 => self.rotate(RotateCW::Zero),
            1 => self.rotate(RotateCW::Five),
            2 => self.rotate(RotateCW::Four),
            3 => self.rotate(RotateCW::Three),
            4 => self.rotate(RotateCW::Two),
            5 => self.rotate(RotateCW::One),
            _ => unreachable!(),
        }
    }

    pub fn token(mut self, ix: usize, name: &'static str) -> Self {
        self.toks.push((ix, name));
        self
    }

    pub fn tokens(mut self, toks: &[(usize, &'static str)]) -> Self {
        for tok in toks {
            self.toks.push(*tok)
        }
        self
    }
}
