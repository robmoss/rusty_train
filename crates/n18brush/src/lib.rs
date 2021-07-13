use cairo::Context;

use n18hex::Hex;
use n18map::{HexAddress, HexIter, Map};
use n18route::{Path, Route, Step, StopLocation, Visit};
use n18tile::{Connection, DitShape, Draw, Tile, TokenSpace};
use n18token::Token;

pub fn draw_hex_backgrounds(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
) {
    hex_iter.restart();
    for _ in &mut hex_iter {
        // Draw a thick black border on all hexes.
        // This will give map edges a clear border.
        hex.define_boundary(ctx);
        ctx.set_source_rgb(0.741, 0.86, 0.741);
        ctx.fill();
    }
    hex_iter.restart();
    for _ in &mut hex_iter {
        hex.define_boundary(ctx);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.05);
        ctx.stroke();
    }

    hex_iter.restart();
}

pub fn draw_tiles(hex: &Hex, ctx: &Context, mut hex_iter: &mut HexIter<'_>) {
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        if let Some((tile, token_spaces)) = hex_state.tile_state {
            // Draw the tile and any tokens.
            tile.draw(&ctx, &hex);
            for (token_space, map_token) in token_spaces.iter() {
                if tile.define_token_space(&token_space, &hex, &ctx) {
                    let name = hex_state
                        .available_tokens
                        .get_name(&map_token)
                        .unwrap();
                    map_token.draw(&hex, &ctx, name, hex_state.tile_rotation);
                } else {
                    println!("Could not define token space.")
                }
            }
        } else {
            // Fill empty hexes with a background colour.
            draw_empty_hex(&hex, &ctx);
        }
    }

    hex_iter.restart();
}

pub fn draw_empty_hex(hex: &Hex, ctx: &Context) {
    hex.define_boundary(ctx);
    ctx.set_source_rgb(0.741, 0.86, 0.741);
    ctx.fill();
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
            ctx.set_source_rgb(0.7, 0.7, 0.7);
            hex.define_boundary(ctx);
            ctx.set_line_width(hex.max_d * 0.01);
            ctx.stroke();
        }
    }

    hex_iter.restart();
}

pub fn draw_map(hex: &Hex, ctx: &Context, hex_iter: &mut HexIter<'_>) {
    draw_hex_backgrounds(hex, ctx, hex_iter);
    draw_tiles(hex, ctx, hex_iter);
    outline_empty_hexes(hex, ctx, hex_iter);
}

pub fn draw_barriers_subset(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    mut hex_iter: &mut HexIter<'_>,
) {
    let cap = ctx.get_line_cap();
    ctx.set_line_cap(cairo::LineCap::Round);
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
            ctx.set_line_width(hex.max_d * 0.05);
            ctx.set_source_rgb(0.1, 0.1, 0.6);
            ctx.move_to(c0.x, c0.y);
            ctx.line_to(c1.x, c1.y);
            ctx.stroke();
        }
    }
    ctx.set_line_cap(cap);
}

pub fn draw_barriers(hex: &Hex, ctx: &Context, map: &Map) {
    let cap = ctx.get_line_cap();
    ctx.set_line_cap(cairo::LineCap::Round);
    for (addr, face) in map.barriers() {
        let m = map.prepare_to_draw(*addr, hex, ctx);
        let corners = face.corners();
        let c0 = hex.corner_coord(&corners.0);
        let c1 = hex.corner_coord(&corners.1);
        ctx.set_line_width(hex.max_d * 0.05);
        ctx.set_source_rgb(0.1, 0.1, 0.6);
        ctx.move_to(c0.x, c0.y);
        ctx.line_to(c1.x, c1.y);
        ctx.stroke();
        ctx.set_matrix(m);
    }
    ctx.set_line_cap(cap);
}

