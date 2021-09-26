//! # 1830: Railways and Robber Barons
//!
//! Initial version of 1830 map and tiles.
//!

use super::Company;
use n18catalogue::Catalogue;
use n18hex::{self, Colour, Hex, HexFace, Orientation, RotateCW};
use n18map::{Coordinates, FirstRow, HexAddress, Letters, Map};
use n18route::{Bonus, ConflictRule, Train, TrainType};
use n18tile::Label;
use n18token::{Token, TokenStyle};

const ORIENTATION: Orientation = Orientation::PointedTop;

const COORDS: Coordinates = Coordinates {
    orientation: ORIENTATION,
    first_row: FirstRow::OddColumns,
    letters: Letters::AsColumns,
};

mod tiles;

#[doc(inline)]
pub use tiles::catalogue;

mod locns;

#[doc(inline)]
pub use locns::Location;

/// Defines the trains, tiles, and map for 1830: Railways and Robber Barons.
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
    barriers: Vec<(HexAddress, HexFace)>,
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
            ("7", TrainType::MustStop.with_max_stops(7)),
            ("D", TrainType::MustStop.with_unlimited_stops()),
        ];

        let companies: Vec<Company> = [
            (
                "PRR",
                "Pennsylvania",
                Colour::from((15, 127, 15)),
                Colour::from((63, 191, 63)),
                Colour::WHITE,
            ),
            (
                "NYC",
                "New York Central",
                Colour::from((191, 31, 31)),
                Colour::from((159, 159, 59)),
                Colour::WHITE,
            ),
            (
                "CPR",
                "Canadian Pacific",
                Colour::from((31, 31, 31)),
                Colour::from((223, 31, 31)),
                Colour::WHITE,
            ),
            (
                "B&O",
                "Baltimore & Ohio",
                Colour::from((0, 91, 127)),
                Colour::from((0, 127, 223)),
                Colour::WHITE,
            ),
            (
                "C&O",
                "Chesapeake & Ohio",
                Colour::from((0, 63, 127)),
                Colour::from((239, 239, 31)),
                Colour::WHITE,
            ),
            (
                "ERIE",
                "Erie",
                Colour::from((239, 239, 31)),
                Colour::from((0, 0, 0)),
                Colour::BLACK,
            ),
            (
                "NNH",
                "New York, New Haven, & Hartford",
                Colour::from((239, 159, 31)),
                Colour::from((91, 91, 91)),
                Colour::BLACK,
            ),
            (
                "B&M",
                "Boston & Maine",
                Colour::from((127, 31, 31)),
                Colour::from((239, 239, 31)),
                Colour::WHITE,
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

        // Define barriers for the two lakes.
        let parse = |text| COORDS.parse(text);
        let barriers = vec![
            // Lake Erie.
            (parse("G5").unwrap(), HexFace::LowerRight),
            // Lake Ontario.
            (parse("L4").unwrap(), HexFace::Top),
            (parse("L4").unwrap(), HexFace::UpperLeft),
        ];

        // NOTE:
        // - Green tiles available from phase 3;
        // - Brown tiles available from phase 5; and
        // - Grey tiles available from phase 6 (irrelevant for base 1830).
        // - Off-board revenues use the lower value for phases 2-4,
        //   and the higher value from phase 5 (i.e., Brown, Grey).
        let phase_names = vec!["2", "3", "4", "5", "6", "7"];
        let phase = 0;

        Game {
            companies,
            trains,
            catalogue,
            barriers,
            phase,
            phase_names,
        }
    }
}

impl super::Game for Game {
    fn name(&self) -> &str {
        "1830: Railways and Robber Barons"
    }

    /// The orientation of the map hexes.
    fn hex_orientation(&self) -> Orientation {
        COORDS.orientation
    }

    /// The coordinate system used to identify map hexes.
    fn coordinate_system(&self) -> Coordinates {
        COORDS
    }

    fn companies(&self) -> &[Company] {
        &self.companies
    }

    fn trains(&self) -> &[(&str, Train)] {
        &self.trains
    }

