//! # 1861: The Railways Of The Russian Empire
//!
//! Initial version of 1861 map and tiles.
//!

use std::collections::BTreeMap;

use super::Company;
use n18catalogue::{Builder, Catalogue, Kind};
use n18hex::{
    Colour, Hex, HexColour, HexFace, HexPosition, Orientation, RotateCW,
};
use n18map::{HexAddress, Map};
use n18route::{Bonus, ConflictRule, Train, TrainType};
use n18tile::{Label, Tile};
use n18token::{Token, TokenStyle};

/// Defines the trains, tiles, and map for 1861: The Railways Of The Russian Empire.
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
            ("2", TrainType::SkipTowns.with_max_stops(2)),
            ("3", TrainType::SkipTowns.with_max_stops(3)),
            ("4", TrainType::SkipTowns.with_max_stops(4)),
            ("5", TrainType::SkipTowns.with_max_stops(5)),
            ("6", TrainType::SkipTowns.with_max_stops(6)),
            ("7", TrainType::SkipTowns.with_max_stops(7)),
            ("8", TrainType::SkipTowns.with_max_stops(8)),
            (
                "2+2",
                TrainType::SkipTowns.with_max_stops(2).with_multiplier(2),
            ),
            (
                "5+5E",
                TrainType::SkipAny.with_max_stops(5).with_multiplier(2),
            ),
        ];
        let catalogue = make_catalogue();
        // Define 24 tokens for the 16 minor and 8 major companies, as per the
        // `draw_tokens` example.
        // - Distinguish between minor and major companies with yellow and
        //   green background colours.
        // - Combine four different token styles with four different secondary
        //   colours to make unique tokens for each company.
        let company_names: Vec<(&str, &str)> = vec![
            // Yellow phase minors.
            ("KB", "Kiev-Brest"),
            ("KK", "Kiev-Kursk"),
            ("KR", "Kharkov-Rostov"),
            ("MK", "Moscow-Kursk"),
            ("MNN", "Moscow-Nizhnii Novgorod"),
            ("MV", "Moscow-Voronezh"),
            ("N", "Nikolaev"),
            ("OK", "Odessa-Kiev"),
            ("RO", "Riga-Orel"),
            ("SPW", "St Petersburg-Warsaw"),
            // Green phase minors.
            ("D", "Donetsk"),
            ("E", "Ekaterinin"),
            ("MB", "Moscow-Brest"),
            ("SV", "Samara-Vyazma"),
            ("TR", "Tsaritsyn-Riga"),
            ("V", "Vladikavkaz"),
            // Major.
            ("GRR", "Grand Russian Railway"),
            ("MK", "Moscow & Kazan Railway"),
            ("MKN", "Moscow, Kursk & Nizhnii Novgorod"),
            ("MKV", "Moscow, Kiev & Voronezh"),
            ("MVR", "Moscow, Vindava & Rybinsk Railway"),
            ("NW", "Northwestern Railway"),
            ("SE", "Southeastern Railway"),
            ("SW", "Southwestern Railway"),
        ];

        // Background colours for minor (yellow) and major (green) companies.
        let bg_yellow = Colour::from((223, 223, 0));
        let bg_green = Colour::from((0, 153, 63));
        let bg_iter = std::iter::repeat(bg_yellow)
            .take(16)
            .chain(std::iter::repeat(bg_green).take(8));

        let fg_colours = vec![
            Colour::from((0, 204, 204)), // Aqua
            Colour::from((0, 63, 204)),  // Blue
            Colour::from((223, 0, 0)),   // Red
            Colour::from((127, 0, 223)), // Purple
        ];
        let fg_count = fg_colours.len();
        let fg_iter = fg_colours.into_iter().cycle();

        let mut companies: Vec<Company> = bg_iter
            .zip(fg_iter)
            .enumerate()
            .map(|(ix, (bg, fg))| {
                // Use black text on yellow, and white text on green.
                let text = if bg == bg_yellow {
                    Colour::from((0, 0, 0))
                } else {
                    Colour::from((255, 255, 255))
                };
                let style = match ix / fg_count {
                    0 => TokenStyle::TopLines { bg, fg, text },
                    1 => TokenStyle::TopTriangles { bg, fg, text },
                    2 => TokenStyle::TopArcs { bg, fg, text },
                    3 => TokenStyle::TripleTriangles { bg, fg, text },
                    4 => TokenStyle::TopLines { bg, fg, text },
                    _ => TokenStyle::TopTriangles { bg, fg, text },
                };
                Company {
                    abbrev: company_names[ix].0.to_string(),
                    full_name: company_names[ix].1.to_string(),
                    token: Token::new(style),
                }
            })
            .collect();

        // Add the Russian State Railway as an extra company.
        // This uses the white-blue-red flag, which was the Russian Empire's
        // merchant flag from 1705 and the Russian national flag from 1883.
        companies.push(Company {
            abbrev: "RNR".to_string(),
            full_name: "Russian National Railway".to_string(),
            token: Token::new(TokenStyle::TricolourH {
                top: Colour::from((255, 255, 255)),
                middle: Colour::from((0, 57, 166)),
                bottom: Colour::from((213, 43, 30)),
                text: Colour::WHITE,
            }),
        });

        let barriers = vec![];
        let phase = 0;
        let phase_names = vec!["2", "3", "4", "5", "6", "7", "8"];
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
    /// The name of this game.
    fn name(&self) -> &str {
        "1861: The Railways of the Russian Empire"
    }

    /// The orientation of the map hexes.
    fn hex_orientation(&self) -> Orientation {
        Orientation::FlatTop
    }

    /// Returns the companies in this game.
    fn companies(&self) -> &[Company] {
        &self.companies
    }

    /// Returns the named train types in this game, in the order that they
    /// become available (where applicable).
    fn trains(&self) -> &[(&str, Train)] {
        &self.trains
    }

    /// Optional route bonuses that a company may hold.
    fn bonus_options(&self) -> Vec<&'static str> {
        vec![
            // $10 bonus for Odessa.
            "Black Sea Shipping Company",
            // $10 bonus for Moscow.
            "Moscow-Yaroslavl Railway",
            // $10 bonus for Moscow.
            "Moscow-Ryazan Railway",
            // $10 bonus for Poland.
            "Warsaw-Vienna Railway",
        ]
    }

    fn bonuses(&self, bonus_options: &[bool]) -> Vec<Bonus> {
        let ekaterinburg = Bonus::ConnectionBonus {
            from: (2, 16).into(),
            to_any: vec![
                (4, 7).into(), // Moscow
            ],
            bonus: 40,
        };
        let mut bonuses = vec![ekaterinburg];
        if bonus_options.len() == 4 {
            if bonus_options[0] {
                // Odessa
                bonuses.push(Bonus::VisitBonus {
                    locn: (10, 3).into(),
                    bonus: 10,
                });
            }
            if bonus_options[1] {
                // Moscow
                bonuses.push(Bonus::VisitBonus {
                    locn: (4, 7).into(),
                    bonus: 10,
                });
            }
            if bonus_options[2] {
                // Moscow
                bonuses.push(Bonus::VisitBonus {
                    locn: (4, 7).into(),
                    bonus: 10,
                });
            }
            if bonus_options[3] {
                // Poland
                bonuses.push(Bonus::VisitBonus {
                    locn: (8, 0).into(),
                    bonus: 10,
                });
            }
        } else {
            panic!("Invalid number of bonus options: {}", bonus_options.len())
        }
        bonuses
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
        let tokens = self.create_tokens();
        let hexes: Vec<HexAddress> =
            addrs().iter().map(|coords| coords.into()).collect();
        let mut map = Map::new(
            self.catalogue.clone(),
            tokens,
            hexes,
            self.hex_orientation(),
        );
        for (addr, (tile_name, rotn)) in initial_tiles() {
            if !map.place_tile(addr, tile_name, rotn) {
                println!("Could not place tile {} at {}", tile_name, addr)
            }
        }
        for (addr, label) in hex_labels() {
            // NOTE: these are used to identify valid tile upgrades.
            // They do not appear on the map itself.
            map.add_label_at(addr, label);
        }
        for (addr, face) in &self.barriers {
            map.add_barrier(*addr, *face)
        }
        // TODO: mark tiles that are not modifiable.
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
        let red_cities: Vec<(HexAddress, &str)> = vec![
            ((7, 16).into(), "Central Asia"),
            ((10, 1).into(), "Romania"),
            ((11, 8).into(), "Caucasus"),
            ((8, 0).into(), "Poland"),
        ];
        let suffix = match phase {
            0 => "Yw",
            1 | 2 => "Gn",
            3 => "Bn",
            _ => "Gy",
        };
        for (addr, city_name) in &red_cities {
            let tile_name = format!("{} {}", city_name, suffix);
            if !map.place_tile(*addr, &tile_name, RotateCW::Zero) {
                println!("Could not place tile {} at {}", tile_name, addr)
            }
        }
        // Show a green token space in Ekaterinburg for phase 2,
        // and an empty token space for all other phases.
        let ekat_addr: HexAddress = (2, 16).into();
        let ekat_tile = if phase == 0 { "Ekat Yw" } else { "Ekat Gr" };
        if !map.place_tile(ekat_addr, ekat_tile, RotateCW::Zero) {
            println!("Could not place tile {} at {}", ekat_tile, ekat_addr)
        }
        true
    }

    /// Return the name of each game phase.
    fn phase_names(&self) -> &[&str] {
        &self.phase_names
    }
}

