//! # 1867: The Railways Of Canada
//!
//! Initial version of 1867 map and tiles.
//!

use std::collections::BTreeMap;

use super::Company;
use n18catalogue::{Builder, Catalogue, Kind};
use n18hex::{Colour, Hex, HexColour, HexFace, HexPosition, RotateCW};
use n18map::{HexAddress, Map};
use n18route::{Bonus, ConflictRule, Train, TrainType};
use n18tile::{Label, Tile};
use n18token::{Token, TokenStyle};

fn addrs() -> Vec<(isize, isize)> {
    vec![
        // Rows 1 and 2.
        (0, 3),
        (0, 4),
        (0, 5),
        // Rows 3 and 4.
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (1, 6),
        (1, 7),
        // Rows 5 and 6.
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
        // Rows 6 and 7.
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
        // Rows 7 and 8.
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
        // Rows 9 and 10.
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
        // Rows 11 and 12.
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
        (6, 14),
        // Rows 13 and 14.
        (7, 1),
        (7, 2),
        (7, 3),
        (7, 4),
        (7, 5),
        (7, 6),
        (7, 7),
        (7, 8),
        (7, 12),
        // Rows 15 and 16.
        (8, 0),
        (8, 1),
        (8, 2),
        (8, 3),
        (8, 4),
        (8, 5),
        // Rows 17 and 18.
        (9, 0),
        (9, 2),
        (9, 4),
    ]
}

fn initial_tiles() -> BTreeMap<HexAddress, (&'static str, RotateCW)> {
    let tiles: Vec<(HexAddress, (&str, RotateCW))> = vec![
        // Blue off-board tiles.
        // Unnamed port ($10)
        ((7, 7).into(), ("Port2", RotateCW::Zero)),
        // Unnamed port ($10)
        ((9, 4).into(), ("Port1", RotateCW::Zero)),
        // Red off-board tiles.
        // Sault Ste. Marie
        ((3, 0).into(), ("Sault Ste Marie Yw", RotateCW::Zero)),
        // Maritime Provinces
        ((3, 15).into(), ("Maritime Provinces Yw", RotateCW::Zero)),
        // Maine
        ((6, 14).into(), ("Maine Yw", RotateCW::Zero)),
        // New England
        ((7, 12).into(), ("New England Yw", RotateCW::Zero)),
        // Detroit
        ((8, 0).into(), ("Detroit2", RotateCW::Zero)),
        // Buffalo
        ((8, 5).into(), ("Buffalo Yw", RotateCW::Zero)),
        // Detroit
        ((9, 0).into(), ("Detroit Yw", RotateCW::Zero)),
        // Grey (fixed) tiles:
        // Timmins (3 tiles)
        ((0, 3).into(), ("Timmins Yw", RotateCW::Zero)),
        ((0, 4).into(), ("Grey2", RotateCW::One)),
        ((1, 2).into(), ("Grey2", RotateCW::Zero)),
        // South of Montreal.
        ((6, 11).into(), ("Grey1", RotateCW::Three)),
        // North of Sarnia, north-west of London.
        ((7, 1).into(), ("Grey1", RotateCW::Zero)),
        // Towns without track.
        ((4, 7).into(), ("Pembroke", RotateCW::Zero)),
        ((4, 11).into(), ("St Jerome", RotateCW::Zero)),
        ((6, 7).into(), ("Belleville", RotateCW::Zero)),
        ((6, 10).into(), ("Cornwall", RotateCW::Zero)),
        ((6, 12).into(), ("Granby", RotateCW::Zero)),
        ((7, 2).into(), ("Goderich", RotateCW::Zero)),
        ((8, 1).into(), ("Sarnia", RotateCW::Zero)),
        // Cities without track.
        ((3, 3).into(), ("Sudbury", RotateCW::Zero)),
        ((3, 5).into(), ("North Bay", RotateCW::Zero)),
        ((4, 12).into(), ("Trois-Rivières", RotateCW::Zero)),
        ((5, 13).into(), ("Sherbrooke", RotateCW::Zero)),
        ((6, 4).into(), ("Barrie", RotateCW::Zero)),
        ((7, 4).into(), ("Guelph", RotateCW::Zero)),
        ((7, 6).into(), ("Peter\u{ad}borough", RotateCW::Zero)),
        ((7, 8).into(), ("Kingston", RotateCW::Zero)),
        ((8, 2).into(), ("London", RotateCW::Zero)),
        // Y Cities without track.
        ((3, 14).into(), ("Quebec", RotateCW::Zero)),
        ((5, 9).into(), ("Ottawa", RotateCW::Zero)),
        ((7, 3).into(), ("Berlin", RotateCW::Zero)),
        ((8, 4).into(), ("Hamilton", RotateCW::Zero)),
        // Cities with initial track.
        ((5, 11).into(), ("Montreal", RotateCW::Zero)),
        ((7, 5).into(), ("Toronto", RotateCW::Zero)),
    ];
    tiles.into_iter().collect()
}

