pub use std::f64::consts::PI;

use std::f64::consts::FRAC_PI_4;
use std::f64::consts::FRAC_PI_6;

/// π/6
pub const PI_1_6: f64 = FRAC_PI_6;

/// 2π/6
pub const PI_2_6: f64 = 2.0 * FRAC_PI_6;

/// 3π/6 = 2π/4 = π/2
pub const PI_3_6: f64 = 3.0 * FRAC_PI_6;

/// 4π/6
pub const PI_4_6: f64 = 4.0 * FRAC_PI_6;

/// 5π/6
pub const PI_5_6: f64 = 5.0 * FRAC_PI_6;

/// π/4
pub const PI_1_4: f64 = FRAC_PI_4;

/// 3π/4
pub const PI_3_4: f64 = 3.0 * FRAC_PI_4;

#[doc(inline)]
pub use rusty_hex::Hex;

#[doc(inline)]
pub use rusty_hex::HexColour;

#[doc(inline)]
pub use rusty_hex::HexCorner;

#[doc(inline)]
pub use rusty_hex::HexFace;

#[doc(inline)]
pub use rusty_hex::HexPosition;

#[doc(inline)]
pub use rusty_hex::Delta;

#[doc(inline)]
pub use rusty_hex::Direction;

#[doc(inline)]
pub use rusty_tile::City;

#[doc(inline)]
pub use rusty_tile::Rotation;

#[doc(inline)]
pub use rusty_tile::Track;

#[doc(inline)]
pub use rusty_tile::TrackCurve;

#[doc(inline)]
pub use rusty_tile::TrackEnd;

#[doc(inline)]
pub use rusty_tile::DitShape;

#[doc(inline)]
pub use rusty_tile::Label;

#[doc(inline)]
pub use rusty_tile::Tile;

#[doc(inline)]
pub use rusty_tile::Connection;

#[doc(inline)]
pub use rusty_token::Tokens;

#[doc(inline)]
pub use rusty_token::Token;

#[doc(inline)]
pub use rusty_token::TokenStyle;

#[doc(inline)]
pub use rusty_token::Colour;

#[doc(inline)]
pub use rusty_catalogue::tile_catalogue;

#[doc(inline)]
pub use rusty_io::read_tile;

#[doc(inline)]
pub use rusty_io::write_tile;

#[doc(inline)]
pub use rusty_io::read_tiles;

#[doc(inline)]
pub use rusty_io::write_tiles;

#[doc(inline)]
pub use rusty_map::Map;

#[doc(inline)]
pub use rusty_map::HexAddress;

#[doc(inline)]
pub use rusty_map::RotateCW;

#[doc(inline)]
pub use rusty_map::TokensTable;

#[doc(inline)]
pub use rusty_route::Path;

#[doc(inline)]
pub use rusty_route::conflict::ConflictRule;

#[doc(inline)]
pub use rusty_route::search::Criteria;

#[doc(inline)]
pub use rusty_route::search::paths_for_token;

#[doc(inline)]
pub use rusty_route::train::Train;

#[doc(inline)]
pub use rusty_route::train::Trains;

#[doc(inline)]
pub use rusty_route::train::Pairing;

#[doc(inline)]
pub use rusty_game::Game;

#[doc(inline)]
pub use rusty_ui::UI;