fn addrs() -> Vec<(isize, isize)> {
    vec![
        // Row 0
        (0, 15),
        // Rows 1 and 2.
        (1, 3),
        (1, 4),
        (1, 5),
        (1, 7),
        (1, 9),
        (1, 11),
        (1, 13),
        (1, 14),
        (1, 15),
        // Rows 3 and 4.
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (2, 6),
        (2, 7),
        (2, 8),
        (2, 9),
        (2, 10),
        (2, 11),
        (2, 12),
        (2, 13),
        (2, 14),
        (2, 15),
        (2, 16),
        // Rows 5 and 6.
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
        (3, 4),
        (3, 5),
        (3, 6),
        (3, 7),
        (3, 8),
        (3, 9),
        (3, 10),
        (3, 11),
        (3, 12),
        (3, 13),
        (3, 14),
        (3, 15),
        (3, 16),
        // Rows 7 and 8.
        (4, 1),
        (4, 2),
        (4, 3),
        (4, 4),
        (4, 5),
        (4, 6),
        (4, 7),
        (4, 8),
        (4, 9),
        (4, 10),
        (4, 11),
        (4, 12),
        (4, 13),
        (4, 14),
        (4, 15),
        (4, 16),
        // Rows 9 and 10.
        (5, 0),
        (5, 1),
        (5, 2),
        (5, 3),
        (5, 4),
        (5, 5),
        (5, 6),
        (5, 7),
        (5, 8),
        (5, 9),
        (5, 10),
        (5, 11),
        (5, 12),
        (5, 13),
        (5, 14),
        (5, 15),
        // Rows 11 and 12.
        (6, 0),
        (6, 1),
        (6, 2),
        (6, 3),
        (6, 4),
        (6, 5),
        (6, 6),
        (6, 7),
        (6, 8),
        (6, 9),
        (6, 10),
        (6, 11),
        (6, 12),
        (6, 13),
        (6, 14),
        (6, 15),
        (6, 16),
        // Rows 13 and 14.
        (7, 0),
        (7, 1),
        (7, 2),
        (7, 3),
        (7, 4),
        (7, 5),
        (7, 6),
        (7, 7),
        (7, 8),
        (7, 9),
        (7, 10),
        (7, 11),
        (7, 16),
        // Rows 15 and 16.
        (8, 0),
        (8, 1),
        (8, 2),
        (8, 3),
        (8, 4),
        (8, 5),
        (8, 6),
        (8, 7),
        (8, 8),
        (8, 9),
        (8, 10),
        // Rows 17 and 18.
        (9, 1),
        (9, 2),
        (9, 3),
        (9, 4),
        (9, 5),
        (9, 6),
        (9, 7),
        (9, 8),
        (9, 9),
        (9, 10),
        (9, 11),
        // Rows 19 and 20.
        (10, 1),
        (10, 2),
        (10, 3),
        (10, 4),
        (10, 5),
        (10, 6),
        (10, 7),
        (10, 8),
        (10, 9),
        (10, 10),
        (10, 11),
        (10, 12),
        (10, 13),
        // Row 21.
        (11, 2),
        (11, 8),
        (11, 10),
        (11, 12),
    ]
}

