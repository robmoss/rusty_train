//! # Example
//!
//! The `UI` struct encapsulates event handling and drawing the map. It
//! requires a `gtk::DrawingArea` widget, and the `UI` methods must be
//! connected to the appropriate GTK signals:
//!
//! - `drawing_area.connect_draw` should dispatch to `UI::draw`;
//! - `drawing_area.connect_button_press_event` should dispatch to
//!   `UI::button_press`; and
//! - `drawing_area.connect_key_press_event` should dispatch to
//!   `UI::key_press`.
//!
//! See the code below for a complete example of how to do this.
//!
//! ```rust,no_run
//! use std::cell::RefCell;
//! use std::rc::Rc;
//!
//! use gtk::prelude::*;
//! use gtk::DrawingArea;
//! use cairo::Context;
//!
//! use rusty_hex::*;
//! use rusty_tile::*;
//! use rusty_map::*;
//! use rusty_game::*;
//! use rusty_ui::UI;
//!
//! // Define the tile geometry.
//! let hex_diameter = 125.0;
//! let hex = Hex::new(hex_diameter);
//!
//! // Use a provided game.
//! let game = _1867::Game::new(&hex);
//! let map = game.create_map(&hex);
//!
//! // Create the initial UI state.
//! let game_box = Box::new(game);
//! let state = Rc::new(RefCell::new(UI::new(hex, game_box, map)));
//!
//! // Create a GTK application.
//! let application = gtk::Application::new(
//!     Some("rusty_train.test_ui"),
//!     Default::default(),
//! )
//! .expect("Initialisation failed...");
//!
//! // Create the GTK widgets that will be used to draw the map.
//! let window = gtk::ApplicationWindow::new(&application);
//! let drawing_area = Box::new(DrawingArea::new)();
//!
//! // Let the UI handle mouse button events.
//! let app = state.clone();
//! let da = drawing_area.clone();
//! let w = window.clone();
//! drawing_area.connect_button_press_event(move |_widget, event| {
//!     let mut ui = app.borrow_mut();
//!     ui.button_press(&w, &da, event)
//! });
//! window.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
//! drawing_area.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
//!
//! // Let the UI handle keyboard events.
//! let app = state.clone();
//! let da = drawing_area.clone();
//! let w = window.clone();
//! window.connect_key_press_event(move |_widget, event| {
//!     let mut ui = app.borrow_mut();
//!     ui.key_press(&w, &da, event)
//! });
//! window.add_events(gdk::EventMask::KEY_PRESS_MASK);
//!
//! // Let the UI draw on the window.
//! drawing_area.connect_draw(move |area, ctx| {
//!     let w = area.get_allocated_width();
//!     let h = area.get_allocated_height();
//!     let ui = state.borrow();
//!     ui.draw(w, h, ctx);
//!     Inhibit(false)
//! });
//!
//! // Display the window.
//! let (width, height) = (1366, 740);
//! window.set_default_size(width, height);
//! window.add(&drawing_area);
//! window.show_all();
//! ```
//!
use cairo::Context;
use gtk::{GtkWindowExt, Inhibit, WidgetExt};

use rusty_game::Game;
use rusty_hex::Hex;
use rusty_map::Map;

/// Create custom dialog windows.
pub mod dialog;
/// The different states of the user interface.
pub mod state;
/// Various utility UI functions.
pub mod util;

use state::State;

/// Defines the non-UI game state components.
pub struct Content {
    hex: Hex,
    map: Map,
    game: Box<dyn Game>,
}

/// Defines the current state of the user interface.
pub struct UI {
    content: Content,
    state: Option<Box<dyn State>>,
}

/// The actions that may be required when the UI state changes.
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
    pub fn new(hex: Hex, game: Box<dyn Game>, map: Map) -> Self {
        let b: Box<dyn State> = Box::new(state::default::Default::new(&map));
        let state = Some(b);
        let content = Content { hex, game, map };
        UI { content, state }
    }

    /// Draws the current state of the user interface.
    pub fn draw(&self, w: i32, h: i32, ctx: &Context) {
        if let Some(ref state) = self.state {
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.rectangle(0.0, 0.0, w as f64, h as f64);
            ctx.fill();
            state.draw(&self.content, w, h, ctx);
        }
    }

    /// Responds to a key being pressed.
    pub fn key_press(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> Inhibit {
        let state_opt = self.state.take();
        if let Some(curr_state) = state_opt {
            let action = global_keymap(
                &curr_state,
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
                        area.queue_draw();
                    }
                }
                Action::Redraw => {
                    area.queue_draw();
                }
                Action::Quit => {
                    window.close();
                }
                Action::None => {}
            }
            inhibit
        } else {
            Inhibit(false)
        }
    }

    /// Responds to a mouse button being clicked.
    pub fn button_press(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventButton,
    ) -> Inhibit {
        let state_opt = self.state.take();
        if let Some(curr_state) = state_opt {
            let (new_state, inhibit, action) = curr_state.button_press(
                &mut self.content,
                window,
                area,
                event,
            );
            self.state = Some(new_state);
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
                        area.queue_draw();
                    }
                }
                Action::Redraw => {
                    area.queue_draw();
                }
                Action::Quit => {
                    window.close();
                }
                Action::None => {}
            }
            inhibit
        } else {
            Inhibit(false)
        }
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
/// - `Ctrl+o`, `Ctrl+O`: load a map from disk.
/// - `Ctrl+s`, `Ctrl+S`: save the current map to disk.
pub fn global_keymap<S: State + ?Sized>(
    state: &Box<S>,
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
