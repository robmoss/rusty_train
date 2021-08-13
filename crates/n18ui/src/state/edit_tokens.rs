//! Adds and removes tokens from the current tile.
use cairo::Context;

use n18hex::HexColour;
use n18map::{HexAddress, Map, TokensTable};
use n18tile::TokenSpace;
use n18token::Token;

use crate::{Assets, UiState};

/// Placing or removing tokens from a tile.
pub struct EditTokens {
    active_hex: HexAddress,
    token_spaces: Vec<TokenSpace>,
    selected: usize,
    original_tokens: TokensTable,
}

impl EditTokens {
    pub fn try_new(map: &Map, addr: HexAddress) -> Option<Self> {
        let hex_state = map.hex_state(addr)?;
        let tile = map.tile_at(addr)?;
        if tile.colour == HexColour::Red {
            return None;
        }
        let token_spaces = tile.token_spaces();
        if token_spaces.is_empty() {
            return None;
        }
        let original_tokens = hex_state.tokens().clone();
        Some(EditTokens {
            active_hex: addr,
            token_spaces,
            selected: 0,
            original_tokens,
        })
    }

    pub fn active_hex(&self) -> HexAddress {
        self.active_hex
    }

    pub fn restore_tokens(&self, map: &mut Map) {
        let restore = self
            .original_tokens
            .iter()
            .map(|(ts, tok)| (*ts, *tok))
            .collect();
        if let Some(hs) = map.hex_state_mut(self.active_hex) {
            hs.set_tokens(restore)
        }
    }

    pub fn clear_token_space(&self, map: &mut Map) {
        let token_space = &self.token_spaces[self.selected];
        if let Some(hs) = map.hex_state_mut(self.active_hex) {
            hs.remove_token_at(token_space)
        }
    }

    pub fn previous_token_space(&mut self) {
        if self.selected == 0 {
            self.selected = self.token_spaces.len() - 1;
        } else {
            self.selected -= 1
        }
    }

    pub fn next_token_space(&mut self) {
        self.selected += 1;
        if self.selected >= self.token_spaces.len() {
            self.selected = 0
        }
    }

    pub fn select_previous_token(&mut self, assets: &mut Assets) {
        let token_space = &self.token_spaces[self.selected];
        // NOTE: we cannot borrow map.tokens() to get the next token,
        // so we have to take a reference to the game's tokens.
        let game = assets.games.active();
        if let Some(hs) = assets.map.hex_state_mut(self.active_hex) {
            let next: &Token = hs
                .token_at(token_space)
                .and_then(|t| game.prev_token(t))
                .unwrap_or_else(|| game.last_token());
            hs.set_token_at(token_space, *next);
        }
    }

    pub fn select_next_token(&mut self, assets: &mut Assets) {
        let token_space = &self.token_spaces[self.selected];
        // NOTE: we cannot borrow map.tokens() to get the next token,
        // so we have to take a reference to the game's tokens.
        let game = assets.games.active();
        if let Some(hs) = assets.map.hex_state_mut(self.active_hex) {
            let next: &Token = hs
                .token_at(token_space)
                .and_then(|t| game.next_token(t))
                .unwrap_or_else(|| game.first_token());
            hs.set_token_at(token_space, *next);
        }
    }
}

impl UiState for EditTokens {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
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
        let border = n18hex::Colour::from((76, 76, 76));
        n18brush::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &Some(self.active_hex),
            border,
        );
    }
}
