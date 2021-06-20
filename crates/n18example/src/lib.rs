#![allow(dead_code)]

use cairo::{Content, Context, Format, RecordingSurface};
use n18brush as brush;
use n18game::Game;
use n18hex::Hex;
use n18map::{HexAddress, Map, RotateCW};
use n18route::{Path, Route};
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
        let all_tiles = n18catalogue::tile_catalogue(&hex);
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
        let rec_ctx = Context::new(&rec_surf);
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
        let rec_ctx = Context::new(&rec_surf);
        Example {
            hex,
            map,
            rec_surf,
            rec_ctx,
        }
    }

    pub fn place_tiles(&mut self, tiles: Vec<PlacedTile>) {
        let token_mgr = self.map.tokens().clone();
        for tile in tiles {
            let addr = tile.addr.parse().unwrap();
            assert!(self.map.place_tile(addr, tile.name, tile.rotn));
            if !tile.toks.is_empty() {
                let hex_tile = self.map.tile_at(addr).unwrap();
                let tok_spaces = hex_tile.token_spaces();
                let map_hex = self.map.get_hex_mut(addr).unwrap();
                for tok in &tile.toks {
                    let token = token_mgr.get_token(tok.1).unwrap();
                    map_hex.set_token_at(&tok_spaces[tok.0], *token)
                }
            }
        }
    }

    pub fn draw_map(&self) {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        let mut hex_iter = self.map.hex_iter(hex, ctx);
        brush::draw_hex_backgrounds(hex, ctx, &mut hex_iter);
        brush::draw_tiles(hex, ctx, &mut hex_iter);
        brush::outline_empty_hexes(hex, ctx, &mut hex_iter);
        brush::draw_barriers(hex, ctx, &self.map);
    }

    pub fn draw_map_subset<P>(&self, include: P)
    where
        P: FnMut(&HexAddress) -> bool,
    {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        let mut hex_iter = self.map.hex_subset_iter(hex, ctx, include);
        brush::draw_hex_backgrounds(hex, ctx, &mut hex_iter);
        brush::draw_tiles(hex, ctx, &mut hex_iter);
        brush::outline_empty_hexes(hex, ctx, &mut hex_iter);
        brush::draw_barriers_subset(hex, ctx, &self.map, &mut hex_iter);
    }

    pub fn draw_path(&self, path: &Path, rgba: (f64, f64, f64, f64)) {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        let (red, green, blue, alpha) = rgba;
        self.rec_ctx.set_source_rgba(red, green, blue, alpha);
        brush::highlight_path(hex, ctx, &self.map, path);
    }

    pub fn draw_route(&self, route: &Route, rgba: (f64, f64, f64, f64)) {
        let hex = &self.hex;
        let ctx = &self.rec_ctx;
        let (red, green, blue, alpha) = rgba;
        self.rec_ctx.set_source_rgba(red, green, blue, alpha);
        brush::highlight_route(hex, ctx, &self.map, route);
    }

    pub fn new_label<'a, T: ToString>(&'a self, text: T) -> LabelBuilder<'a> {
        LabelBuilder::new(&self.rec_ctx, &self.hex, text)
    }

    pub fn get_context(&self) -> &Context {
        &self.rec_ctx
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }

    pub fn get_map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn get_hex(&self) -> &Hex {
        &self.hex
    }

    pub fn content_size(&self) -> (f64, f64) {
        let (_x0, _y0, ink_w, ink_h) = self.rec_surf.ink_extents();
        (ink_w, ink_h)
    }

    pub fn erase_all(&mut self) -> Result<(), cairo::Error> {
        let rec_surf = RecordingSurface::create(Content::ColorAlpha, None)?;
        let rec_ctx = Context::new(&rec_surf);
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

    fn copy_surface<T: Deref<Target = cairo::Surface>>(
        &self,
        surf: &T,
        dx: f64,
        dy: f64,
        bg_rgba: Option<(f64, f64, f64, f64)>,
    ) {
        let ctx = Context::new(&surf);

        if let Some((red, green, blue, alpha)) = bg_rgba {
            ctx.set_source_rgba(red, green, blue, alpha);
            ctx.paint();
        }
        ctx.set_source_surface(&self.rec_surf, dx, dy);
        ctx.paint();
    }

    pub fn write_png<P: AsRef<std::path::Path>>(
        &self,
        margin: usize,
        bg_rgba: Option<(f64, f64, f64, f64)>,
        path: P,
    ) {
        let (width, height, dx, dy) = self.image_size(margin);
        let surf = cairo::ImageSurface::create(
            Format::ARgb32,
            width as i32,
            height as i32,
        )
        .expect("Can't create surface");
        self.copy_surface(&surf, dx, dy, bg_rgba);
        let mut file =
            std::fs::File::create(path).expect("Can't create output file");
        surf.write_to_png(&mut file)
            .expect("Can't write output file")
    }

    pub fn write_pdf<P: AsRef<std::path::Path>>(
        &self,
        margin: usize,
        bg_rgba: Option<(f64, f64, f64, f64)>,
        path: P,
    ) {
        let (width, height, dx, dy) = self.image_size(margin);
        let surf = cairo::PdfSurface::new(width, height, path)
            .expect("Can't create surface");
        self.copy_surface(&surf, dx, dy, bg_rgba);
        surf.finish();
    }

    pub fn write_svg<P: AsRef<std::path::Path>>(
        &self,
        margin: usize,
        bg_rgba: Option<(f64, f64, f64, f64)>,
        path: P,
    ) {
        let (width, height, dx, dy) = self.image_size(margin);
        let surf = cairo::SvgSurface::new(width, height, Some(path))
            .expect("Can't create surface");
        self.copy_surface(&surf, dx, dy, bg_rgba);
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

pub fn create_map(hex: &Hex, tiles: &[PlacedTile], tokens: Tokens) -> Map {
    let catalogue = n18catalogue::tile_catalogue(hex);
    let hexes: Vec<HexAddress> =
        tiles.iter().map(|t| t.addr.parse().unwrap()).collect();
    let map_tokens = tokens.clone();
    let mut map = Map::new(catalogue, map_tokens, hexes);
    for tile in tiles {
        let addr = tile.addr.parse().unwrap();
        assert!(map.place_tile(addr, tile.name, tile.rotn));
        if !tile.toks.is_empty() {
            let hex_tile = map.tile_at(addr).unwrap();
            let tok_spaces = hex_tile.token_spaces();
            let map_hex = map.get_hex_mut(addr).unwrap();
            for tok in &tile.toks {
                let token = tokens.get_token(tok.1).unwrap();
                map_hex.set_token_at(&tok_spaces[tok.0], *token)
            }
        }
    }
    map
}

pub struct Label<'a> {
    ctx: &'a Context,
    layout: pango::Layout,
    font_descr: pango::FontDescription,
    hjust: f64,
    vjust: f64,
}

impl<'a> Label<'a> {
    pub fn dims(&self) -> (i32, i32) {
        pangocairo::update_layout(&self.ctx, &self.layout);
        // TODO: is it best to return the ink extents or the logical extents?
        let (_ink_rect, rect) = self.layout.get_pixel_extents();
        (rect.width, rect.height)
    }

    pub fn draw_at(&self, x: f64, y: f64) {
        pangocairo::update_layout(&self.ctx, &self.layout);
        let (_ink_rect, rect) = self.layout.get_pixel_extents();
        let tx = x - self.hjust * (rect.width as f64) - rect.x as f64;
        let ty = y - self.vjust * (rect.height as f64) - rect.y as f64;
        self.ctx.move_to(tx, ty);
        pangocairo::show_layout(&self.ctx, &self.layout);
    }

    pub fn draw(&self) {
        pangocairo::show_layout(&self.ctx, &self.layout);
    }
}

pub struct LabelBuilder<'a> {
    ctx: &'a Context,
    hex: &'a Hex,
    text: String,
    font_size: f64,
    font_family: Option<String>,
    alignment: pango::Alignment,
    style: pango::Style,
    weight: pango::Weight,
    hjust: f64,
    vjust: f64,
}

