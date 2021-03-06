use cairo::Context;
use log::debug;

use n18hex::{Colour, Hex, HexColour};
use n18map::{HexAddress, HexIter, Map};
use n18route::{Path, Route, Step, StopLocation, Visit};
use n18tile::{Connection, DitShape, Draw, Tile, TokenSpace};
use n18token::Token;

/// Clears the surface with a uniform colour, or makes the surface entirely
/// transparent if no colour is provided.
pub fn clear_surface<C>(ctx: &Context, colour: C)
where
    C: Into<Option<Colour>>,
{
    // See https://www.cairographics.org/FAQ/#clear_a_surface for details.
    let colour_opt = colour.into();
    if let Some(colour) = colour_opt {
        let operator = ctx.operator();
        ctx.set_operator(cairo::Operator::Source);
        colour.apply_colour(ctx);
        ctx.paint().unwrap();
        ctx.set_operator(operator);
    } else {
        let operator = ctx.operator();
        ctx.set_operator(cairo::Operator::Clear);
        ctx.paint().unwrap();
        ctx.set_operator(operator);
    }
}

pub fn draw_hex_backgrounds(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
) {
    // Fill each hex with the default background colour.
    hex_iter.restart();
    for _ in &mut hex_iter {
        hex.define_boundary(ctx);
        hex.theme.apply_hex_colour(ctx, HexColour::Empty);
        ctx.fill().unwrap();
    }
    // Draw a thick border around each hex, so that after the map tiles and
    // other layers have been drawn on top of this, the map edges will have a
    // clear border.
    hex_iter.restart();
    for _ in &mut hex_iter {
        hex.define_boundary(ctx);
        hex.theme.map_border.apply_line_and_stroke(ctx, hex);
        ctx.stroke().unwrap();
    }

    hex_iter.restart();
}

pub fn draw_tiles(hex: &Hex, ctx: &Context, mut hex_iter: &mut HexIter<'_>) {
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        if let Some((tile, token_spaces)) = hex_state.tile_state {
            // Draw the tile and any tokens.
            tile.draw(ctx, hex);
            for (token_space, map_token) in token_spaces.iter() {
                if tile.define_token_space(token_space, hex, ctx) {
                    let name =
                        hex_state.available_tokens.name(map_token).unwrap();
                    map_token.draw(hex, ctx, name, hex_state.tile_rotation);
                } else {
                    println!("Could not define token space.")
                }
            }
        } else {
            // Fill empty hexes with a background colour.
            draw_empty_hex(hex, ctx);
        }
    }

    hex_iter.restart();
}

pub fn draw_empty_hex(hex: &Hex, ctx: &Context) {
    hex.define_boundary(ctx);
    hex.theme.apply_hex_colour(ctx, HexColour::Empty);
    ctx.fill().unwrap();
}

pub fn outline_empty_hexes(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
) {
    // Draw a thin grey border around empty hexes.
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        if hex_state.tile_state.is_none() {
            hex.define_boundary(ctx);
            hex.theme.hex_border.apply_line_and_stroke(ctx, hex);
            ctx.stroke().unwrap();
        }
    }

    hex_iter.restart();
}

/// Draws the core map layers: hex backgrounds, tiles, empty hex borders, and
/// track barriers.
pub fn draw_map(hex: &Hex, ctx: &Context, hex_iter: &mut HexIter<'_>) {
    draw_hex_backgrounds(hex, ctx, hex_iter);
    draw_tiles(hex, ctx, hex_iter);
    outline_empty_hexes(hex, ctx, hex_iter);
    // Note: use the fully-quantified syntax to call HexIter::map() rather
    // than Iterator::map() on `hex_iter`.
    draw_barriers(hex, ctx, HexIter::map(hex_iter));
}

/// Draws the core map layers for a subset of map hexes: hex backgrounds,
/// tiles, empty hex borders, and track barriers.
///
/// The subset is defined by `hex_iter`; see [Map::hex_subset_iter].
pub fn draw_map_subset(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    hex_iter: &mut HexIter<'_>,
) {
    draw_hex_backgrounds(hex, ctx, hex_iter);
    draw_tiles(hex, ctx, hex_iter);
    outline_empty_hexes(hex, ctx, hex_iter);
    draw_barriers_subset(hex, ctx, map, hex_iter);
}

pub fn draw_barriers_subset(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    mut hex_iter: &mut HexIter<'_>,
) {
    let barriers = map.barriers();
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        let hex_addr = hex_state.addr;
        for (addr, face) in barriers {
            if hex_addr != *addr {
                continue;
            }
            let corners = face.corners();
            let c0 = hex.corner_coord(&corners.0);
            let c1 = hex.corner_coord(&corners.1);
            ctx.move_to(c0.x, c0.y);
            ctx.line_to(c1.x, c1.y);
            hex.theme.hex_barrier.apply_line_and_stroke(ctx, hex);
            ctx.stroke().unwrap();
        }
    }
}