fn hex_labels() -> Vec<(HexAddress, Label)> {
    vec![
        ((3, 14).into(), Label::y()),
        ((5, 9).into(), Label::City("O".to_string())),
        ((5, 9).into(), Label::y()),
        ((7, 3).into(), Label::y()),
        ((8, 4).into(), Label::y()),
        ((5, 11).into(), Label::City("M".to_string())),
        ((7, 5).into(), Label::City("T".to_string())),
    ]
}

/// Defines the trains, tiles, and map for 1867: The Railways Of Canada.
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
            ("BBG", "Buffalo, Brantford and Goderich Railway"),
            ("BO", "Brockville and Ottawa Railway"),
            ("CS", "Canada Southern Railway"),
            ("CV", "Credit Valley Railway"),
            ("KP", "Kingston and Pembroke Railway"),
            ("LPS", "London and Port Stanley Railway"),
            ("OP", "Ottawa and Prescott Railway"),
            ("SLA", "St. Lawrence and Atlantic Railroad"),
            ("TGB", "Toronto, Grey and Bruce Railway"),
            ("TN", "Toronto and Nipissing Railway"),
            ("AE", "Algoma Eastern Railway"),
            ("CA", "Canada Atlantic Railway"),
            ("NYO", "New York and Ottawa Railway"),
            ("PM", "Pere Marquette Railway"),
            ("QLS", "Quebec and Lake St. John Railway"),
            ("THB", "Toronto, Hamilton and Buffalo Railway"),
            ("CNR", "Canadian Northern Railway"),
            ("CPR", "Canadian Pacific Railway"),
            ("C&O", "Chesapeake and Ohio Railway"),
            ("GT", "Grand Trunk Railway"),
            ("GW", "Great Western Railway"),
            ("IRC", "Intercolonial Railway of Canada"),
            ("NTR", "National Transcontinental Railway"),
            ("NYC", "New York Central Railroad"),
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

        let companies: Vec<Company> = bg_iter
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

        let barriers = vec![
            // The two ports.
            ("E19".parse().unwrap(), n18hex::HexFace::UpperLeft),
            ("H16".parse().unwrap(), n18hex::HexFace::Top),
            // Lake Huron
            ("C11".parse().unwrap(), n18hex::HexFace::Top),
            ("C11".parse().unwrap(), n18hex::HexFace::Bottom),
            ("D10".parse().unwrap(), n18hex::HexFace::UpperLeft),
            ("D10".parse().unwrap(), n18hex::HexFace::LowerLeft),
            ("D10".parse().unwrap(), n18hex::HexFace::Bottom),
            ("D10".parse().unwrap(), n18hex::HexFace::LowerRight),
            ("D12".parse().unwrap(), n18hex::HexFace::UpperRight),
            // St Lawrence River
            ("M11".parse().unwrap(), n18hex::HexFace::UpperLeft),
            ("M11".parse().unwrap(), n18hex::HexFace::Top),
            ("N10".parse().unwrap(), n18hex::HexFace::UpperLeft),
            ("N10".parse().unwrap(), n18hex::HexFace::Top),
            ("O9".parse().unwrap(), n18hex::HexFace::UpperLeft),
            ("O9".parse().unwrap(), n18hex::HexFace::Top),
        ];
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
        "1867: The Railways of Canada"
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
            // $10 bonus for Buffalo.
            "Niagara Falls Bridge",
            // $10 bonus for Montreal.
            "Montreal Bridge",
            // $10 bonus for Quebec.
            "Quebec Bridge",
            // $10 bonus for Detroit.
            "St. Clair Tunnel",
        ]
    }

    fn bonuses(&self, bonus_options: &[bool]) -> Vec<Bonus> {
        let timmins = Bonus::ConnectionBonus {
            from: (0, 3).into(),
            to_any: vec![
                (7, 5).into(),  // Toronto
                (5, 11).into(), // Montreal
                (3, 14).into(), // Quebec
            ],
            bonus: 40,
        };
        let mut bonuses = vec![timmins];
        if bonus_options.len() == 4 {
            if bonus_options[0] {
                // Buffalo
                bonuses.push(Bonus::VisitBonus {
                    locn: (8, 5).into(),
                    bonus: 10,
                });
            }
            if bonus_options[1] {
                // Montreal
                bonuses.push(Bonus::VisitBonus {
                    locn: (5, 11).into(),
                    bonus: 10,
                });
            }
            if bonus_options[2] {
                // Quebec
                bonuses.push(Bonus::VisitBonus {
                    locn: (3, 14).into(),
                    bonus: 10,
                });
            }
            if bonus_options[3] {
                // Detroit
                bonuses.push(Bonus::VisitBonus {
                    locn: (8, 0).into(),
                    bonus: 10,
                });
                bonuses.push(Bonus::VisitBonus {
                    locn: (9, 0).into(),
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
        let mut map = Map::new(self.catalogue.clone(), tokens, hexes);
        for (addr, (tile_name, rotn)) in initial_tiles() {
            if !map.place_tile(addr, tile_name, rotn) {
                println!("Could not place tile {} at {}", tile_name, addr)
            }
        }
        for (addr, label) in hex_labels() {
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
            ((3, 0).into(), "Sault Ste Marie"),
            ((3, 15).into(), "Maritime Provinces"),
            ((6, 14).into(), "Maine"),
            ((7, 12).into(), "New England"),
            ((8, 5).into(), "Buffalo"),
            ((9, 0).into(), "Detroit"),
        ];
        let suffix = match phase {
            0 => "Yw",
            1 | 2 => "Gn",
            3 | 4 => "Bn",
            _ => "Gy",
        };
        for (addr, city_name) in &red_cities {
            let tile_name = format!("{} {}", city_name, suffix);
            if !map.place_tile(*addr, &tile_name, RotateCW::Zero) {
                println!("Could not place tile {} at {}", tile_name, addr)
            }
        }
        let timmins_addr: HexAddress = (0, 3).into();
        let timmins_tile = if phase == 0 {
            "Timmins Yw"
        } else {
            "Timmins Gr"
        };
        if !map.place_tile(timmins_addr, timmins_tile, RotateCW::Zero) {
            println!(
                "Could not place tile {} at {}",
                timmins_tile, timmins_addr
            )
        }
        true
    }

    /// Return the name of each game phase.
    fn phase_names(&self) -> &[&str] {
        &self.phase_names
    }
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
        (Kind::_15, 4),
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
        (Kind::_120, 1),
        (Kind::_204, 2),
        (Kind::_207, 5),
        (Kind::_208, 2),
        (Kind::_619, 2),
        (Kind::_622, 2),
        (Kind::_624, 1),
        (Kind::_625, 1),
        (Kind::_626, 1),
        (Kind::_637, 1),
        (Kind::X1, 1),
        (Kind::X2, 1),
        (Kind::X3, 1),
        (Kind::X4, 1),
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
        (Kind::_70, 2),
        (Kind::_611, 3),
        (Kind::_623, 3),
        (Kind::_801, 2),
        (Kind::_911, 3),
        (Kind::_122, 1),
        (Kind::X5, 1),
        (Kind::X6, 1),
        (Kind::X7, 1),
        // Grey tiles.
        (Kind::_124, 1),
        (Kind::_639, 1),
        (Kind::X8, 1),
    ];
    Builder::with_available_tiles(tiles).unwrap()
}

fn make_catalogue() -> Catalogue {
    let mut builder = player_tiles();
    let hex = builder.hex();
    // NOTE: hide tile names on all starting tiles, off-board tiles, etc.
    let town_tiles = starting_town_tiles(hex)
        .into_iter()
        .map(|t| t.hide_tile_name());
    let city_tiles = starting_city_tiles(hex)
        .into_iter()
        .map(|t| t.hide_tile_name());
    let offb_tiles =
        offboard_tiles(hex).into_iter().map(|t| t.hide_tile_name());
    let misc_tiles = miscellaneous_tiles(hex)
        .into_iter()
        .map(|t| t.hide_tile_name());
    builder.add_unavailable_tiles(town_tiles);
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

/// Position labels above the bottom hex face.
///
/// The default nudge is 0.215 towards the tile centre.
fn above_bottom_face<F: Into<Option<f64>>>(pos: F) -> HexPosition {
    let frac = pos.into().unwrap_or(0.215);
    HexFace::Bottom.to_centre(frac)
}

/// Tiles that are specific to 1867 and which cannot be placed by the player.
fn starting_city_tiles(hex: &Hex) -> Vec<Tile> {
    use n18hex::{Direction::*, *};
    use n18tile::*;
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;

    let cities = vec![
        "Sudbury",
        "North Bay",
        "Trois-Rivières",
        "Sherbrooke",
        "Barrie",
        "Guelph",
        "Peter\u{ad}borough",
        "Kingston",
        "London",
    ];

    let city_label_pos = off_centre(N, 0.525);
    let y_label_pos = off_centre(S, 0.525);
    let oy_y_label_pos = off_centre(S60W, 0.625);
    let oy_o_label_pos = off_centre(S60E, 0.625);

    let cities_y = vec!["Quebec", "Berlin", "Hamilton"];
    let cities_oy = vec!["Ottawa"];

    let montreal = Tile::new(
        Yellow,
        "Montreal",
        vec![
            Track::straight(LowerLeft).with_span(0.0, 0.3),
            Track::straight(Top).with_span(0.0, 0.3),
        ],
        vec![
            City::single(40).in_dir(S60W, 0.4),
            City::single(40).in_dir(N, 0.4),
            City::single(40).in_dir(S60E, 0.4),
        ],
        hex,
    )
    .label(
        Label::City("M".to_string()),
        UpperLeft.in_dir(Direction::E, 0.1),
    )
    .label(Label::Revenue(0), UpperRight.in_dir(Direction::W, 0.1));

    let toronto = Tile::new(
        Yellow,
        "Toronto",
        vec![
            Track::straight(LowerLeft).with_span(0.0, 0.3),
            Track::straight(UpperRight).with_span(0.0, 0.3),
        ],
        vec![
            City::single(30).in_dir(S60W, 0.4),
            City::single(30).in_dir(N60E, 0.4),
        ],
        hex,
    )
    .label(
        Label::City("T".to_string()),
        UpperLeft.in_dir(Direction::E, 0.15),
    )
    .label(Label::Revenue(0), BottomRight.to_centre(0.1));

    let timmins_yw = Tile::new(
        Grey,
        "Timmins Yw",
        vec![
            Track::straight(LowerLeft).with_span(0.0, 0.5),
            Track::straight(Bottom).with_span(0.0, 0.5),
            Track::straight(LowerRight).with_span(0.0, 0.5),
            Track::straight(UpperRight).with_span(0.0, 0.5),
        ],
        vec![City::single(40).with_fill(HexColour::Green)],
        hex,
    )
    .label(Label::Revenue(0), BottomRight.to_centre(0.1))
    .label(
        Label::MapLocation("Timmins".to_string()),
        HexPosition::Centre(None).in_dir(Direction::N, 0.525),
    );

    let timmins_gr = Tile::new(
        Grey,
        "Timmins Gr",
        vec![
            Track::straight(LowerLeft).with_span(0.0, 0.5),
            Track::straight(Bottom).with_span(0.0, 0.5),
            Track::straight(LowerRight).with_span(0.0, 0.5),
            Track::straight(UpperRight).with_span(0.0, 0.5),
        ],
        vec![City::single(40)],
        hex,
    )
    .label(Label::Revenue(0), BottomRight.to_centre(0.1))
    .label(
        Label::MapLocation("Timmins".to_string()),
        HexPosition::Centre(None).in_dir(Direction::N, 0.525),
    );

    cities
        .into_iter()
        .map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::MapLocation(name.to_string()), city_label_pos)
        })
        .chain(cities_y.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::y(), y_label_pos)
                .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_oy.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::y(), oy_y_label_pos)
                .label(Label::City("O".to_string()), oy_o_label_pos)
                .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(vec![toronto, montreal, timmins_yw, timmins_gr].into_iter())
        .collect()
}

