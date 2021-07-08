//! Replaces the current tile in a hex (if any) with another tile.
//!
//! This mode allows the user to step through available tiles and select one
//! to replace the current tile.
//!
//! # Placed tokens
//!
//! This mode attempts to draw all of the tokens placed on the original tile.
//! Replacement tiles may not have a token space to match each token space on
//! the original tile, and so not all of the placed tokens may be drawn.
//! Note that each token space is identified by two indices: one for the city
//! and one for the token space in that city.
//! So even if the replacement tile has a sufficient number of token spaces,
//! it isn't clear how to identify an appropriate "equivalent" token space.
//!
//! For now, this mode only draws tokens for which there is a matching token
//! space (i.e., a matching city index and a matching token index).
//!
//! Once a replacement tile has been selected, the user may need to manually
//! place tokens on this tile.
//!
//! # Upgrading tiles
//!
//! This mode allows the user to replace a tile with any available tile, and
//! does not enforce any criteria for upgrade tiles.
//! Note that 18xx games may involve one of three
//! [different criteria](https://www.railsonboards.com/2020/12/26/permissive-restrictive-semi-restrictive-what-it-means/):
//! permissive, semi-restrictive, and restrictive.
//!
//! To support any or all of these criteria, this mode would need to record
//! the current tile's connections (including tokens) and only accept a
//! replacement tile if it satisfies all of these connections with its chosen
//! rotation.

use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;

use crate::Content;
use n18map::{HexAddress, Map, RotateCW};

/// Replacing one tile with another.
pub struct ReplaceTile {
    active_hex: HexAddress,
    candidates: Vec<usize>,
    selected: usize,
    show_original: bool,
    rotation: RotateCW,
}

impl ReplaceTile {
    pub(super) fn with_any(map: &Map, addr: HexAddress) -> Self {
        let candidates: Vec<usize> = (0..(map.tiles().len())).collect();
        ReplaceTile {
            active_hex: addr,
            candidates,
            selected: 0,
            show_original: false,
            rotation: RotateCW::Zero,
        }
    }

    pub(super) fn with_candidates(
        addr: HexAddress,
        candidates: Vec<usize>,
    ) -> Self {
        ReplaceTile {
            active_hex: addr,
            candidates,
            selected: 0,
            show_original: false,
            rotation: RotateCW::Zero,
        }
    }
}

impl State for ReplaceTile {
    fn draw(&self, content: &Content, ctx: &Context) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_hex_backgrounds(hex, ctx, &mut hex_iter);
        n18brush::draw_tiles(hex, ctx, &mut hex_iter);

        // Draw the replacement tile over the current tile.
        if !self.show_original {
            let map_hex = map.get_hex(self.active_hex);
            let m = map.prepare_to_draw(self.active_hex, hex, ctx);

            // Apply the appropriate tile rotation.
            let extra_angle = map_hex.map(|hs| -hs.radians()).unwrap_or(0.0);
            let rotn = self.rotation.radians() + extra_angle;
            ctx.rotate(rotn);

            // Draw the replacement tile.
            let tile_ix = self.candidates[self.selected];
            let tile = &map.tiles()[tile_ix];
            tile.draw(ctx, hex);

            // Draw any tokens that have been placed, if there is a matching
            // token space (i.e., matching city index and token index).
            // See the module doc comment, above, for details.
            if let Some(hs) = map_hex {
                for (token_space, token) in hs.get_tokens() {
                    if tile.define_token_space(&token_space, &hex, ctx) {
                        let tok_name = map.try_token_name(token);
                        if let Some(name) = tok_name {
                            token.draw(&hex, ctx, &name, rotn);
                        } else {
                            println!("Could not get token name.")
                        }
                    } else {
                        println!("Could not define token space.")
                    }
                }
            };

            ctx.set_matrix(m);
        }

        n18brush::outline_empty_hexes(hex, ctx, &mut hex_iter);
        n18brush::draw_barriers(hex, ctx, map);

        // Draw the active hex with a blue border.
        n18brush::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &Some(self.active_hex),
            0.0,
            0.0,
            0.7,
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
            gdk::keys::constants::Escape => (
                Box::new(super::default::Default::at_hex(Some(
                    self.active_hex,
                ))),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::keys::constants::Return => {
                if self.show_original {
                    (self, Inhibit(false), Action::None)
                } else {
                    // Replace the original tile with the current selection.
                    let tile_ix = self.candidates[self.selected];
                    let tile_name = map.tiles()[tile_ix].name.clone();
                    map.place_tile(
                        self.active_hex,
                        &tile_name,
                        self.rotation,
                    );
                    (
                        Box::new(super::default::Default::at_hex(Some(
                            self.active_hex,
                        ))),
                        Inhibit(false),
                        Action::Redraw,
                    )
                }
            }
            gdk::keys::constants::o | gdk::keys::constants::O => {
                self.show_original = !self.show_original;
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::Up => {
                if self.show_original {
                    (self, Inhibit(false), Action::None)
                } else {
                    if self.selected == 0 {
                        self.selected = self.candidates.len() - 1
                    } else {
                        self.selected -= 1
                    }
                    (self, Inhibit(false), Action::Redraw)
                }
            }
            gdk::keys::constants::Down => {
                if self.show_original {
                    (self, Inhibit(false), Action::None)
                } else {
                    self.selected += 1;
                    if self.selected >= self.candidates.len() {
                        self.selected = 0;
                    }
                    (self, Inhibit(false), Action::Redraw)
                }
            }
            gdk::keys::constants::less | gdk::keys::constants::comma => {
                self.rotation = self.rotation.rotate_anti_cw();
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::keys::constants::greater | gdk::keys::constants::period => {
                self.rotation = self.rotation.rotate_cw();
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
