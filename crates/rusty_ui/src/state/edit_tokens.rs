use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;

use crate::Content;
use rusty_brush;
use rusty_hex::HexColour;
use rusty_map::{HexAddress, Map, TokensTable};
use rusty_tile::TokenSpace;
use rusty_token::Token;

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
        if tile.colour == HexColour::Red {
            return None;
        }
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
        content: &Content,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        rusty_brush::draw_map(hex, ctx, &mut hex_iter);
        rusty_brush::draw_barriers(hex, ctx, map);

        // Highlight the active token space.
        if let Some(tile) = map.tile_at(self.active_hex) {
            let m = map.prepare_to_draw(self.active_hex, hex, ctx);
            let token_space = &self.token_spaces[self.selected];
            tile.define_token_space(token_space, hex, ctx);
            ctx.set_source_rgb(0.8, 0.2, 0.2);
            ctx.set_line_width(hex.max_d * 0.025);
            ctx.stroke_preserve();
            ctx.set_matrix(m);
        }

        // Draw the active hex with a grey border.
        rusty_brush::highlight_active_hex(
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
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let map = &mut content.map;
        let key = event.get_keyval();
        match key {
            gdk::keys::constants::Escape => {
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
            gdk::keys::constants::Return => (
                // NOTE: no changes to apply, just exit this mode.
                Box::new(super::default::Default::at_hex(Some(
                    self.active_hex,
                ))),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::keys::constants::Left => {
                if self.selected == 0 {
                    self.selected = self.token_spaces.len() - 1;
                } else {
                    self.selected -= 1
                }
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Up => {
                let token_space = &self.token_spaces[self.selected];
                // NOTE: we cannot borrow map.tokens() to get the next token,
                // so we have to take a reference to the game's tokens.
                let tokens = content.game.company_tokens();
                map.get_hex_mut(self.active_hex).map(|hex_state| {
                    let next: Token = hex_state
                        .get_token_at(token_space)
                        .and_then(|t| tokens.next_token(t))
                        .unwrap_or(tokens.first_token());
                    hex_state.set_token_at(token_space, next);
                });
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Down => {
                let token_space = &self.token_spaces[self.selected];
                // NOTE: we cannot borrow map.tokens() to get the next token,
                // so we have to take a reference to the game's tokens.
                let tokens = content.game.company_tokens();
                map.get_hex_mut(self.active_hex).map(|hex_state| {
                    let next: Token = hex_state
                        .get_token_at(token_space)
                        .and_then(|t| tokens.prev_token(t))
                        .unwrap_or(tokens.last_token());
                    hex_state.set_token_at(token_space, next);
                });
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::_0
            | gdk::keys::constants::KP_0
            | gdk::keys::constants::BackSpace
            | gdk::keys::constants::Delete => {
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
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}
