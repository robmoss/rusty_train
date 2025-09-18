//! # 1889: History of Shikoku Railways (Shikoku 1889)
//!
//! Initial version of 1889 map and tiles.
//!

use super::{Company, DividendKind, DividendOptions, Rounding};
use n18catalogue::Catalogue;
use n18hex::{self, Colour, Hex, Orientation, RotateCW};
use n18map::{Coordinates, FirstRow, HexAddress, Letters, Map};
use n18route::{Bonus, ConflictRule, Train, TrainType};
use n18tile::Label;
use n18token::{Token, TokenStyle};

const ORIENTATION: Orientation = Orientation::FlatTop;

const COORDS: Coordinates = Coordinates {
    orientation: ORIENTATION,
    first_row: FirstRow::EvenColumns,
    letters: Letters::AsColumns,
};

mod tiles;

#[doc(inline)]
pub use tiles::catalogue;

mod locns;

#[doc(inline)]
pub use locns::Location;

/// Defines the trains, tiles, and map for 1889: History of Shikoku Railways
/// (Shikoku 1889).
///
/// - Each game starts in phase 2.
/// - Green tiles are available from phase 3.
/// - Brown tiles are available from phase 5.
/// - Off-board locations provide the lower revenue for phases 2-4, and the
///   higher revenue from phase 5.
pub struct Game {
    companies: Vec<Company>,
    trains: Vec<(&'static str, Train)>,
    catalogue: Catalogue,
    phase: usize,
    phase_names: Vec<&'static str>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        let trains = vec![
            ("2", TrainType::MustStop.with_max_stops(2)),
            ("3", TrainType::MustStop.with_max_stops(3)),
            ("4", TrainType::MustStop.with_max_stops(4)),
            ("5", TrainType::MustStop.with_max_stops(5)),
            ("6", TrainType::MustStop.with_max_stops(6)),
            ("D", TrainType::MustStop.with_unlimited_stops()),
        ];

        let companies: Vec<Company> = [
            (
                "AR",
                "Awa Railroad",
                Colour::from((15, 127, 15)),
                Colour::from((63, 191, 63)),
                Colour::WHITE,
            ),
            (
                "IR",
                "Iyo Railway",
                Colour::from((191, 31, 31)),
                Colour::from((159, 159, 59)),
                Colour::WHITE,
            ),
            (
                "SR",
                "Sanuki Railway",
                Colour::from((31, 31, 31)),
                Colour::from((223, 31, 31)),
                Colour::WHITE,
            ),
            (
                "KO",
                "Takamatsu & Kotohira Electric Railway",
                Colour::from((0, 91, 127)),
                Colour::from((0, 127, 223)),
                Colour::WHITE,
            ),
            (
                "TR",
                "Tosa Electric Railway",
                Colour::from((0, 63, 127)),
                Colour::from((239, 239, 31)),
                Colour::WHITE,
            ),
            (
                "KU",
                "Tosa Kuroshio Railway",
                Colour::from((239, 239, 31)),
                Colour::from((0, 0, 0)),
                Colour::BLACK,
            ),
            (
                "UR",
                "Uwajima Railway",
                Colour::from((239, 159, 31)),
                Colour::from((91, 91, 91)),
                Colour::BLACK,
            ),
        ]
        .iter()
        .map(|&(abbrev, full_name, bg, fg, text)| Company {
            abbrev: abbrev.to_string(),
            full_name: full_name.to_string(),
            token: Token::new(TokenStyle::TopArcs { bg, fg, text }),
        })
        .collect();

        // Create the tile catalogue.
        let catalogue = tiles::catalogue();

        // NOTE:
        // - Green tiles available from phase 3;
        // - Brown tiles available from phase 5; and
        // - Off-board revenues use the lower value for phases 2-4,
        //   and the higher value from phase 5.
        let phase_names = vec!["2", "3", "4", "5", "6", "D"];
        let phase = 0;