impl<'a> LabelBuilder<'a> {
    pub fn new<T: ToString>(ctx: &'a Context, hex: &'a Hex, text: T) -> Self {
        LabelBuilder {
            ctx,
            hex,
            text: text.to_string(),
            font_size: 12.0,
            font_family: None,
            alignment: pango::Alignment::Left,
            style: pango::Style::Normal,
            weight: pango::Weight::Normal,
            hjust: 0.0,
            vjust: 0.0,
        }
    }

    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn font_family<T: ToString>(mut self, font_family: T) -> Self {
        self.font_family = Some(font_family.to_string());
        self
    }

    pub fn alignment(mut self, alignment: pango::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn style(mut self, style: pango::Style) -> Self {
        self.style = style;
        self
    }

    pub fn weight(mut self, weight: pango::Weight) -> Self {
        self.weight = weight;
        self
    }

    pub fn bold(mut self) -> Self {
        self.weight = pango::Weight::Bold;
        self
    }

    pub fn hjust(mut self, hjust: f64) -> Self {
        self.hjust = hjust;
        self
    }

    pub fn vjust(mut self, vjust: f64) -> Self {
        self.vjust = vjust;
        self
    }

    pub fn into_label(self) -> Option<Label<'a>> {
        let layout = pangocairo::create_layout(self.ctx)?;
        layout.set_text(&self.text);
        let mut font_descr = pango::FontDescription::new();
        // NOTE: apply the specified font settings.
        if let Some(family) = self.font_family {
            font_descr.set_family(&family);
        }
        let scale = self.hex.max_d / 125.0;
        // NOTE: font size in *points* is used by set_size(), while
        // *device units* as used by set_absolute_size().
        let dev_size = self.font_size * scale * pango::SCALE as f64;
        font_descr.set_absolute_size(dev_size);
        font_descr.set_style(self.style);
        font_descr.set_weight(self.weight);
        layout.set_font_description(Some(&font_descr));
        layout.set_alignment(self.alignment);
        Some(Label {
            ctx: self.ctx,
            layout,
            font_descr,
            hjust: self.hjust,
            vjust: self.vjust,
        })
    }
}