/// Tiles that are specific to 1867 and which cannot be placed by the player.
fn starting_town_tiles(hex: &Hex) -> Vec<Tile> {
    use n18hex::{Direction::*, *};
    use n18tile::DitShape::*;
    use n18tile::*;
    use HexColour::*;
    use HexFace::*;
    use TrackEnd::*;

    let towns = vec![
        "Pembroke",
        "St Jerome",
        "Belleville",
        "Cornwall",
        "Granby",
        "Goderich",
        "Sarnia",
    ];

    let town_label_pos = off_centre(N, 0.425);

    towns
        .into_iter()
        .map(|name| {
            Tile::new(
                Empty,
                name,
                vec![Track::straight(Bottom)
                    .with_span(0.5, 0.5)
                    .with_dit(End, 10, Circle)],
                vec![],
                hex,
            )
            .label(Label::MapLocation(name.to_string()), town_label_pos)
        })
        .collect()
}

fn sault_ste_marie(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*, *};
    use n18tile::*;

    let name = "Sault Ste Marie";
    vec![20, 30, 40, 40]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_r(LowerRight).with_span(0.0, 0.5),
                    Track::hard_l(UpperRight).with_span(0.0, 0.5),
                ],
                vec![City::single_at_corner(revenue, &HexCorner::Right)],
                hex,
            )
            .with_offboard_faces([UpperRight, LowerRight])
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, 20, ix == 0),
                    (HexColour::Green, 30, ix == 1),
                    (HexColour::Brown, 40, ix == 2),
                    (HexColour::Grey, 40, ix == 3),
                ]),
                above_bottom_face(None),
            )
            .label(Label::MapLocation(name.to_string()), off_centre(N, 0.575))
        })
        .collect()
}