/// Highlights tokens that satisfy a predicate by drawing borders around them
/// and optionally filling the token space with, e.g., a semi-transparent
/// colour.
pub fn highlight_tokens<P>(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
    mut predicate: P,
    border: n18token::Colour,
    fill: Option<n18token::Colour>,
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
                        fill_colour.apply_to(ctx);
                        ctx.fill_preserve();
                    }
                    border.apply_to(ctx);
                    ctx.set_line_width(hex.max_d * 0.025);
                    ctx.stroke_preserve();
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
    border: n18token::Colour,
) {
    if let Some(tile) = map.tile_at(hex_addr) {
        let m = map.prepare_to_draw(hex_addr, hex, ctx);
        tile.define_token_space(token_space, hex, ctx);
        border.apply_to(ctx);
        ctx.set_line_width(hex.max_d * 0.025);
        ctx.stroke_preserve();
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
    border: Option<n18token::Colour>,
) where
    P: FnMut(&HexAddress) -> bool,
{
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        let highlight = predicate(&hex_state.addr);
        if highlight {
            // Draw the active hex with a coloured border.
            if let Some(colour) = border {
                colour.apply_to(ctx);
                ctx.set_line_width(hex.max_d * 0.02);
                hex.define_boundary(ctx);
                ctx.stroke();
            }
        } else {
            // Cover all other tiles with a partially-transparent layer.
            ctx.set_source_rgba(1.0, 1.0, 1.0, 0.25);
            hex.define_boundary(ctx);
            ctx.fill();
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
    r: f64,
    g: f64,
    b: f64,
) {
    hex_iter.restart();
    for hex_state in &mut hex_iter {
        if active_hex == &Some(hex_state.addr) {
            // Draw the active hex with a coloured border.
            ctx.set_source_rgb(r, g, b);
            ctx.set_line_width(hex.max_d * 0.02);
            hex.define_boundary(ctx);
            ctx.stroke();
        } else {
            // Cover all other tiles with a partially-transparent layer.
            ctx.set_source_rgba(1.0, 1.0, 1.0, 0.25);
            hex.define_boundary(ctx);
            ctx.fill();
        }
    }

    hex_iter.restart();
}

// TODO: provide a default colour cycle?
pub fn highlight_routes<C, R>(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    routes: &[R],
    colour_fn: C,
) where
    C: Fn(usize) -> (f64, f64, f64, f64),
    R: AsRef<Route>,
{
    for (ix, route) in routes.iter().enumerate() {
        let (red, green, blue, alpha) = colour_fn(ix);
        ctx.set_source_rgba(red, green, blue, alpha);
        highlight_route(&hex, &ctx, &map, route.as_ref())
    }
}

pub fn highlight_paths<C>(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    paths: &[Path],
    colour_fn: C,
) where
    C: Fn(usize) -> (f64, f64, f64, f64),
{
    for (ix, path) in paths.iter().enumerate() {
        let (red, green, blue, alpha) = colour_fn(ix);
        ctx.set_source_rgba(red, green, blue, alpha);
        highlight_path(&hex, &ctx, &map, &path)
    }
}

fn highlight_steps(hex: &Hex, ctx: &Context, map: &Map, steps: &[Step]) {
    ctx.set_line_width(hex.max_d * 0.025);

    // Draw track segments first.
    for step in steps {
        let m = map.prepare_to_draw(step.addr, &hex, &ctx);
        let tile = map.tile_at(step.addr).expect("Invalid step hex");

        if let Connection::Track { ix, end: _ } = step.conn {
            let track = tile.tracks()[ix];
            track.define_path(&hex, &ctx);
            // NOTE: hack to replace the black part of the track.
            ctx.set_line_width(hex.max_d * 0.08);
            ctx.stroke();
        }
        ctx.set_matrix(m);
    }
}

fn highlight_visits(hex: &Hex, ctx: &Context, map: &Map, visits: &[Visit]) {
    ctx.set_line_width(hex.max_d * 0.025);
    let source = ctx.get_source();
    let line_width = ctx.get_line_width();

    for visit in visits {
        let m = map.prepare_to_draw(visit.addr, &hex, &ctx);
        let tile = map.tile_at(visit.addr).expect("Invalid step hex");
        match visit.visits {
            StopLocation::City { ix } => {
                let city = tile.cities()[ix];
                city.draw_fg(&hex, &ctx);
                // Draw the tokens first.
                if let Some(hex_state) = map.get_hex(visit.addr) {
                    let rotn = hex_state.radians();
                    let tokens_table = hex_state.get_tokens();
                    for (token_space, map_token) in tokens_table.iter() {
                        if tile.define_token_space(&token_space, &hex, &ctx) {
                            let name = map.get_token_name(&map_token);
                            map_token.draw(&hex, &ctx, name, rotn);
                        } else {
                            println!("Could not define token space.")
                        }
                    }
                }
                // Then draw a border around the city.
                if visit.revenue > 0 {
                    ctx.set_source(&source);
                } else {
                    // NOTE: the train did not stop here.
                    ctx.set_source_rgb(0.0, 0.0, 0.0);
                }
                ctx.set_line_width(line_width);
                city.define_boundary(&hex, &ctx);
                ctx.stroke();
            }
            StopLocation::Dit { ix } => {
                let dit = tile.dits()[ix];
                let track = tile.tracks()[dit.track_ix];
                if visit.revenue > 0 {
                    ctx.set_source(&source);
                } else {
                    // NOTE: the train did not stop here.
                    ctx.set_source_rgb(0.0, 0.0, 0.0);
                }
                let dit_shape = track.dit.unwrap().2;
                // TODO: need a better API for drawing dit
                // background and foreground.
                match dit_shape {
                    DitShape::Bar => {
                        ctx.set_line_width(hex.max_d * 0.08);
                        track.draw_dit_ends(0.10, &hex, &ctx);
                    }
                    DitShape::Circle => {
                        track.define_circle_dit(&hex, &ctx);
                        ctx.fill_preserve();
                        ctx.set_source_rgb(1.0, 1.0, 1.0);
                        ctx.set_line_width(hex.max_d * 0.01);
                        ctx.stroke();
                    }
                }
            }
        }
        ctx.set_matrix(m);
    }
}

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
                let ctx = cairo::Context::new(&surf);
                draw_fn(&ctx);
                surf.finish();
            }
            Png => {
                let surf = cairo::ImageSurface::create(
                    cairo::Format::ARgb32,
                    width as i32,
                    height as i32,
                )?;
                let ctx = cairo::Context::new(&surf);
                let mut out_file = std::fs::File::create(dest.as_ref())
                    .expect("Could not create output file");
                draw_fn(&ctx);
                surf.write_to_png(&mut out_file)?;
            }
            Svg => {
                let surf = cairo::SvgSurface::new(width, height, Some(dest))?;
                let ctx = cairo::Context::new(&surf);
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
    let ctx = cairo::Context::new(&rec_surf);
    draw_fn(&ctx);
    let exts = rec_surf.ink_extents();
    let width = exts.2 + 2.0 * exts.0;
    let height = exts.3 + 2.0 * exts.1;
    Some((width, height))
}