/// Returns the tiles that are available to players at the start of the game.
fn player_tiles() -> Builder {
    let tiles = vec![
        // Yellow tiles.
        (Kind::_3, 2),
        (Kind::_4, 4),
        (Kind::_5, 2),
        (Kind::_6, 2),
        (Kind::_7, 3),
        (Kind::_8, 19),
        (Kind::_9, 24),
        (Kind::_57, 2),
        (Kind::_58, 4),
        (Kind::_201, 3),
        (Kind::_202, 3),
        (Kind::_621, 2),
        // Green tiles.
        (Kind::_14, 2),
        (Kind::_15, 2),
        (Kind::_16, 2),
        (Kind::_17, 2),
        (Kind::_18, 2),
        (Kind::_19, 2),
        (Kind::_20, 2),
        (Kind::_21, 2),
        (Kind::_22, 2),
        (Kind::_23, 5),
        (Kind::_24, 5),
        (Kind::_25, 4),
        (Kind::_26, 2),
        (Kind::_27, 2),
        (Kind::_28, 2),
        (Kind::_29, 2),
        (Kind::_30, 2),
        (Kind::_31, 2),
        (Kind::_87, 2),
        (Kind::_88, 2),
        (Kind::_204, 2),
        (Kind::_207, 5),
        (Kind::_208, 2),
        (Kind::_619, 2),
        (Kind::_622, 2),
        (Kind::_624, 1),
        (Kind::_625, 1),
        (Kind::_626, 1),
        (Kind::_635, 1),
        (Kind::_637, 1),
        // Brown tiles.
        (Kind::_39, 2),
        (Kind::_40, 2),
        (Kind::_41, 2),
        (Kind::_42, 2),
        (Kind::_43, 2),
        (Kind::_44, 2),
        (Kind::_45, 2),
        (Kind::_46, 2),
        (Kind::_47, 2),
        (Kind::_63, 3),
        (Kind::_611, 3),
        (Kind::_623, 3),
        (Kind::_636, 1),
        (Kind::_638, 1),
        (Kind::_641, 1),
        (Kind::_801, 2),
        (Kind::_911, 3),
        // Grey tiles.
        (Kind::_639, 1),
        (Kind::_640, 1),
        (Kind::_642, 1),
    ];
    Builder::with_available_tiles(tiles).unwrap()
}