fn maritime_provinces(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*, *};
    use n18tile::*;

    let name = "Maritime Provinces";
    vec![30, 30, 40, 40]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_r(UpperLeft).with_span(0.0, 0.5),
                    Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                ],
                vec![City::single_at_corner(revenue, &HexCorner::Left)],
                hex,
            )
            .with_offboard_faces([LowerLeft, UpperLeft])
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, 30, ix == 0),
                    (HexColour::Green, 30, ix == 1),
                    (HexColour::Brown, 40, ix == 2),
                    (HexColour::Grey, 40, ix == 3),
                ]),
                above_bottom_face(None),
            )
            .label(Label::MapLocation(name.to_string()), off_centre(N, 0.6))
        })
        .collect()
}

fn maine(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*, *};
    use n18tile::*;

    let name = "Maine";
    vec![20, 30, 40, 40]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_r(Top).with_span(0.0, 0.5),
                    Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                ],
                vec![City::single_at_corner(revenue, &HexCorner::TopLeft)],
                hex,
            )
            .with_offboard_faces([UpperLeft, Top])
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, 20, ix == 0),
                    (HexColour::Green, 30, ix == 1),
                    (HexColour::Brown, 40, ix == 2),
                    (HexColour::Grey, 40, ix == 3),
                ]),
                above_bottom_face(None),
            )
            .label(Label::MapLocation(name.to_string()), off_centre(S, 0.22))
        })
        .collect()
}

