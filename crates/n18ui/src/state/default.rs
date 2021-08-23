//! Selects tiles and switches to editing and route-finding modes.
use cairo::Context;
use log::info;

use super::{Action, State};
use crate::{Content, Ping};
use n18map::{HexAddress, Map};

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
    fn draw(&self, content: &Content, ctx: &Context) {
        let hex = &content.hex;
        let map = &content.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_map(hex, ctx, &mut hex_iter);

        // Draw the active hex with a red border.
        let border = n18hex::Colour::from((179, 0, 0));
        n18brush::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &self.active_hex,
            border,
        );
    }

    fn key_press(
        &mut self,
        content: &mut Content,
        window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
        ping_tx: &Ping,
    ) -> (Option<Box<dyn State>>, Action) {
        let map = &mut content.map;
        let key = event.keyval();
        match key {
            gdk::keys::constants::e | gdk::keys::constants::E => {
                if let Some(addr) = self.active_hex {
                    let state = Box::new(
                        super::replace_tile::ReplaceTile::with_any(map, addr),
                    );
                    (Some(state), Action::Redraw)
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::p | gdk::keys::constants::P => {
                // Note: use &* because Box<T> implements Deref<Target = T>.
                // So &*content.game converts from Box<dyn Game> to &dyn Game.
                let phase_opt = crate::dialog::select_phase(
                    window,
                    content.games.active(),
                );
                if let Some(phase) = phase_opt {
                    content
                        .games
                        .active_mut()
                        .set_phase_ix(&mut content.map, phase);
                    (None, Action::Redraw)
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::u | gdk::keys::constants::U => {
                if let Some(addr) = self.active_hex {
                    if let Some(tile) = map.tile_at(addr) {
                        let candidates: Vec<usize> = map
                            .available_tiles_iter()
                            .enumerate()
                            .filter(|(_ix, t)| {
                                map.can_upgrade_to(addr, t)
                                    && tile.can_upgrade_to(t)
                            })
                            .map(|(ix, _t)| ix)
                            .collect();
                        if !candidates.is_empty() {
                            let state = Box::new(
                                super::replace_tile::ReplaceTile::with_candidates(
                                    addr, candidates,
                                ),
                            );
                            (Some(state), Action::Redraw)
                        } else {
                            info!("No upgrade candidates for {}", tile.name);
                            (None, Action::None)
                        }
                    } else {
                        // NOTE: attempting to place a tile on an empty hex.
                        let candidates: Vec<usize> = map
                            .available_tiles_iter()
                            .enumerate()
                            .filter(|(_ix, t)| {
                                map.can_place_on_empty(addr, t)
                            })
                            .map(|(ix, _t)| ix)
                            .collect();
                        if !candidates.is_empty() {
                            let state = Box::new(
                                super::replace_tile::ReplaceTile::with_candidates(
                                    addr, candidates,
                                ),
                            );
                            (Some(state), Action::Redraw)
                        } else {
                            info!("No placement candidates for empty hex");
                            (None, Action::None)
                        }
                    }
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::t | gdk::keys::constants::T => {
                if let Some(addr) = self.active_hex {
                    if let Some(state) =
                        super::edit_tokens::EditTokens::try_new(map, addr)
                    {
                        (Some(Box::new(state)), Action::Redraw)
                    } else {
                        (None, Action::None)
                    }
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::r | gdk::keys::constants::R => {
                // Allow the user to select a company and trains, and find the
                // routes that earn the most revenue.
                let state_opt = super::find_routes::FindRoutes::try_new(
                    content,
                    self.active_hex.as_ref(),
                    window,
                    ping_tx,
                );
                if let Some(state) = state_opt {
                    (Some(Box::new(state)), Action::Redraw)
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::Left => {
                // TODO: these are boilerplate, define a common function?
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_col(addr);
                    if new_addr == addr {
                        (None, Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (None, Action::Redraw)
                    }
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::Right => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_col(addr);
                    if new_addr == addr {
                        (None, Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (None, Action::Redraw)
                    }
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::Up => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_row(addr);
                    if new_addr == addr {
                        (None, Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (None, Action::Redraw)
                    }
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::Down => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_row(addr);
                    if new_addr == addr {
                        (None, Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (None, Action::Redraw)
                    }
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::less | gdk::keys::constants::comma => {
                if let Some(addr) = self.active_hex {
                    if let Some(hs) = map.hex_mut(addr) {
                        hs.rotate_anti_cw()
                    }
                    (None, Action::Redraw)
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::greater | gdk::keys::constants::period => {
                if let Some(addr) = self.active_hex {
                    if let Some(hs) = map.hex_mut(addr) {
                        hs.rotate_cw()
                    }
                    (None, Action::Redraw)
                } else {
                    (None, Action::None)
                }
            }
            gdk::keys::constants::BackSpace
            | gdk::keys::constants::Delete => {
                if let Some(addr) = self.active_hex {
                    // TODO: allow this action to be undone?
                    map.remove_tile(addr);
                    (None, Action::Redraw)
                } else {
                    (None, Action::None)
                }
            }
            _ => (None, Action::None),
        }
    }

    fn button_press(
        &mut self,
        content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventButton,
        _ping_tx: &Ping,
    ) -> (Option<Box<dyn State>>, Action) {
        let hex = &content.hex;
        let map = &mut content.map;
        // Allow the user to select hexes with a single click of any button.
        let (x, y) = event.position();
        let hexes = map.hexes();
        let ctx = hex.context();
        let addr = hexes.iter().find(|addr| {
            let m = map.prepare_to_draw(**addr, hex, ctx);
            hex.define_boundary(ctx);
            ctx.set_matrix(m);
            ctx.in_fill(x, y).unwrap()
        });
        if let Some(a) = addr {
            self.active_hex = Some(*a);
            (None, Action::Redraw)
        } else {
            (None, Action::None)
        }
    }
}
