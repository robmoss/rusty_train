//! Provides a user interface for managing 18xx game maps and calculating the
//! optimal revenue for each company.
//!
//! See the `rusty_train` binary for an example of using this crate.
//!
//! # Overview
//!
//! The [UserInterface] struct encapsulates map drawing and event handling.
//! It is divided into several components:
//!
//! - [Assets]: the current [Map] and the available [Games].
//! - [Canvas]: the surface on which the map is drawn.
//! - [State]: the current user interface state (or "mode"), which defines the
//!   actions available to the user and how the map is drawn.
//! - [Keymap]: responds to keyboard and mouse input by triggering state
//!   actions.
//! - [Controller]: manages user interface elements and collects input from
//!   the user (e.g., selecting a game file to load or save).
//!
//! # Events and event handlers
//!
//! There are three types of events that [UserInterface] handles:
//!
//! - Mouse button events, which are described by [ButtonPress] structs and
//!   are handled by [UserInterface::handle_button_press].
//!
//! - Keyboard events, which are described by [KeyPress] structs and are
//!   handled by [UserInterface::handle_key_press].
//!
//! - "Ping" events, which are triggered by something else (e.g., a message
//!   from a task running in a separate thread) and are handled by
//!   [UserInterface::ping].
//!   Pings can be sent using a [PingSender], which can be obtained by calling
//!   [Controller::ping_tx](UiController::ping_tx).
//!
//! Note that event details, such as key identifiers, are defined by the
//! [ButtonPress] and [KeyPress] structs, which use types from the GDK 3
//! library.
//!
//! # Responses to event handlers
//!
//! Each of the [UserInterface] event handlers (listed above) return a
//! [UiResponse] value, which indicates what actions the [UserInterface]
//! should take (if any) once the event has been handled.
//! These actions are performed by [UserInterface::respond].
//!

use log::error;
use std::sync::mpsc::{Receiver, Sender};

use n18game::Game;
use n18hex::{Colour, Hex};
use n18map::Map;

/// Manage drawing surfaces.
pub mod canvas;
/// Control UI elements.
pub mod control;
/// Response to keyboard and mouse events.
pub mod keymap;
/// The different states of the user interface.
pub mod state;

#[doc(inline)]
pub use canvas::Canvas;

#[doc(inline)]
pub use control::{Controller, PingSender, UiController};

#[doc(inline)]
pub use keymap::{ButtonPress, KeyPress, Keymap};

#[doc(inline)]
pub use state::{State, UiState};

/// Identify which part of the UI should respond to a "ping".
pub enum PingDest {
    /// Ping the [UserInterface].
    TopLevel,
    /// Ping the current [State].
    State,
}

/// Ordered collections of available games.
pub struct Games {
    // NOTE: it would be nice to index games by name (e.g., in a
    // BTreeMap<String, Box<dyn Game>>) but then we'd need to store the name
    // of the active game as a separate String field, and create a new string
    // each time we change the active game.
    games: Vec<Box<dyn Game>>,
    game_ix: usize,
}

impl Games {
    /// Creates a collection of games.
    pub fn new<T>(games: T) -> Self
    where
        T: IntoIterator<Item = Box<dyn Game>>,
    {
        let games: Vec<_> = games.into_iter().collect();
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

    /// Changes the active game by name.
    pub fn set_active_name(&mut self, name: &str) -> bool {
        let ix_opt = self
            .games
            .iter()
            .enumerate()
            .find(|(_ix, game)| game.name() == name)
            .map(|(ix, _game)| ix);
        if let Some(ix) = ix_opt {
            self.game_ix = ix;
            true
        } else {
            false
        }
    }

    /// Changes the active game by index.
    pub fn set_active_index(&mut self, ix: usize) -> bool {
        if ix < self.games.len() {
            self.game_ix = ix;
            true
        } else {
            false
        }
    }
}

/// Defines the non-UI game state components.
pub struct Assets {
    pub hex: Hex,
    pub map: Map,
    pub games: Games,
}

/// Global UI actions, which are not specific to the current [State].
pub struct UiAction {
    action: Action,
}

/// Global UI actions, which are not specific to the current [State].
/// We wrap this type in a [UiAction] struct, in order to make the type public
/// while keeping the enum variants private.
enum Action {
    /// Create a new instance of the `nth` game in the library.
    NewGame(usize),
    /// Load a game from the provided path.
    LoadGame(std::path::PathBuf),
    /// Save the current game to the provided path.
    SaveGame(std::path::PathBuf),
    /// Save an image to the provided path.
    SaveImage(std::path::PathBuf, cairo::ImageSurface),
}

impl From<UiAction> for Action {
    fn from(action: UiAction) -> Self {
        action.action
    }
}

impl From<Action> for UiAction {
    fn from(action: Action) -> Self {
        UiAction { action }
    }
}

/// UI responses that may be required when the UI state changes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiResponse {
    /// No action required.
    None,
    /// Quit the application.
    Quit,
    /// Redraw the surface.
    Redraw,
    /// In response to changing the active game, resize the drawing area and
    /// redraw the surface.
    ResetGame,
    /// Increase the hex size.
    ZoomIn,
    /// Decrease the hex size.
    ZoomOut,
}

