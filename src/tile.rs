use crate::city::City;
use crate::draw::Draw;
use crate::hex::{Hex, HexColour};
use crate::label::Label;
use crate::track::Track;
use cairo::Context;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DrawLayer {
    Under,
    Normal,
    Over,
    Topmost,
}

impl DrawLayer {
    pub fn below(&self) -> Option<Self> {
        use DrawLayer::*;
        match self {
            Under => None,
            Normal => Some(Under),
            Over => Some(Normal),
            Topmost => Some(Over),
        }
    }

    pub fn above(&self) -> Option<Self> {
        use DrawLayer::*;
        match self {
            Under => Some(Normal),
            Normal => Some(Over),
            Over => Some(Topmost),
            Topmost => None,
        }
    }
}

pub type LabelAndPos = (crate::label::Label, crate::hex::HexPosition);

pub type Tiles = Vec<Tile>;

/// A tile contains some number of track segments and cities.
#[derive(PartialEq, Debug)]
pub struct Tile {
    pub colour: HexColour,
    pub name: String,
    tracks: Vec<Track>,
    cities: Vec<City>,
    // Track indices by drawing layer.
    tracks_tbl: HashMap<DrawLayer, Vec<usize>>,
    // City indices by drawing layer.
    cities_tbl: HashMap<DrawLayer, Vec<usize>>,
    // The revenue(s) for any dits and/or cities.
    revenues: Vec<usize>,
    // Tile labels: tile name, revenue, city name, etc.
    labels: Vec<LabelAndPos>,
}

impl Tile {
    pub fn new(
        colour: HexColour,
        name: String,
        tracks: Vec<Track>,
        cities: Vec<City>,
        hex: &Hex,
    ) -> Self {
        // TODO: check track connectivity and crossing, determine layers
        // Also save this information in a form that's amenable for
        // building track networks ... ???
        // TODO: detect track segments that cross a city (and that this isn't
        // part of the clipped path) and break them into separate segments
        // (e.g., straight -> mid + mid; gentle_l -> ...)
        // Hmmm ... maybe not
        let ctx = hex.context();
        let mut tracks_tbl = HashMap::new();
        let mut cities_tbl = HashMap::new();
        let default_layer = DrawLayer::Normal;
        let dt = 0.10;
        let mut track_layers = HashMap::new();
        // TODO: can we divide this up into smaller functions and check
        // some invariant conditions and write test cases?
        let verbose = false;
        if verbose {
            println!("Inspecting tile {} ...", name);
        }
        for i in 0..tracks.len() {
            let track = tracks[i];
            for j in (i + 1)..tracks.len() {
                let other = tracks[j];
                if track.crosses(&other, hex, dt, ctx) {
                    if verbose {
                        println!("    Tracks {} and {} cross", i, j);
                    }
                    // NOTE: if the underlying track's layer is Over, the
                    // overlying track needs to be in the Top layer.
                    if let Some(l) = track_layers.get(&i) {
                        if l == &DrawLayer::Over {
                            track_layers.insert(j, DrawLayer::Topmost);
                            continue;
                        }
                    }
                    let this_layer = default_layer.below().unwrap();
                    let other_layer = default_layer.above().unwrap();
                    track_layers.insert(i, this_layer);
                    track_layers.insert(j, other_layer);
                } else {
                    if track.connected(&other, hex, ctx) {
                        if verbose {
                            println!("    Tracks {} and {} connect", i, j);
                        }
                    }
                    track_layers.entry(i).or_insert(default_layer);
                }
            }
            track_layers.entry(i).or_insert(default_layer);
        }
        if verbose {
            println!("    Have {} tracks", tracks.len());
            for (key, val) in track_layers.iter() {
                println!("        key: {} val: {:?}", key, val);
            }
        }
        // NOTE: there can be zero, one, or multiple revenues for a tile.
        let mut revenues: Vec<usize> = tracks
            .iter()
            .filter_map(|t| t.dit.map(|(_, revenue)| revenue))
            .chain(cities.iter().map(|c| c.revenue))
            .collect();
        revenues.sort();
        revenues.dedup();
        for (cx, city) in cities.iter().enumerate() {
            let mut layer = DrawLayer::Under;
            for (i, track) in tracks.iter().enumerate() {
                if track.intersects_fill(city, hex, dt, ctx) {
                    let track_layer = track_layers.get(&i).unwrap_or(&layer);
                    layer = std::cmp::max(layer, *track_layer);
                }
            }
            if verbose {
                println!("    City #{} in layer {:?}", cx, layer);
            }
            cities_tbl.entry(layer).or_insert(vec![]).push(cx)
        }
        for (i, _track) in tracks.iter().enumerate() {
            let layer = track_layers.get(&i).unwrap();
            tracks_tbl.entry(*layer).or_insert(vec![]).push(i)
        }
        Self {
            colour,
            name,
            tracks,
            cities,
            tracks_tbl,
            cities_tbl,
            revenues,
            labels: vec![],
        }
    }

    // TODO: verify labels (e.g., one revenue label for each revenue ix)

