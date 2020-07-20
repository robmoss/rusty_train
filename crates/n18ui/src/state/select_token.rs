use super::{Action, State};

use cairo::Context;
use gtk::GtkWindowExt;
use gtk::Inhibit;
use std::collections::HashMap;

use log::info;

use crate::Content;
use n18brush;
use n18hex::HexColour;
use n18map::{HexAddress, Map};
use n18route::{
    paths_for_token, ConflictRule, Criteria, Pairing, PathLimit, Trains,
};
use n18tile::TokenSpace;
use n18token::Token;

/// Selecting a company's tokens for route-finding.
pub struct SelectToken {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    matches: HashMap<HexAddress, Vec<TokenSpace>>,
    token_trains: HashMap<Token, (Trains, Vec<bool>)>,
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

    /// Updates the window title so that it shows the train route criteria and
    /// the revenue earned from the best path (if any).
    fn update_title(
        &self,
        window: &gtk::ApplicationWindow,
        content: &Content,
    ) {
        let title = self
            .best_routes
            .as_ref()
            .map(|(token, pairing)| {
                let tok_name = content.map.tokens().get_name(token).unwrap();
                format!("{}: ${}", tok_name, pairing.net_revenue)
            })
            .unwrap_or("No routes".to_string());
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
                self.best_routes = None;
                self.update_title(window, content);
                return Action::Redraw;
            }
            let token = token_opt.unwrap();
            if self.token_trains.contains_key(&token) {
                self.best_routes = self.best_routes_for(content, token);
                self.update_title(window, content);
                return Action::Redraw;
            }
            let tok_name = map.tokens().get_name(token).unwrap();
            let trains_opt =
                crate::dialog::select_trains(window, &content.game, tok_name);
            if let Some((trains, bonuses)) = trains_opt {
                self.token_trains.insert(*token, (trains, bonuses));
                self.best_routes = self.best_routes_for(content, token);
                self.update_title(window, content);
                Action::Redraw
            } else {
                self.best_routes = None;
                self.update_title(window, content);
                Action::Redraw
            }
        } else {
            Action::None
        };
        self.update_title(window, content);
        action
    }

    /// Finds a path from the currently-selected token that yields the maximum
    /// revenue.
    fn best_routes_for(
        &mut self,
        content: &Content,
        token: &Token,
    ) -> Option<(Token, Pairing)> {
        let (trains, bonus_options) = match self.token_trains.get(token) {
            Some(value) => value,
            None => return None,
        };

        // Handle the case where no trains were selected.
        if trains.train_count() == 0 {
            return None;
        }

        let start = std::time::Instant::now();
        info!("");
        info!("Searching for the best routes ...");

        let path_limit = trains.path_limit();
        self.path_limit = path_limit;
        let criteria = Criteria {
            token: *token,
            path_limit: path_limit,
            conflict_rule: ConflictRule::TrackOrCityHex,
            route_conflict_rule: ConflictRule::TrackOnly,
        };

        let now = std::time::Instant::now();
        let map = &content.map;
        let paths = paths_for_token(map, &criteria);
        info!(
            "Enumerated {} routes in {}",
            paths.len(),
            now.elapsed().as_secs_f64()
        );
        let now = std::time::Instant::now();
        // Determine the route bonuses that may apply.
        let bonuses = content.game.get_bonuses(bonus_options);
        let best_routes = trains.select_routes(paths, bonuses);
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
                    content.game.train_name(&pair.train).unwrap_or("???"),
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
    fn draw(&self, content: &Content, ctx: &Context) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_map(hex, ctx, &mut hex_iter);
        n18brush::draw_barriers(hex, ctx, map);

        // Highlight all matching token spaces on the map, before drawing each
        // route. Note that the routes may pass through these token spaces
        // without stopping at them.
        hex_iter.restart();
        for hex_state in &mut hex_iter {
            if let Some(spaces) = self.matches.get(&hex_state.addr) {
                // Highlight and/or fill token spaces
                if let Some((tile, _tokens)) = hex_state.tile_state {
                    for token_space in spaces {
                        let (r, g, b, a) = (0.9, 0.1, 0.1, 0.25);
                        tile.define_token_space(token_space, hex, ctx);
                        ctx.set_source_rgb(r, g, b);
                        ctx.set_line_width(hex.max_d * 0.025);
                        ctx.stroke_preserve();
                        if self.active_hex != hex_state.addr {
                            ctx.set_source_rgba(r, g, b, a);
                            ctx.fill_preserve();
                        }
                    }
                }
            }
        }

        // Draw each route.
        if let Some((_token, pairing)) = &self.best_routes {
            n18brush::highlight_routes(
                &hex,
                &ctx,
                &map,
                &pairing.pairs,
                |ix| match ix % 3 {
                    0 => (0.7, 0.1, 0.1, 1.0),
                    1 => (0.1, 0.7, 0.1, 1.0),
                    _ => (0.1, 0.1, 0.7, 1.0),
                },
            );
        }

        n18brush::highlight_active_hex(
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
            gdk::keys::constants::Escape | gdk::keys::constants::Return => {
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
            gdk::keys::constants::Left => {
                if self.selected == 0 {
                    self.selected = self.token_spaces.len() - 1;
                } else {
                    self.selected -= 1
                }
                // NOTE: always redraw, the selected token has changed.
                let _action = self.update_search(content, window);
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                // NOTE: always redraw, the selected token has changed.
                let _action = self.update_search(content, window);
                (self, Inhibit(false), Action::Redraw)
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