/// Defines the user interface.
pub struct UserInterface {
    pub assets: Assets,
    pub state: State,
    pub controller: Controller,
    pub keymap: Keymap,
    pub canvas: Canvas,
    sender: Sender<UiAction>,
    receiver: Receiver<UiAction>,
    min_hex_diameter: f64,
    max_hex_diameter: f64,
}

impl UserInterface {
    /// Creates the initial user interface state.
    pub fn new<T, C>(games: T, controller: C, keymap: Keymap) -> Self
    where
        T: IntoIterator<Item = Box<dyn Game>>,
        C: Into<Controller>,
    {
        let games = Games::new(games);
        let start_state = state::start::Start::new();
        let map = start_state.dummy_map();
        let controller = controller.into();

        let hex = Hex::default();
        let min_hex_diameter: f64 = 56.0;
        let max_hex_diameter: f64 = 164.0;

        let mut assets = Assets { hex, map, games };
        let state = State::Start(start_state);

        // Determine the surface dimensions necessary to contain the state
        // output at the highest zoom level.
        let dims =
            canvas::max_surface_dims(&state, &mut assets, max_hex_diameter);

        // Create the canvas and paint it white.
        let canvas = Canvas::new(dims.0, dims.1);
        n18brush::clear_surface(canvas.context(), Colour::WHITE);

        let (sender, receiver) = std::sync::mpsc::channel();

        UserInterface {
            assets,
            state,
            controller,
            keymap,
            canvas,
            sender,
            receiver,
            min_hex_diameter,
            max_hex_diameter,
        }
    }

    /// Draws the current state of the user interface.
    pub fn draw(&self) {
        self.state.draw(&self.assets, self.canvas.context())
    }

    /// Returns the minimum allowed hex diameter, which limits zooming out.
    pub fn min_hex_diameter(&self) -> f64 {
        self.min_hex_diameter
    }

    /// Returns the maximum allowed hex diameter, which limits zooming in.
    pub fn max_hex_diameter(&self) -> f64 {
        self.max_hex_diameter
    }

    #[must_use = "pass the response to UserInterface::respond"]
    pub fn handle_key_press(&mut self, event: &KeyPress) -> UiResponse {
        let response = self.keymap.handle_key_press(
            &mut self.assets,
            &mut self.controller,
            &mut self.state,
            &mut self.canvas,
            &self.sender,
            event,
        );
        if let Some((response, new_state_opt)) = response {
            if let Some(new_state) = new_state_opt {
                if new_state.is_default_state() {
                    self.set_default_title();
                }
                self.state = new_state
            }
            response
        } else {
            UiResponse::None
        }
    }

    #[must_use = "pass the response to UserInterface::respond"]
    pub fn handle_button_press(&mut self, event: &ButtonPress) -> UiResponse {
        let response = self.keymap.handle_button_press(
            &mut self.assets,
            &mut self.controller,
            &mut self.state,
            &mut self.canvas,
            &self.sender,
            event,
        );
        if let Some((response, new_state_opt)) = response {
            if let Some(new_state) = new_state_opt {
                if new_state.is_default_state() {
                    self.set_default_title();
                }
                self.state = new_state
            }
            response
        } else {
            UiResponse::None
        }
    }

    pub fn respond(&mut self, response: UiResponse) {
        match response {
            UiResponse::ZoomIn => {
                let new_max_d = self.assets.hex.max_d + 10.0;
                if new_max_d < self.max_hex_diameter() {
                    // NOTE: may need to increase surface, draw area size?
                    self.assets.hex.resize(new_max_d);
                    self.zoom_and_redraw();
                }
            }
            UiResponse::ZoomOut => {
                let new_max_d = self.assets.hex.max_d - 10.0;
                if new_max_d > self.min_hex_diameter() {
                    // NOTE: may need to decrease surface, draw area size?
                    self.assets.hex.resize(new_max_d);
                    self.zoom_and_redraw();
                }
            }
            UiResponse::Redraw => {
                // NOTE: must redraw to the backing surface.
                self.draw();
                self.controller.redraw();
            }
            UiResponse::ResetGame => {
                // NOTE: request this size request is only required when the
                // game map has been replaced (e.g., by starting a new game or
                // by loading a saved game).
                self.reset_and_redraw();
            }
            UiResponse::Quit => self.controller.quit(),
            UiResponse::None => {}
        }
    }

    /// Returns the dimensions of the current game map, in pixels.
    pub fn map_size(&self) -> (i32, i32) {
        canvas::required_dims(&self.state, &self.assets)
    }