    pub fn label<P>(mut self, label: Label, pos: P) -> Self
    where
        P: Into<crate::hex::HexPosition>,
    {
        self.labels.push((label, pos.into()));
        self
    }

    pub fn tracks(&self) -> &[Track] {
        self.tracks.as_slice()
    }

    pub fn cities(&self) -> &[City] {
        self.cities.as_slice()
    }

    pub fn labels(&self) -> &[LabelAndPos] {
        self.labels.as_slice()
    }

    fn layer_bg(&self, layer: &DrawLayer, ctx: &Context, hex: &Hex) {
        let empty = vec![];
        for ix in self.tracks_tbl.get(layer).unwrap_or(&empty) {
            let track = self.tracks[*ix];
            track.draw_bg(hex, ctx)
        }
        let empty = vec![];
        for ix in self.cities_tbl.get(layer).unwrap_or(&empty) {
            let city = self.cities[*ix];
            city.draw_bg(hex, ctx)
        }
    }

    #[allow(dead_code)]
    fn coords_in_red(&self, layer: &DrawLayer, ctx: &Context, hex: &Hex) {
        let empty = vec![];

        for ix in self.tracks_tbl.get(layer).unwrap_or(&empty).iter() {
            let track = self.tracks[*ix];
            ctx.set_source_rgb(1.0, 0.0, 0.0);
            let line_cap = ctx.get_line_cap();
            ctx.set_line_cap(cairo::LineCap::Round);
            for coord in track.coords(hex, 0.1) {
                ctx.new_path();
                ctx.move_to(coord.x, coord.y);
                ctx.line_to(coord.x, coord.y);
                ctx.stroke();
            }
            ctx.set_line_cap(line_cap);
        }
    }

    #[allow(dead_code)]
    fn dit_coords_in_red(&self, ctx: &Context, hex: &Hex) {
        use DrawLayer::*;

        // Draw the centre of each dit on a track segment as a red dot.
        for layer in vec![&Under, &Normal, &Over, &Topmost] {
            let empty = vec![];
            ctx.set_source_rgb(1.0, 0.0, 0.0);
            let line_cap = ctx.get_line_cap();
            ctx.set_line_cap(cairo::LineCap::Round);
            for ix in self.tracks_tbl.get(layer).unwrap_or(&empty) {
                let track = self.tracks[*ix];
                if let Some(coord) = track.dit_coord(hex) {
                    ctx.new_path();
                    ctx.move_to(coord.x, coord.y);
                    ctx.line_to(coord.x, coord.y);
                    ctx.stroke();
                }
            }
            ctx.set_line_cap(line_cap);
        }
    }

    fn layer_fg(&self, layer: &DrawLayer, ctx: &Context, hex: &Hex) {
        let empty = vec![];
        for ix in self.tracks_tbl.get(layer).unwrap_or(&empty) {
            let track = self.tracks[*ix];
            track.draw_fg(hex, ctx);
        }

        // NOTE: draw coordinates along track in red.
        // self.coords_in_red(layer, ctx, hex);

        let empty = vec![];
        for ix in self.cities_tbl.get(layer).unwrap_or(&empty) {
            let city = self.cities[*ix];
            city.draw_fg(hex, ctx)
        }
    }

    fn label_text(&self, label: &Label) -> Option<String> {
        match label {
            Label::City(name) => Some(name.to_string()),
            Label::Y => Some("Y".to_string()),
            Label::TileName => Some(self.name.to_string()),
            Label::Revenue(ref ix) => {
                self.revenues.get(*ix).map(|r| format!("{}", r))
            }
        }
    }

    pub fn draw(self: &Self, ctx: &Context, hex: &Hex) {
        use DrawLayer::*;

        // Draw the tile background.
        hex.draw_background(self.colour, ctx);
        // Draw the background for the bottom two layers.
        self.layer_bg(&Under, ctx, hex);
        self.layer_bg(&Normal, ctx, hex);
        // Draw the foreground for the bottom-most layer.
        self.layer_fg(&Under, ctx, hex);
        // Draw the background of the covering layer.
        self.layer_bg(&Over, ctx, hex);
        // Draw the foreground of the normal and covering layers.
        self.layer_fg(&Normal, ctx, hex);
        self.layer_fg(&Over, ctx, hex);
        // Draw the top-most layer.
        self.layer_bg(&Topmost, ctx, hex);
        self.layer_fg(&Topmost, ctx, hex);
        // Draw the tile name.
        Label::TileName.select_font(ctx);
        hex.draw_tile_name(&self.name, ctx);
        // Draw other tile labels.
        for (label, pos) in &self.labels {
            // TODO: can we avoid needing to pass ix for revenue labels?
            if let Some(text) = self.label_text(label) {
                label.select_font(ctx);
                if let &Label::Revenue(_ix) = label {
                    hex.circ_text(text.as_ref(), *pos, ctx)
                } else {
                    hex.text(text.as_ref(), *pos, ctx)
                }
            }
        }
    }

