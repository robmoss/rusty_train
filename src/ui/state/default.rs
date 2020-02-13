use cairo::Context;
use gtk::Inhibit;

use super::{Action, State};
use crate::draw::Draw;
use crate::hex::Hex;
use crate::map::{HexAddress, Map, Token};
use crate::ui::util;

/// The default state: selecting a tile.
pub struct Default {
    active_hex: Option<HexAddress>,
}

impl Default {
    pub fn new(map: &Map) -> Self {
        Default {
            active_hex: map.default_hex(),
        }
    }

    pub fn at_hex(addr: Option<HexAddress>) -> Self {
        Default { active_hex: addr }
    }
}

impl State for Default {
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        let mut hex_iter = map.hex_iter(hex, ctx);

        for _ in &mut hex_iter {
            // Draw a thick black border on all hexes.
            // This will give map edges a clear border.
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            hex.define_boundary(ctx);
            ctx.set_line_width(hex.max_d * 0.05);
            ctx.stroke();
        }

        hex_iter.restart();
        for (_addr, tile_opt) in &mut hex_iter {
            if let Some((tile, token_spaces)) = tile_opt {
                // Draw the tile and any tokens.
                tile.draw(ctx, hex);
                for (token_space, map_token) in token_spaces.iter() {
                    tile.define_token_space(&token_space, &hex, ctx);
                    map_token.draw_token(&hex, ctx);
                }
            } else {
                // Draw an empty hex.
                // TODO: draw "crosshairs" at hex intersections?
                ctx.set_source_rgb(0.7, 0.7, 0.7);
                hex.define_boundary(ctx);
                ctx.set_line_width(hex.max_d * 0.01);
                ctx.stroke();
            }
        }

