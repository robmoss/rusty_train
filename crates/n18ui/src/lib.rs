//! The [UI] struct encapsulates event handling, and draws the map on a
//! `gtk::DrawingArea` widget.
//!
//! See the `rusty_train` code for an example of how to use the [UI] struct.
//!
use cairo::Context;
use gtk::{GtkWindowExt, Inhibit, WidgetExt};

use n18game::Game;
use n18hex::Hex;
use n18map::Map;

/// Create custom dialog windows.
pub mod dialog;
/// The different states of the user interface.
pub mod state;
/// Various utility UI functions.
pub mod util;

use state::State;

/// Ordered collections of available games.
pub struct Games {
    games: Vec<Box<dyn Game>>,
    game_ix: usize,
}

impl Games {
    /// Creates a collection of games.
    pub fn new(games: Vec<Box<dyn Game>>) -> Self {
        Games { games, game_ix: 0 }
    }

    /// Returns a reference to the active game.
    pub fn active(&self) -> &dyn Game {
        &*self.games[self.game_ix]
    }

    /// Returns a mutable reference to the active game.
    pub fn active_mut(&mut self) -> &mut dyn Game {
        &mut *self.games[self.game_ix]
    }

    /// Returns the name of each game in the collection.
    pub fn names(&self) -> Vec<&str> {
        self.games.iter().map(|g| g.name()).collect()
    }

    /// Returns an iterator over the games in the collection.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Game> {
        // Note: de-reference &Box twice to obtain a dyn Game value.
        self.games.iter().map(|g| &**g)
    }

    /// Changes the active game.
    pub fn set_active(&mut self, ix: usize) -> bool {
        if ix < self.games.len() {
            self.game_ix = ix;
            true
        } else {
            false
        }
    }
}

/// Defines the non-UI game state components.
pub struct Content {
    hex: Hex,
    map: Map,
    games: Games,
}

/// Defines the current state of the user interface.
pub struct UI {
    content: Content,
    state: Option<Box<dyn State>>,
}

/// The actions that may be required when the UI state changes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    /// No action required.
    None,
    /// Quit the application.
    Quit,
    /// Redraw the surface.
    Redraw,
    /// Increase the hex size.
    ZoomIn,
    /// Decrease the hex size.
    ZoomOut,
}

// TODO: collect hex, map, window, area, event into a struct Event<T>
// where T is EventKey or EventButton?
// pub struct Event<'a, T> {
//     hex: &'a Hex,
//     map: &'a mut Map,
//     window: &'a gtk::ApplicationWindow,
//     area: &'a gtk::DrawingArea,
//     event: &'a T,
// }

impl UI {
    /// Creates the initial user interface state.
    pub fn new(hex: Hex, games: Vec<Box<dyn Game>>, map: Map) -> Self {
        let b: Box<dyn State> = Box::new(state::default::Default::new(&map));
        let state = Some(b);
        let games = Games::new(games);
        let content = Content { hex, map, games };
        UI { content, state }
    }

    /// Draws the current state of the user interface.
    pub fn draw(&self, ctx: &Context) {
        if let Some(ref state) = self.state {
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.reset_clip();
            let (x1, y1, x2, y2) = ctx.clip_extents();
            ctx.rectangle(x1, y1, x2, y2);
            ctx.fill();
            state.draw(&self.content, ctx);
        }
    }

    pub fn handle_action(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        action: Action,
        ctx: &Context,
    ) {
        match action {
            Action::ZoomIn => {
                if self.content.hex.max_d < 154.0 {
                    // NOTE: may need to increase surface, draw area size?
                    self.content.hex =
                        Hex::new(self.content.hex.max_d + 10.0);
                    let surf_w = ((self.content.map.max_col as f64)
                        * self.content.hex.min_d)
                        as i32;
                    let surf_h = ((self.content.map.max_row as f64)
                        * self.content.hex.max_d)
                        as i32;
                    area.set_size_request(surf_w, surf_h);
                    // NOTE: must redraw to the backing surface.
                    self.draw(ctx);
                    area.queue_draw();
                }
            }
            Action::ZoomOut => {
                if self.content.hex.max_d > 66.0 {
                    // NOTE: may need to decrease surface, draw area size?
                    self.content.hex =
                        Hex::new(self.content.hex.max_d - 10.0);
                    let surf_w = ((self.content.map.max_col as f64)
                        * self.content.hex.min_d)
                        as i32;
                    let surf_h = ((self.content.map.max_row as f64)
                        * self.content.hex.max_d)
                        as i32;
                    area.set_size_request(surf_w, surf_h);
                    // NOTE: must redraw to the backing surface.
                    self.draw(ctx);
                    area.queue_draw();
                }
            }
            Action::Redraw => {
                // NOTE: must redraw to the backing surface.
                self.draw(ctx);
                area.queue_draw();
            }
            Action::Quit => {
                window.close();
            }
            Action::None => {}
        }
    }