pub fn draw_barriers(hex: &Hex, ctx: &Context, map: &Map) {
    for (addr, face) in map.barriers() {
        let m = map.prepare_to_draw(*addr, hex, ctx);
        let corners = face.corners();
        let c0 = hex.corner_coord(&corners.0);
        let c1 = hex.corner_coord(&corners.1);
        ctx.move_to(c0.x, c0.y);
        ctx.line_to(c1.x, c1.y);
        hex.theme.hex_barrier.apply_line_and_stroke(ctx, hex);
        ctx.stroke().unwrap();
        ctx.set_matrix(m);
    }
}

/// Highlights tokens that satisfy a predicate by drawing borders around them
/// and optionally filling the token space with, e.g., a semi-transparent
/// colour.
pub fn highlight_tokens<P>(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
    mut predicate: P,
    border: n18hex::Colour,
    fill: Option<n18hex::Colour>,
) where
    P: FnMut(&HexAddress, &Tile, &TokenSpace, &Token) -> bool,
{
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        let hex_addr = &hex_state.addr;
        if let Some((tile, tokens)) = hex_state.tile_state {
            for (token_space, token) in tokens {
                if predicate(hex_addr, tile, token_space, token) {
                    tile.define_token_space(token_space, hex, ctx);
                    if let Some(fill_colour) = fill {
                        fill_colour.apply_colour(ctx);
                        ctx.fill_preserve().unwrap();
                    }
                    border.apply_colour(ctx);
                    hex.theme.token_space_highlight.apply_line(ctx, hex);
                    ctx.stroke_preserve().unwrap();
                }
            }
        }
    }
}

/// Highlights a token space by drawing a border around it.
pub fn highlight_token_space(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    hex_addr: HexAddress,
    token_space: &TokenSpace,
    border: n18hex::Colour,
) {
    if let Some(tile) = map.tile_at(hex_addr) {
        let m = map.prepare_to_draw(hex_addr, hex, ctx);
        tile.define_token_space(token_space, hex, ctx);
        border.apply_colour(ctx);
        hex.theme.token_space_highlight.apply_line(ctx, hex);
        ctx.stroke_preserve().unwrap();
        ctx.set_matrix(m);
    }
}

/// Highlights map hexes that satisfy a predicate by covering all other hexes
/// with a partially-transparent layer.
///
/// This also draws a coloured border around the highlighted hexes if `border`
/// is not `None`.
pub fn highlight_hexes<P>(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
    mut predicate: P,
    border: Option<n18hex::Colour>,
) where
    P: FnMut(&HexAddress) -> bool,
{
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        let highlight = predicate(&hex_state.addr);
        if highlight {
            // Draw the active hex with a coloured border.
            if let Some(colour) = border {
                colour.apply_colour(ctx);
                hex.theme.hex_highlight.apply_line(ctx, hex);
                hex.define_boundary(ctx);
                ctx.stroke().unwrap();
            }
        } else {
            // Cover all other tiles with a partially-transparent layer.
            hex.theme.hex_border.apply_fill(ctx);
            hex.define_boundary(ctx);
            ctx.fill().unwrap();
        }
    }
}

/// Highlights the active map hex by covering all other hexes with a
/// partially-transparent layer.
///
/// This also draws a coloured border around the active map hex.
pub fn highlight_active_hex(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
    active_hex: &Option<HexAddress>,
    border: n18hex::Colour,
) {
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        if active_hex == &Some(hex_state.addr) {
            // Draw the active hex with a coloured border.
            border.apply_colour(ctx);
            hex.theme.hex_highlight.apply_line(ctx, hex);
            hex.define_boundary(ctx);
            ctx.stroke().unwrap();
        } else {
            // Cover all other tiles with a partially-transparent layer.
            hex.theme.hex_border.apply_fill(ctx);
            hex.define_boundary(ctx);
            ctx.fill().unwrap();
        }
    }

    hex_iter.restart();
}

/// Highlights routes, using a different colour for each route.
pub fn highlight_routes<F, C, R>(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    routes: &[R],
    colour_fn: F,
) where
    F: Fn(usize) -> C,
    C: Into<Colour>,
    R: AsRef<Route>,
{
    for (ix, route) in routes.iter().enumerate() {
        colour_fn(ix).into().apply_colour(ctx);
        highlight_route(hex, ctx, map, route.as_ref())
    }
}