fn initial_tiles() -> BTreeMap<HexAddress, (&'static str, RotateCW)> {
    let tiles: Vec<(HexAddress, (&str, RotateCW))> = vec![
        // Grey off-board tiles.
        ((0, 15).into(), ("Grey1", RotateCW::Zero)),
        ((2, 16).into(), ("Ekat Yw", RotateCW::Zero)),
        ((3, 16).into(), ("Grey2", RotateCW::One)),
        ((3, 0).into(), ("Grey1", RotateCW::Four)),
        ((4, 16).into(), ("Grey1", RotateCW::One)),
        ((11, 2).into(), ("Grey1", RotateCW::Three)),
        ((11, 12).into(), ("Grey1", RotateCW::Two)),
        // Grey on-board tile.
        ((3, 2).into(), ("Grey3", RotateCW::Zero)),
        // Cities with token spaces.
        ((1, 4).into(), ("St Petersburg", RotateCW::Zero)),
        ((2, 1).into(), ("Riga", RotateCW::Zero)),
        ((4, 7).into(), ("Moscow", RotateCW::Zero)),
        ((4, 10).into(), ("Nizhnii Novgorod", RotateCW::Zero)),
        ((5, 13).into(), ("Samara", RotateCW::Zero)),
        ((7, 3).into(), ("Kiev", RotateCW::Zero)),
        ((8, 6).into(), ("Kharkov", RotateCW::Zero)),
        ((10, 3).into(), ("Odessa", RotateCW::Zero)),
        ((10, 12).into(), ("Astrakhan", RotateCW::Zero)),
        ((5, 4).into(), ("Smolensk", RotateCW::Zero)),
        ((9, 7).into(), ("Yuzovka", RotateCW::Zero)),
        ((10, 8).into(), ("Rostov", RotateCW::Zero)),
        ((9, 10).into(), ("Tsaritsyn", RotateCW::Zero)),
        ((1, 15).into(), ("Perm", RotateCW::Zero)),
        ((4, 12).into(), ("Kazan", RotateCW::Zero)),
        ((4, 15).into(), ("Ufa", RotateCW::Zero)),
        ((6, 11).into(), ("Saratov", RotateCW::Zero)),
        ((9, 5).into(), ("Ekaterin\u{ad}oslav", RotateCW::Zero)),
        // Towns without token spaces.
        ((4, 1).into(), ("Vilnius", RotateCW::Zero)),
        ((5, 2).into(), ("Minsk", RotateCW::Zero)),
        ((6, 4).into(), ("Gomel", RotateCW::Zero)),
        ((7, 4).into(), ("Chernigov", RotateCW::Zero)),
        ((7, 6).into(), ("Kursk", RotateCW::Zero)),
        ((7, 8).into(), ("Voronezh", RotateCW::Zero)),
        ((10, 6).into(), ("Alexan\u{ad}drovsk", RotateCW::Zero)),
        ((5, 7).into(), ("Tula", RotateCW::Zero)),
        ((3, 8).into(), ("Yaroslavl", RotateCW::Zero)),
        ((5, 12).into(), ("Lugansk", RotateCW::Zero)),
        ((6, 10).into(), ("Penza", RotateCW::Zero)),
        ((5, 12).into(), ("Simbirsk", RotateCW::Zero)),
        // Yellow track and Tver.
        // NOTE: use the custom "8 initial" tile so that the tile name is not
        // displayed.
        ((3, 6).into(), ("Tver", RotateCW::Zero)),
        ((1, 5).into(), ("8 initial", RotateCW::Two)),
        ((2, 5).into(), ("8 initial", RotateCW::Five)),
        ((4, 6).into(), ("8 initial", RotateCW::Five)),
        ((1, 3).into(), ("8 initial", RotateCW::Zero)),
        ((2, 3).into(), ("8 initial", RotateCW::Three)),
        ((4, 2).into(), ("8 initial", RotateCW::Three)),
        // Red off-board tiles.
        ((6, 16).into(), ("Central Asia2", RotateCW::Zero)),
        ((7, 16).into(), ("Central Asia Yw", RotateCW::Zero)),
        ((10, 1).into(), ("Romania Yw", RotateCW::Zero)),
        ((9, 1).into(), ("Romania2", RotateCW::Zero)),
        ((11, 8).into(), ("Caucasus Yw", RotateCW::Zero)),
        ((10, 9).into(), ("Caucasus2", RotateCW::Zero)),
        ((11, 10).into(), ("Caucasus3", RotateCW::Zero)),
        ((5, 0).into(), ("Poland2", RotateCW::Zero)),
        ((6, 0).into(), ("Poland3", RotateCW::Zero)),
        ((7, 0).into(), ("Poland3", RotateCW::Zero)),
        ((8, 0).into(), ("Poland Yw", RotateCW::Zero)),
    ];
    tiles.into_iter().collect()
}