    /// Requests the drawing area to update its size, and redraws the current
    /// game state.
    ///
    /// This should be called when the user has zoomed in or zoomed out.
    pub fn zoom_and_redraw(&mut self) {
        let (width, height) =
            canvas::required_dims(&self.state, &self.assets);
        self.controller.resize(width, height);
        self.draw();
        self.controller.redraw();
    }

    /// Resets the drawing surface, requests the drawing area to update its
    /// size, and redraws the current game state.
    ///
    /// This should only be called when the active game has changed (e.g.,
    /// when creating a new game or loading a saved game).
    pub fn reset_and_redraw(&mut self) {
        let hex_d = self.max_hex_diameter();
        let (width, height) =
            canvas::max_surface_dims(&self.state, &mut self.assets, hex_d);

        // Resize the underlying image surface if it is too small.
        let curr_width = self.canvas.width();
        let curr_height = self.canvas.height();
        let resize = (curr_width < width) || (curr_height < height);
        if resize {
            self.canvas.resize(width, height);
            // Paint the new surface white.
            n18brush::clear_surface(self.canvas.context(), Colour::WHITE);
        }

        // NOTE: resize the drawing area to fit the current state at the
        // current zoom level, not at the maximum zoom level.
        self.zoom_and_redraw()
    }

    /// Sets the window title to the game name, replacing any state-specific
    /// title.
    pub fn set_default_title(&mut self) {
        let title = self.assets.games.active().name();
        self.controller.set_window_title(title);
    }

    /// Responds to an event triggered by something other than a UI event
    /// (e.g., a message from a task running in a separate thread).
    #[must_use = "pass the response to UserInterface::respond"]
    pub fn ping(&mut self, dest: PingDest) -> UiResponse {
        match dest {
            PingDest::State => {
                let (response, new_state_opt) =
                    self.state.ping(&mut self.assets, &mut self.controller);
                if let Some(new_state) = new_state_opt {
                    if new_state.is_default_state() {
                        self.set_default_title();
                    }
                    self.state = new_state;
                }
                response
            }
            PingDest::TopLevel => {
                let msg = self.receiver.recv().unwrap();
                let action = msg.into();
                match action {
                    Action::NewGame(game_ix) => self.new_game(game_ix),
                    Action::LoadGame(path) => self.load_game(path),
                    Action::SaveGame(path) => self.save_game(path),
                    Action::SaveImage(path, image) => {
                        self.save_image(path, image)
                    }
                }
            }
        }
    }

    /// Creates a new game, identified by index into the game library.
    #[must_use = "pass the response to UserInterface::respond"]
    pub fn new_game(&mut self, game_ix: usize) -> UiResponse {
        if self.assets.games.set_active_index(game_ix) {
            self.assets.map =
                self.assets.games.active().create_map(&self.assets.hex);
            self.set_default_title();
            let active_hex = self.assets.map.default_hex();
            self.state = State::default_state(active_hex);
            UiResponse::ResetGame
        } else {
            UiResponse::None
        }
    }

    /// Loads a saved game state from `path`.
    #[must_use = "pass the response to UserInterface::respond"]
    pub fn load_game(&mut self, path: std::path::PathBuf) -> UiResponse {
        let game_state = n18io::read_game_state(&path).unwrap_or_else(|_| {
            panic!("Could not read '{}'", path.display())
        });
        if self.assets.games.set_active_name(&game_state.game) {
            if let Some(new_map) = self
                .assets
                .games
                .active_mut()
                .load(&self.assets.hex, game_state)
            {
                self.set_default_title();
                self.assets.map = new_map;
                let active_hex = self.assets.map.default_hex();
                self.state = State::default_state(active_hex);
                UiResponse::ResetGame
            } else {
                error!("game.load() returned None");
                UiResponse::None
            }
        } else {
            error!("no game called '{}'", game_state.game);
            UiResponse::None
        }
    }

    /// Saves the current game state to `path`.
    #[must_use = "pass the response to UserInterface::respond"]
    pub fn save_game(&mut self, path: std::path::PathBuf) -> UiResponse {
        let game_state = self.assets.games.active().save(&self.assets.map);
        n18io::write_game_state(&path, game_state, true).unwrap_or_else(
            |_| panic!("Could not write '{}'", path.display()),
        );
        UiResponse::None
    }

    /// Saves an image to `path`.
    #[must_use = "pass the response to UserInterface::respond"]
    pub fn save_image(
        &self,
        path: std::path::PathBuf,
        image: cairo::ImageSurface,
    ) -> UiResponse {
        let mut file = std::fs::File::create(&path).unwrap_or_else(|_| {
            panic!("Couldn't create '{}'", path.display())
        });
        image.write_to_png(&mut file).unwrap_or_else(|_| {
            panic!("Couldn't write '{}'", path.display())
        });
        UiResponse::None
    }
}
