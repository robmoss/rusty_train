//! Searches for the best routes that a company can operate.

use cairo::Context;
use std::sync::mpsc::Receiver;

use n18game::Company;
use n18map::HexAddress;
use n18route::{Routes, Trains};
use n18token::Token;

use crate::{
    Assets, Controller, PingDest, State, UiController, UiResponse, UiState,
};

/// Prompts the user to select a company that has at least one token placed on
/// the map.
pub struct SelectCompany {
    active_hex: HexAddress,
    receiver: Receiver<Option<String>>,
}

impl SelectCompany {
    pub fn new(
        assets: &Assets,
        controller: &mut Controller,
        active_hex: HexAddress,
    ) -> Self {
        let companies = valid_companies(assets);
        let company_names: Vec<&str> =
            companies.iter().map(|c| c.full_name.as_str()).collect();
        let (sender, receiver) = std::sync::mpsc::channel();
        let ping_tx = controller.ping_tx();
        controller.select_string(
            "Select a company",
            &company_names,
            move |name_opt| {
                sender.send(name_opt).unwrap();
                ping_tx.send_ping(PingDest::State).unwrap();
            },
        );
        SelectCompany {
            active_hex,
            receiver,
        }
    }
}

impl UiState for SelectCompany {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
        let mut hex_iter = map.hex_iter(hex, ctx);
        n18brush::draw_map(hex, ctx, &mut hex_iter);
    }

    fn ping(
        &mut self,
        assets: &mut Assets,
        controller: &mut Controller,
    ) -> (UiResponse, Option<State>) {
        let name_opt = self.receiver.recv().unwrap();
        if let Some(chosen_name) = name_opt {
            let companies = valid_companies(assets);
            let abbrev_opt = companies.iter().find_map(|c| {
                if c.full_name == chosen_name {
                    Some(c.abbrev.clone())
                } else {
                    None
                }
            });
            if let Some(abbrev) = abbrev_opt {
                if let Some(token) = assets.map.try_token(&abbrev) {
                    let b = State::FindRoutesTrains(SelectTrains::new(
                        assets,
                        controller,
                        self.active_hex,
                        abbrev,
                        token,
                    ));
                    return (UiResponse::Redraw, Some(b));
                }
            }
        }

        // Return to the default state.
        // let def = super::default::Default::at_hex(self.active_hex);
        // (Action::Redraw, Some(UiState::Default(def)))
        (UiResponse::Redraw, Some(self.active_hex.into()))
    }
}

/// Prompts the user to select the trains owned by the selected company, and
/// any relevant bonuses.
pub struct SelectTrains {
    active_hex: HexAddress,
    receiver: Receiver<Option<(Trains, Vec<bool>)>>,
    abbrev: String,
    token: Token,
}

impl SelectTrains {
    pub fn new(
        assets: &Assets,
        controller: &mut Controller,
        active_hex: HexAddress,
        abbrev: String,
        token: Token,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let ping_tx = controller.ping_tx();
        controller.select_trains(
            assets.games.active(),
            &abbrev,
            move |trains_opt| {
                sender.send(trains_opt).unwrap();
                ping_tx.send_ping(PingDest::State).unwrap();
            },
        );
        SelectTrains {
            active_hex,
            receiver,
            abbrev,
            token,
        }
    }
}

impl UiState for SelectTrains {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
        let mut hex_iter = map.hex_iter(hex, ctx);
        n18brush::draw_map(hex, ctx, &mut hex_iter);
    }

    fn ping(
        &mut self,
        assets: &mut Assets,
        controller: &mut Controller,
    ) -> (UiResponse, Option<State>) {
        let trains_opt = self.receiver.recv().unwrap();
        if let Some((trains, bonuses)) = trains_opt {
            let state = State::FindRoutesSearch(Search::new(
                assets,
                controller,
                self.active_hex,
                self.abbrev.clone(),
                self.token,
                trains,
                bonuses,
            ));
            return (UiResponse::Redraw, Some(state));
        }

        // Return to the default state.
        (UiResponse::Redraw, Some(self.active_hex.into()))
    }
}

/// Searches for the optimal routes for the selected company.
pub struct Search {
    active_hex: HexAddress,
    abbrev: String,
    receiver: std::sync::mpsc::Receiver<Option<(Token, Routes)>>,
}

impl Search {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assets: &Assets,
        controller: &mut dyn UiController,
        active_hex: HexAddress,
        abbrev: String,
        token: Token,
        trains: Trains,
        bonuses: Vec<bool>,
    ) -> Self {
        let new_title = format!("{}: searching ...", abbrev);
        controller.set_window_title(&new_title);

        // Search for the best routes in a separate thread, to avoid making
        // the user interface unresponsive, and ping this state when the
        // route-finding has finished.
        let ping_tx = controller.ping_tx();

        // NOTE: we also need to clone the map, because the thread cannot take
        // a reference unless we somehow define an appropriate lifetime.
        let map = assets.map.clone();
        // Create a channel from which to retrieve the best routes.
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);

        // Spawn the new thread.
        let active_game = assets.games.active();
        let search_fn =
            active_game.best_routes_closure(map, token, trains, bonuses);
        std::thread::spawn(move || {
            // Find the best routes.
            let best_routes = search_fn().map(|routes| (token, routes));
            // Send the best routes back to this state.
            sender.send(best_routes).unwrap();
            // Ping this state so that it can retrieve the best routes.
            ping_tx.send_ping(PingDest::State).unwrap();
        });

        Search {
            active_hex,
            abbrev,
            receiver,
        }
    }
}

