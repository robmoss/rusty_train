use crate::city::City;
use crate::draw::Draw;
use crate::hex::{Hex, HexColour, HexPosition};
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

/// A tile contains some number of track segments and cities.
#[derive(PartialEq)]
pub struct Tile {
    pub colour: HexColour,
    pub name: String,
    tracks: Vec<Track>,
    cities: Vec<City>,
    // Track indices by drawing layer.
    tracks_tbl: HashMap<DrawLayer, Vec<usize>>,
    // City indices by drawing layer.
    cities_tbl: HashMap<DrawLayer, Vec<usize>>,
    // TODO: revenue here as Option<usize> ???
    // TODO: label(s) as Vec<Label> ???
    // TODO: revenue_pos: Vec<Position> and display unique revenues in order
    // from smallest to largest!
    // revenue: Option<usize>,
    revenues: Vec<usize>,
    labels: Vec<(Label, HexPosition)>,
    // corner_labels: Vec<(Label, HexCorner)>,
    // face_labels: Vec<(Label, HexFace)>,
    // TODO: have a way to look up token spaces by index!
}

impl Tile {
    // pub fn new<P>(
    pub fn new(
        colour: HexColour,
        name: String,
        tracks: Vec<Track>,
        cities: Vec<City>,
        // labels: Vec<(Label, P)>,
        ctx: &Context,
        hex: &Hex,
    ) -> Self
// where
    //     P: Into<HexPosition>,
    {
        // TODO: check track connectivity and crossing, determine layers
        // Also save this information in a form that's amenable for
        // building track networks ... ???
        // TODO: detect track segments that cross a city (and that this isn't
        // part of the clipped path) and break them into separate segments
        // (e.g., straight -> mid + mid; gentle_l -> ...)
        // Hmmm ... maybe not
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
                    // TODO: need to detect if track is already at Over
                    // and, if so, set other's layer to Top
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
        // TODO: there isn't necessarily a single revenue!!!
        // e.g., dit vs city
        // So we can't have a single label.
        // let track_revenues: Vec<usize> = tracks
        //     .iter()
        //     .filter_map(|t| t.dit.map(|(_, revenue)| revenue))
        //     .collect();
        // let city_revenues: Vec<usize> =
        //     cities.iter().map(|c| c.revenue).collect();
        let mut revenues: Vec<usize> = tracks
            .iter()
            .filter_map(|t| t.dit.map(|(_, revenue)| revenue))
            .chain(cities.iter().map(|c| c.revenue))
            .collect();
        revenues.sort();
        revenues.dedup();
        // let track_min_rev = tracks
        //     .iter()
        //     .filter_map(|t| t.dit.map(|(_, rev)| rev))
        //     .min();
        // let track_max_rev = tracks
        //     .iter()
        //     .filter_map(|t| t.dit.map(|(_, rev)| rev))
        //     .max();
        // let track_revenue = if track_min_rev == track_max_rev {
        //     track_min_rev
        // } else {
        //     panic!(
        //         "Tile dit revenue ranges from {:?} to {:?}",
        //         track_min_rev, track_max_rev
        //     )
        // };
        // // TODO: track segments may have dits with revenue!
        // let city_min_rev = cities.iter().map(|c| c.revenue).min();
        // let city_max_rev = cities.iter().map(|c| c.revenue).max();
        // let city_revenue = if city_min_rev == city_max_rev {
        //     city_min_rev
        // } else {
        //     panic!(
        //         "Tile city revenue ranges from {:?} to {:?}",
        //         city_min_rev, city_max_rev
        //     )
        // };
        // let revenue = if track_revenue.is_none() {
        //     city_revenue
        // } else if city_revenue.is_none() {
        //     track_revenue
        // } else {
        //     panic!("Tile has dit and city revenue")
        // };
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
            // TODO: accept the labels as an argument!
            labels: vec![],
            // corner_labels: vec![],
            // face_labels: vec![],
        }
    }

    // pub fn label_at_corner(
    //     mut self,
    //     label: Label,
    //     corner: HexCorner,
    // ) -> Self {
    //     self.corner_labels.push((label, corner));
    //     self
    // }

    // pub fn label_at_face(mut self, label: Label, face: HexFace) -> Self {
    //     self.face_labels.push((label, face));
    //     self
    // }

    // TODO: verify labels (e.g., one revenue label for each revenue ix)

    pub fn label<P>(mut self, label: Label, pos: P) -> Self
    where
        P: Into<crate::hex::HexPosition>,
    {
        self.labels.push((label, pos.into()));
        self
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
                // self.revenue.map(|r| format!("{}", r)),
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
        // for (label, corner) in &self.corner_labels {
        //     if let Some(text) = self.label_text(label) {
        //         label.select_font(ctx);
        //         if let &Label::Revenue(_ix) = label {
        //             hex.draw_circ_text_corner(text.as_ref(), corner, ctx)
        //         } else {
        //             hex.draw_text_corner(text.as_ref(), corner, ctx)
        //         }
        //     }
        // }
        // for (label, face) in &self.face_labels {
        //     if let Some(text) = self.label_text(label) {
        //         label.select_font(ctx);
        //         if let &Label::Revenue(_ix) = label {
        //             hex.draw_circ_text_face(text.as_ref(), face, ctx)
        //         } else {
        //             hex.draw_text_face(text.as_ref(), face, ctx)
        //         }
        //     }
        // }
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
}
