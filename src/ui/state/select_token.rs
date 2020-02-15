use super::{Action, State};

use cairo::Context;
use gtk::GtkWindowExt;
use gtk::Inhibit;
use std::collections::HashMap;

use crate::draw::Draw;
use crate::hex::Hex;
use crate::map::{HexAddress, Map, Token};
use crate::tile::TokenSpace;

/// Selecting a company's tokens for route-finding.
pub struct SelectToken {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    matches: HashMap<HexAddress, Vec<TokenSpace>>,
    max_visits: Option<usize>,
    skip_cities: bool,
    skip_dits: bool,
    double_revenue: bool,
    best_path: Option<crate::route::Path>,
    original_window_title: Option<String>,
}

fn token_matches(
    map: &Map,
    token_opt: Option<&Token>,
) -> HashMap<HexAddress, Vec<TokenSpace>> {
    let pairs = token_opt
        .map(|token| {
            map.find_placed_tokens(token)
                .iter()
                .map(|(addr, _state, _tile, token_space)| {
                    (**addr, **token_space)
                })
                .collect()
        })
        .unwrap_or(vec![]);
    let mut matches = HashMap::new();
    for (addr, token_space) in pairs {
        let spaces = matches.entry(addr).or_insert(vec![]);
        spaces.push(token_space)
    }
    matches
}

impl SelectToken {
    pub(super) fn try_new(
        map: &Map,
        addr: HexAddress,
        window: &gtk::ApplicationWindow,
    ) -> Option<Self> {
        let hex_state = if let Some(hex_state) = map.get_hex(addr) {
            hex_state
        } else {
            return None;
        };
        let tile = if let Some(tile) = map.tile_at(addr) {
            tile
        } else {
            return None;
        };
        let token_spaces = tile.token_spaces();
        if token_spaces.is_empty() {
            return None;
        }
        let selected = 0;
        let space = token_spaces[selected];
        let city_ix = space.city_ix();
        let token_opt = hex_state.get_token_at(&space);
        let matches = token_matches(map, token_opt);
        let window_title = window.get_title().map(|s| s.as_str().to_string());
        let mut state = SelectToken {
            active_hex: addr,
            token_spaces: token_spaces,
            selected: selected,
            matches: matches,
            // NOTE: set the default search parameters.
            max_visits: Some(2),
            skip_cities: false,
            skip_dits: true,
            double_revenue: false,
            // NOTE: need to calculate the best path from this token.
            best_path: None,
            original_window_title: window_title,
        };
        state.best_path = state.best_path(map, addr, city_ix, &token_opt);
        state.update_title(window);
        Some(state)
    }

    /// Returns a description of the train route criteria.
    fn describe_query(&self) -> String {
        let visits = self
            .max_visits
            .map(|n| n.to_string())
            .unwrap_or("D".to_string());
        let suffix = if self.skip_cities { "E" } else { "" };
        if self.double_revenue {
            format!("{}+{}{}", visits, visits, suffix)
        } else {
            format!("{}{}", visits, suffix)
        }
    }

    /// Updates the window title so that it shows the train route criteria and
    /// the revenue earned from the best path (if any).
    fn update_title(&self, window: &gtk::ApplicationWindow) {
        let descr = self.describe_query();
        let revenue = self
            .best_path
            .as_ref()
            .map(|path| {
                if self.double_revenue {
                    2 * path.revenue
                } else {
                    path.revenue
                }
            })
            .unwrap_or(0);
        let title = format!("{} train: ${}", descr, revenue);
        window.set_title(&title);
    }

    /// Searches for the best path that matches the current criteria, updates
    /// the window title, and returns the UI action that should be taken as a
    /// result of this search.
    fn update_search(
        &mut self,
        map: &Map,
        window: &gtk::ApplicationWindow,
    ) -> Action {
        let action = if let Some(hex_state) = map.get_hex(self.active_hex) {
            // Update the matching tokens across the map.
            let space = self.token_spaces[self.selected];
            let token_opt = hex_state.get_token_at(&space);
            self.matches = token_matches(map, token_opt);
            // NOTE: calculate the best path from this token.
            self.best_path = self.best_path(
                map,
                self.active_hex,
                space.city_ix(),
                &token_opt,
            );
            Action::Redraw
        } else {
            Action::None
        };
        self.update_title(window);
        action
    }

