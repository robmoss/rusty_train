use super::{Action, State};

use cairo::Context;
use gtk::GtkWindowExt;
use gtk::Inhibit;
use std::collections::HashMap;

use log::info;

use crate::draw::Draw;
use crate::hex::HexColour;
use crate::map::{HexAddress, Map, Token};
use crate::route::search::{paths_for_token, Criteria, PathLimit};
use crate::route::train::{Pairing, Trains};
use crate::tile::TokenSpace;
use crate::ui::util;
use crate::ui::Content;

/// Selecting a company's tokens for route-finding.
pub struct SelectToken {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    matches: HashMap<HexAddress, Vec<TokenSpace>>,
    token_trains: HashMap<Token, Trains>,
    path_limit: Option<PathLimit>,
    best_routes: Option<(Token, Pairing)>,
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
                .map(|(addr, token_space)| (**addr, **token_space))
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
        content: &Content,
        addr: HexAddress,
        window: &gtk::ApplicationWindow,
    ) -> Option<Self> {
        let map = &content.map;
        let tile = if let Some(tile) = map.tile_at(addr) {
            tile
        } else {
            return None;
        };
        if tile.colour == HexColour::Red {
            return None;
        }
        let token_spaces = tile.token_spaces();
        if token_spaces.is_empty() {
            return None;
        }
        let selected = 0;
        let window_title = window.get_title().map(|s| s.as_str().to_string());
        let mut state = SelectToken {
            active_hex: addr,
            token_spaces: token_spaces,
            selected: selected,
            matches: HashMap::new(),
            token_trains: HashMap::new(),
            path_limit: None,
            best_routes: None,
            original_window_title: window_title,
        };
        // Prompt the user to select trains for the active token.
        state.update_search(content, window);
        Some(state)
    }

    /// Returns a description of the train route criteria.
    fn describe_query(&self) -> String {
        let visits = self
            .path_limit
            .map(|n| match n {
                PathLimit::Cities { count } => count.to_string(),
                PathLimit::CitiesAndTowns { count } => count.to_string(),
                PathLimit::Hexes { count } => format!("H{}", count),
            })
            .unwrap_or("D".to_string());
        visits
    }

    /// Updates the window title so that it shows the train route criteria and
    /// the revenue earned from the best path (if any).
    fn update_title(&self, window: &gtk::ApplicationWindow) {
        let descr = self.describe_query();
        let revenue = self
            .best_routes
            .as_ref()
            .map(|(_token, pairing)| pairing.net_revenue)
            .unwrap_or(0);
        let title = format!("{} train: ${}", descr, revenue);
        window.set_title(&title);
    }

    /// Searches for the best path that matches the current criteria, updates
    /// the window title, and returns the UI action that should be taken as a
    /// result of this search.
    fn update_search(
        &mut self,
        content: &Content,
        window: &gtk::ApplicationWindow,
    ) -> Action {
        let map = &content.map;
        let action = if let Some(hex_state) = map.get_hex(self.active_hex) {
            // Update the matching tokens across the map.
            let space = self.token_spaces[self.selected];
            let token_opt = hex_state.get_token_at(&space);
            self.matches = token_matches(map, token_opt);
            if token_opt == None {
                self.update_title(window);
                self.best_routes = None;
                return Action::Redraw;
            }
            let token = token_opt.unwrap();
            if self.token_trains.contains_key(&token) {
                self.best_routes = self.best_routes_for(map, token);
                self.update_title(window);
                return Action::Redraw;
            }
            let tok_name = token.text();
            let trains_opt =
                crate::ui::dialog::select(window, &content.game, tok_name);
            if let Some((trains, _bonuses)) = trains_opt {
                self.token_trains.insert(*token, trains);
                self.best_routes = self.best_routes_for(map, token);
                Action::Redraw
            } else {
                self.best_routes = None;
                Action::Redraw
            }
        } else {
            Action::None
        };
        self.update_title(window);
        action
    }

    /// Finds a path from the currently-selected token that yields the maximum
    /// revenue.
    fn best_routes_for(
        &mut self,
        map: &Map,
        token: &Token,
    ) -> Option<(Token, Pairing)> {
        let trains = match self.token_trains.get(token) {
            Some(trains) => trains,
            None => return None,
        };

        let start = std::time::Instant::now();
        info!("");
        info!("Searching for the best routes ...");

        let path_limit = trains.path_limit();
        self.path_limit = path_limit;
        let criteria = Criteria {
            token: *token,
            path_limit: path_limit,
            conflict_rule:
                crate::route::conflict::ConflictRule::TrackOrCityHex,
            route_conflict_rule:
                crate::route::conflict::ConflictRule::TrackOnly,
        };

        let now = std::time::Instant::now();
        let paths = paths_for_token(map, &criteria);
        info!(
            "Enumerated {} routes in {}",
            paths.len(),
            now.elapsed().as_secs_f64()
        );
        let now = std::time::Instant::now();
        let best_routes = trains.select_routes(paths);
        info!(
            "Calculated (train, path) revenues in {}",
            now.elapsed().as_secs_f64()
        );
        if let Some(pairing) = &best_routes {
            info!(
                "BEST NET REVENUE FOR {:?} IS ${}",
                token, pairing.net_revenue
            );
            for pair in &pairing.pairs {
                info!(
                    "{}: ${} for {} to {}",
                    pair.train.describe(),
                    pair.revenue,
                    pair.path.visits.first().unwrap().addr,
                    pair.path.visits.last().unwrap().addr
                );
            }
        }

        info!(
            "Searching for the best routes took {}",
            start.elapsed().as_secs_f64()
        );

        best_routes.map(|pairing| (*token, pairing))
    }
}

