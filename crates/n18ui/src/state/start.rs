//! Defines the starting UI state.
use cairo::Context;
use gtk::Inhibit;

use n18example::LabelBuilder;

use super::{Action, State};
use crate::Content;

/// The starting UI state: no game, no map.
pub struct Start {}

impl Start {
    pub fn new() -> Self {
        Start {}
    }

    pub fn dummy_map(&self) -> n18map::Map {
        let tiles = vec![];
        let tokens = vec![].into();
        // NOTE: a map must have at least one hex.
        // And these hexes currently define the size of the image buffer.
        let hexes = vec![
            n18map::HexAddress::new(0, 0),
            n18map::HexAddress::new(50, 50),
        ];
        n18map::Map::new(tiles, tokens, hexes)
    }
}

impl Default for Start {
    fn default() -> Self {
        Start {}
    }
}

impl State for Start {
    fn draw(&self, content: &Content, ctx: &Context) {
        ctx.set_source_rgb(0.9, 0.9, 0.9);
        ctx.paint();
        let usage = LabelBuilder::new(
            ctx,
            &content.hex,
            "Ctrl+N: Start a new game\nCtrl+O: Load a saved game",
        )
        .font_size(16.0)
        .into_label()
        .unwrap();
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        usage.draw_at(20.0, 20.0);
    }

    fn key_press(
        self: Box<Self>,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
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