/// Defines the labels that identify which map hexes can contain specific
/// tiles.
fn hex_labels() -> Vec<(HexAddress, Label)> {
    vec![
        // St Petersburg
        ((1, 4).into(), Label::City("S".to_string())),
        // Riga
        ((2, 1).into(), Label::y()),
        // Moscow
        ((4, 7).into(), Label::City("M".to_string())),
        // Nizhnii Novgorod
        ((4, 10).into(), Label::y()),
        // Samara
        ((5, 13).into(), Label::y()),
        // Kiev
        ((7, 3).into(), Label::City("K".to_string())),
        // Kharkov
        ((8, 6).into(), Label::y()),
        ((8, 6).into(), Label::City("Kh".to_string())),
        // Odessa
        ((10, 3).into(), Label::y()),
        // Astrakhan
        ((10, 12).into(), Label::y()),
    ]
}

fn make_catalogue() -> Catalogue {
    let mut builder = player_tiles();
    let hex = builder.hex();
    // NOTE: hide tile names on all starting tiles, off-board tiles, etc.
    let city_tiles = starting_city_tiles(hex)
        .into_iter()
        .map(|t| t.hide_tile_name());
    let offb_tiles =
        offboard_tiles(hex).into_iter().map(|t| t.hide_tile_name());
    let misc_tiles = miscellaneous_tiles(hex)
        .into_iter()
        .map(|t| t.hide_tile_name());
    builder.add_unavailable_tiles(city_tiles);
    builder.add_unavailable_tiles(offb_tiles);
    builder.add_unavailable_tiles(misc_tiles);
    builder.build()
}

/// Position labels relative to the tile centre.
///
/// This means the label will be centred horizontally and vertically, relative
/// to its position coordinates.
fn off_centre(dir: n18hex::Direction, frac: f64) -> HexPosition {
    HexPosition::Centre(Some(n18hex::Delta::InDir(dir, frac)))
}

