//! Defines the starting UI state.
use cairo::Context;
use gtk::Inhibit;

use n18hex::theme::Text;
use n18hex::Coord;

use super::{Action, State};
use crate::Content;

/// The starting UI state: no game, no map.
pub struct Start {}

impl Start {
    pub fn new() -> Self {
        Start {}
    }

    pub fn dummy_map(&self) -> n18map::Map {
        let tiles: Vec<n18tile::Tile> = vec![];
        let tokens = vec![].into();
        // NOTE: a map must have at least one hex.
        let hexes = vec![n18map::HexAddress::new(0, 0)];
        n18map::Map::new(tiles.into(), tokens, hexes)
    }
}

impl Default for Start {
    fn default() -> Self {
        Start {}
    }
}

impl State for Start {
    fn draw(&self, content: &Content, ctx: &Context) {
        let usage_str =
            "Ctrl+N: Start a new game\nCtrl+O: Load a saved game\nQ: Quit";
        let labeller =
            Text::new().font_size(16.0).labeller(ctx, &content.hex);
        labeller.draw(usage_str, Coord::from((20.0, 20.0)));
    }

    fn key_press(
        &mut self,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventKey,
    ) -> (Option<Box<dyn State>>, Inhibit, Action) {
        (None, Inhibit(false), Action::None)
    }

    fn button_press(
        &mut self,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Option<Box<dyn State>>, Inhibit, Action) {
        (None, Inhibit(false), Action::None)
    }
}
