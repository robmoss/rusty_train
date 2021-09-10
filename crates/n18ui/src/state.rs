use cairo::Context;

use super::{Action, Content, Ping};

pub mod default;
pub mod edit_tokens;
pub mod find_routes;
pub mod replace_tile;
pub mod start;

/// The methods that are required in order to manipulate the user interface.
pub trait State {
    /// Draws the current state of the map.
    ///
    /// Note that this method should not draw a background by filling the
    /// entire surface, because this makes it impossible to determine the
    /// appropriate surface dimensions.
    fn draw(&self, content: &Content, ctx: &Context);

    /// Responds to a key being pressed, and returns the new state.
    fn key_press(
        &mut self,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &super::KeyPress,
        _ping_tx: &Ping,
    ) -> (Option<Box<dyn State>>, Action) {
        (None, Action::None)
    }

    /// Responds to a mouse button being clicked, and returns the new state.
    fn button_press(
        &mut self,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &super::ButtonPress,
        _ping_tx: &Ping,
    ) -> (Option<Box<dyn State>>, Action) {
        (None, Action::None)
    }

    fn ping(
        &mut self,
        _content: &mut Content,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _ping_tx: &Ping,
    ) -> (Option<Box<dyn State>>, Action) {
        (None, Action::None)
    }
}
