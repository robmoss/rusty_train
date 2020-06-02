use cairo::Context;
use gtk::Inhibit;
use log::info;

use super::{Action, State};
use crate::Content;
use rusty_brush;
use rusty_map::{HexAddress, Map};

/// The default state: selecting a tile.
pub struct Default {
    active_hex: Option<HexAddress>,
}

impl Default {
    pub fn new(map: &Map) -> Self {
        Default {
            active_hex: map.default_hex(),
        }
    }

    pub fn at_hex(addr: Option<HexAddress>) -> Self {
        Default { active_hex: addr }
    }
}

impl State for Default {
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

        // Draw the active hex with a red border.
        rusty_brush::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &self.active_hex,
            0.7,
            0.0,
            0.0,
        );
    }

    fn key_press(
        mut self: Box<Self>,
        content: &mut Content,
        window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let map = &mut content.map;
        let key = event.get_keyval();
        match key {
            gdk::enums::key::e | gdk::enums::key::E => {
                if let Some(addr) = self.active_hex {
                    let state = Box::new(
                        super::replace_tile::ReplaceTile::with_any(map, addr),
                    );
                    (state, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::p | gdk::enums::key::P => {
                let phase_opt =
                    crate::dialog::select_phase(window, &content.game);
                if let Some(phase) = phase_opt {
                    content.game.set_phase(&mut content.map, phase);
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::u | gdk::enums::key::U => {
                if let Some(addr) = self.active_hex {
                    if let Some(tile) = map.tile_at(addr) {
                        let candidates: Vec<usize> = map
                            .tiles()
                            .iter()
                            .enumerate()
                            .filter(|(_ix, t)| {
                                map.can_upgrade_to(addr, t)
                                    && tile.can_upgrade_to(t)
                            })
                            .map(|(ix, _t)| ix)
                            .collect();
                        if candidates.len() > 0 {
                            let state = Box::new(
                                super::replace_tile::ReplaceTile::with_candidates(
                                    addr, candidates,
                                ),
                            );
                            (state, Inhibit(false), Action::Redraw)
                        } else {
                            info!("No upgrade candidates for {}", tile.name);
                            (self, Inhibit(false), Action::None)
                        }
                    } else {
                        // NOTE: attempting to place a tile on an empty hex.
                        let candidates: Vec<usize> = map
                            .tiles()
                            .iter()
                            .enumerate()
                            .filter(|(_ix, t)| {
                                map.can_place_on_empty(addr, t)
                            })
                            .map(|(ix, _t)| ix)
                            .collect();
                        if candidates.len() > 0 {
                            let state = Box::new(
                                super::replace_tile::ReplaceTile::with_candidates(
                                    addr, candidates,
                                ),
                            );
                            (state, Inhibit(false), Action::Redraw)
                        } else {
                            info!("No placement candidates for empty hex");
                            (self, Inhibit(false), Action::None)
                        }
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::t | gdk::enums::key::T => {
                if let Some(addr) = self.active_hex {
                    if let Some(state) =
                        super::edit_tokens::EditTokens::try_new(map, addr)
                    {
                        (Box::new(state), Inhibit(false), Action::Redraw)
                    } else {
                        (self, Inhibit(false), Action::None)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::r | gdk::enums::key::R => {
                // Allow the user to select tokens, and highlight all matching
                // tokens on the map.
                if let Some(addr) = self.active_hex {
                    if let Some(state) =
                        super::select_token::SelectToken::try_new(
                            content, addr, window,
                        )
                    {
                        (Box::new(state), Inhibit(false), Action::Redraw)
                    } else {
                        (self, Inhibit(false), Action::None)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
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
            gdk::enums::key::less | gdk::enums::key::comma => {
                if let Some(addr) = self.active_hex {
                    map.get_hex_mut(addr).map(|hs| hs.rotate_anti_cw());
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::greater | gdk::enums::key::period => {
                if let Some(addr) = self.active_hex {
                    map.get_hex_mut(addr).map(|hs| hs.rotate_cw());
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::BackSpace | gdk::enums::key::Delete => {
                if let Some(addr) = self.active_hex {
                    // TODO: allow this action to be undone?
                    map.remove_tile(addr);
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            _ => (self, Inhibit(false), Action::None),
        }
    }

    fn button_press(
        mut self: Box<Self>,
        content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let hex = &content.hex;
        let map = &mut content.map;
        // Allow the user to select hexes with a single click of any button.
        let (x, y) = event.get_position();
        let hexes = map.hexes();
        let ctx = hex.context();
        let addr = hexes.iter().find(|addr| {
            let m = map.prepare_to_draw(**addr, hex, ctx);
            hex.define_boundary(ctx);
            ctx.set_matrix(m);
            ctx.in_fill(x, y)
        });
        if let Some(a) = addr {
            self.active_hex = Some(*a);
            return (self, Inhibit(false), Action::Redraw);
        } else {
            return (self, Inhibit(false), Action::None);
        }
    }
}
