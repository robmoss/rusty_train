//! The [UI] struct encapsulates event handling, and draws the map on a
//! `gtk::DrawingArea` widget.
//!
//! See the `rusty_train` code for an example of how to use the [UI] struct.
//!
use cairo::{Context, ImageSurface};
use gtk::prelude::{GtkWindowExt, WidgetExt};
use log::info;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

use n18game::Game;
use n18hex::{Colour, Hex};
use n18map::Map;

/// Create custom dialog windows.
pub mod dialog;
/// The different states of the user interface.
pub mod state;

/// Identify which part of the UI should respond to a "ping".
pub enum PingDest {
    /// Ping the user interface.
    TopLevel,
    /// Ping the current (internal) UI state.
    State,
}

/// The channel type used to send "pings" to update the UI state in response
/// to events other than UI events.
pub type Ping = glib::Sender<PingDest>;

use state::State;

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
pub struct Content {
    hex: Hex,
    map: Map,
    games: Games,
}

/// Global UI actions, which are not specific to the current [State].
pub enum GlobalAction {
    /// Create a new instance of the `nth` game in the library.
    NewGame(usize),
    /// Load a game from the provided path.
    LoadGame(std::path::PathBuf),
    /// Save the current game to the provided path.
    SaveGame(std::path::PathBuf),
    /// Save a screenshot of the current game to the provided path.
    SaveScreenshot(std::path::PathBuf),
}

/// Defines the current state of the user interface.
pub struct UI {
    content: Content,
    state: Box<dyn State>,
    surface: Arc<RwLock<ImageSurface>>,
    context: Context,
    sender: Sender<GlobalAction>,
    receiver: Receiver<GlobalAction>,
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
    /// In response to changing the active game, resize the drawing area and
    /// redraw the surface.
    ResetGame,
    /// Increase the hex size.
    ZoomIn,
    /// Decrease the hex size.
    ZoomOut,
}

/// Describes a mouse button being clicked.
pub struct ButtonPress {
    /// The x coordinate of the click.
    pub x: f64,
    /// The y coordinate of the click.
    pub y: f64,
    /// The button that was clicked; `1` typically corresponds to the left
    /// button, `2` to the middle button, and `3` to the right button.
    pub button: u32,
}

impl From<&gdk::EventButton> for ButtonPress {
    fn from(event: &gdk::EventButton) -> Self {
        let (x, y) = event.position();
        let button = event.button();
        ButtonPress { x, y, button }
    }
}

/// Describes a keyboard key being pressed.
pub struct KeyPress {
    /// The key that was pressed.
    pub key: gdk::keys::Key,
    /// Whether the Control key was also pressed.
    pub ctrl: bool,
    /// Whether the Alt key was also pressed.
    pub alt: bool,
    /// Whether the Shift key was also pressed.
    pub shift: bool,
}

impl From<&gdk::EventKey> for KeyPress {
    fn from(event: &gdk::EventKey) -> Self {
        let key = event.keyval();
        let modifiers = event.state();
        let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
        let alt = modifiers.contains(gdk::ModifierType::MOD1_MASK);
        let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);

        KeyPress {
            key,
            ctrl,
            alt,
            shift,
        }
    }
}

/// Draws the state onto the provided context.
fn draw_state(state: &dyn State, content: &Content, ctx: &Context) {
    Colour::WHITE.apply_colour(ctx);
    ctx.reset_clip();
    let (x1, y1, x2, y2) = ctx.clip_extents().unwrap();
    ctx.rectangle(x1, y1, x2, y2);
    ctx.fill().unwrap();
    state.draw(content, ctx);
}

/// Returns the ink bounding box `(x0, y0, width, height)` for the provided
/// state.
fn ink_extents(state: &dyn State, content: &Content) -> (f64, f64, f64, f64) {
    let surf = cairo::RecordingSurface::create(cairo::Content::Color, None)
        .expect("Could not create RecordingSurface");

    let ctx =
        cairo::Context::new(&surf).expect("Could not create cairo::Context");
    state.draw(content, &ctx);
    // Note: (x0, y0, width, height)
    surf.ink_extents()
}

