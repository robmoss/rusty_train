//! # 1867: The Railways Of Canada
//!
//! Initial version of 1867 map and tiles.
//!

use std::collections::HashMap;

use n18catalogue::tile_catalogue;
use n18hex::{Hex, HexColour, HexFace, HexPosition};
use n18map::{HexAddress, Map, RotateCW};
use n18route::{Bonus, Train};
use n18tile::{Label, Tile};
use n18token::{Token, TokenStyle, Tokens};

fn addrs() -> Vec<(usize, usize)> {
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

fn initial_tiles() -> HashMap<HexAddress, (&'static str, RotateCW)> {
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
        ((3, 14).into(), Label::Y),
        ((5, 9).into(), Label::City("O".to_string())),
        ((5, 9).into(), Label::Y),
        ((7, 3).into(), Label::Y),
        ((8, 4).into(), Label::Y),
        ((5, 11).into(), Label::City("M".to_string())),
        ((7, 5).into(), Label::City("T".to_string())),
    ]
}

/// Defines the trains, tiles, and map for 1867: The Railways Of Canada.
pub struct Game {
    trains: Vec<Train>,
    names: Vec<&'static str>,
    all_tiles: Vec<Tile>,
    tokens: Tokens,
    barriers: Vec<(HexAddress, HexFace)>,
    phase: usize,
    phase_names: Vec<&'static str>,
}

impl Game {
    pub fn new(hex: &Hex) -> Self {
        let trains = vec![
            Train::new_2_train(),
            Train::new_3_train(),
            Train::new_4_train(),
            Train::new_5_train(),
            Train::new_6_train(),
            Train::new_7_train(),
            Train::new_8_train(),
            Train::new_2p2_train(),
            Train::new_5p5e_train(),
        ];
        let names = vec!["2", "3", "4", "5", "6", "7", "8", "2+2", "5+5E"];
        let all_tiles = game_tiles(&hex);
        let tokens = vec![
            (
                "CNR".to_string(),
                Token::new(TokenStyle::SideArcs {
                    fg: (176, 176, 176).into(),
                    bg: (66, 0, 0).into(),
                    text: (255, 255, 255).into(),
                }),
            ),
            (
                "CPR".to_string(),
                Token::new(TokenStyle::TopArcs {
                    fg: (176, 176, 176).into(),
                    bg: (0, 66, 0).into(),
                    text: (255, 255, 255).into(),
                }),
            ),
            (
                "C&O".to_string(),
                Token::new(TokenStyle::SideArcs {
                    fg: (176, 176, 176).into(),
                    bg: (0, 0, 66).into(),
                    text: (255, 255, 255).into(),
                }),
            ),
            (
                "GT".to_string(),
                Token::new(TokenStyle::TripleTriangles {
                    fg: (0, 143, 31).into(),
                    bg: (223, 223, 0).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
            (
                "A".to_string(),
                Token::new(TokenStyle::TopSquares {
                    fg: (176, 0, 0).into(),
                    bg: (15, 15, 127).into(),
                    text: (255, 255, 255).into(),
                }),
            ),
            (
                "B".to_string(),
                Token::new(TokenStyle::TopLines {
                    fg: (0, 143, 31).into(),
                    bg: (223, 223, 0).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
            (
                "C".to_string(),
                Token::new(TokenStyle::TopTriangles {
                    fg: (0, 143, 31).into(),
                    bg: (223, 223, 0).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
            (
                "D".to_string(),
                Token::new(TokenStyle::TopArcs {
                    fg: (176, 176, 176).into(),
                    bg: (66, 0, 0).into(),
                    text: (255, 255, 255).into(),
                }),
            ),
            (
                "E".to_string(),
                Token::new(TokenStyle::TopTriangles {
                    fg: (255, 255, 102).into(),
                    bg: (204, 16, 16).into(),
                    text: (255, 255, 255).into(),
                }),
            ),
        ]
        .into();
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
            trains,
            names,
            all_tiles,
            tokens,
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

    /// The train types that companies can purchase and operate.
    fn train_types(&self) -> Vec<Train> {
        self.trains.clone()
    }

    fn train_name(&self, train: &Train) -> Option<&str> {
        for i in 0..self.trains.len() {
            if self.trains[i] == *train {
                return Some(self.names[i]);
            }
        }
        return None;
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

    fn get_bonuses(&self, bonus_options: &Vec<bool>) -> Vec<Bonus> {
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

    /// Create the initial map for 1867.
    fn create_map(&self, _hex: &Hex) -> Map {
        let tokens = self.company_tokens().clone();
        let hexes: Vec<HexAddress> =
            addrs().iter().map(|coords| coords.into()).collect();
        let mut map = Map::new(self.all_tiles.clone(), tokens, hexes);
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

    /// Return the tiles that players are allowed to place on the map.
    fn player_tiles(&self) -> &[Tile] {
        // TODO: this currently also returns special map tiles.
        &self.all_tiles
    }

    /// Return the unique tokens (one per company).
    fn company_tokens(&self) -> &Tokens {
        &self.tokens
    }

    /// Return the number of game phases.
    fn phase_count(&self) -> usize {
        7
    }

    /// Return the current game phase.
    fn get_phase(&self) -> usize {
        self.phase
    }

    /// Change the current game phase, which may update the map.
    fn set_phase(&mut self, map: &mut Map, phase: usize) {
        if phase > 6 {
            return;
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
    }

    /// Return the name of a game phase.
    fn phase_name(&self, phase: usize) -> Option<&str> {
        self.phase_names.get(phase).map(|s| *s)
    }

    /// Return the name of each game phase.
    fn phase_names(&self) -> &[&str] {
        &self.phase_names
    }
}

fn game_tiles(hex: &Hex) -> Vec<Tile> {
    let mut all_tiles = tile_catalogue(&hex);
    // NOTE: hide tile names on all starting tiles, off-board tiles, etc.
    let mut town_tiles = starting_town_tiles(&hex)
        .into_iter()
        .map(|t| t.hide_tile_name())
        .collect();
    let mut city_tiles = starting_city_tiles(&hex)
        .into_iter()
        .map(|t| t.hide_tile_name())
        .collect();
    let mut offb_tiles = offboard_tiles(&hex)
        .into_iter()
        .map(|t| t.hide_tile_name())
        .collect();
    let mut misc_tiles = miscellaneous_tiles(&hex)
        .into_iter()
        .map(|t| t.hide_tile_name())
        .collect();
    all_tiles.append(&mut town_tiles);
    all_tiles.append(&mut city_tiles);
    all_tiles.append(&mut offb_tiles);
    all_tiles.append(&mut misc_tiles);
    all_tiles
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
                .label(Label::Y, y_label_pos)
                .label(Label::MapLocation(name.to_string()), city_label_pos)
        }))
        .chain(cities_oy.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::Y, oy_y_label_pos)
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

fn sault_ste_marie(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
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

fn maritime_provinces(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
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

fn maine(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
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

fn new_england(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
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

fn buffalo(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
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

fn detroit(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
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
    tiles.push(Tile::new(
        Red,
        "Detroit2",
        vec![Track::hard_l(LowerRight)],
        vec![],
        hex,
    ));
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
