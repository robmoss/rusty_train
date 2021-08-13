use crate::{City, Connection, Connections, Dit, Draw, Label, Track};
use cairo::Context;
use n18hex::{Colour, Hex, HexColour, HexCorner, HexPosition};
use std::collections::BTreeMap;

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

pub type LabelAndPos = (Label, HexPosition);

pub type Tiles = Vec<Tile>;

/// A tile contains some number of track segments and cities.
#[derive(PartialEq, Debug, Clone)]
pub struct Tile {
    pub colour: HexColour,
    pub name: String,
    tracks: Vec<Track>,
    cities: Vec<City>,
    // Track indices by drawing layer.
    tracks_tbl: BTreeMap<DrawLayer, Vec<usize>>,
    // City indices by drawing layer.
    cities_tbl: BTreeMap<DrawLayer, Vec<usize>>,
    // The revenue(s) for any dits and/or cities.
    revenues: Vec<usize>,
    // Tile labels: tile name, revenue, city name, etc.
    labels: Vec<LabelAndPos>,
    // Whether to show the tile name.
    show_tile_name: bool,
    // Connections between tracks, dits, cities, and hex faces.
    conns: Connections,
}

impl Tile {
    /// Creates a new tile and determines the connectivity between track
    /// segments, revenue centres, and hex faces.
    ///
    /// Note that `hex` should be the [**default hexagon**](Hex::default),
    /// to ensure that track connectivity is valid and consistent.
    /// By making this an argument, a single [Hex] can be used to construct
    /// all required tiles, rather than creating a new [Hex] for each tile.
    pub fn new<S: Into<String>>(
        colour: HexColour,
        name: S,
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
        let mut tracks_tbl = BTreeMap::new();
        let mut cities_tbl = BTreeMap::new();
        let default_layer = DrawLayer::Normal;
        let dt = 0.10;
        let mut track_layers = BTreeMap::new();
        let name = name.into();
        // TODO: can we divide this up into smaller functions and check
        // some invariant conditions and write test cases?
        let verbose = false;
        if verbose {
            println!("Inspecting tile {} ...", name);
        }
        for (i, track) in tracks.iter().enumerate() {
            for (j, other) in tracks.iter().enumerate().skip(i + 1) {
                if track.crosses(other, hex, dt, ctx) {
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
            .filter_map(|t| t.dit.map(|(_, revenue, _)| revenue))
            .chain(cities.iter().map(|c| c.revenue))
            .collect();
        revenues.sort_unstable();
        revenues.dedup();
        for (cx, city) in cities.iter().enumerate() {
            let mut layer = DrawLayer::Under;
            for (i, track) in tracks.iter().enumerate() {
                // Tracks must start or end at a city, rather than passing
                // through a city. This allows routes to identify a track
                // by tile and the track index, rather than needing to worry
                // about subsets of a track.
                if track.connected_to_fill(city, hex, ctx) {
                    let track_layer = track_layers.get(&i).unwrap_or(&layer);
                    layer = std::cmp::max(layer, *track_layer);
                } else if track.intersects_fill(city, hex, dt, ctx) {
                    println!("WARNING: track crosses city, tile {}", name);
                    let track_layer = track_layers.get(&i).unwrap_or(&layer);
                    layer = std::cmp::max(layer, *track_layer);
                }
            }
            if verbose {
                println!("    City #{} in layer {:?}", cx, layer);
            }
            cities_tbl.entry(layer).or_insert_with(Vec::new).push(cx)
        }
        for (i, _track) in tracks.iter().enumerate() {
            let layer = track_layers.get(&i).unwrap();
            tracks_tbl.entry(*layer).or_insert_with(Vec::new).push(i)
        }
        let conns = Connections::new(&tracks, &cities, hex);
        Self {
            colour,
            name,
            tracks,
            cities,
            tracks_tbl,
            cities_tbl,
            revenues,
            labels: vec![],
            show_tile_name: true,
            conns,
        }
    }

    /// Do not display the tile name when drawing the tile.
    pub fn hide_tile_name(mut self) -> Self {
        self.show_tile_name = false;
        self
    }

    /// Returns whether the tile name is displayed when drawing the tile.
    pub fn is_tile_name_visible(&self) -> bool {
        self.show_tile_name
    }

    pub fn connections(&self, from: &Connection) -> Option<&[Connection]> {
        self.conns.from(from)
    }

    // TODO: verify labels (e.g., one revenue label for each revenue ix)

    pub fn label<P>(mut self, label: Label, pos: P) -> Self
    where
        P: Into<HexPosition>,
    {
        self.labels.push((label, pos.into()));
        self
    }

    pub fn tracks(&self) -> &[Track] {
        self.tracks.as_slice()
    }

    pub fn dits(&self) -> &[Dit] {
        self.conns.dits()
    }

    pub fn cities(&self) -> &[City] {
        self.cities.as_slice()
    }

    pub fn labels(&self) -> &[LabelAndPos] {
        self.labels.as_slice()
    }

    /// Returns the city that corresponds to the provided token location.
    pub fn city(&self, space: &TokenSpace) -> Option<&City> {
        self.cities.get(space.city_ix)
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
            let line_cap = ctx.line_cap();
            ctx.set_line_cap(cairo::LineCap::Round);
            for coord in track.coords(hex, 0.1) {
                ctx.new_path();
                ctx.move_to(coord.x, coord.y);
                ctx.line_to(coord.x, coord.y);
                ctx.stroke().unwrap();
            }
            ctx.set_line_cap(line_cap);
        }
    }

    #[allow(dead_code)]
    fn dit_coords_in_red(&self, ctx: &Context, hex: &Hex) {
        use DrawLayer::*;

        // Draw the centre of each dit on a track segment as a red dot.
        for layer in &[Under, Normal, Over, Topmost] {
            let empty = vec![];
            ctx.set_source_rgb(1.0, 0.0, 0.0);
            let line_cap = ctx.line_cap();
            ctx.set_line_cap(cairo::LineCap::Round);
            for ix in self.tracks_tbl.get(layer).unwrap_or(&empty) {
                let track = self.tracks[*ix];
                if let Some(coord) = track.dit_coord(hex) {
                    ctx.new_path();
                    ctx.move_to(coord.x, coord.y);
                    ctx.line_to(coord.x, coord.y);
                    ctx.stroke().unwrap();
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

        for ix in self.tracks_tbl.get(layer).unwrap_or(&empty) {
            let track = self.tracks[*ix];
            track.draw_circle_dit(hex, ctx);
        }

        // NOTE: draw coordinates along track in red.
        // self.coords_in_red(layer, ctx, hex);

        let empty = vec![];
        for ix in self.cities_tbl.get(layer).unwrap_or(&empty) {
            let city = self.cities[*ix];
            city.draw_fg(hex, ctx)
        }
    }

    pub fn revenues(&self) -> &[usize] {
        &self.revenues
    }

    pub fn draw(&self, ctx: &Context, hex: &Hex) {
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
        // Draw the tile name, except for special tiles such as those that are
        // part of the initial map and are not truly "tiles" as such.
        if self.show_tile_name {
            Colour::BLACK.apply_colour(ctx);
            let hex_pos = HexPosition::Corner(HexCorner::BottomRight, None);
            Label::TileName.draw(ctx, hex, &hex_pos, self);
        }
        // Draw other tile labels.
        for (label, pos) in &self.labels {
            label.draw(ctx, hex, pos, self);
        }
    }

    pub fn token_spaces(&self) -> Vec<TokenSpace> {
        self.cities
            .iter()
            .enumerate()
            .flat_map(|(city_ix, city)| {
                city.token_ixs()
                    .into_iter()
                    .map(|token_ix| TokenSpace { city_ix, token_ix })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn city_token_spaces(&self, city_ix: usize) -> Vec<TokenSpace> {
        self.cities[city_ix]
            .token_ixs()
            .into_iter()
            .map(|token_ix| TokenSpace { city_ix, token_ix })
            .collect()
    }

    pub fn token_space_count(&self) -> usize {
        self.token_spaces().len()
    }

    pub fn dit_count(&self) -> usize {
        self.tracks
            .iter()
            .filter(|track| track.dit.is_some())
            .count()
    }

    pub fn define_token_space(
        &self,
        space: &TokenSpace,
        hex: &Hex,
        ctx: &Context,
    ) -> bool {
        if space.city_ix >= self.cities.len() {
            return false;
        }
        let city = self.cities[space.city_ix];
        city.define_token_path(space.token_ix, hex, ctx)
    }

    /// Check whether a tile can be upgraded to another tile.
    pub fn can_upgrade_to(&self, other: &Tile) -> bool {
        // Check whether the new tile's colour is correct.
        if let Some(colour) = self.colour.next_phase() {
            if other.colour != colour {
                return false;
            }
        }
        // Tiles must have the same number of dits.
        if self.dit_count() != other.dit_count() {
            return false;
        }
        let self_tok_spaces = self.token_space_count();
        let other_tok_spaces = other.token_space_count();
        // City tiles can only be upgraded to from existing city tiles.
        if self_tok_spaces == 0 && other_tok_spaces > 0 {
            return false;
        }
        // Check whether the new tile has at least as many token spaces.
        if self_tok_spaces > other_tok_spaces {
            return false;
        }
        // TODO: other checks, such as preserving track connectivity?
        // That would require having access to the map, so this would have to
        // be an additional layer of filtering provided by the map itself.
        true
    }

    /// Determines the surface size for this tile, which includes a small
    /// margin on all four sides.
    fn surface_width(&self, hex: &Hex) -> f64 {
        let margin = hex.theme.tile_margin.absolute(hex);
        hex.max_d + 2.0 * margin
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
        let width = self.surface_width(hex);
        let dim = width as i32;
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, dim, dim)
                .map_err(|_status| "Can't create surface")?;
        let ctx = cairo::Context::new(&surface)
            .expect("Can't create cairo::Context");
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
        let width = self.surface_width(hex);
        let surface = cairo::SvgSurface::for_stream(width, width, stream)
            .map_err(|_status| "Can't create surface")?;
        let ctx = cairo::Context::new(&surface)
            .expect("Can't create cairo::Context");
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
        let ctx = cairo::Context::new(&surface)
            .expect("Can't create cairo::Context");
        ctx.translate(width / 2.0, width / 2.0);
        self.draw(&ctx, hex);
        surface
            .finish_output_stream()
            .map(|_stream| ())
            .map_err(|err| err.error.into())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenSpace {
    city_ix: usize,
    token_ix: usize,
}

impl TokenSpace {
    pub fn city_ix(&self) -> usize {
        self.city_ix
    }
}

#[cfg(test)]
/// Tests that check whether `Tile` appropriately layers the tile elements and
/// correctly detects their connectivity.
mod tests {
    use crate::*;

    use super::DrawLayer::*;
    use n18hex::{Hex, HexColour::*, HexFace::*};
    use TrackEnd::*;

    static HEX_DIAMETER: f64 = 150.0;

    #[test]
    /// Constructs a tile with two track segments that do not cross each
    /// other, and checks that both track segments are drawn in the `Normal`
    /// layer.
    fn no_overlaps_one_layer() {
        let hex = Hex::new(HEX_DIAMETER);
        let tile = Tile::new(
            Yellow,
            "Test".to_string(),
            vec![
                Track::hard_l(Bottom).with_span(0.0, 0.5).with_dit(
                    End,
                    10,
                    DitShape::Bar,
                ),
                Track::hard_l(Bottom).with_span(0.5, 1.0),
            ],
            vec![],
            &hex,
        );
        let items_opt = tile.tracks_tbl.get(&Normal);
        assert!(items_opt.is_some(), "No items in Normal draw layer");
        let items = items_opt.unwrap();
        assert_eq!(
            items.len(),
            2,
            "Expected two tracks in Normal draw layer"
        );
        assert_eq!(
            tile.tracks_tbl.len(),
            1,
            "Expected only one drawing layer"
        );
    }

    #[test]
    /// Constructs a tile with two track segments that cross each other, and
    /// checks that the track segments are drawn in different layers.
    fn one_overlap_two_layers() {
        let hex = Hex::new(HEX_DIAMETER);
        let tile = Tile::new(
            Yellow,
            "Test".to_string(),
            vec![Track::straight(Bottom), Track::straight(UpperLeft)],
            vec![],
            &hex,
        );
        for layer in &[Under, Over] {
            let items_opt = tile.tracks_tbl.get(layer);
            assert!(
                items_opt.is_some(),
                "No items in {:?} draw layer",
                layer
            );
            let items = items_opt.unwrap();
            assert_eq!(
                items.len(),
                1,
                "Expected one track in {:?} draw layer",
                layer
            );
        }
        assert_eq!(tile.tracks_tbl.len(), 2, "Expected two drawing layers");
    }

    #[test]
    /// Constructs a tile with three track segments that cross each other, and
    /// checks that the track segments are drawn in different layers.
    fn two_overlaps_three_layers() {
        let hex = Hex::new(HEX_DIAMETER);
        let tile = Tile::new(
            Yellow,
            "Test".to_string(),
            vec![
                Track::straight(Bottom),
                Track::straight(UpperLeft),
                Track::straight(UpperRight),
            ],
            vec![],
            &hex,
        );
        for layer in &[Under, Over, Topmost] {
            let items_opt = tile.tracks_tbl.get(layer);
            assert!(
                items_opt.is_some(),
                "No items in {:?} draw layer",
                layer
            );
            let items = items_opt.unwrap();
            assert_eq!(
                items.len(),
                1,
                "Expected one track in {:?} draw layer",
                layer
            );
        }
        assert_eq!(tile.tracks_tbl.len(), 3, "Expected three drawing layers");
    }
}