fn starting_city_tiles(hex: &Hex) -> Vec<Tile> {
    use n18hex::{Direction::*, *};
    use n18tile::*;
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;

    let dits = vec![
        "Vilnius",
        "Minsk",
        "Gomel",
        "Chernigov",
        "Kursk",
        "Alexan\u{ad}drovsk",
        "Tula",
        "Yaroslavl",
        "Lugansk",
        "Penza",
        "Simbirsk",
    ];
    let cities =
        vec!["Saratov", "Ekaterin\u{ad}oslav", "Kazan", "Ufa", "Perm"];
    let cities_yellow = vec!["Voronezh"];
    let cities_y = vec!["Astrakhan"];
    let cities_y_green = vec!["Samara"];
    let cities_y_yellow = vec!["Riga", "Odessa"];
    let cities_y_kh_yellow = vec!["Kharkov"];
    let cities_green = vec!["Smolensk", "Yuzovka", "Rostov", "Tsaritsyn"];

    let city_label_pos = off_centre(N, 0.525);
    let y_label_pos = off_centre(S, 0.525);

    let tver = Tile::new(
        Yellow,
        "Tver",
        vec![
            Track::gentle_l(Bottom).with_span(0.0, 0.5).with_dit(
                TrackEnd::End,
                10,
                DitShape::Bar,
            ),
            Track::gentle_l(Bottom).with_span(0.5, 1.0),
        ],
        vec![],
        hex,
    )
    .label(Label::Revenue(0), off_centre(N30E, 0.25))
    .label(Label::MapLocation("Tver".to_string()), Top.to_centre(0.2));

    let nizhnii = Tile::new(
        Yellow,
        "Nizhnii Novgorod",
        vec![
            Track::gentle_r(LowerLeft).with_span(0.0, 0.5),
            Track::gentle_r(LowerLeft).with_span(0.5, 1.0),
        ],
        vec![City::single(30).in_dir(S, 0.25)],
        hex,
    )
    .label(Label::Revenue(0), Left.to_centre(0.2))
    .label(Label::y(), Right.to_centre(0.2))
    .label(
        Label::MapLocation("Nizhnii Novgorod".to_string()),
        Top.to_centre(0.1),
    );

    let moscow = Tile::new(
        Yellow,
        "Moscow",
        vec![
            Track::straight(Bottom).with_span(0.0, 0.3),
            Track::straight(UpperLeft).with_span(0.0, 0.3),
            Track::straight(UpperRight).with_span(0.0, 0.3),
        ],
        vec![
            City::single_at_face(40, &Bottom)
                .in_dir(N, 0.2)
                .with_fill(Yellow),
            City::single_at_face(40, &UpperLeft)
                .in_dir(S60E, 0.2)
                .with_fill(Yellow),
            City::single_at_face(40, &UpperRight)
                .in_dir(S60W, 0.2)
                .with_fill(Yellow),
        ],
        hex,
    )
    .label(Label::Revenue(0), LowerLeft.in_dir(S60E, 0.15))
    .label(Label::City("M".to_string()), LowerRight.in_dir(S60W, 0.15))
    .label(Label::MapLocation("Moscow".to_string()), Top.to_centre(0.1));

    let kiev = Tile::new(
        Yellow,
        "Kiev",
        vec![
            Track::straight(Bottom).with_span(0.0, 0.3),
            Track::straight(UpperLeft).with_span(0.0, 0.3),
            Track::straight(UpperRight).with_span(0.0, 0.3),
        ],
        vec![
            City::single_at_face(30, &Bottom).in_dir(N, 0.2),
            City::single_at_face(30, &UpperLeft)
                .in_dir(S60E, 0.2)
                .with_fill(Yellow),
            City::single_at_face(30, &UpperRight)
                .in_dir(S60W, 0.2)
                .with_fill(Yellow),
        ],
        hex,
    )
    .label(Label::Revenue(0), LowerLeft.in_dir(S60E, 0.15))
    .label(Label::City("K".to_string()), LowerRight.in_dir(S60W, 0.15))
    .label(Label::MapLocation("Kiev".to_string()), Top.to_centre(0.1));

    let st_petersburg = Tile::new(
        Green,
        "St Petersburg",
        vec![
            Track::gentle_r(LowerLeft).with_span(0.0, 0.3),
            Track::gentle_l(LowerRight).with_span(0.0, 0.3),
        ],
        vec![
            City::single_at_face(40, &LowerLeft)
                .in_dir(N60E, 0.15)
                .with_fill(Yellow),
            City::single_at_face(40, &LowerRight)
                .in_dir(N60W, 0.15)
                .with_fill(Grey),
        ],
        hex,
    )
    .label(Label::Revenue(0), Bottom.to_centre(0.15))
    .label(Label::City("S".to_string()), Right.to_centre(0.15))
    .label(
        Label::MapLocation("St Petersburg".to_string()),
        Top.to_centre(0.1),
    );

    let cities_with_track = vec![tver, nizhnii, moscow, kiev, st_petersburg];

    cities_with_track
        .into_iter()
        .chain(dits.into_iter().map(|name| {
            Tile::new(
                Empty,
                name,
                // NOTE: create the dit with a zero-length track.
                vec![Track::straight(Bottom).with_span(0.5, 0.5).with_dit(
                    TrackEnd::End,
                    10,
                    DitShape::Circle,
                )],
                vec![],
                hex,
            )
            .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_yellow.into_iter().map(|name| {
            Tile::new(
                Empty,
                name,
                vec![],
                vec![City::single(0).with_fill(Yellow)],
                hex,
            )
            .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_y.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::y(), y_label_pos)
                .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_y_yellow.into_iter().map(|name| {
            Tile::new(
                Empty,
                name,
                vec![],
                vec![City::single(0).with_fill(Yellow)],
                hex,
            )
            .label(Label::y(), y_label_pos)
            .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_y_kh_yellow.into_iter().map(|name| {
            Tile::new(
                Empty,
                name,
                vec![],
                vec![City::single(0).with_fill(Yellow)],
                hex,
            )
            .label(Label::y(), y_label_pos)
            .label(Label::City("Kh".to_string()), Right.to_centre(0.15))
            .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_y_green.into_iter().map(|name| {
            Tile::new(
                Empty,
                name,
                vec![],
                vec![City::single(0).with_fill(Green)],
                hex,
            )
            .label(Label::y(), y_label_pos)
            .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_green.into_iter().map(|name| {
            Tile::new(
                Empty,
                name,
                vec![],
                vec![City::single(0).with_fill(Green)],
                hex,
            )
            .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .collect()
}

fn offboard_tiles(hex: &Hex) -> Vec<Tile> {
    let suffixes = vec!["Yw", "Gn", "Bn", "Gy"];
    let mut tiles = vec![];
    tiles.append(&mut central_asia(hex, &suffixes));
    tiles.append(&mut romania(hex, &suffixes));
    tiles.append(&mut caucasus(hex, &suffixes));
    tiles.append(&mut poland(hex, &suffixes));
    tiles
}

/// Position labels above the bottom hex face.
///
/// The default nudge is 0.215 towards the tile centre.
fn above_bottom_face<F: Into<Option<f64>>>(pos: F) -> HexPosition {
    let frac = pos.into().unwrap_or(0.215);
    HexFace::Bottom.to_centre(frac)
}

fn central_asia(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexCorner::*, HexFace::*};
    use n18tile::*;

    let name = "Central Asia";
    let revenues = vec![10, 20, 30, 40];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                    Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(revenue, &TopLeft).with_fill(Red)
                ],
                hex,
            )
            .with_offboard_faces([UpperLeft])
            .label(Label::MapLocation(name.to_string()), off_centre(S, 0.15))
            .label(
                Label::PhaseRevenue(vec![
                    (Yellow, revenues[0], ix == 0),
                    (Green, revenues[1], ix == 1),
                    (Brown, revenues[2], ix == 2),
                    (Grey, revenues[3], ix == 3),
                ]),
                above_bottom_face(0.15),
            )
        })
        .chain(vec![Tile::new(
            Red,
            format!("{}2", name),
            vec![Track::gentle_l(Bottom), Track::hard_l(Bottom)],
            vec![],
            hex,
        )
        .with_offboard_faces([LowerLeft, UpperLeft])])
        .collect()
}

fn romania(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexCorner::*, HexFace::*};
    use n18tile::*;

    let name = "Romania";
    let revenues = vec![10, 20, 30, 30];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_r(UpperRight).with_span(0.0, 0.5),
                    Track::hard_r(UpperRight).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(revenue, &TopRight).with_fill(Red)
                ],
                hex,
            )
            .with_offboard_faces([UpperRight])
            .label(Label::MapLocation(name.to_string()), off_centre(S, 0.12))
            .label(
                Label::PhaseRevenue(vec![
                    (Yellow, revenues[0], ix == 0),
                    (Green, revenues[1], ix == 1),
                    (Brown, revenues[2], ix == 2),
                    (Grey, revenues[3], ix == 3),
                ]),
                above_bottom_face(None),
            )
        })
        .chain(vec![Tile::new(
            Red,
            format!("{}2", name),
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::hard_r(Bottom),
            ],
            vec![],
            hex,
        )
        .with_offboard_faces([Top, UpperRight, LowerRight])])
        .collect()
}

fn caucasus(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexCorner::*, HexFace::*};
    use n18tile::*;

    let name = "Caucasus";
    let revenues = vec![10, 20, 40, 60];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_r(UpperRight).with_span(0.0, 0.5),
                    Track::hard_r(UpperRight).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(revenue, &TopRight).with_fill(Red)
                ],
                hex,
            )
            .with_offboard_faces([Top])
            .label(Label::MapLocation(name.to_string()), off_centre(S, 0.12))
            .label(
                Label::PhaseRevenue(vec![
                    (Yellow, revenues[0], ix == 0),
                    (Green, revenues[1], ix == 1),
                    (Brown, revenues[2], ix == 2),
                    (Grey, revenues[3], ix == 3),
                ]),
                above_bottom_face(None),
            )
        })
        .chain(vec![
            Tile::new(
                Red,
                format!("{}2", name),
                vec![
                    Track::gentle_r(LowerLeft),
                    Track::straight(LowerLeft),
                    Track::gentle_l(LowerLeft),
                    Track::hard_l(LowerLeft),
                ],
                vec![],
                hex,
            )
            .with_offboard_faces([UpperLeft, Top, UpperRight]),
            Tile::new(
                Red,
                format!("{}3", name),
                vec![Track::hard_l(UpperLeft), Track::gentle_l(UpperLeft)],
                vec![],
                hex,
            )
            .with_offboard_faces([Top, UpperRight]),
        ])
        .collect()
}