impl UiState for Search {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
        let mut hex_iter = map.hex_iter(hex, ctx);
        n18brush::draw_map(hex, ctx, &mut hex_iter);

        // NOTE: fade out the entire map and return.
        let fill = n18hex::Colour::WHITE.with_alpha(159);
        fill.apply_colour(ctx);
        ctx.paint().unwrap();
    }

    fn ping(
        &mut self,
        assets: &mut Assets,
        controller: &mut Controller,
    ) -> (UiResponse, Option<State>) {
        let best_routes = self.receiver.recv().unwrap();
        let state = State::FindRoutesFound(Found::new(
            assets,
            controller,
            self.active_hex,
            self.abbrev.clone(),
            best_routes,
        ));
        (UiResponse::Redraw, Some(state))
    }
}

/// Displays the optimal routes for the selected company, once they have been
/// found.
pub struct Found {
    active_hex: HexAddress,
    abbrev: String,
    best_routes: Option<(Token, Routes)>,
    active_route: Option<usize>,
}

impl Found {
    pub fn new(
        assets: &Assets,
        controller: &mut dyn UiController,
        active_hex: HexAddress,
        abbrev: String,
        best_routes: Option<(Token, Routes)>,
    ) -> Self {
        let state = Found {
            active_hex,
            abbrev,
            best_routes,
            active_route: None,
        };
        controller.set_window_title(&state.window_title(assets));
        state
    }

    pub fn active_hex(&self) -> HexAddress {
        self.active_hex
    }

    /// Returns the window title, which shows the company name and either the
    /// net revenue, or the revenue for the currently-selected route.
    pub fn window_title(&self, assets: &Assets) -> String {
        if let Some((_token, routes)) = &self.best_routes {
            if let Some(ix) = self.active_route {
                let route = &routes.train_routes[ix];
                let train = &route.train;
                let train_name =
                    assets.games.active().train_name(train).unwrap();
                format!(
                    "{} {}-train: ${}",
                    self.abbrev, train_name, route.revenue
                )
            } else {
                format!("{}: ${}", self.abbrev, routes.net_revenue)
            }
        } else {
            format!("{}: No routes", &self.abbrev)
        }
    }

    pub fn highlight_previous_route(&mut self) -> bool {
        if let Some((_token, routes)) = &self.best_routes {
            let routes_vec = routes.routes();
            let num_routes = routes_vec.len();
            if num_routes < 2 {
                return false;
            }
            if let Some(curr_ix) = self.active_route {
                if curr_ix == 0 {
                    self.active_route = None;
                } else {
                    self.active_route = Some(curr_ix - 1);
                }
            } else {
                self.active_route = Some(num_routes - 1);
            }
            true
        } else {
            false
        }
    }

    pub fn highlight_next_route(&mut self) -> bool {
        if let Some((_token, routes)) = &self.best_routes {
            let num_routes = routes.routes().len();
            if num_routes < 2 {
                return false;
            }
            if let Some(curr_ix) = self.active_route {
                if curr_ix == num_routes - 1 {
                    self.active_route = None;
                } else {
                    self.active_route = Some(curr_ix + 1);
                }
            } else {
                self.active_route = Some(0);
            }
            true
        } else {
            false
        }
    }
}

impl UiState for Found {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_map(hex, ctx, &mut hex_iter);

        // Slightly fade hexes that are not part of any route.
        if let Some((_token, routes)) = &self.best_routes {
            let hexes: std::collections::BTreeSet<&HexAddress> = routes
                .routes()
                .iter()
                .flat_map(|route| route.steps.iter().map(|step| &step.addr))
                .collect();
            n18brush::highlight_hexes(
                hex,
                ctx,
                &mut hex_iter,
                |addr| hexes.contains(addr),
                None,
            );
        }

        // Draw each route.
        // Note that this also redraws the token spaces at each visit.
        if let Some((_token, routes)) = &self.best_routes {
            if let Some(ix) = self.active_route {
                // Draw only a single route, in the same colour as when
                // drawing all routes.
                let colour = hex.theme.nth_highlight_colour(ix);
                colour.apply_colour(ctx);
                let route = routes.routes()[ix];
                n18brush::highlight_route(hex, ctx, map, route);
            } else {
                n18brush::highlight_routes(
                    hex,
                    ctx,
                    map,
                    &routes.routes(),
                    |ix| hex.theme.nth_highlight_colour(ix),
                );
            }
        }

        // Highlight all matching token spaces on the map.
        // We do this after highlighting each route, because the routes will
        // redraw all of the token spaces that they pass through.
        // Note that the routes may pass through these token spaces
        // without stopping at them.
        n18brush::highlight_tokens(
            hex,
            ctx,
            &mut hex_iter,
            |_addr, _tile, _token_space, token| {
                map.try_token_name(token)
                    .map(|name| name == self.abbrev)
                    .unwrap_or(false)
            },
            (230, 25, 25).into(),
            Some((230, 25, 25, 31).into()),
        );
    }
}

/// Returns the companies that have placed tokens on the map.
fn valid_companies(assets: &Assets) -> Vec<&Company> {
    let companies = assets.games.active().companies();
    let placed = assets.map.unique_placed_tokens();
    let placed_names: Vec<&str> = placed
        .iter()
        .filter_map(|token| assets.map.try_token_name(token))
        .collect();
    let companies: Vec<&Company> = companies
        .iter()
        .filter(|c| placed_names.iter().any(|name| c.abbrev == *name))
        .collect();
    companies
}
