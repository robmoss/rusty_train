use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;

use crate::Content;
use n18brush;
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

        for hex_state in &mut hex_iter {
            if hex_state.addr == self.active_hex && !self.show_original {
                // Draw the currently-selected replacement tile.
                // NOTE: must account for the current tile's rotation.
                let extra_angle =
                    if let Some(hs) = map.get_hex(hex_state.addr) {
                        -hs.radians()
                    } else {
                        0.0
                    };
                let rotn = self.rotation.radians() + extra_angle;
                ctx.rotate(rotn);
                let tile_ix = self.candidates[self.selected];
                let tile = &map.tiles()[tile_ix];
                tile.draw(ctx, hex);
                if let Some((_tile, token_spaces)) = hex_state.tile_state {
                    // Draw any tokens that have been placed.
                    // NOTE: the replacement tile may not have a matching
                    // token space; when editing a tile there may be fewer
                    // token spaces, and when upgrading there may be fewer
                    // cities --- and the token_space is linked to the city
                    // index. So we really need to identify an appropriate
                    // "equivalent" token space, if one exists. For now, this
                    // only draws the token if there is a matching space
                    // (i.e., matching city index and token index).
                    for (token_space, map_token) in token_spaces.iter() {
                        // Determine if the tile has a matching token space.
                        if tile.define_token_space(&token_space, &hex, ctx) {
                            let tok_name = content
                                .map
                                .tokens()
                                .get_name(map_token)
                                .unwrap();
                            map_token.draw(&hex, ctx, &tok_name, rotn);
                        } else {
                            println!("Could not define token space.")
                        }
                    }
                }
                ctx.rotate(-self.rotation.radians() - extra_angle);
            } else if let Some((tile, token_spaces)) = hex_state.tile_state {
                // Draw the tile and any tokens.
                tile.draw(ctx, hex);
                let rotn = hex_state.tile_rotation;
                for (token_space, map_token) in token_spaces.iter() {
                    if tile.define_token_space(&token_space, &hex, ctx) {
                        let tok_name =
                            content.map.tokens().get_name(map_token).unwrap();
                        map_token.draw(&hex, ctx, &tok_name, rotn);
                    } else {
                        println!("Could not define token space.")
                    }
                }
            } else {
                // Draw an empty hex.
                n18brush::draw_empty_hex(hex, ctx);
            }
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