fn new_england(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*, *};
    use n18tile::*;

    let name = "New England";
    vec![30, 40, 50, 60]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![Track::straight(Top).with_span(0.0, 0.25)],
                vec![City::single(revenue).in_dir(Direction::N, 0.4)],
                hex,
            )
            .with_offboard_faces([Top])
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, 30, ix == 0),
                    (HexColour::Green, 40, ix == 1),
                    (HexColour::Brown, 50, ix == 2),
                    (HexColour::Grey, 60, ix == 3),
                ]),
                above_bottom_face(0.115),
            )
            .label(Label::MapLocation(name.to_string()), off_centre(S, 0.2))
        })
        .collect()
}

fn buffalo(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*, *};
    use n18tile::*;

    let name = "Buffalo";
    vec![30, 40, 50, 60]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![Track::straight(UpperLeft).with_span(0.0, 0.5)],
                vec![City::single(revenue)],
                hex,
            )
            .with_offboard_faces([UpperLeft])
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, 30, ix == 0),
                    (HexColour::Green, 40, ix == 1),
                    (HexColour::Brown, 50, ix == 2),
                    (HexColour::Grey, 60, ix == 3),
                ]),
                above_bottom_face(None),
            )
            .label(Label::MapLocation(name.to_string()), off_centre(N, 0.525))
        })
        .collect()
}

