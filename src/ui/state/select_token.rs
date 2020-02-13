use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;
use std::collections::HashMap;

use crate::hex::Hex;
use crate::map::{HexAddress, Map, Token};
use crate::tile::TokenSpace;

/// Selecting a company's tokens for route-finding.
pub struct SelectToken {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    matches: HashMap<HexAddress, Vec<TokenSpace>>,
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
    pub(super) fn try_new(map: &Map, addr: HexAddress) -> Option<Self> {
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
        let token_opt = hex_state.get_token_at(&space);
        let matches = token_matches(map, token_opt);
        // TODO: calculate the paths for the current token here!
        Some(SelectToken {
            active_hex: addr,
            token_spaces: token_spaces,
            selected: selected,
            matches: matches,
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

        // TODO: draw the best path from the current token?
    }

    fn key_press(
        mut self: Box<Self>,
        _hex: &Hex,
        map: &mut Map,
        _window: &gtk::ApplicationWindow,
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
                if let Some(hex_state) = map.get_hex(self.active_hex) {
                    // Update the matching tokens across the map.
                    let space = self.token_spaces[self.selected];
                    let token_opt = hex_state.get_token_at(&space);
                    self.matches = token_matches(map, token_opt);
                    // TODO: calculate the best path from this token?
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                if let Some(hex_state) = map.get_hex(self.active_hex) {
                    // Update the matching tokens across the map.
                    let space = self.token_spaces[self.selected];
                    let token_opt = hex_state.get_token_at(&space);
                    self.matches = token_matches(map, token_opt);
                    // TODO: calculate the best path from this token?
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