    /// This game does not include any route bonuses.
    fn bonus_options(&self) -> Vec<&'static str> {
        vec![]
    }

    /// This game does not include any route bonuses.
    fn bonuses(&self, _bonus_options: &[bool]) -> Vec<Bonus> {
        vec![]
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
            if let Some((tile_name, rotation)) = tile_opt {
                if !map.place_tile(addr, tile_name, rotation) {
                    eprintln!("Could not place {} at {}", tile_name, addr);
                }
            }
        }

        // Add City and CityKind labels to the map, so that appropriate tile
        // upgrades can be identified.

        // The "OO" locations are the two-city yellow tiles with no track.
        for locn in [
            Location::Detroit,
            Location::Toronto,
            Location::Buffalo,
            Location::Philadelphia,
        ] {
            let addr = locn.address();
            map.add_label_at(addr, Label::CityKind("OO".to_string()));
        }
        // Baltimore and Boston: "B".
        for locn in [Location::Baltimore, Location::Boston] {
            let addr = locn.address();
            map.add_label_at(addr, Label::City("B".to_string()));
        }
        // New York: "NY".
        for locn in [Location::NewYork] {
            let addr = locn.address();
            map.add_label_at(addr, Label::City("NY".to_string()));
        }

        // Add barriers for Lake Erie and Lake Ontario.
        for (addr, face) in &self.barriers {
            map.add_barrier(*addr, *face)
        }

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
            0 => "Yw",
            1 | 2 => "Gn",
            _ => "Bn",
        };
        let offboard_phase_locns = [
            Location::Chicago,
            Location::GulfOfMexicoS,
            Location::CanadianWestE,
            Location::DeepSouth,
            Location::MaritimeProvinces,
        ];
        for locn in offboard_phase_locns {
            let locn_addr = locn.address();
            let locn_name = locn.as_str();
            let tile_name = format!("{}_{}", locn_name, suffix);
            if !map.place_tile(locn_addr, &tile_name, RotateCW::Zero) {
                println!(
                    "Could not place tile {} at {}",
                    tile_name, locn_addr
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
        // Column A contains a single off-board location, Gulf of Mexico.
        ("A9", Some(("Gulf of Mexico 2", Zero))),
        // Column B contains Lansing, Chicago, and Gulf of Mexico.
        ("B4", Some(("Lansing", Zero))),
        ("B6", Some(("Chicago_Yw", Zero))),
        ("B8", None),
        ("B10", Some(("Gulf of Mexico_Yw", Zero))),
        // Column C contains empty hexes.
        ("C5", None),
        ("C7", None),
        ("C9", None),
        // Column D contains Toledo and Columbus.
        ("D4", Some(("Flint", Zero))),
        ("D6", Some(("Toledo", Zero))),
        ("D8", Some(("Columbus", Zero))),
        ("D10", None),
        // Column E contains Detroit/Windsor.
        ("E5", Some(("Detroit/Windsor", Zero))),
        ("E7", None),
        ("E9", None),
        // Column F contains Cleveland.
        ("F4", Some(("Cost_80", Zero))),
        ("F6", Some(("Cleveland", Zero))),
        ("F8", None),
        ("F10", None),
        // Column G contains London,
        ("G3", None),
        ("G5", Some(("London", Zero))),
        ("G7", Some(("Akron/Canton", Zero))),
        ("G9", None),
        // Column H contains no cities.
        ("H4", None),
        ("H6", None),
        ("H8", None),
        ("H10", None),
        // Column I contains the first tile of Canadian West.
        ("I1", Some(("Canadian West 2", Zero))),
        ("I3", None),
        ("I5", Some(("Grey1", Two))),
        ("I7", None),
        ("I9", None),
        // Column J contains Barre, Toronto, and Pittsburgh.
        ("J2", Some(("Barre", Zero))),
        ("J4", Some(("Hamilton/Toronto", Zero))),
        ("J6", Some(("Erie", Zero))),
        ("J8", Some(("Pittsburgh", Zero))),
        ("J10", Some(("Cost_120", Zero))),
        // Column K contains the second tile of Canadian West, and Buffalo.
        ("K1", Some(("Canadian West_Yw", Zero))),
        ("K3", None),
        ("K5", Some(("Buffalo/Dunkirk", Zero))),
        ("K7", None),
        ("K9", Some(("Cost_120", Zero))),
        // Column L contains Altoona.
        ("L2", None),
        ("L4", None),
        ("L6", None),
        ("L8", Some(("Altoona", Zero))),
        ("L10", Some(("Cost_120", Zero))),
        // Column M contains the off-board location Deep South.
        ("M3", None),
        ("M5", None),
        ("M7", Some(("Cost_120", Zero))),
        ("M9", None),
        ("M11", Some(("Deep South_Yw", Zero))),
        // Column N contains Rochester and Washington.
        ("N2", None),
        ("N4", Some(("Rochester", Zero))),
        ("N6", None),
        ("N8", None),
        ("N10", Some(("Washington", Zero))),
        // Column O contains Baltimore and Richmond.
        ("O3", Some(("Kingston", Zero))),
        ("O5", None),
        ("O7", Some(("Cost_120", Zero))),
        ("O9", Some(("Baltimore", Zero))),
        ("O11", Some(("Richmond", Zero))),
        // Column P contains Ottawa, Scranton, and Lancaster.
        ("P2", Some(("Ottawa", Zero))),
        ("P4", None),
        ("P6", Some(("Scranton", Zero))),
        ("P8", Some(("Lancaster", Zero))),
        // Column Q contains no major cities.
        ("Q1", Some(("Grey1", Five))),
        ("Q3", Some(("Cost_120", Zero))),
        ("Q5", Some(("Cost_120", Zero))),
        ("Q7", Some(("Allentown/Reading", Zero))),
        ("Q9", Some(("Cost_80", Zero))),
        // Column R contains Trenton.
        ("R2", Some(("Cost_80", Zero))),
        ("R4", None),
        ("R6", None),
        ("R8", Some(("Trenton/Philadelphia", Zero))),
        // Column S contains Montreal, Albany, and New York.
        ("S1", Some(("Montreal", Zero))),
        ("S3", Some(("Cost_80", Zero))),
        ("S5", Some(("Albany", Zero))),
        ("S7", Some(("New York", Zero))),
        ("S9", Some(("Atlantic City", Zero))),
        // Column T contains no major cities.
        ("T2", Some(("Burlington", Zero))),
        ("T4", None),
        ("T6", Some(("Hartfort/New Haven", Zero))),
        // Column U contains no major cities.
        ("U3", Some(("Cost_120", Zero))),
        ("U5", Some(("Cost_120", Zero))),
        // Column V contains Providence.
        ("V2", None),
        ("V4", Some(("Cost_120", Zero))),
        ("V6", Some(("Providence", Zero))),
        // Column W contains Boston.
        ("W3", None),
        ("W5", Some(("Boston", Zero))),
        // Column X contains the off-board location Maritime Provinces.
        ("X2", Some(("Maritime Provinces_Yw", Zero))),
        ("X4", Some(("Grey1", Zero))),
        ("X6", Some(("Fall River", Zero))),
    ]
    .into_iter()
    .map(|(addr, tile_opt)| (COORDS.parse(addr).unwrap(), tile_opt))
    .collect()
}