fn detroit(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    use n18hex::{Direction::*, HexColour::*, HexFace::*, *};
    use n18tile::*;

    let name = "Detroit";
    let mut tiles: Vec<Tile> = vec![30, 40, 50, 70]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::hard_r(UpperRight).with_span(0.0, 0.5),
                    Track::hard_l(Top).with_span(0.0, 0.5),
                ],
                vec![City::single_at_corner(revenue, &HexCorner::TopRight)],
                hex,
            )
            .with_offboard_faces([UpperRight])
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, 30, ix == 0),
                    (HexColour::Green, 40, ix == 1),
                    (HexColour::Brown, 50, ix == 2),
                    (HexColour::Grey, 70, ix == 3),
                ]),
                above_bottom_face(None),
            )
            .label(Label::MapLocation(name.to_string()), off_centre(S, 0.22))
        })
        .collect();
    tiles.push(
        Tile::new(
            Red,
            "Detroit2",
            vec![Track::hard_l(LowerRight)],
            vec![],
            hex,
        )
        .with_offboard_faces([LowerRight]),
    );
    tiles
}

/// Tiles that are specific to 1867 and which cannot be placed by the player.
fn offboard_tiles(hex: &Hex) -> Vec<Tile> {
    use n18hex::*;
    use n18tile::DitShape::*;
    use n18tile::*;
    use HexColour::*;
    use HexFace::*;
    use TrackEnd::*;

    let mut ports = vec![
        Tile::new(
            Blue,
            "Port1",
            vec![Track::straight(Top)
                .with_span(0.0, 0.5)
                .with_dit(End, 10, Circle)],
            vec![],
            hex,
        )
        .with_offboard_faces([Top])
        .label(Label::Revenue(0), UpperLeft.to_centre(0.25)),
        Tile::new(
            Blue,
            "Port2",
            vec![
                Track::straight(UpperLeft)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Circle),
                Track::straight(UpperRight).with_span(0.0, 0.5),
            ],
            vec![],
            hex,
        )
        .with_offboard_faces([UpperLeft, UpperRight])
        .label(Label::Revenue(0), Top.to_centre(0.3)),
    ];

    let suffixes = vec!["Yw", "Gn", "Bn", "Gy"];
    ports.append(&mut sault_ste_marie(hex, &suffixes));
    ports.append(&mut maritime_provinces(hex, &suffixes));
    ports.append(&mut maine(hex, &suffixes));
    ports.append(&mut new_england(hex, &suffixes));
    ports.append(&mut buffalo(hex, &suffixes));
    ports.append(&mut detroit(hex, &suffixes));
    ports
}

fn miscellaneous_tiles(hex: &Hex) -> Vec<Tile> {
    use n18hex::*;
    use n18tile::*;
    use HexColour::*;
    use HexFace::*;

    vec![
        Tile::new(Grey, "Grey1", vec![Track::hard_r(Bottom)], vec![], hex),
        Tile::new(Grey, "Grey2", vec![Track::gentle_r(Bottom)], vec![], hex),
    ]
}
