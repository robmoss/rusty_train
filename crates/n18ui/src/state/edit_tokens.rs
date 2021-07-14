//! Adds and removes tokens from the current tile.
use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;

use crate::Content;
use n18hex::HexColour;
use n18map::{HexAddress, Map, TokensTable};
use n18tile::TokenSpace;
use n18token::Token;

/// Placing or removing tokens from a tile.
pub struct EditTokens {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    original_tokens: TokensTable,
}

impl EditTokens {
    pub(super) fn try_new(map: &Map, addr: HexAddress) -> Option<Self> {
        let hex_state = map.get_hex(addr)?;
        let tile = map.tile_at(addr)?;
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
            token_spaces,
            selected: 0,
            original_tokens,
        })
    }
}

impl State for EditTokens {
    fn draw(&self, content: &Content, ctx: &Context) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_map(hex, ctx, &mut hex_iter);

        // Highlight the active token space.
        let token_space = &self.token_spaces[self.selected];
        n18brush::highlight_token_space(
            hex,
            ctx,
            map,
            self.active_hex,
            token_space,
            (204, 51, 51).into(),
        );

        // Draw the active hex with a grey border.
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
        &mut self,
        content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Option<Box<dyn State>>, Inhibit, Action) {
        let map = &mut content.map;
        let key = event.get_keyval();
        match key {
            gdk::keys::constants::Escape => {
                // NOTE: revert any edits before exiting this mode.
                let restore = self
                    .original_tokens
                    .iter()
                    .map(|(ts, tok)| (*ts, *tok))
                    .collect();
                if let Some(hs) = map.get_hex_mut(self.active_hex) {
                    hs.set_tokens(restore)
                }
                (
                    Some(Box::new(super::default::Default::at_hex(Some(
                        self.active_hex,
                    )))),
                    Inhibit(false),
                    Action::Redraw,
                )
            }
            gdk::keys::constants::Return => (
                // NOTE: no changes to apply, just exit this mode.
                Some(Box::new(super::default::Default::at_hex(Some(
                    self.active_hex,
                )))),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::keys::constants::Left => {
                if self.selected == 0 {
                    self.selected = self.token_spaces.len() - 1;
                } else {
                    self.selected -= 1
                }
                (None, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Right => {
                self.selected += 1;
                if self.selected >= self.token_spaces.len() {
                    self.selected = 0
                }
                (None, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Up => {
                let token_space = &self.token_spaces[self.selected];
                // NOTE: we cannot borrow map.tokens() to get the next token,
                // so we have to take a reference to the game's tokens.
                let game = content.games.active();
                if let Some(hs) = map.get_hex_mut(self.active_hex) {
                    let next: &Token = hs
                        .get_token_at(token_space)
                        .and_then(|t| game.next_token(t))
                        .unwrap_or_else(|| game.first_token());
                    hs.set_token_at(token_space, *next);
                }
                (None, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Down => {
                let token_space = &self.token_spaces[self.selected];
                // NOTE: we cannot borrow map.tokens() to get the next token,
                // so we have to take a reference to the game's tokens.
                let game = content.games.active();
                if let Some(hs) = map.get_hex_mut(self.active_hex) {
                    let next: &Token = hs
                        .get_token_at(token_space)
                        .and_then(|t| game.prev_token(t))
                        .unwrap_or_else(|| game.last_token());
                    hs.set_token_at(token_space, *next);
                };
                (None, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::_0
            | gdk::keys::constants::KP_0
            | gdk::keys::constants::BackSpace
            | gdk::keys::constants::Delete => {
                let token_space = &self.token_spaces[self.selected];
                if let Some(hs) = map.get_hex_mut(self.active_hex) {
                    hs.remove_token_at(token_space)
                }
                (None, Inhibit(false), Action::Redraw)
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
