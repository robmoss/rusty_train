//! Searches for the best routes that a company can operate.
use super::{Action, State};

use cairo::Context;
use gtk::{GtkWindowExt, Inhibit};

use log::info;

use crate::dialog::{select_item, select_trains};
use crate::Content;
use n18game::Company;
use n18map::HexAddress;
use n18route::{paths_for_token, ConflictRule, Criteria, Routes, Trains};
use n18token::Token;

/// Search for the best routes that a company can operate.
pub struct FindRoutes {
    active_hex: Option<HexAddress>,
    abbrev: String,
    best_routes: Option<(Token, Routes)>,
    original_window_title: Option<String>,
    active_route: Option<usize>,
}

impl FindRoutes {
    pub(super) fn try_new(
        content: &Content,
        active_hex: Option<&HexAddress>,
        window: &gtk::ApplicationWindow,
    ) -> Option<Self> {
        // Select a company from those that have placed tokens on the map.
        let companies = valid_companies(content);
        let company_names: Vec<&str> =
            companies.iter().map(|c| c.full_name.as_str()).collect();
        let ix = select_item(window, "Select a company", &company_names)?;

        // Identify the company name and token.
        let abbrev = &companies[ix].abbrev;
        let token = content.map.try_token(abbrev)?;

        // Select the company trains and bonuses.
        // Note: use &* because Box<T> implements Deref<Target = T>.
        // So &*content.game converts from Box<dyn Game> to &dyn Game.
        let (trains, bonuses) =
            select_trains(window, content.games.active(), abbrev)?;

        // Find the best routes.
        let best_routes = best_routes_for(content, &token, trains, bonuses);
        let active_route = None;

        let original_window_title =
            window.get_title().map(|s| s.as_str().to_string());
        let state = FindRoutes {
            active_hex: active_hex.copied(),
            abbrev: abbrev.clone(),
            best_routes,
            original_window_title,
            active_route,
        };

        // Update the window title.
        // update_title(window, abbrev, &best_routes);
        state.set_window_title(window, content);

        Some(state)
    }

    /// Updates the window title so that it shows the company name and either
    /// the net revenue, or the revenue for the currently-selected route.
    fn set_window_title(
        &self,
        window: &gtk::ApplicationWindow,
        content: &Content,
    ) {
        let title = if let Some((_token, routes)) = &self.best_routes {
            if let Some(ix) = self.active_route {
                let route = &routes.train_routes[ix];
                let train = &route.train;
                let train_name =
                    content.games.active().train_name(train).unwrap();
                format!(
                    "{} {}-train: ${}",
                    self.abbrev, train_name, route.revenue
                )
            } else {
                format!("{}: ${}", self.abbrev, routes.net_revenue)
            }
        } else {
            format!("{}: No routes", &self.abbrev)
        };
        window.set_title(&title);
    }
}

/// Returns the companies that have placed tokens on the map.
fn valid_companies(content: &Content) -> Vec<&Company> {
    let companies = content.games.active().companies();
    let placed = content.map.unique_placed_tokens();
    let placed_names: Vec<&str> = placed
        .iter()
        .filter_map(|token| content.map.try_token_name(token))
        .collect();
    let companies: Vec<&Company> = companies
        .iter()
        .filter(|c| placed_names.iter().any(|name| c.abbrev == *name))
        .collect();
    companies
}

/// Finds a path from the currently-selected token that yields the maximum
/// revenue.
fn best_routes_for(
    content: &Content,
    token: &Token,
    trains: Trains,
    bonus_options: Vec<bool>,
) -> Option<(Token, Routes)> {
    // Handle the case where no trains were selected.
    if trains.train_count() == 0 {
        return None;
    }

    let start = std::time::Instant::now();
    info!("");
    info!("Searching for the best routes ...");

    let path_limit = trains.path_limit();
    let criteria = Criteria {
        token: *token,
        path_limit,
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
    let bonuses = content.games.active().get_bonuses(&bonus_options);
    let best_routes = trains.select_routes(paths, bonuses);
    info!(
        "Calculated (train, path) revenues in {}",
        now.elapsed().as_secs_f64()
    );
    info!(
        "Searching for the best routes took {}",
        start.elapsed().as_secs_f64()
    );

    best_routes.map(|pairing| (*token, pairing))
}

impl State for FindRoutes {
    fn draw(&self, content: &Content, ctx: &Context) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_map(hex, ctx, &mut hex_iter);
        // Slightly fade hexes that are not part of any route.
        if let Some((_token, routes)) = &self.best_routes {
            let hexes: std::collections::HashSet<&HexAddress> = routes
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
                let colour = match ix % 3 {
                    0 => (0.7, 0.1, 0.1, 1.0),
                    1 => (0.1, 0.7, 0.1, 1.0),
                    _ => (0.1, 0.1, 0.7, 1.0),
                };
                ctx.set_source_rgba(colour.0, colour.1, colour.2, colour.3);
                let route = routes.routes()[ix];
                n18brush::highlight_route(&hex, &ctx, &map, route);
            } else {
                n18brush::highlight_routes(
                    &hex,
                    &ctx,
                    &map,
                    &routes.routes(),
                    |ix| match ix % 3 {
                        0 => (0.7, 0.1, 0.1, 1.0),
                        1 => (0.1, 0.7, 0.1, 1.0),
                        _ => (0.1, 0.1, 0.7, 1.0),
                    },
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
                map.try_token_name(&token)
                    .map(|name| name == self.abbrev)
                    .unwrap_or(false)
            },
            (230, 25, 25).into(),
            Some((230, 25, 25, 31).into()),
        );
    }

    fn key_press(
        &mut self,
        content: &mut Content,
        window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Option<Box<dyn State>>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
            gdk::keys::constants::Escape | gdk::keys::constants::Return => {
                // Exit this mode.
                if let Some(title) = &self.original_window_title {
                    window.set_title(&title);
                } else {
                    window.set_title("");
                }
                let state = Box::new(super::default::Default::at_hex(
                    self.active_hex,
                ));
                (Some(state), Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Left | gdk::keys::constants::Up => {
                // Draw the previous route, if any, by itself.
                if let Some((_token, routes)) = &self.best_routes {
                    let routes_vec = routes.routes();
                    let num_routes = routes_vec.len();
                    if let Some(curr_ix) = self.active_route {
                        if curr_ix == 0 {
                            self.active_route = None;
                        } else {
                            self.active_route = Some(curr_ix - 1);
                        }
                    } else {
                        self.active_route = Some(num_routes - 1);
                    }
                    self.set_window_title(window, content);
                    (None, Inhibit(false), Action::Redraw)
                } else {
                    (None, Inhibit(false), Action::None)
                }
            }
            gdk::keys::constants::Right | gdk::keys::constants::Down => {
                // Draw the next route, if any, by itself.
                if let Some((_token, routes)) = &self.best_routes {
                    let num_routes = routes.routes().len();
                    if let Some(curr_ix) = self.active_route {
                        if curr_ix == num_routes - 1 {
                            self.active_route = None;
                        } else {
                            self.active_route = Some(curr_ix + 1);
                        }
                    } else {
                        self.active_route = Some(0);
                    }
                    self.set_window_title(window, content);
                    (None, Inhibit(false), Action::Redraw)
                } else {
                    (None, Inhibit(false), Action::None)
                }
            }
            _ => (None, Inhibit(false), Action::None),
        }
    }

    fn button_press(
        &mut self,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Option<Box<dyn State>>, Inhibit, Action) {
        (None, Inhibit(false), Action::None)
    }
}