fn poland(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*};
    use n18tile::*;

    let name = "Poland";
    let revenues = vec![30, 40, 50, 70];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_l(Top).with_span(0.0, 0.2),
                    Track::hard_l(Top).with_span(0.3, 1.0),
                    Track::gentle_l(Top).with_span(0.3, 1.0),
                ],
                vec![City::single_at_face(revenue, &Top).with_fill(Red)],
                hex,
            )
            .with_offboard_faces([UpperRight, LowerRight])
            .label(
                Label::MapLocation(name.to_string()),
                off_centre(S60W, 0.35),
            )
            .label(
                Label::PhaseRevenue(vec![
                    (Yellow, revenues[0], ix == 0),
                    (Green, revenues[1], ix == 1),
                    (Brown, revenues[2], ix == 2),
                    (Grey, revenues[3], ix == 3),
                ]),
                above_bottom_face(0.15),
            )
        })
        .chain(vec![
            Tile::new(
                Red,
                format!("{}2", name),
                vec![Track::gentle_r(Bottom), Track::hard_r(Bottom)],
                vec![],
                hex,
            )
            .with_offboard_faces([UpperRight, LowerRight]),
            Tile::new(
                Red,
                format!("{}3", name),
                vec![
                    Track::straight(Bottom),
                    Track::gentle_r(Bottom),
                    Track::hard_r(Bottom),
                ],
                vec![],
                hex,
            )
            .with_offboard_faces([UpperRight, LowerRight]),
        ])
        .collect()
}

