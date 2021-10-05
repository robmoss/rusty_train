use crate::state::edit_tokens::EditTokens;
use crate::state::replace_tile::ReplaceTile;
use crate::state::search::SelectCompany;
use crate::{
    Action, Assets, Canvas, Controller, PingDest, Sender, State, UiAction,
    UiController, UiResponse,
};

/// Type alias for key identifiers.
pub type Key = gdk::keys::Key;

/// Alias for key identifier constants.
use gdk::keys::constants as key;

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
    pub key: Key,
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

pub trait Submap {
    fn name(&self) -> &str;

    fn handle_key_press(
        &self,
        _assets: &mut Assets,
        _controller: &mut Controller,
        _state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        _event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        None
    }

    fn handle_button_press(
        &self,
        _assets: &mut Assets,
        _controller: &mut Controller,
        _state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        _event: &ButtonPress,
    ) -> Option<(UiResponse, Option<State>)> {
        None
    }
}

pub struct Keymap {
    submaps: Vec<Box<dyn Submap>>,
}

impl Default for Keymap {
    fn default() -> Self {
        let mut submaps = vec![];
        let global: Box<dyn Submap> = Box::new(Global {});
        submaps.push(global);
        submaps.push(Box::new(DefaultMode {}));
        submaps.push(Box::new(ReplaceTileMode {}));
        submaps.push(Box::new(EditTokensMode {}));
        submaps.push(Box::new(FoundRoutesMode {}));
        Keymap { submaps }
    }
}

impl Keymap {
    pub fn empty() -> Self {
        Keymap { submaps: vec![] }
    }

    pub fn add_submap(&mut self, submap: Box<dyn Submap>) {
        self.submaps.push(submap);
    }

    pub fn handle_key_press(
        &self,
        assets: &mut Assets,
        controller: &mut Controller,
        state: &mut State,
        canvas: &mut Canvas,
        sender: &Sender<UiAction>,
        event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        self.submaps.iter().find_map(|submap| {
            submap.handle_key_press(
                assets, controller, state, canvas, sender, event,
            )
        })
    }

    pub fn handle_button_press(
        &self,
        assets: &mut Assets,
        controller: &mut Controller,
        state: &mut State,
        canvas: &mut Canvas,
        sender: &Sender<UiAction>,
        event: &ButtonPress,
    ) -> Option<(UiResponse, Option<State>)> {
        self.submaps.iter().find_map(|submap| {
            submap.handle_button_press(
                assets, controller, state, canvas, sender, event,
            )
        })
    }
}

/// The keymap for the default UI mode.
pub struct DefaultMode {}

impl Submap for DefaultMode {
    fn name(&self) -> &str {
        "Default mode"
    }

