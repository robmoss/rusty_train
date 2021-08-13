//! Selects tiles and switches to editing and route-finding modes.
use cairo::Context;
use std::sync::mpsc::{Receiver, Sender};

use n18map::{HexAddress, Map};

use crate::{
    Assets, Controller, PingDest, UiController, UiResponse, UiState,
};

/// The default state: selecting a tile.
pub struct Default {
    active_hex: HexAddress,
    sender: Sender<usize>,
    receiver: Receiver<usize>,
}

impl Default {
    pub fn new(map: &Map) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Default {
            active_hex: map.default_hex(),
            sender,
            receiver,
        }
    }

    pub fn at_hex(addr: HexAddress) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Default {
            active_hex: addr,
            sender,
            receiver,
        }
    }

    pub fn active_hex(&self) -> HexAddress {
        self.active_hex
    }

    pub fn set_active_hex(&mut self, addr: HexAddress) {
        self.active_hex = addr;
    }

    pub fn select_phase(&self, assets: &Assets, controller: &mut Controller) {
        let self_tx = self.sender.clone();
        let ping_tx = controller.ping_tx();
        controller.select_phase(assets.games.active(), move |ix_opt| {
            if let Some(ix) = ix_opt {
                self_tx.send(ix).unwrap();
                ping_tx.send_ping(PingDest::State).unwrap();
            }
        });
    }
}

impl UiState for Default {
    fn draw(&self, assets: &Assets, ctx: &Context) {
        let hex = &assets.hex;
        let map = &assets.map;
        let mut hex_iter = map.hex_iter(hex, ctx);

        n18brush::draw_map(hex, ctx, &mut hex_iter);

        // Draw the active hex with a red border.
        let border = n18hex::Colour::from((179, 0, 0));
        n18brush::highlight_active_hex(
            hex,
            ctx,
            &mut hex_iter,
            &Some(self.active_hex),
            border,
        );
    }

    fn ping(
        &mut self,
        assets: &mut Assets,
        _controller: &mut Controller,
    ) -> (UiResponse, Option<crate::State>) {
        let phase_ix = self.receiver.recv().unwrap();
        assets
            .games
            .active_mut()
            .set_phase_ix(&mut assets.map, phase_ix);
        (UiResponse::Redraw, None)
    }
}