        Game {
            companies,
            trains,
            catalogue,
            phase,
            phase_names,
        }
    }
}

impl super::Game for Game {
    fn name(&self) -> &str {
        "1889: History of Shikoku Railways"
    }

    /// The orientation of the map hexes.
    fn hex_orientation(&self) -> Orientation {
        COORDS.orientation
    }

    /// The coordinate system used to identify map hexes.
    fn coordinate_system(&self) -> Coordinates {
        COORDS
    }

    /// Returns the companies in this game.
    fn companies(&self) -> &[Company] {
        &self.companies
    }

    /// Returns the options available to a company for distributing dividends
    /// to shareholders.
    fn dividend_options(&self, _abbrev: &str) -> Option<DividendOptions> {
        Some(DividendOptions {
            share_count: 10,
            dividend_options: vec![(DividendKind::Full, Rounding::Exact)],
        })
    }

    fn trains(&self) -> &[(&str, Train)] {
        &self.trains
    }

    /// This game does not include any route bonuses.
    fn bonus_options(&self) -> Vec<&'static str> {
        vec![]
    }

    /// This game includes route bonuses for diesel trains.
    fn bonuses(&self, _bonus_options: &[bool]) -> Vec<Bonus> {
        // The diesel route bonuses are not applicable until the Diesel phase
        // is reached.
        if self.phase < 5 {
            return vec![];
        }

        // Diesel trains receive a bonus for visiting off-board locations.
        let offboard_locns = vec![
            Location::Imabari,
            Location::SakaideAndOkoyama,
            Location::NarutoAndAwaji,
        ];
        let train = self.trains[5].1;
        let bonus = 40;
        offboard_locns
            .into_iter()
            .map(|locn| {
                let locn = locn.address();
                Bonus::VisitWithTrainBonus { locn, train, bonus }
            })
            .collect()
    }

    /// Defines the elements that cannot be shared in a single route.
    ///
    /// A single route cannot reuse any track segment, any revenue centre
    /// (city or dit), or multiple revenue centres on a single hex.
    fn single_route_conflicts(&self) -> ConflictRule {
        ConflictRule::TrackOrCityHex
    }

    /// Defines the elements that cannot be shared between routes.
    ///
    /// Routes cannot have any track segments in common.
    fn multiple_routes_conflicts(&self) -> ConflictRule {
        ConflictRule::TrackOnly
    }

    /// Create the initial map for 1867.
    fn create_map(&self, _hex: &Hex) -> Map {
        // Create the map assets.
        let tokens = self.create_tokens();
        let hexes_and_tiles = initial_map();
        let hexes: Vec<HexAddress> = hexes_and_tiles
            .iter()
            .map(|(addr, _)| addr)
            .copied()
            .collect();
        let mut map = n18map::Map::new(
            self.catalogue.clone(),
            tokens,
            hexes,
            self.hex_orientation(),
        );

        // Place the initial tiles.
        for (addr, tile_opt) in hexes_and_tiles.into_iter() {
            if let Some((tile_name, rotation)) = tile_opt
                && !map.place_tile(addr, tile_name, rotation) {
                    eprintln!("Could not place {tile_name} at {addr}");
                }
        }

        // Add City labels to the map, so that appropriate tile upgrades can
        // be identified.
        map.add_label_at(
            Location::Kotohira.address(),
            Label::City("K".to_string()),
        );
        map.add_label_at(
            Location::Kouchi.address(),
            Label::City("Ki".to_string()),
        );
        map.add_label_at(
            Location::Takamatsu.address(),
            Label::City("T".to_string()),
        );

        map
    }

    /// Returns all game tiles, including special tiles that players cannot
    /// place on the map.
    fn catalogue(&self) -> &Catalogue {
        &self.catalogue
    }

    /// Returns the index of the current game phase.
    fn phase_ix(&self) -> usize {
        self.phase
    }

    /// Changes the current game phase, which may update the map.
    fn set_phase_ix(&mut self, map: &mut Map, phase: usize) -> bool {
        if phase >= self.phase_names.len() {
            return false;
        }
        self.phase = phase;

        let suffix = match phase {
            0..=2 => "Yw",
            _ => "Bn",
        };

        let offboard_locns = vec![
            Location::Imabari,
            Location::SakaideAndOkoyama,
            Location::NarutoAndAwaji,
        ];

        for locn in offboard_locns {
            let locn_addr = locn.address();
            let locn_name = locn.as_str();
            let tile_name = format!("{locn_name}_{suffix}");
            if !map.place_tile(locn_addr, &tile_name, RotateCW::Zero) {
                println!(
                    "Could not place tile {tile_name} at {locn_addr}"
                )
            }
        }

        true
    }

    /// Return the name of each game phase.
    fn phase_names(&self) -> &[&str] {
        &self.phase_names
    }
}

