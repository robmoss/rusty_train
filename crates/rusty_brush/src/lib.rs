use cairo::Context;

use rusty_hex::Hex;
use rusty_map::{HexAddress, HexIter, Map};
use rusty_route::{Pair, Path, StopLocation};
use rusty_tile::connection::Connection;
use rusty_tile::draw::Draw;
use rusty_tile::track::DitShape;

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

pub fn highlight_routes<C>(
    hex: &Hex,
    ctx: &Context,
    map: &Map,
    pairs: &[Pair],
    colour_fn: C,
) where
    C: Fn(usize) -> (f64, f64, f64, f64),
{
    for (ix, pair) in pairs.iter().enumerate() {
        let (red, green, blue, alpha) = colour_fn(ix);
        ctx.set_source_rgba(red, green, blue, alpha);
        highlight_route(&hex, &ctx, &map, &pair.path)
    }
}

pub fn highlight_route(hex: &Hex, ctx: &Context, map: &Map, path: &Path) {
    ctx.set_line_width(hex.max_d * 0.025);
    let source = ctx.get_source();
    let line_width = ctx.get_line_width();

    // Draw track segments first.
    for step in &path.steps {
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

    // Then draw visited cities and dits.
    for visit in &path.visits {
        let m = map.prepare_to_draw(visit.addr, &hex, &ctx);
        let tile = map.tile_at(visit.addr).expect("Invalid step hex");
        if visit.revenue > 0 {
            println!("Stopping at {} for ${}", visit.addr, visit.revenue)
        } else {
            println!("Skipping {}", visit.addr)
        }
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
                            let name =
                                map.tokens().get_name(&map_token).unwrap();
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
