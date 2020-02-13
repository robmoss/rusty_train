use super::{Action, State};

use cairo::Context;
use gtk::Inhibit;

use crate::hex::Hex;
use crate::map::{HexAddress, Map, RotateCW};

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
        for (addr, tile_opt) in &mut hex_iter {
            if addr == self.active_hex && !self.show_original {
                // Draw the currently-selected replacement tile.
                // NOTE: must account for the current tile's rotation.
                let extra_angle = if let Some(hs) = map.get_hex(addr) {
                    -hs.radians()
                } else {
                    0.0
                };
                ctx.rotate(self.rotation.radians() + extra_angle);
                let tile_ix = self.candidates[self.selected];
                let tile = &map.tiles()[tile_ix];
                tile.draw(ctx, hex);
                if let Some((_tile, token_spaces)) = tile_opt {
                    // Draw any tokens that have been placed.
                    for (token_space, map_token) in token_spaces.iter() {
                        tile.define_token_space(&token_space, &hex, ctx);
                        map_token.draw_token(&hex, ctx);
                    }
                }
                ctx.rotate(-self.rotation.radians() - extra_angle);
            } else if let Some((tile, token_spaces)) = tile_opt {
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
        for (addr, _tile_opt) in &mut hex_iter {
            if self.active_hex == addr {
                // Draw the active hex with a blue border.
                ctx.set_source_rgb(0.0, 0.0, 0.7);
                ctx.set_line_width(hex.max_d * 0.02);
                hex.define_boundary(ctx);
                ctx.stroke();
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
            gdk::enums::key::q | gdk::enums::key::Q => {
                (self, Inhibit(false), Action::Quit)
            }
            gdk::enums::key::Escape => (
                Box::new(super::default::Default::at_hex(Some(
                    self.active_hex,
                ))),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::enums::key::Return => {
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
            gdk::enums::key::o | gdk::enums::key::O => {
                self.show_original = !self.show_original;
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::Up => {
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
            gdk::enums::key::Down => {
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
            gdk::enums::key::less | gdk::enums::key::comma => {
                self.rotation = self.rotation.rotate_anti_cw();
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::greater | gdk::enums::key::period => {
                self.rotation = self.rotation.rotate_cw();
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