fn miscellaneous_tiles(hex: &Hex) -> Vec<Tile> {
    use n18hex::*;
    use n18tile::*;
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;

    vec![
        Tile::new(Grey, "Grey1", vec![Track::hard_l(Bottom)], vec![], hex),
        Tile::new(
            Grey,
            "Grey2",
            vec![Track::hard_r(UpperLeft), Track::gentle_r(UpperLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Grey,
            "Grey3",
            vec![
                Track::straight(LowerRight).with_span(0.0, 0.8).with_dit(
                    TrackEnd::End,
                    10,
                    DitShape::Bar,
                ),
                Track::straight(LowerRight).with_span(0.8, 1.0),
                Track::gentle_r(Bottom),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), LowerLeft.to_centre(0.1))
        .label(
            Label::MapLocation("Daugavpils".to_string()),
            Top.to_centre(0.1),
        ),
        Tile::new(
            Grey,
            "Ekat Yw",
            vec![
                Track::gentle_r(UpperLeft).with_span(0.0, 0.2),
                Track::gentle_r(UpperLeft).with_span(0.3, 1.0),
                Track::hard_r(UpperLeft).with_span(0.3, 1.0),
            ],
            vec![City::single_at_face(40, &UpperLeft)
                .in_dir(Direction::S30E, 0.2)
                .with_fill(HexColour::Green)],
            hex,
        )
        .label(Label::Revenue(0), BottomRight.to_centre(0.1))
        .label(
            Label::MapLocation("Ekaterin\u{ad}burg".to_string()),
            HexPosition::Centre(None).in_dir(Direction::N, 0.525),
        ),
        Tile::new(
            Grey,
            "Ekat Gr",
            vec![
                Track::gentle_r(UpperLeft).with_span(0.0, 0.2),
                Track::gentle_r(UpperLeft).with_span(0.3, 1.0),
                Track::hard_r(UpperLeft).with_span(0.3, 1.0),
            ],
            vec![City::single_at_face(40, &UpperLeft)
                .in_dir(Direction::S30E, 0.2)],
            hex,
        )
        .label(Label::Revenue(0), BottomRight.to_centre(0.1))
        .label(
            Label::MapLocation("Ekaterin\u{ad}burg".to_string()),
            HexPosition::Centre(None).in_dir(Direction::N, 0.525),
        ),
        Tile::new(
            Yellow,
            "8 initial",
            vec![Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
    ]
}