    /// Finds a path from the currently-selected token that yields the maximum
    /// revenue.
    fn best_path(
        &self,
        map: &Map,
        addr: HexAddress,
        city_ix: usize,
        token_opt: &Option<&Token>,
    ) -> Option<crate::route::Path> {
        let token = if let Some(t) = token_opt {
            t
        } else {
            return None;
        };

        let query = crate::route::search::Query {
            addr: addr,
            from: crate::connection::Connection::City { ix: city_ix },
            token: **token,
            max_visits: self.max_visits, //Some(3),
            skip_cities: self.skip_cities,
            skip_dits: self.skip_dits,
            conflict_rule:
                crate::route::conflict::ConflictRule::TrackOrCityHex,
        };
        let now = std::time::Instant::now();
        let paths = crate::route::search::paths_from(map, &query);
        println!(
            "Enumerated {} routes in {}",
            paths.len(),
            now.elapsed().as_secs_f64()
        );
        paths
            .iter()
            .map(|path| path.revenue)
            .max()
            .and_then(|max_revenue| {
                println!("Maximum revenue is: {}", max_revenue);
                let num_max = paths
                    .iter()
                    .filter(|path| path.revenue == max_revenue)
                    .count();
                println!("{} paths return maximum revenue", num_max);
                paths.into_iter().find(|path| path.revenue == max_revenue)
            })
    }
}

impl State for SelectToken {
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

        // NOTE: draw the best path from the current token.
        if let Some(best_path) = &self.best_path {
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

        // Highlight all matching token spaces on the map.
        hex_iter.restart();
        for (addr, tile_opt) in &mut hex_iter {
            if let Some(spaces) = self.matches.get(&addr) {
                // Highlight and/or fill token spaces
                if let Some((tile, _tokens)) = tile_opt {
                    for token_space in spaces {
                        let (r, g, b, a) = (0.9, 0.1, 0.1, 0.25);
                        tile.define_token_space(token_space, hex, ctx);
                        ctx.set_source_rgb(r, g, b);
                        ctx.set_line_width(hex.max_d * 0.025);
                        ctx.stroke_preserve();
                        if self.active_hex != addr {
                            ctx.set_source_rgba(r, g, b, a);
                            ctx.fill_preserve();
                        }
                    }
                }
            }

            if self.active_hex == addr {
                // Draw the active hex with a grey border.
                ctx.set_source_rgb(0.3, 0.3, 0.3);
                ctx.set_line_width(hex.max_d * 0.02);
                hex.define_boundary(ctx);
                ctx.stroke();

                // Highlight the active token space.
                // NOTE: this still needs to be done, as the active token
                // space may be empty and thus no spaces will be highlighted
                // by the code above.
                if let Some((tile, _tokens)) = tile_opt {
                    let token_space = &self.token_spaces[self.selected];
                    tile.define_token_space(token_space, hex, ctx);
                    ctx.set_source_rgb(0.8, 0.2, 0.2);
                    ctx.set_line_width(hex.max_d * 0.025);
                    ctx.stroke_preserve();
                }
            } else {
                // Cover all other tiles with a partially-transparent layer.
                ctx.set_source_rgba(1.0, 1.0, 1.0, 0.25);
                hex.define_boundary(ctx);
                ctx.fill();
            }
        }
    }

    fn key_press(
        mut self: Box<Self>,
        _hex: &Hex,
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
            gdk::enums::key::q | gdk::enums::key::Q => {
                (self, Inhibit(false), Action::Quit)
            }
            gdk::enums::key::Escape | gdk::enums::key::Return => {
                // Exit this mode.
                // Once the token is selected, switch to EnterTrains state;
                // Once the trains have been entered, calculate the optimal
                // routes and revenue.
                if let Some(title) = &self.original_window_title {
                    window.set_title(&title);
                } else {
                    window.set_title("");
                }
                let state = Box::new(super::default::Default::at_hex(Some(
                    self.active_hex,
                )));
                (state, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::Left => {
                if self.selected == 0 {
                    self.selected = self.token_spaces.len() - 1;
                } else {
                    self.selected -= 1
                }
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::D | gdk::enums::key::d => {
                self.skip_dits = !self.skip_dits;
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::E | gdk::enums::key::e => {
                self.skip_cities = !self.skip_cities;
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::plus => {
                self.double_revenue = !self.double_revenue;
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_2 => {
                self.max_visits = Some(2);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_3 => {
                self.max_visits = Some(3);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_4 => {
                self.max_visits = Some(4);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_5 => {
                self.max_visits = Some(5);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_6 => {
                self.max_visits = Some(6);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_7 => {
                self.max_visits = Some(7);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_8 => {
                self.max_visits = Some(8);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_9 => {
                self.max_visits = Some(9);
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::_0 => {
                self.max_visits = None;
                let action = self.update_search(map, window);
                (self, Inhibit(false), action)
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
