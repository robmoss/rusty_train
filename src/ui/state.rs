use cairo::Context;
use gtk::Inhibit;

use crate::hex::Hex;
use crate::map::{HexAddress, Map};

/// The actions that may be required when the UI state changes.
pub enum Action {
    /// No action required.
    None,
    /// Quit the application.
    Quit,
    /// Redraw the surface.
    Redraw,
}

/// The methods that are required in order to manipulate the user interface.
pub trait State {
    /// Draws the current state of the map.
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        width: i32,
        height: i32,
        ctx: &Context,
    );

    /// Responds to a key being pressed, and returns the new state.
    fn key_press(
        self: Box<Self>,
        hex: &Hex,
        map: &Map,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action);

    /// Responds to a mouse button being clicked, and returns the new state.
    fn button_press(
        self: Box<Self>,
        hex: &Hex,
        map: &Map,
        event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action);
}

/// The default state: selecting a tile.
pub struct Default {
    active_hex: Option<HexAddress>,
}

/// Replacing one tile with another.
pub struct Edit {
    active_hex: Option<HexAddress>,
}

impl Default {
    pub fn new(map: &Map) -> Self {
        Default {
            active_hex: map.default_hex(),
        }
    }
}

impl State for Default {
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        for (_addr, tile_opt) in map.hex_iter(hex, ctx) {
            if let Some((tile, tokens)) = tile_opt {
                // Draw the tile and any tokens.
                tile.draw(ctx, hex);
                for (tok, map_token) in tokens.iter() {
                    tile.define_tok_path(&tok, &hex, ctx);
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

        for (addr, _tile_opt) in map.hex_iter(hex, ctx) {
            if self.active_hex == Some(addr) {
                // Draw the active hex with a red border.
                ctx.set_source_rgb(0.7, 0.0, 0.0);
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
        map: &Map,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
            gdk::enums::key::q | gdk::enums::key::Q => {
                (self, Inhibit(false), Action::Quit)
            }
            gdk::enums::key::e | gdk::enums::key::E => (
                Box::new(Edit {
                    active_hex: self.active_hex,
                }),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::enums::key::Left => {
                // TODO: these are boilerplate, define a common function?
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_col(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Right => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_col(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Up => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_row(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Down => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_row(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
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
        _map: &Map,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}

impl State for Edit {
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        for (_addr, tile_opt) in map.hex_iter(hex, ctx) {
            if let Some((tile, tokens)) = tile_opt {
                // Draw the tile and any tokens.
                tile.draw(ctx, hex);
                for (tok, map_token) in tokens.iter() {
                    tile.define_tok_path(&tok, &hex, ctx);
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

        for (addr, _tile_opt) in map.hex_iter(hex, ctx) {
            if self.active_hex == Some(addr) {
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
        self: Box<Self>,
        _hex: &Hex,
        _map: &Map,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        if key == gdk::enums::key::q || key == gdk::enums::key::Q {
            (self, Inhibit(false), Action::Quit)
        } else if key == gdk::enums::key::Escape {
            (
                Box::new(Default {
                    active_hex: self.active_hex,
                }),
                Inhibit(false),
                Action::Redraw,
            )
        } else {
            (self, Inhibit(false), Action::None)
        }
    }

    fn button_press(
        self: Box<Self>,
        _hex: &Hex,
        _map: &Map,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}