pub fn highlight_paths<F, C>(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    paths: &[Path],
    colour_fn: F,
) where
    F: Fn(usize) -> C,
    C: Into<Colour>,
{
    for (ix, path) in paths.iter().enumerate() {
        colour_fn(ix).into().apply_colour(ctx);
        highlight_path(hex, ctx, map, path)
    }
}

fn highlight_steps(hex: &Hex, ctx: &Context, map: &Map, steps: &[Step]) {
    // Draw track segments first.
    for step in steps {
        let m = map.prepare_to_draw(step.addr, hex, ctx);
        let tile = map.tile_at(step.addr).expect("Invalid step hex");

        // For tiles that only show off-board track segments, highlight only
        // these segments.
        if tile.only_draw_offboard_track() {
            if let Connection::Face { face } = step.conn {
                if tile.define_offboard_track_inner_path(ctx, hex, &face) {
                    ctx.fill().unwrap()
                }
            }
        } else if let Connection::Track { ix, end: _ } = step.conn {
            let track = tile.tracks()[ix];
            track.define_path(hex, ctx);
            // NOTE: cover the inner (black) part of the track.
            hex.theme.track_inner.apply_line(ctx, hex);
            ctx.stroke().unwrap();
        }
        ctx.set_matrix(m);
    }
}

fn highlight_visits(hex: &Hex, ctx: &Context, map: &Map, visits: &[Visit]) {
    let source = ctx.source();

    for visit in visits {
        let tile = map.tile_at(visit.addr).expect("Invalid step hex");

        // Don't highlight visits on off-board tiles that only show their
        // off-board track segments.
        if tile.only_draw_offboard_track() {
            continue;
        }

        let m = map.prepare_to_draw(visit.addr, hex, ctx);
        match visit.visits {
            StopLocation::City { ix } => {
                let city = tile.cities()[ix];
                city.draw_fg(hex, ctx);
                // Draw the tokens first.
                if let Some(hex_state) = map.hex_state(visit.addr) {
                    let rotn = hex_state.radians();
                    let tokens_table = hex_state.tokens();
                    for (token_space, map_token) in tokens_table.iter() {
                        if tile.define_token_space(token_space, hex, ctx) {
                            let name = map.token_name(map_token);
                            map_token.draw(hex, ctx, name, rotn);
                        } else {
                            println!("Could not define token space.")
                        }
                    }
                }
                // Then draw a border around the city.
                if visit.revenue > 0 {
                    ctx.set_source(&source).unwrap();
                } else {
                    // NOTE: the train did not stop here, use the default
                    // track colour.
                    hex.theme.track_inner.apply_stroke(ctx);
                }
                hex.theme.token_space_highlight.apply_line(ctx, hex);
                city.define_boundary(hex, ctx);
                ctx.stroke().unwrap();
            }
            StopLocation::Dit { ix } => {
                let dit = tile.dits()[ix];
                let track = tile.tracks()[dit.track_ix];
                if visit.revenue > 0 {
                    ctx.set_source(&source).unwrap();
                } else {
                    // NOTE: the train did not stop here, use the default dit
                    // colour.
                    hex.theme.dit_inner.apply_stroke(ctx);
                }
                let dit_shape = track.dit.unwrap().2;
                // TODO: need a better API for drawing dit
                // background and foreground.
                match dit_shape {
                    DitShape::Bar => {
                        hex.theme.dit_inner.apply_line(ctx, hex);
                        track.draw_dit_ends_fg(hex, ctx);
                    }
                    DitShape::Circle => {
                        track.define_circle_dit(hex, ctx);
                        ctx.fill_preserve().unwrap();
                        hex.theme.dit_circle.apply_line_and_stroke(ctx, hex);
                        ctx.stroke().unwrap();
                    }
                }
            }
        }
        ctx.set_matrix(m);
    }
}

/// Highlights a single route, using the current source.
pub fn highlight_route(hex: &Hex, ctx: &Context, map: &Map, route: &Route) {
    // Draw track segments first.
    highlight_steps(hex, ctx, map, &route.steps);
    // Then draw visited cities and dits.
    highlight_visits(hex, ctx, map, &route.visits);
}

pub fn highlight_path(hex: &Hex, ctx: &Context, map: &Map, path: &Path) {
    // Draw track segments first.
    highlight_steps(hex, ctx, map, &path.steps);
    // Then draw visited cities and dits.
    highlight_visits(hex, ctx, map, &path.visits);
}

/// Draw an arbitrary tile at the specified map hex, rather than the tile that
/// is currently placed at the map hex (if any).
pub fn draw_tile_at(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    addr: &HexAddress,
    tile: &Tile,
    radians: f64,
) {
    let m = map.prepare_to_draw(*addr, hex, ctx);
    ctx.rotate(radians);
    tile.draw(ctx, hex);
    ctx.set_matrix(m);
}