/// Returns the address of each map hex, and the tile that should be placed
/// there (if any) to create new game map.
pub fn initial_map() -> Vec<(HexAddress, Option<(&'static str, RotateCW)>)> {
    use RotateCW::*;
    vec![
        ("A8", Some(("Mountain", Zero))),
        ("A10", Some(("Sukumo", Zero))),
        ("B3", Some(("Yawatahama", Zero))),
        ("B5", None),
        ("B7", Some(("Uwajima", Zero))),
        ("B9", Some(("Mountain", Zero))),
        ("B11", Some(("Nakamura", Zero))),
        ("C4", Some(("Ohzu", Zero))),
        ("C6", Some(("Mountain", Zero))),
        ("C8", None),
        ("C10", Some(("Kubokawa", Zero))),
        ("D3", None),
        ("D5", Some(("Mountain", Zero))),
        ("D7", Some(("Mountain", Zero))),
        ("D9", None),
        ("E2", Some(("Matsuyama", Zero))),
        ("E4", Some(("Mountain", Zero))),
        ("E6", Some(("Mountain", Zero))),
        ("E8", None),
        ("F1", Some(("Imabari_Yw", Zero))),
        ("F3", Some(("Saijou", Zero))),
        ("F5", Some(("Mountain", Zero))),
        ("F7", Some(("Mountain", Zero))),
        ("F9", Some(("Kouchi", Zero))),
        ("G4", Some(("Niihama", Zero))),
        ("G6", Some(("Mountain", Zero))),
        ("G8", Some(("Mountain", Zero))),
        ("G10", Some(("Nangoku", Zero))),
        ("G12", Some(("Nahari", Zero))),
        ("G14", Some(("Muroto", Zero))),
        ("H3", None),
        ("H5", Some(("River", Zero))),
        ("H7", Some(("Ikeda", Zero))),
        ("H9", Some(("Mountain", Zero))),
        ("H11", Some(("Mountain", Zero))),
        ("H13", Some(("Mountain", Zero))),
        ("I2", Some(("Marugame", Zero))),
        ("I4", Some(("Kotohira", Zero))),
        ("I6", Some(("River", Zero))),
        ("I8", None),
        ("I10", None),
        ("I12", Some(("Muki", Zero))),
        ("J1", Some(("Sakaide & Okoyama_Yw", Zero))),
        ("J3", None),
        ("J5", Some(("Ritsurin Kouen", Zero))),
        ("J7", Some(("Grey_Gentle", Zero))),
        ("J9", Some(("Komatsujima", Zero))),
        ("J11", Some(("Anan", Zero))),
        ("K4", Some(("Takamatsu", Zero))),
        ("K6", Some(("River", Zero))),
        ("K8", Some(("Tokushima", Zero))),
        ("L7", Some(("Naruto & Awaji_Yw", Zero))),
    ]
    .into_iter()
    .map(|(addr, tile_opt)| (COORDS.parse(addr).unwrap(), tile_opt))
    .collect()
}