impl State for SelectToken {
    fn draw(
        &self,
        content: &Content,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        util::draw_hex_backgrounds(hex, ctx, &mut hex_iter);

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
                util::draw_empty_hex(hex, ctx);
            }
        }

        util::outline_empty_hexes(hex, ctx, &mut hex_iter);

        // Draw each of the best routes.
        if let Some((_token, pairing)) = &self.best_routes {
            for (ix, pair) in pairing.pairs.iter().enumerate() {
                // NOTE: cycle through colours for each path.
                if ix % 3 == 0 {
                    ctx.set_source_rgb(0.7, 0.1, 0.1);
                } else if ix % 3 == 1 {
                    ctx.set_source_rgb(0.1, 0.7, 0.1);
                } else {
                    ctx.set_source_rgb(0.1, 0.1, 0.7);
                }
                ctx.set_line_width(hex.max_d * 0.025);
                let source = ctx.get_source();
                let line_width = ctx.get_line_width();

                // Draw track segments first.
                for step in &pair.path.steps {
                    let m = map.prepare_to_draw(step.addr, hex, ctx);
                    let tile =
                        map.tile_at(step.addr).expect("Invalid step hex");

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

                // Then draw visited cities and dits.
                println!("Running train {:?}", pair.train);
                for visit in &pair.path.visits {
                    let m = map.prepare_to_draw(visit.addr, hex, ctx);
                    let tile =
                        map.tile_at(visit.addr).expect("Invalid step hex");
                    use crate::route::StopLocation;
                    match visit.visits {
                        StopLocation::City { ix } => {
                            let city = tile.cities()[ix];
                            city.draw_fg(hex, ctx);
                            if visit.revenue > 0 {
                                ctx.set_source(&source);
                            } else {
                                // NOTE: the train did not stop here.
                                ctx.set_source(&source);
                                ctx.set_source_rgb(0.0, 0.0, 0.0);
                            }
                            ctx.set_line_width(line_width);
                            city.define_boundary(hex, ctx);
                            ctx.stroke();
                            if let Some(hex_state) = map.get_hex(visit.addr) {
                                let tokens_table = hex_state.get_tokens();
                                for (token_space, map_token) in
                                    tokens_table.iter()
                                {
                                    tile.define_token_space(
                                        &token_space,
                                        &hex,
                                        ctx,
                                    );
                                    map_token.draw_token(&hex, ctx);
                                }
                            }
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
                            use crate::track::DitShape::*;
                            match dit_shape {
                                Bar => {
                                    ctx.set_line_width(hex.max_d * 0.08);
                                    track.draw_dit_ends(0.10, hex, ctx);
                                }
                                Circle => {
                                    track.define_circle_dit(hex, ctx);
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
            }
        }

        util::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &Some(self.active_hex),
            0.3,
            0.3,
            0.3,
        );
    }

    fn key_press(
        mut self: Box<Self>,
        content: &mut Content,
        window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
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
                let action = self.update_search(content, window);
                (self, Inhibit(false), action)
            }
            gdk::enums::key::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                let action = self.update_search(content, window);
                (self, Inhibit(false), action)
            }
            _ => (self, Inhibit(false), Action::None),
        }
    }

    fn button_press(
        self: Box<Self>,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}