/// Draw an arbitrary tile and tokens at the specified map hex, rather than
/// the tile and tokens that are currently placed at the map hex (if any).
///
/// Note that the rotation `radians` is applied **in addition to** the
/// currently-placed tile's rotation (if any).
///
/// This only draws tokens for which there is a matching token space (i.e., a
/// matching city index and a matching token index).
/// It ignores tokens spaces that do not belong to the provided tile, and
/// unknown token names.
/// It outputs a debug logging message for each ignored token space and token.
pub fn draw_tile_and_tokens_at<'a, T>(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    addr: &HexAddress,
    tile: &Tile,
    radians: f64,
    tokens: T,
) where
    T: IntoIterator<Item = (&'a TokenSpace, &'a Token)>,
{
    let m = map.prepare_to_draw(*addr, hex, ctx);
    // Retrieve the original tile's rotation, which has been applied.
    let orig_rotn =
        map.hex_state(*addr).map(|hs| hs.radians()).unwrap_or(0.0);
    // Apply this additional rotation to the specified tile.
    ctx.rotate(radians);
    // Account for the effective rotation (i.e., the combination of the
    // original tile and the specified tile) so that it can be corrected for
    // by `token.draw()`, below.
    let token_rotn = radians + orig_rotn;
    tile.draw(ctx, hex);
    for (token_space, token) in tokens.into_iter() {
        if tile.define_token_space(token_space, hex, ctx) {
            let tok_name = map.try_token_name(token);
            if let Some(name) = tok_name {
                // NOTE: `token_rotn` is the rotation (in radians) that will
                // be *reversed* when drawing the token.
                token.draw(hex, ctx, name, token_rotn);
            } else {
                debug!("Invalid token for this map: {:?}", token);
            }
        } else {
            debug!("Tile {} has no {:?}", tile.name, token_space);
        }
    }
    ctx.set_matrix(m);
}

/// Supported output image formats.
#[derive(Clone, Copy, Debug)]
pub enum ImageFormat {
    /// Write PDF (vector) files.
    Pdf,
    /// Write PNG (bitmap) files.
    Png,
    /// Write SVG (vector) files.
    Svg,
}

impl ImageFormat {
    /// Returns the filename extension associated with the image format.
    pub fn extension(&self) -> &str {
        use ImageFormat::*;
        match self {
            Pdf => "pdf",
            Png => "png",
            Svg => "svg",
        }
    }

    /// Saves the image drawn by `draw_fn` to an output file.
    pub fn save_image<F, P>(
        &self,
        width: f64,
        height: f64,
        draw_fn: F,
        dest: P,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&cairo::Context),
        P: std::convert::AsRef<std::path::Path>,
    {
        use ImageFormat::*;
        match self {
            Pdf => {
                let surf = cairo::PdfSurface::new(width, height, dest)?;
                let ctx = cairo::Context::new(&surf)?;
                draw_fn(&ctx);
                surf.finish();
            }
            Png => {
                let surf = cairo::ImageSurface::create(
                    cairo::Format::ARgb32,
                    width as i32,
                    height as i32,
                )?;
                let ctx = cairo::Context::new(&surf)?;
                let mut out_file = std::fs::File::create(dest.as_ref())
                    .expect("Could not create output file");
                draw_fn(&ctx);
                surf.write_to_png(&mut out_file)?;
            }
            Svg => {
                let surf = cairo::SvgSurface::new(width, height, Some(dest))?;
                let ctx = cairo::Context::new(&surf)?;
                draw_fn(&ctx);
                surf.finish();
            }
        }
        Ok(())
    }
}

/// Returns the width and height of the image drawn by `draw_fn`.
///
/// The returned dimensions include horizontal and vertical margins, which are
/// defined to be the horizontal and vertical distances between the origin and
/// the top-left corner `(x0, y0)` of the image.
/// If either `x0` or `y0` is negative, the returned dimensions will result in
/// a cropped image.
///
/// Returns `None` if unable to create a `cairo::RecordingSurface`.
pub fn image_size<F>(draw_fn: F) -> Option<(f64, f64)>
where
    F: FnOnce(&cairo::Context),
{
    let rec_surf =
        cairo::RecordingSurface::create(cairo::Content::Color, None).ok()?;
    let ctx = cairo::Context::new(&rec_surf).ok()?;
    draw_fn(&ctx);
    let exts = rec_surf.ink_extents();
    let width = exts.2 + 2.0 * exts.0;
    let height = exts.3 + 2.0 * exts.1;
    Some((width, height))
}