        hex_iter.restart();
        for (addr, _tile_opt) in &mut hex_iter {
            if self.active_hex == Some(addr) {
                // Draw the active hex with a red border.
                ctx.set_source_rgb(0.7, 0.0, 0.0);
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

        // Search for paths that start from a specific city.
        // TODO: allow the user to select a token and switch to path-finding
        // mode, where pressing digits 0..9 set the maximum number of stops
        // (0 being unlimited).
        // TODO: move this code to the state-updating part, so it isn't
        // recalculated every time we need to draw!
        let now = std::time::Instant::now();
        let query = crate::route::search::Query {
            addr: HexAddress::new(3, 8),
            from: crate::connection::Connection::City { ix: 0 },
            token: Token::LP,
            max_visits: Some(3),
            skip_cities: false,
            skip_dits: true,
            conflict_rule:
                crate::route::conflict::ConflictRule::TrackOrCityHex,
        };
        let paths = crate::route::search::paths_from(map, &query);
        println!(
            "Enumerated {} routes in {}",
            paths.len(),
            now.elapsed().as_secs_f64()
        );
        if paths.len() == 0 {
            return;
        }

        // If there was at least one path, find one with the greatest revenue.
        let max_revenue =
            paths.iter().map(|path| path.revenue).max().unwrap();
        println!("Maximum revenue is {}", max_revenue);
        let best_path = paths
            .iter()
            .find(|path| path.revenue == max_revenue)
            .unwrap();

        // Draw this path in dark red.
        ctx.set_source_rgb(0.7, 0.1, 0.1);
        ctx.set_line_width(hex.max_d * 0.025);
        let source = ctx.get_source();
        let line_width = ctx.get_line_width();

        // Draw track segments first.
        for step in &best_path.steps {
            let m = map.prepare_to_draw(step.addr, hex, ctx);
            let tile = map.tile_at(step.addr).expect("Invalid step hex");

            use crate::connection::Connection::*;
            if let Track { ix, end: _ } = step.conn {
                let track = tile.tracks()[ix];
                track.define_path(hex, ctx);
                // NOTE: hack to replace the black part of the track.
                ctx.set_line_width(hex.max_d * 0.08);
                ctx.stroke();
            }
            ctx.set_matrix(m);
        }

        // Then draw visited cities.
        for step in &best_path.steps {
            let m = map.prepare_to_draw(step.addr, hex, ctx);
            let tile = map.tile_at(step.addr).expect("Invalid step hex");

            use crate::connection::Connection::*;
            if let City { ix } = step.conn {
                let city = tile.cities()[ix];
                city.draw_fg(hex, ctx);
                ctx.set_source(&source);
                ctx.set_line_width(line_width);
                city.define_boundary(hex, ctx);
                if city.tokens == crate::city::Tokens::Dit {
                    ctx.fill_preserve();
                }
                ctx.stroke();
                if let Some(hex_state) = map.get_hex(step.addr) {
                    let tokens_table = hex_state.get_tokens();
                    for (token_space, map_token) in tokens_table.iter() {
                        tile.define_token_space(&token_space, &hex, ctx);
                        map_token.draw_token(&hex, ctx);
                    }
                }
            }
            ctx.set_matrix(m);
        }
    }

    fn key_press(
        mut self: Box<Self>,
        hex: &Hex,
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        let modifiers = event.get_state();
        let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
        match key {
            gdk::enums::key::o | gdk::enums::key::O => {
                if ctrl {
                    match util::load_map(window, map) {
                        Ok(action) => (self, Inhibit(false), action),
                        Err(error) => {
                            eprintln!("Error loading map: {}", error);
                            (self, Inhibit(false), Action::None)
                        }
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::q | gdk::enums::key::Q => {
                (self, Inhibit(false), Action::Quit)
            }
            gdk::enums::key::e | gdk::enums::key::E => {
                if let Some(addr) = self.active_hex {
                    let state = Box::new(
                        super::replace_tile::ReplaceTile::with_any(map, addr),
                    );
                    (state, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::u | gdk::enums::key::U => {
                if let Some(addr) = self.active_hex {
                    if let Some(tile) = map.tile_at(addr) {
                        let candidates: Vec<usize> = map
                            .tiles()
                            .iter()
                            .enumerate()
                            .filter(|(_ix, t)| tile.can_upgrade_to(t))
                            .map(|(ix, _t)| ix)
                            .collect();
                        let state = Box::new(
                            super::replace_tile::ReplaceTile::with_candidates(
                                addr, candidates,
                            ),
                        );
                        (state, Inhibit(false), Action::Redraw)
                    } else {
                        (self, Inhibit(false), Action::None)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::t | gdk::enums::key::T => {
                if let Some(addr) = self.active_hex {
                    if let Some(state) =
                        super::edit_tokens::EditTokens::try_new(map, addr)
                    {
                        (Box::new(state), Inhibit(false), Action::Redraw)
                    } else {
                        (self, Inhibit(false), Action::None)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::r | gdk::enums::key::R => {
                // Allow the user to select tokens, and highlight all matching
                // tokens on the map.
                if let Some(addr) = self.active_hex {
                    if let Some(state) =
                        super::select_token::SelectToken::try_new(map, addr)
                    {
                        (Box::new(state), Inhibit(false), Action::Redraw)
                    } else {
                        (self, Inhibit(false), Action::None)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::s | gdk::enums::key::S => {
                // NOTE: FileChooserNative requires gtk 3.20.
                // let dialog = gtk::FileChooserNative::new(
                //     Some("Save Screenshot"),
                //     Some(window),
                //     gtk::FileChooserAction::Save,
                //     None,
                //     None,
                // );
                if ctrl {
                    match util::save_map(window, map) {
                        Ok(action) => (self, Inhibit(false), action),
                        Err(error) => {
                            eprintln!("Error saving map: {}", error);
                            (self, Inhibit(false), Action::None)
                        }
                    }
                } else {
                    match util::save_screenshot(&self, window, area, hex, map)
                    {
                        Ok(action) => (self, Inhibit(false), action),
                        Err(error) => {
                            eprintln!("Error saving screenshot: {}", error);
                            (self, Inhibit(false), Action::None)
                        }
                    }
                }
            }
            gdk::enums::key::Left => {
                // TODO: these are boilerplate, define a common function?
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_col(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Right => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_col(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Up => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_row(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Down => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_row(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::less | gdk::enums::key::comma => {
                if let Some(addr) = self.active_hex {
                    map.get_hex_mut(addr).map(|hs| hs.rotate_anti_cw());
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::greater | gdk::enums::key::period => {
                if let Some(addr) = self.active_hex {
                    map.get_hex_mut(addr).map(|hs| hs.rotate_cw());
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::_0
            | gdk::enums::key::KP_0
            | gdk::enums::key::BackSpace
            | gdk::enums::key::Delete => {
                if let Some(addr) = self.active_hex {
                    // TODO: allow this action to be undone?
                    map.remove_tile(addr);
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            _ => (self, Inhibit(false), Action::None),
        }
    }

    fn button_press(
        self: Box<Self>,
        _hex: &Hex,
        _map: &mut Map,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}
