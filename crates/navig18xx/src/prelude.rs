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
pub use n18hex::Hex;

#[doc(inline)]
pub use n18hex::HexColour;

#[doc(inline)]
pub use n18hex::HexCorner;

#[doc(inline)]
pub use n18hex::HexFace;

#[doc(inline)]
pub use n18hex::HexPosition;

#[doc(inline)]
pub use n18hex::Delta;

#[doc(inline)]
pub use n18hex::Direction;

#[doc(inline)]
pub use n18tile::City;

#[doc(inline)]
pub use n18tile::Rotation;

#[doc(inline)]
pub use n18tile::Track;

#[doc(inline)]
pub use n18tile::TrackCurve;

#[doc(inline)]
pub use n18tile::TrackEnd;

#[doc(inline)]
pub use n18tile::DitShape;

#[doc(inline)]
pub use n18tile::Label;

#[doc(inline)]
pub use n18tile::Tile;

#[doc(inline)]
pub use n18tile::Connection;

#[doc(inline)]
pub use n18token::Tokens;

#[doc(inline)]
pub use n18token::Token;

#[doc(inline)]
pub use n18token::TokenStyle;

#[doc(inline)]
pub use n18token::Colour;

#[doc(inline)]
pub use n18catalogue::tile_catalogue;

#[doc(inline)]
pub use n18io::read_tile;

#[doc(inline)]
pub use n18io::write_tile;

#[doc(inline)]
pub use n18io::read_tiles;

#[doc(inline)]
pub use n18io::write_tiles;

#[doc(inline)]
pub use n18io::read_routes;

#[doc(inline)]
pub use n18io::write_routes;

#[doc(inline)]
pub use n18map::Map;

#[doc(inline)]
pub use n18map::HexAddress;

#[doc(inline)]
pub use n18map::RotateCW;

#[doc(inline)]
pub use n18map::TokensTable;

#[doc(inline)]
pub use n18route::Path;

#[doc(inline)]
pub use n18route::Route;

#[doc(inline)]
pub use n18route::Routes;

#[doc(inline)]
pub use n18route::ConflictRule;

#[doc(inline)]
pub use n18route::Criteria;

#[doc(inline)]
pub use n18route::paths_for_token;

#[doc(inline)]
pub use n18route::Train;

#[doc(inline)]
pub use n18route::Trains;

#[doc(inline)]
pub use n18route::TrainRoute;

#[doc(inline)]
pub use n18game::Game;

#[cfg(feature = "ui")]
#[doc(inline)]
pub use n18ui::UI;

#[doc(inline)]
pub use n18example::Example;

#[doc(inline)]
pub use n18example::tile_at;
