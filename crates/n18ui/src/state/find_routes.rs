use super::{Action, State};

use cairo::Context;
use gtk::{GtkWindowExt, Inhibit};

use log::info;

use crate::dialog::{select_company, select_trains};
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
}

impl FindRoutes {
    pub(super) fn try_new(
        content: &Content,
        active_hex: Option<&HexAddress>,
        window: &gtk::ApplicationWindow,
    ) -> Option<Self> {
        // Select a company from those that have placed tokens on the map.
        let companies = valid_companies(content);
        let ix = select_company(window, &companies)?;

        // Identify the company name and token.
        let abbrev = &companies[ix].abbrev;
        let token = content.map.tokens().get_token(abbrev)?;

        // Select the company trains and bonuses.
        // Note: use &* because Box<T> implements Deref<Target = T>.
        // So &*content.game converts from Box<dyn Game> to &dyn Game.
        let (trains, bonuses) =
            select_trains(window, &*content.game, abbrev)?;

        // Find the best routes.
        let best_routes = best_routes_for(content, token, trains, bonuses);

        // Update the window title.
        let original_window_title =
            window.get_title().map(|s| s.as_str().to_string());
        update_title(window, abbrev, &best_routes);

        Some(FindRoutes {
            active_hex: active_hex.copied(),
            abbrev: abbrev.clone(),
            best_routes,
            original_window_title,
        })
    }
}

/// Returns the companies that have placed tokens on the map.
fn valid_companies(content: &Content) -> Vec<&Company> {
    let companies = content.game.companies();
    let placed = content.map.unique_placed_tokens();
    let placed_names: Vec<&str> = placed
        .iter()
        .filter_map(|token| content.map.tokens().get_name(token))
        .collect();
    let companies: Vec<&Company> = companies
        .iter()
        .filter(|c| placed_names.iter().any(|name| c.abbrev == *name))
        .collect();
    companies
}

/// Updates the window title so that it shows the company name and revenue.
fn update_title(
    window: &gtk::ApplicationWindow,
    abbrev: &str,
    routes: &Option<(Token, Routes)>,
) {
    let title = routes
        .as_ref()
        .map(|(_token, pairing)| {
            format!("{}: ${}", abbrev, pairing.net_revenue)
        })
        .unwrap_or(format!("{}: No routes", abbrev));
    window.set_title(&title);
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
    let bonuses = content.game.get_bonuses(&bonus_options);
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
        n18brush::draw_barriers(hex, ctx, map);

        // Highlight all matching token spaces on the map, before drawing each
        // route. Note that the routes may pass through these token spaces
        // without stopping at them.
        hex_iter.restart();
        for hex_state in &mut hex_iter {
            if let Some((tile, tokens)) = hex_state.tile_state {
                for (token_space, token) in tokens {
                    if let Some(name) = map.tokens().get_name(&token) {
                        if name == self.abbrev {
                            // Highlight this matching token space.
                            let (r, g, b, a) = (0.9, 0.1, 0.1, 0.25);
                            tile.define_token_space(&token_space, hex, ctx);
                            ctx.set_source_rgb(r, g, b);
                            ctx.set_line_width(hex.max_d * 0.025);
                            ctx.stroke_preserve();
                            ctx.set_source_rgba(r, g, b, a);
                            ctx.fill_preserve();
                        }
                    }
                }
            }
        }

        // Draw each route.
        if let Some((_token, routes)) = &self.best_routes {
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

    fn key_press(
        self: Box<Self>,
        _content: &mut Content,
        window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
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
                (state, Inhibit(false), Action::Redraw)
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
