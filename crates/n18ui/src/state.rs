use cairo::Context;
use n18hex::Colour;
use n18map::HexAddress;

use crate::{Assets, Controller, UiResponse};

pub mod default;
pub mod edit_tokens;
pub mod replace_tile;
pub mod search;
pub mod start;

/// The methods that are required in order to manipulate the user interface.
pub trait UiState {
    /// Draws the current state of the map.
    ///
    /// Note that this method should not draw a background by filling the
    /// entire surface, because this makes it impossible to determine the
    /// appropriate surface dimensions.
    fn draw(&self, assets: &Assets, ctx: &Context);

    fn ping(
        &mut self,
        _assets: &mut Assets,
        _controller: &mut Controller,
    ) -> (UiResponse, Option<State>) {
        (UiResponse::None, None)
    }
}

/// The different user interface states.
pub enum State {
    Start(start::Start),
    Default(default::Default),
    EditTokens(edit_tokens::EditTokens),
    ReplaceTile(replace_tile::ReplaceTile),
    FindRoutesCompany(search::SelectCompany),
    FindRoutesTrains(search::SelectTrains),
    FindRoutesSearch(search::Search),
    FindRoutesFound(search::Found),
}

/// Returns the default UI state, with the provided active map hex.
impl From<HexAddress> for State {
    fn from(active_hex: HexAddress) -> Self {
        State::default_state(active_hex)
    }
}

impl From<start::Start> for State {
    fn from(state: start::Start) -> Self {
        State::Start(state)
    }
}

impl From<default::Default> for State {
    fn from(state: default::Default) -> Self {
        State::Default(state)
    }
}

impl From<edit_tokens::EditTokens> for State {
    fn from(state: edit_tokens::EditTokens) -> Self {
        State::EditTokens(state)
    }
}

impl From<replace_tile::ReplaceTile> for State {
    fn from(state: replace_tile::ReplaceTile) -> Self {
        State::ReplaceTile(state)
    }
}

impl From<search::SelectCompany> for State {
    fn from(state: search::SelectCompany) -> Self {
        State::FindRoutesCompany(state)
    }
}

impl From<search::SelectTrains> for State {
    fn from(state: search::SelectTrains) -> Self {
        State::FindRoutesTrains(state)
    }
}

impl From<search::Search> for State {
    fn from(state: search::Search) -> Self {
        State::FindRoutesSearch(state)
    }
}

impl From<search::Found> for State {
    fn from(state: search::Found) -> Self {
        State::FindRoutesFound(state)
    }
}

impl State {
    pub fn default_state(active_hex: HexAddress) -> Self {
        let state = default::Default::at_hex(active_hex);
        state.into()
    }

    pub fn is_default_state(&self) -> bool {
        matches!(self, State::Default(_))
    }

    pub fn as_default(&self) -> Option<&default::Default> {
        match self {
            State::Default(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_start(&self) -> Option<&start::Start> {
        match self {
            State::Start(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_edit_tokens(&self) -> Option<&edit_tokens::EditTokens> {
        match self {
            State::EditTokens(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_replace_tile(&self) -> Option<&replace_tile::ReplaceTile> {
        match self {
            State::ReplaceTile(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_company(&self) -> Option<&search::SelectCompany> {
        match self {
            State::FindRoutesCompany(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_trains(&self) -> Option<&search::SelectTrains> {
        match self {
            State::FindRoutesTrains(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_search(&self) -> Option<&search::Search> {
        match self {
            State::FindRoutesSearch(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_found(&self) -> Option<&search::Found> {
        match self {
            State::FindRoutesFound(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_default_mut(&mut self) -> Option<&mut default::Default> {
        match self {
            State::Default(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_start_mut(&mut self) -> Option<&mut start::Start> {
        match self {
            State::Start(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_edit_tokens_mut(
        &mut self,
    ) -> Option<&mut edit_tokens::EditTokens> {
        match self {
            State::EditTokens(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_replace_tile_mut(
        &mut self,
    ) -> Option<&mut replace_tile::ReplaceTile> {
        match self {
            State::ReplaceTile(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_company_mut(
        &mut self,
    ) -> Option<&mut search::SelectCompany> {
        match self {
            State::FindRoutesCompany(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_trains_mut(
        &mut self,
    ) -> Option<&mut search::SelectTrains> {
        match self {
            State::FindRoutesTrains(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_search_mut(
        &mut self,
    ) -> Option<&mut search::Search> {
        match self {
            State::FindRoutesSearch(state) => Some(state),
            _ => None,
        }
    }

    pub fn as_find_routes_found_mut(&mut self) -> Option<&mut search::Found> {
        match self {
            State::FindRoutesFound(state) => Some(state),
            _ => None,
        }
    }

    pub fn draw(&self, assets: &Assets, context: &Context) {
        Colour::WHITE.apply_colour(context);
        context.reset_clip();
        let (x1, y1, x2, y2) = context.clip_extents().unwrap();
        context.rectangle(x1, y1, x2, y2);
        context.fill().unwrap();
        self.as_ref().draw(assets, context)
    }

    pub fn ping(
        &mut self,
        assets: &mut Assets,
        controller: &mut Controller,
    ) -> (UiResponse, Option<State>) {
        self.as_mut().ping(assets, controller)
    }
}

/// Note the `'static` lifetime in the return type, which is necessary for
/// this to compile.
/// See [this
/// thread](https://users.rust-lang.org/t/where-does-the-static-bound-come-from-in-this-error-implementing-asref/60065)
/// on the Rust forum, and [how static lifetimes apply to trait
/// bounds](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound):
///
/// > As a trait bound, it means the type does not contain any non-static
/// > references.
/// > E.g., the receiver can hold on to the type for as long as they want and
/// > it will never become invalid until they drop it.
impl AsRef<dyn UiState> for State {
    fn as_ref(&self) -> &(dyn UiState + 'static) {
        use State::*;
        match self {
            Start(state) => state,
            Default(state) => state,
            EditTokens(state) => state,
            ReplaceTile(state) => state,
            FindRoutesCompany(state) => state,
            FindRoutesTrains(state) => state,
            FindRoutesSearch(state) => state,
            FindRoutesFound(state) => state,
        }
    }
}

impl AsMut<dyn UiState> for State {
    fn as_mut(&mut self) -> &mut (dyn UiState + 'static) {
        use State::*;
        match self {
            Start(state) => state,
            Default(state) => state,
            EditTokens(state) => state,
            ReplaceTile(state) => state,
            FindRoutesCompany(state) => state,
            FindRoutesTrains(state) => state,
            FindRoutesSearch(state) => state,
            FindRoutesFound(state) => state,
        }
    }
}