    /// Responds to a key being pressed.
    pub fn key_press_action(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Inhibit, Action) {
        let state_opt = self.state.take();
        if let Some(curr_state) = state_opt {
            // Note: use &* because Box<T> implements Deref<Target = T>.
            // So &*curr_state converts from Box<dyn State> to &dyn State.
            let action = global_keymap(
                &*curr_state,
                &mut self.content,
                window,
                area,
                event,
            );
            let (new_state, inhibit, action) =
                if let Some((reset_state, inhibit, action)) = action {
                    match reset_state {
                        ResetState::No => (curr_state, inhibit, action),
                        ResetState::Yes => {
                            let b: Box<dyn State> =
                                Box::new(state::default::Default::new(
                                    &self.content.map,
                                ));
                            (b, inhibit, action)
                        }
                    }
                } else {
                    curr_state.key_press(
                        &mut self.content,
                        window,
                        area,
                        event,
                    )
                };
            self.state = Some(new_state);
            (inhibit, action)
        } else {
            (Inhibit(false), Action::None)
        }
    }

    /// Responds to a key being pressed.
    pub fn key_press(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
        ctx: &Context,
    ) -> Inhibit {
        let (inhibit, action) = self.key_press_action(window, area, event);
        self.handle_action(window, area, action, ctx);
        inhibit
    }

    /// Responds to a mouse button being clicked.
    pub fn button_press_action(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventButton,
    ) -> (Inhibit, Action) {
        let state_opt = self.state.take();
        if let Some(curr_state) = state_opt {
            let (new_state, inhibit, action) = curr_state.button_press(
                &mut self.content,
                window,
                area,
                event,
            );
            self.state = Some(new_state);
            (inhibit, action)
        } else {
            (Inhibit(false), Action::None)
        }
    }

    /// Responds to a mouse button being clicked.
    pub fn button_press(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventButton,
        ctx: &Context,
    ) -> Inhibit {
        let (inhibit, action) = self.button_press_action(window, area, event);
        self.handle_action(window, area, action, ctx);
        inhibit
    }
}

/// Indicates whether the UI should reset to its default state.
///
/// For example, this is required when the global keymap results in a new map
/// being loaded from disk, since the current UI state may not be valid.
pub enum ResetState {
    /// Retain the current UI state.
    No,
    /// Reset the UI state to the default state.
    Yes,
}

/// The global keymap defines key bindings that apply regardless of the
/// current UI state.
///
/// - `q`, `Q`: quit;
/// - `s`, `S`: save a screenshot of the current map;
/// - `Ctrl+n`, `Ctrl+N`: load the starting map.
/// - `Ctrl+o`, `Ctrl+O`: load a map from disk.
/// - `Ctrl+s`, `Ctrl+S`: save the current map to disk.
pub fn global_keymap(
    state: &dyn State,
    content: &mut Content,
    window: &gtk::ApplicationWindow,
    area: &gtk::DrawingArea,
    event: &gdk::EventKey,
) -> Option<(ResetState, Inhibit, Action)> {
    let key = event.get_keyval();
    let modifiers = event.get_state();
    let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
    match (key, ctrl) {
        (gdk::keys::constants::q, false)
        | (gdk::keys::constants::Q, false) => {
            Some((ResetState::No, Inhibit(false), Action::Quit))
        }
        (gdk::keys::constants::n, true) | (gdk::keys::constants::N, true) => {
            // Prompt the user to select a game, and load its starting map.
            let game_names: Vec<&str> = content.games.names();
            let ix_opt =
                dialog::select_item(window, "Select a game", &game_names);
            if let Some(ix) = ix_opt {
                if content.games.set_active(ix) {
                    content.map =
                        content.games.active().create_map(&content.hex);
                    return Some((
                        ResetState::Yes,
                        Inhibit(false),
                        Action::Redraw,
                    ));
                }
            }
            Some((ResetState::No, Inhibit(false), Action::None))
        }
        (gdk::keys::constants::o, true) | (gdk::keys::constants::O, true) => {
            match util::load_map(window, &mut content.map) {
                Ok(action) => {
                    let reset = match action {
                        Action::None => ResetState::No,
                        _ => ResetState::Yes,
                    };
                    Some((reset, Inhibit(false), action))
                }
                Err(error) => {
                    eprintln!("Error loading map: {}", error);
                    Some((ResetState::No, Inhibit(false), Action::None))
                }
            }
        }
        (gdk::keys::constants::s, true) | (gdk::keys::constants::S, true) => {
            match util::save_map(window, &mut content.map) {
                Ok(action) => {
                    let reset = match action {
                        Action::None => ResetState::No,
                        _ => ResetState::Yes,
                    };
                    Some((reset, Inhibit(false), action))
                }
                Err(error) => {
                    eprintln!("Error saving map: {}", error);
                    Some((ResetState::No, Inhibit(false), Action::None))
                }
            }
        }
        (gdk::keys::constants::s, false)
        | (gdk::keys::constants::S, false) => {
            match util::save_screenshot(state, window, area, content) {
                Ok(action) => Some((ResetState::No, Inhibit(false), action)),
                Err(error) => {
                    eprintln!("Error saving screenshot: {}", error);
                    Some((ResetState::No, Inhibit(false), Action::None))
                }
            }
        }
        (gdk::keys::constants::plus, false)
        | (gdk::keys::constants::equal, false) => {
            Some((ResetState::No, Inhibit(false), Action::ZoomIn))
        }
        (gdk::keys::constants::minus, false)
        | (gdk::keys::constants::underscore, false) => {
            Some((ResetState::No, Inhibit(false), Action::ZoomOut))
        }
        _ => None,
    }
}