    fn handle_key_press(
        &self,
        assets: &mut Assets,
        controller: &mut Controller,
        state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        state.as_default_mut().and_then(|state| {
            match (&event.key, event.ctrl) {
                (&key::e, false) | (&key::E, false) => {
                    ReplaceTile::with_any(&assets.map, state.active_hex())
                        .map(|new_state| {
                            (UiResponse::Redraw, Some(new_state.into()))
                        })
                        .or(Some((UiResponse::None, None)))
                }
                (&key::p, false) | (&key::P, false) => {
                    state.select_phase(assets, controller);
                    Some((UiResponse::None, None))
                }
                (&key::r, false) | (&key::R, false) => {
                    // Allow the user to select a company and trains, and find the
                    // routes that earn the most revenue.
                    SelectCompany::new(assets, controller, state.active_hex())
                        .map(|new_state| {
                            (UiResponse::Redraw, Some(new_state.into()))
                        })
                        .or(Some((UiResponse::None, None)))
                }
                (&key::t, false) | (&key::T, false) => {
                    EditTokens::try_new(&assets.map, state.active_hex())
                        .map(|new_state| {
                            (UiResponse::Redraw, Some(new_state.into()))
                        })
                        .or(Some((UiResponse::None, None)))
                }
                (&key::u, false) | (&key::U, false) => {
                    // Upgrade tile or place tile on empty hex.
                    ReplaceTile::maybe_upgrade(assets, state.active_hex())
                        .map(|new_state| {
                            (UiResponse::Redraw, Some(new_state.into()))
                        })
                        .or(Some((UiResponse::None, None)))
                }
                (&key::Left, false) => {
                    let new_addr = assets.map.prev_col(state.active_hex());
                    if new_addr == state.active_hex() {
                        Some((UiResponse::None, None))
                    } else {
                        state.set_active_hex(new_addr);
                        Some((UiResponse::Redraw, None))
                    }
                }
                (&key::Right, false) => {
                    let new_addr = assets.map.next_col(state.active_hex());
                    if new_addr == state.active_hex() {
                        Some((UiResponse::None, None))
                    } else {
                        state.set_active_hex(new_addr);
                        Some((UiResponse::Redraw, None))
                    }
                }
                (&key::Up, false) => {
                    let new_addr = assets.map.prev_row(state.active_hex());
                    if new_addr == state.active_hex() {
                        Some((UiResponse::None, None))
                    } else {
                        state.set_active_hex(new_addr);
                        Some((UiResponse::Redraw, None))
                    }
                }
                (&key::Down, false) => {
                    let new_addr = assets.map.next_row(state.active_hex());
                    if new_addr == state.active_hex() {
                        Some((UiResponse::None, None))
                    } else {
                        state.set_active_hex(new_addr);
                        Some((UiResponse::Redraw, None))
                    }
                }
                (&key::less, false) | (&key::comma, false) => {
                    // NOTE: unlike upgrading a tile, when rotating the current
                    // tile we should not try moving the currently-placed tokens
                    // to maintain their connectivity.
                    // This cannot be done without either:
                    // (a) losing information; or
                    // (b) recording additional state information.
                    // For example, in the event that the tokens cannot be
                    // successfully placed, they would be removed unless we
                    // separately recorded the "original" token configuration for
                    // the current tile.
                    // This additional state information should then presumably be
                    // discarded once the user chooses *any* action except further
                    // rotations of the current tile.
                    if let Some(hs) =
                        assets.map.hex_state_mut(state.active_hex())
                    {
                        hs.rotate_anti_cw()
                    }
                    Some((UiResponse::Redraw, None))
                }
                (&key::greater, false) | (&key::period, false) => {
                    if let Some(hs) =
                        assets.map.hex_state_mut(state.active_hex())
                    {
                        hs.rotate_cw()
                    }
                    Some((UiResponse::Redraw, None))
                }
                (&key::BackSpace, false) | (&key::Delete, false) => {
                    // TODO: allow this action to be undone?
                    assets.map.remove_tile(state.active_hex());
                    Some((UiResponse::Redraw, None))
                }
                _ => None,
            }
        })
    }

    fn handle_button_press(
        &self,
        assets: &mut Assets,
        _controller: &mut Controller,
        state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        event: &ButtonPress,
    ) -> Option<(UiResponse, Option<State>)> {
        // Allow the user to select hexes with a single click of any button.
        state.as_default_mut().map(|state| {
            let hex = &assets.hex;
            let map = &mut assets.map;
            let ctx = hex.context();
            let addr = map.hex_address_iter().find(|addr| {
                let m = map.prepare_to_draw(**addr, hex, ctx);
                hex.define_boundary(ctx);
                ctx.set_matrix(m);
                ctx.in_fill(event.x, event.y).unwrap()
            });
            if let Some(a) = addr {
                state.set_active_hex(*a);
                (UiResponse::Redraw, None)
            } else {
                (UiResponse::None, None)
            }
        })
    }
}

/// The keymap for the found routes UI mode.
pub struct FoundRoutesMode {}

impl Submap for FoundRoutesMode {
    fn name(&self) -> &str {
        "Find routes mode"
    }

