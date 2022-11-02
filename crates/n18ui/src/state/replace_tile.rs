//! Replaces the current tile in a hex (if any) with another tile.
//!
//! This mode allows the user to step through available tiles and select one
//! to replace the current tile.
//!
//! # Placed tokens
//!
//! This mode places all tokens from the original tile on the replacement
//! tile, preserving their track connectivity, whenever possible.
//! When this is not possible, this mode places **no tokens** on the
//! replacement tile.
//!
//! See [try_placing_tokens](n18map::map::try_placing_tokens) and the
//! [n18tile::ekmf] module for further details about placing tokens.
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

use cairo::Context;
use log::info;

use n18hex::RotateCW;
use n18map::{HexAddress, Map};
use n18tile::Tile;

use crate::{Assets, UiState};

/// Replacing one tile with another.
pub struct ReplaceTile {
    active_hex: HexAddress,
    candidates: Vec<usize>,
    selected: usize,
    show_original: bool,
    extra_rotation: RotateCW,
    original_rotation: RotateCW,
}

impl ReplaceTile {
    pub fn with_any(map: &Map, addr: HexAddress) -> Option<Self> {
        let candidates: Vec<usize> = (0..(map.num_tiles())).collect();
        // NOTE: must ensure there is at least one candidate tile.
        if candidates.is_empty() {
            return None;
        }
        // NOTE: record the current tile's rotation.
        let original_rotation = if let Some(hs) = map.hex_state(addr) {
            *hs.rotation()
        } else {
            RotateCW::Zero
        };
        Some(ReplaceTile {
            active_hex: addr,
            candidates,
            selected: 0,
            show_original: false,
            extra_rotation: RotateCW::Zero,
            original_rotation,
        })
    }

    fn with_candidates(addr: HexAddress, candidates: Vec<usize>) -> Self {
        ReplaceTile {
            active_hex: addr,
            candidates,
            selected: 0,
            show_original: false,
            extra_rotation: RotateCW::Zero,
            original_rotation: RotateCW::Zero,
        }
    }

    pub fn maybe_upgrade(assets: &Assets, addr: HexAddress) -> Option<Self> {
        if let Some(tile) = assets.map.tile_at(addr) {
            Self::maybe_upgrade_tile(assets, addr, tile)
        } else {
            Self::maybe_place_on_empty(assets, addr)
        }
    }

    fn maybe_upgrade_tile(
        assets: &Assets,
        addr: HexAddress,
        tile: &Tile,
    ) -> Option<Self> {
        let candidates: Vec<usize> = assets
            .map
            .available_tiles_iter()
            .enumerate()
            .filter(|(_ix, t)| {
                assets.map.can_upgrade_to(addr, t) && tile.can_upgrade_to(t)
            })
            .map(|(ix, _t)| ix)
            .collect();
        if candidates.is_empty() {
            info!("No candidates for tile {} at {}", tile.name, addr);
            None
        } else {
            let mut state = Self::with_candidates(addr, candidates);
            // NOTE: record the current tile's rotation.
            if let Some(hs) = assets.map.hex_state(addr) {
                state.original_rotation = *hs.rotation();
            }
            Some(state)
        }
    }

    fn maybe_place_on_empty(
        assets: &Assets,
        addr: HexAddress,
    ) -> Option<Self> {
        let candidates: Vec<usize> = assets
            .map
            .available_tiles_iter()
            .enumerate()
            .filter(|(_ix, t)| assets.map.can_place_on_empty(addr, t))
            .map(|(ix, _t)| ix)
            .collect();
        if candidates.is_empty() {
            info!("No candidates for empty hex {}", addr);
            None
        } else {
            Some(Self::with_candidates(addr, candidates))
        }
    }

    pub fn active_hex(&self) -> HexAddress {
        self.active_hex
    }

    pub fn net_rotation(&self) -> RotateCW {
        self.original_rotation + self.extra_rotation
    }

    pub fn place_candidate(&self, map: &mut Map) -> bool {
        if self.show_original {
            false
        } else {
            // Replace the original tile with the current selection.
            let tile_ix = self.candidates[self.selected];
            let tile_name = map.nth_tile(tile_ix).name.clone();
            // NOTE: the true rotation of the candidate is the sum of these
            // two rotations, because candidates are drawn with respect to the
            // original tile's rotation (if any).
            let tile_rotation = self.net_rotation();
            map.place_tile(self.active_hex, &tile_name, tile_rotation);
            true
        }
    }

    pub fn toggle_original_tile(&mut self) {
        self.show_original = !self.show_original;
    }

    pub fn select_previous_candidate(&mut self) -> bool {
        if self.show_original {
            false
        } else {
            if self.selected == 0 {
                self.selected = self.candidates.len() - 1
            } else {
                self.selected -= 1
            }
            true
        }
    }

    pub fn select_next_candidate(&mut self) -> bool {
        if self.show_original {
            false
        } else {
            self.selected += 1;
            if self.selected >= self.candidates.len() {
                self.selected = 0;
            }
            true
        }
    }

    pub fn rotate_candidate_anti_cw(&mut self) -> bool {
        if self.show_original {
            false
        } else {
            self.extra_rotation = self.extra_rotation.rotate_anti_cw();
            true
        }
    }

    pub fn rotate_candidate_cw(&mut self) -> bool {
        if self.show_original {
            false
        } else {
            self.extra_rotation = self.extra_rotation.rotate_cw();
            true
        }
    }
}

impl UiState for ReplaceTile {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_hex_backgrounds(hex, ctx, &mut hex_iter);
        n18brush::draw_tiles(hex, ctx, &mut hex_iter);

        // Draw the replacement tile over the current tile.
        if !self.show_original {
            // Find the replacement tile.
            let tile_ix = self.candidates[self.selected];
            let tile = &map.nth_tile(tile_ix);

            // Draw the replacement tile and placed tokens, if any, in
            // matching spaces (i.e., matching city index and token index).
            // See the module doc comment, above, for details.
            let map_hex = map.hex_state(self.active_hex);
            if let Some(hs) = map_hex {
                let tokens = n18map::map::try_placing_tokens(
                    hs.tile(map),
                    hs.rotation(),
                    hs.tokens(),
                    tile,
                    &self.net_rotation(),
                )
                .unwrap_or_default();
                n18brush::draw_tile_and_tokens_at(
                    hex,
                    ctx,
                    map,
                    &self.active_hex,
                    tile,
                    self.extra_rotation.radians(),
                    &tokens,
                );
            } else {
                n18brush::draw_tile_at(
                    hex,
                    ctx,
                    map,
                    &self.active_hex,
                    tile,
                    self.extra_rotation.radians(),
                );
            };
        }

        n18brush::outline_empty_hexes(hex, ctx, &mut hex_iter);
        n18brush::draw_barriers(hex, ctx, map);

        // Draw the active hex with a blue border.
        let border = n18hex::Colour::from((0, 0, 179));
        n18brush::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &Some(self.active_hex),
            border,
        );
    }
}
