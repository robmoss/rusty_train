use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;

use crate::hex::Hex;
use crate::map::{HexAddress, Map, Token, TokensTable};
use crate::tile::TokenSpace;

/// Placing or removing tokens from a tile.
pub struct EditTokens {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    original_tokens: TokensTable,
}

impl EditTokens {
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
        let original_tokens = hex_state.get_tokens().clone();
        Some(EditTokens {
            active_hex: addr,
            token_spaces: token_spaces,
            selected: 0,
            original_tokens: original_tokens,
        })
    }
}

impl State for EditTokens {
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
        for (addr, tile_opt) in &mut hex_iter {
            if self.active_hex == addr {
                // Draw the active hex with a grey border.
                ctx.set_source_rgb(0.3, 0.3, 0.3);
                ctx.set_line_width(hex.max_d * 0.02);
                hex.define_boundary(ctx);
                ctx.stroke();

                // Highlight the active token space.
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
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
            gdk::enums::key::Escape => {
                // NOTE: revert any edits before exiting this mode.
                let restore = self.original_tokens.drain().collect();
                map.get_hex_mut(self.active_hex)
                    .map(|hex_state| hex_state.set_tokens(restore));
                (
                    Box::new(super::default::Default::at_hex(Some(
                        self.active_hex,
                    ))),
                    Inhibit(false),
                    Action::Redraw,
                )
            }
            gdk::enums::key::Return => (
                // NOTE: no changes to apply, just exit this mode.
                Box::new(super::default::Default::at_hex(Some(
                    self.active_hex,
                ))),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::enums::key::Left => {
                if self.selected == 0 {
                    self.selected = self.token_spaces.len() - 1;
                } else {
                    self.selected -= 1
                }
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::Up => {
                let token_space = &self.token_spaces[self.selected];
                map.get_hex_mut(self.active_hex).map(|hex_state| {
                    let next = hex_state
                        .get_token_at(token_space)
                        .map(|t| t.next())
                        .unwrap_or(Token::first());
                    hex_state.set_token_at(token_space, next);
                });
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::Down => {
                let token_space = &self.token_spaces[self.selected];
                map.get_hex_mut(self.active_hex).map(|hex_state| {
                    let next = hex_state
                        .get_token_at(token_space)
                        .map(|t| t.prev())
                        .unwrap_or(Token::last());
                    hex_state.set_token_at(token_space, next);
                });
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::_0
            | gdk::enums::key::KP_0
            | gdk::enums::key::BackSpace
            | gdk::enums::key::Delete => {
                let token_space = &self.token_spaces[self.selected];
                map.get_hex_mut(self.active_hex)
                    .map(|hex_state| hex_state.remove_token_at(token_space));
                (self, Inhibit(false), Action::Redraw)
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