    pub fn toks(&self) -> Vec<Tok> {
        self.cities
            .iter()
            .enumerate()
            .flat_map(|(city_ix, city)| {
                city.token_ixs()
                    .into_iter()
                    .map(|token_ix| Tok { city_ix, token_ix })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn num_toks(&self) -> usize {
        self.toks().len()
    }

    fn has_dits(&self) -> bool {
        if self.tracks.iter().any(|track| track.dit.is_some()) {
            return true;
        }
        if self.cities.iter().any(|city| city.token_ixs().is_empty()) {
            return true;
        }
        return false;
    }

    pub fn define_tok_path(&self, tok: &Tok, hex: &Hex, ctx: &Context) {
        let city = self.cities[tok.city_ix];
        city.define_token_path(tok.token_ix, hex, ctx);
    }

    /// Check whether a tile can be upgraded to another tile.
    pub fn can_upgrade_to(&self, other: &Tile) -> bool {
        // Check whether the new tile's colour is correct.
        if let Some(colour) = self.colour.next_phase() {
            if other.colour != colour {
                return false;
            }
        }
        // Dit tiles can only be upgraded to from other dit tiles.
        if self.has_dits() != other.has_dits() {
            return false;
        }
        let self_toks = self.num_toks();
        let other_toks = other.num_toks();
        // City tiles can only be upgraded to from existing city tiles.
        if self_toks == 0 && other_toks > 0 {
            return false;
        }
        // Check whether the new tile has at least as many token spaces.
        if self_toks > other_toks {
            return false;
        }
        // Check Y label compatibility.
        let self_y = self
            .labels
            .iter()
            .any(|(label, _posn)| label == &crate::label::Label::Y);
        let other_y = other
            .labels
            .iter()
            .any(|(label, _posn)| label == &crate::label::Label::Y);
        if self_y && !other_y {
            return false;
        }
        // Check city-name compatibility.
        let self_city = self.labels.iter().find_map(|(label, _posn)| {
            if let crate::label::Label::City(ref name) = label {
                Some(name)
            } else {
                None
            }
        });
        let other_city = other.labels.iter().find_map(|(label, _posn)| {
            if let crate::label::Label::City(ref name) = label {
                Some(name)
            } else {
                None
            }
        });
        // NOTE: Ottawa, for example, can be covered with tile 623 (Y label)
        // and then upgraded to tile X8 (City("O")).
        // So we should only check that the city names match when the current
        // tile has a city name.
        if self_city.is_some() {
            if self_city != other_city {
                return false;
            }
        }
        // TODO: other checks, such as preserving track connectivity?
        // That would require having access to the map, so this would have to
        // be an additional layer of filtering provided by the map itself.
        return true;
    }

    /// Determines the surface size for this tile, which includes a small
    /// margin on all four sides.
    fn surface_width(&self, hex: &Hex) -> f64 {
        let margin = 0.05;
        hex.max_d * (1.0 + margin)
    }

    /// Saves the tile to a PNG file.
    pub fn save_png<P: AsRef<std::path::Path>>(
        &self,
        hex: &Hex,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(path)?;
        self.write_png(hex, &mut file)
    }

    /// Saves the tile to an SVG file.
    pub fn save_svg<P: AsRef<std::path::Path>>(
        &self,
        hex: &Hex,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        self.write_svg(hex, file)
    }

    /// Saves the tile to a PDF file.
    pub fn save_pdf<P: AsRef<std::path::Path>>(
        &self,
        hex: &Hex,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        self.write_pdf(hex, file)
    }

    /// Writes the tile as a PNG image to the provided stream.
    pub fn write_png<W: std::io::Write>(
        &self,
        hex: &Hex,
        stream: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let margin = 0.05;
        let width = hex.max_d * (1.0 + margin);
        let dim = width as i32;
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, dim, dim)
                .map_err(|_status| "Can't create surface")?;
        let ctx = cairo::Context::new(&surface);
        ctx.translate(width / 2.0, width / 2.0);
        self.draw(&ctx, hex);
        surface.write_to_png(stream)?;
        Ok(())
    }

    /// Writes the tile as an SVG image to the provided stream.
    pub fn write_svg<W: std::io::Write + 'static>(
        &self,
        hex: &Hex,
        stream: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let margin = 0.05;
        let width = hex.max_d * (1.0 + margin);
        let surface = cairo::SvgSurface::for_stream(width, width, stream)
            .map_err(|_status| "Can't create surface")?;
        let ctx = cairo::Context::new(&surface);
        ctx.translate(width / 2.0, width / 2.0);
        self.draw(&ctx, hex);
        surface
            .finish_output_stream()
            .map(|_stream| ())
            .map_err(|err| err.error.into())
    }

    /// Writes the tile as a PDF image to the provided stream.
    pub fn write_pdf<W: std::io::Write + 'static>(
        &self,
        hex: &Hex,
        stream: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let width = self.surface_width(hex);
        let surface = cairo::PdfSurface::for_stream(width, width, stream)
            .map_err(|_status| "Can't create surface")?;
        let ctx = cairo::Context::new(&surface);
        ctx.translate(width / 2.0, width / 2.0);
        self.draw(&ctx, hex);
        surface
            .finish_output_stream()
            .map(|_stream| ())
            .map_err(|err| err.error.into())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tok {
    city_ix: usize,
    token_ix: usize,
}
