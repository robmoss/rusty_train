use cairo::Context;
use gtk::Inhibit;

use super::{Action, Content};

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
        content: &mut Content,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Option<Box<dyn State>>, Inhibit, Action);

    /// Responds to a mouse button being clicked, and returns the new state.
    fn button_press(
        &mut self,
        content: &mut Content,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventButton,
    ) -> (Option<Box<dyn State>>, Inhibit, Action);
}