    fn handle_key_press(
        &self,
        assets: &mut Assets,
        controller: &mut Controller,
        state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        state.as_find_routes_found_mut().and_then(|state| {
            match (&event.key, event.ctrl) {
                (&key::Escape, false) | (&key::Return, false) => {
                    // Exit this mode.
                    let new_state = State::default_state(state.active_hex());
                    Some((UiResponse::Redraw, Some(new_state)))
                }
                (&key::Left, _) | (&key::Up, _) => {
                    let action = if state.highlight_previous_route() {
                        controller
                            .set_window_title(&state.window_title(assets));
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                (&key::Right, _) | (&key::Down, _) => {
                    let action = if state.highlight_next_route() {
                        controller
                            .set_window_title(&state.window_title(assets));
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                (&key::d, _) | (&key::D, _) => {
                    let action = if state.show_dividends(assets, controller) {
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                _ => None,
            }
        })
    }
}

/// The keymap for the edit tokens UI mode.
pub struct EditTokensMode {}

impl Submap for EditTokensMode {
    fn name(&self) -> &str {
        "Edit tokens mode"
    }

    fn handle_key_press(
        &self,
        assets: &mut Assets,
        _controller: &mut Controller,
        state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        state.as_edit_tokens_mut().and_then(|state| {
            match (&event.key, event.ctrl) {
                (&key::Escape, false) => {
                    // Exit this mode, discarding any changes.
                    state.restore_tokens(&mut assets.map);
                    let new_state = State::default_state(state.active_hex());
                    Some((UiResponse::Redraw, Some(new_state)))
                }
                (&key::Return, false) => {
                    // Exit this mode, retaining any changes.
                    let new_state = State::default_state(state.active_hex());
                    Some((UiResponse::Redraw, Some(new_state)))
                }
                (&key::Left, false) => {
                    state.previous_token_space();
                    Some((UiResponse::Redraw, None))
                }
                (&key::Right, false) => {
                    state.next_token_space();
                    Some((UiResponse::Redraw, None))
                }
                (&key::Down, false) => {
                    state.select_previous_token(assets);
                    Some((UiResponse::Redraw, None))
                }
                (&key::Up, false) => {
                    state.select_next_token(assets);
                    Some((UiResponse::Redraw, None))
                }
                (&key::_0, false)
                | (&key::KP_0, false)
                | (&key::BackSpace, false)
                | (&key::Delete, false) => {
                    // Remove the current token
                    state.clear_token_space(&mut assets.map);
                    Some((UiResponse::Redraw, None))
                }
                _ => None,
            }
        })
    }
}

/// The keymap for the replace tile UI mode.
pub struct ReplaceTileMode {}

impl Submap for ReplaceTileMode {
    fn name(&self) -> &str {
        "Replace tile mode"
    }

    fn handle_key_press(
        &self,
        assets: &mut Assets,
        _controller: &mut Controller,
        state: &mut State,
        _canvas: &mut Canvas,
        _sender: &Sender<UiAction>,
        event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        state.as_replace_tile_mut().and_then(|state| {
            match (&event.key, event.ctrl) {
                (&key::Escape, false) => {
                    // Exit this mode, discarding any changes.
                    let new_state = State::default_state(state.active_hex());
                    Some((UiResponse::Redraw, Some(new_state)))
                }
                (&key::Return, false) => {
                    // Exit this mode, retaining any changes.
                    let action = if state.place_candidate(&mut assets.map) {
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    let new_state = State::default_state(state.active_hex());
                    Some((action, Some(new_state)))
                }
                (&key::o, false) | (&key::O, false) => {
                    state.toggle_original_tile();
                    Some((UiResponse::Redraw, None))
                }
                (&key::Down, false) => {
                    let action = if state.select_previous_candidate() {
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                (&key::Up, false) => {
                    let action = if state.select_next_candidate() {
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                (&key::less, false) | (&key::comma, false) => {
                    let action = if state.rotate_candidate_anti_cw() {
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                (&key::greater, false) | (&key::period, false) => {
                    let action = if state.rotate_candidate_cw() {
                        UiResponse::Redraw
                    } else {
                        UiResponse::None
                    };
                    Some((action, None))
                }
                _ => None,
            }
        })
    }
}

/// The global keymap defines key bindings that apply regardless of the
/// current UI state.
///
/// - `q`, `Q`: quit;
/// - `s`, `S`: save a screenshot of the current map;
/// - `Ctrl+n`, `Ctrl+N`: load the starting map.
/// - `Ctrl+o`, `Ctrl+O`: load a map from disk.
/// - `Ctrl+s`, `Ctrl+S`: save the current map to disk.
pub struct Global {}

impl Submap for Global {
    fn name(&self) -> &str {
        "Global"
    }

    fn handle_key_press(
        &self,
        assets: &mut Assets,
        controller: &mut Controller,
        state: &mut State,
        canvas: &mut Canvas,
        sender: &Sender<UiAction>,
        event: &KeyPress,
    ) -> Option<(UiResponse, Option<State>)> {
        let is_start = state.as_start().is_some();
        match (&event.key, event.ctrl) {
            (&key::q, false) | (&key::Q, false) => {
                Some((UiResponse::Quit, None))
            }
            (&key::n, true) | (&key::N, true) => {
                // Prompt the user to select a game, and load its starting map.
                let game_names: Vec<&str> = assets.games.names();
                let ping_tx = controller.ping_tx();
                let send_tx = sender.clone();
                controller.select_index(
                    "Select a game",
                    &game_names,
                    move |ix_opt| {
                        if let Some(ix) = ix_opt {
                            send_tx.send(Action::NewGame(ix).into()).unwrap();
                            ping_tx.send_ping(PingDest::TopLevel).unwrap();
                        }
                    },
                );
                Some((UiResponse::None, None))
            }
            (&key::o, true) | (&key::O, true) => {
                let ping_tx = controller.ping_tx();
                let send_tx = sender.clone();
                controller.select_game_load(
                    "Load game",
                    None,
                    move |path_opt| {
                        if let Some(path) = path_opt {
                            send_tx
                                .send(Action::LoadGame(path).into())
                                .unwrap();
                            ping_tx.send_ping(PingDest::TopLevel).unwrap();
                        }
                    },
                );
                Some((UiResponse::None, None))
            }
            (&key::s, true) | (&key::S, true) => {
                if is_start {
                    return None;
                }
                let ping_tx = controller.ping_tx();
                let send_tx = sender.clone();
                controller.select_game_save(
                    "Save game",
                    None,
                    move |path_opt| {
                        if let Some(path) = path_opt {
                            send_tx
                                .send(Action::SaveGame(path).into())
                                .unwrap();
                            ping_tx.send_ping(PingDest::TopLevel).unwrap();
                        }
                    },
                );
                Some((UiResponse::None, None))
            }
            (&key::s, false) | (&key::S, false) => {
                if is_start {
                    return None;
                }
                let ping_tx = controller.ping_tx();
                let send_tx = sender.clone();
                // NOTE: save the current image contents, so that subsequent
                // updates (e.g., finding optimal routes) do not affect the
                // saved image.
                // We need to create a new surface and paint from the source
                // surface.
                let image = canvas.copy_ink(state, assets);

                // Suggest a filename that contains the current date and time.
                let now = chrono::Local::now();
                let default_dest =
                    now.format("screenshot-%Y-%m-%d-%H%M%S.png").to_string();
                controller.select_screenshot_save(
                    "Save screenshot",
                    Some(&default_dest),
                    move |path_opt| {
                        if let Some(path) = path_opt {
                            // NOTE: need to clone `ss_surf`, because this is
                            // a `Fn` closure, not a `FnOnce` closure.
                            send_tx
                                .send(
                                    Action::SaveImage(path, image.clone())
                                        .into(),
                                )
                                .unwrap();
                            ping_tx.send_ping(PingDest::TopLevel).unwrap();
                        }
                    },
                );
                Some((UiResponse::None, None))
            }
            (&key::plus, false) | (&key::equal, false) => {
                if is_start {
                    return None;
                }
                Some((UiResponse::ZoomIn, None))
            }
            (&key::minus, false) | (&key::underscore, false) => {
                if is_start {
                    return None;
                }
                Some((UiResponse::ZoomOut, None))
            }
            _ => None,
        }
    }
}