/// Returns the ink bounding box `(x0, y0, width, height)` for the provided
/// state, for the specified maximal hex diameter `hex_d`.
fn ink_extents_with_hex(
    state: &dyn State,
    content: &mut Content,
    hex_d: f64,
) -> (f64, f64, f64, f64) {
    let mut new_hex = Hex::new(hex_d);
    std::mem::swap(&mut new_hex, &mut content.hex);
    let exts = ink_extents(state, content);
    std::mem::swap(&mut new_hex, &mut content.hex);
    exts
}

/// Returns the surface dimensions required to draw the provided state.
fn current_surface_dims(state: &dyn State, content: &Content) -> (i32, i32) {
    let exts = ink_extents(state, content);
    let want_width = (exts.2 + 2.0 * exts.0) as i32;
    let want_height = (exts.3 + 2.0 * exts.1) as i32;
    (want_width, want_height)
}

/// Returns the surface dimensions required to draw the provided state at the
/// maximum zoom level.
fn max_surface_dims(state: &dyn State, content: &mut Content) -> (i32, i32) {
    // NOTE: this is the upper limit on the maximum hex size.
    let hex_d = 164.0;
    let exts = ink_extents_with_hex(state, content, hex_d);
    let want_width = (exts.2 + 2.0 * exts.0) as i32;
    let want_height = (exts.3 + 2.0 * exts.1) as i32;
    (want_width, want_height)
}

impl UI {
    /// Creates the initial user interface state.
    pub fn new<T>(hex: Hex, games: T) -> Self
    where
        T: IntoIterator<Item = Box<dyn Game>>,
    {
        let games = Games::new(games);
        let init_state = state::start::Start::new();
        let map = init_state.dummy_map();

        // Determine the surface dimensions necessary to contain the state
        // output at the highest zoom level.
        let mut content = Content { hex, map, games };
        let dims = max_surface_dims(&init_state, &mut content);

        // Create the backing surface and context.
        info!("Creating image surface ({}, {})", dims.0, dims.1,);
        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            dims.0,
            dims.1,
        )
        .expect("Could not create ImageSurface");
        let context =
            Context::new(&surface).expect("Could not create cairo::Context");
        // Paint the new surface white.
        n18brush::clear_surface(&context, Colour::WHITE);
        let surface = Arc::new(RwLock::new(surface));

        let (sender, receiver) = std::sync::mpsc::channel();

