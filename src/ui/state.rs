use cairo::Context;
use gtk::Inhibit;

use super::Action;
use crate::hex::Hex;
use crate::map::Map;

pub mod default;
pub mod edit_tokens;
pub mod replace_tile;
pub mod select_token;

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
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action);

    /// Responds to a mouse button being clicked, and returns the new state.
    fn button_press(
        self: Box<Self>,
        hex: &Hex,
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action);
}