        // Create the UI state struct, and draw the initial state.
        let state: Box<dyn State> = Box::new(init_state);
        let ui_state = UI {
            content,
            state,
            surface,
            context,
            sender,
            receiver,
        };
        ui_state.draw();
        ui_state
    }

    /// Returns the image surface onto which the state is drawn, contained
    /// within a thread-safe reader-writer lock.
    ///
    /// Use this surface as the source for painting the current state:
    ///
    /// ```no_run
    /// # let hex = n18hex::Hex::new(125.0);
    /// # let ui_state = n18ui::UI::new(hex, vec![]);
    /// # let fmt = cairo::Format::ARgb32;
    /// # let s = cairo::ImageSurface::create(fmt, 10, 10).unwrap();
    /// # let context = cairo::Context::new(&s).unwrap();
    /// let surf_lock = ui_state.surface();
    /// let surface = surf_lock.read().expect("Cannot access surface");
    /// context.set_source_surface(&surface, 0.0, 0.0);
    /// context.paint();
    /// ```
    pub fn surface(&self) -> Arc<RwLock<ImageSurface>> {
        Arc::clone(&self.surface)
    }

    /// Returns the dimensions of the current game map, in pixels.
    pub fn map_size(&self) -> (i32, i32) {
        current_surface_dims(&*self.state, &self.content)
    }

    /// Draws the current state of the user interface.
    pub fn draw(&self) {
        draw_state(&*self.state, &self.content, &self.context);
    }

    /// Requests the drawing area to update its size, and redraws the current
    /// game state.
    ///
    /// This should be called when the user has zoomed in or zoomed out.
    fn zoom_and_redraw(&self, area: &gtk::DrawingArea) {
        let curr_exts = ink_extents(&*self.state, &self.content);
        let width = (curr_exts.2 + 2.0 * curr_exts.0) as i32;
        let height = (curr_exts.3 + 2.0 * curr_exts.1) as i32;
        area.set_size_request(width, height);
        // NOTE: must redraw to the backing surface.
        self.draw();
        area.queue_draw();
    }

    /// Resets the drawing surface, requests the drawing area to update its
    /// size, and redraws the current game state.
    ///
    /// This should only be called when the active game has changed (e.g.,
    /// when creating a new game or loading a saved game).
    fn reset_and_redraw(&mut self, area: &gtk::DrawingArea) {
        let want_dims = max_surface_dims(&*self.state, &mut self.content);
        let want_width = want_dims.0;
        let want_height = want_dims.1;

        // Resize the underlying image surface if it is too small.
        let (curr_width, curr_height) = {
            let curr_surface = self
                .surface
                .read()
                .expect("Could not access drawing surface");
            (curr_surface.width(), curr_surface.height())
        };
        let resize = (curr_width < want_width) || (curr_height < want_height);
        if resize {
            info!(
                "Resizing image surface from ({}, {}) to ({}, {})",
                curr_width, curr_height, want_width, want_height
            );
            let surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                want_width,
                want_height,
            )
            .expect("Could not create ImageSurface");
            self.context = Context::new(&surface)
                .expect("Could not create cairo::Context");
            let mut surf_ref = self
                .surface
                .write()
                .expect("Could not modify drawing surface");
            *surf_ref = surface;
            // Paint the new surface white.
            n18brush::clear_surface(&self.context, Colour::WHITE);
        }

        // Request the drawing area to update its size, and redraw the current
        // (i.e., new) game state.
        self.zoom_and_redraw(area);
    }

    pub fn handle_action(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        action: Action,
    ) {
        match action {
            Action::ZoomIn => {
                let new_max_d = self.content.hex.max_d + 10.0;
                if new_max_d < 164.0 {
                    // NOTE: may need to increase surface, draw area size?
                    self.content.hex.resize(new_max_d);
                    self.zoom_and_redraw(area);
                }
            }
            Action::ZoomOut => {
                let new_max_d = self.content.hex.max_d - 10.0;
                if new_max_d > 56.0 {
                    // NOTE: may need to decrease surface, draw area size?
                    self.content.hex.resize(new_max_d);
                    self.zoom_and_redraw(area);
                }
            }
            Action::Redraw => {
                // NOTE: must redraw to the backing surface.
                self.draw();
                area.queue_draw();
            }
            Action::ResetGame => {
                // NOTE: request this size request is only required when the
                // game map has been replaced (e.g., by starting a new game or
                // by loading a saved game).
                self.reset_and_redraw(area);
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
        event: &KeyPress,
        ping_tx: &Ping,
    ) -> Action {
        // Note: use &* because Box<T> implements Deref<Target = T>.
        // So &*curr_state converts from Box<dyn State> to &dyn State.
        let action = global_keymap(
            &*self.state,
            &mut self.content,
            window,
            area,
            event,
            ping_tx,
            &self.sender,
        );
        let (new_state_opt, action) =
            if let Some((reset_state, action)) = action {
                match reset_state {
                    ResetState::No => (None, action),
                    ResetState::Yes => {
                        let b: Box<dyn State> = Box::new(
                            state::default::Default::new(&self.content.map),
                        );
                        (Some(b), action)
                    }
                }
            } else {
                self.state.key_press(
                    &mut self.content,
                    window,
                    area,
                    event,
                    ping_tx,
                )
            };
        if let Some(new_state) = new_state_opt {
            self.state = new_state;
        }
        action
    }

    /// Responds to a mouse button being clicked.
    pub fn button_press_action(
        &mut self,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &ButtonPress,
        ping_tx: &Ping,
    ) -> Action {
        let (new_state_opt, action) = self.state.button_press(
            &mut self.content,
            window,
            area,
            event,
            ping_tx,
        );
        if let Some(new_state) = new_state_opt {
            self.state = new_state;
        }
        action
    }

    /// Responds to an event triggered by something other than a UI event
    /// (e.g., a message from a task running in a separate thread).
    pub fn ping(
        &mut self,
        dest: PingDest,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        ping_tx: &Ping,
    ) -> Action {
        match dest {
            PingDest::State => {
                let (new_state_opt, action) =
                    self.state.ping(&mut self.content, window, area, ping_tx);
                if let Some(new_state) = new_state_opt {
                    self.state = new_state;
                }
                action
            }
            PingDest::TopLevel => {
                let msg = self.receiver.recv().unwrap();
                match msg {
                    GlobalAction::NewGame(game_ix) => {
                        self.new_game(game_ix, window)
                    }
                    GlobalAction::LoadGame(path) => self.load_game(path),
                    GlobalAction::SaveGame(path) => self.save_game(path),
                    GlobalAction::SaveScreenshot(path) => {
                        self.save_screenshot(area, path)
                    }
                }
            }
        }
    }

    /// Resets the internal state to [state::default::Default].
    pub fn reset_to_default_state(&mut self) {
        let b: Box<dyn State> =
            Box::new(state::default::Default::new(&self.content.map));
        self.state = b;
    }

    /// Creates a new game, identified by index into the game library.
    pub fn new_game(
        &mut self,
        game_ix: usize,
        window: &gtk::ApplicationWindow,
    ) -> Action {
        if self.content.games.set_active_index(game_ix) {
            self.content.map =
                self.content.games.active().create_map(&self.content.hex);
            window.set_title(self.content.games.active().name());
            self.reset_to_default_state();
            Action::ResetGame
        } else {
            Action::None
        }
    }

    /// Loads a saved game state from `path`.
    pub fn load_game(&mut self, path: std::path::PathBuf) -> Action {
        let game_state = n18io::read_game_state(&path).unwrap_or_else(|_| {
            panic!("Could not read '{}'", path.display())
        });
        if self.content.games.set_active_name(&game_state.game) {
            if let Some(new_map) = self
                .content
                .games
                .active_mut()
                .load(&self.content.hex, game_state)
            {
                self.content.map = new_map;
                self.reset_to_default_state();
                Action::ResetGame
            } else {
                Action::None
            }
        } else {
            Action::None
        }
    }

    /// Saves the current game state to `path`.
    pub fn save_game(&mut self, path: std::path::PathBuf) -> Action {
        let game_state = self.content.games.active().save(&self.content.map);
        n18io::write_game_state(&path, game_state, true).unwrap_or_else(
            |_| panic!("Could not write '{}'", path.display()),
        );
        Action::None
    }

    /// Saves a screenshot of the current game state to `path`.
    pub fn save_screenshot(
        &self,
        area: &gtk::DrawingArea,
        path: std::path::PathBuf,
    ) -> Action {
        let width = area.allocated_width();
        let height = area.allocated_height();
        let surface =
            ImageSurface::create(cairo::Format::ARgb32, width, height)
                .expect("Can't create surface");
        let icx =
            Context::new(&surface).expect("Can't create cairo::Context");
        // Fill the image with a white background.
        n18brush::clear_surface(&icx, n18hex::Colour::WHITE);
        // Then draw the current map content.
        self.state.draw(&self.content, &icx);
        let mut file = std::fs::File::create(&path).unwrap_or_else(|_| {
            panic!("Couldn't create '{}'", path.display())
        });
        surface.write_to_png(&mut file).unwrap_or_else(|_| {
            panic!("Couldn't write '{}'", path.display())
        });
        Action::None
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
    _state: &dyn State,
    content: &mut Content,
    window: &gtk::ApplicationWindow,
    _area: &gtk::DrawingArea,
    event: &KeyPress,
    ping_tx: &Ping,
    self_tx: &Sender<GlobalAction>,
) -> Option<(ResetState, Action)> {
    match (&event.key, event.ctrl) {
        (&gdk::keys::constants::q, false)
        | (&gdk::keys::constants::Q, false) => {
            Some((ResetState::No, Action::Quit))
        }
        (&gdk::keys::constants::n, true)
        | (&gdk::keys::constants::N, true) => {
            // Prompt the user to select a game, and load its starting map.
            let game_names: Vec<&str> = content.games.names();
            let ping_tx = ping_tx.clone();
            let self_tx = self_tx.clone();
            dialog::select_index(
                window,
                "Select a game",
                &game_names,
                move |ix_opt| {
                    if let Some(ix) = ix_opt {
                        self_tx.send(GlobalAction::NewGame(ix)).unwrap();
                        ping_tx.send(PingDest::TopLevel).unwrap();
                    }
                },
            );
            Some((ResetState::No, Action::None))
        }
        (&gdk::keys::constants::o, true)
        | (&gdk::keys::constants::O, true) => {
            let filters = dialog::game_file_filters();
            let ping_tx = ping_tx.clone();
            let self_tx = self_tx.clone();
            dialog::select_file_load(
                window,
                "Load game",
                &filters,
                None,
                move |path_opt| {
                    if let Some(path) = path_opt {
                        self_tx.send(GlobalAction::LoadGame(path)).unwrap();
                        ping_tx.send(PingDest::TopLevel).unwrap();
                    }
                },
            );
            Some((ResetState::No, Action::None))
        }
        (&gdk::keys::constants::s, true)
        | (&gdk::keys::constants::S, true) => {
            let filters = dialog::game_file_filters();
            let ping_tx = ping_tx.clone();
            let self_tx = self_tx.clone();
            dialog::select_file_save(
                window,
                "Save game",
                &filters,
                None,
                move |path_opt| {
                    if let Some(path) = path_opt {
                        self_tx.send(GlobalAction::SaveGame(path)).unwrap();
                        ping_tx.send(PingDest::TopLevel).unwrap();
                    }
                },
            );
            Some((ResetState::No, Action::None))
        }
        (&gdk::keys::constants::s, false)
        | (&gdk::keys::constants::S, false) => {
            let filters = dialog::image_file_filters();
            let ping_tx = ping_tx.clone();
            let self_tx = self_tx.clone();
            // Suggest a filename that contains the current date and time.
            let now = chrono::Local::now();
            let default_dest =
                now.format("screenshot-%Y-%m-%d-%H%M%S.png").to_string();
            dialog::select_file_save(
                window,
                "Save screenshot",
                &filters,
                Some(&default_dest),
                move |path_opt| {
                    if let Some(path) = path_opt {
                        self_tx
                            .send(GlobalAction::SaveScreenshot(path))
                            .unwrap();
                        ping_tx.send(PingDest::TopLevel).unwrap();
                    }
                },
            );
            Some((ResetState::No, Action::None))
        }
        (&gdk::keys::constants::plus, false)
        | (&gdk::keys::constants::equal, false) => {
            Some((ResetState::No, Action::ZoomIn))
        }
        (&gdk::keys::constants::minus, false)
        | (&gdk::keys::constants::underscore, false) => {
            Some((ResetState::No, Action::ZoomOut))
        }
        _ => None,
    }
}
