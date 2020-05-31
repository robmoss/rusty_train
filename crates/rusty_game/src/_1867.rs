//! # 1867: The Railways Of Canada
//!
//! Initial version of 1867 map and tiles.
//!

use std::collections::HashMap;

use rusty_catalogue::tile_catalogue;
use rusty_hex::Hex;
use rusty_map::{HexAddress, Map, RotateCW};
use rusty_route::{Bonus, Train};
use rusty_tile::{Label, Tile};

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
        ((0, 3).into(), ("Timmins", RotateCW::Zero)),
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
        ((7, 6).into(), ("Peterborough", RotateCW::Zero)),
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
        let phase = 0;
        let phase_names = vec!["2", "3", "4", "5", "6", "7", "8"];
        Game {
            trains,
            names,
            all_tiles,
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
        let hexes: Vec<HexAddress> =
            addrs().iter().map(|coords| coords.into()).collect();
        let mut map = Map::new(self.all_tiles.clone(), hexes);
        for (addr, (tile_name, rotn)) in initial_tiles() {
            if !map.place_tile(addr, tile_name, rotn) {
                println!("Could not place tile {} at {}", tile_name, addr)
            }
        }
        for (addr, label) in hex_labels() {
            map.add_label_at(addr, label);
        }
        // TODO: mark tiles that are not modifiable.
        map
    }

    /// Return the tiles that players are allowed to place on the map.
    fn player_tiles(&self) -> &[Tile] {
        // TODO: this currently also returns special map tiles.
        &self.all_tiles
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
    let mut town_tiles = starting_town_tiles(&hex);
    let mut city_tiles = starting_city_tiles(&hex);
    let mut offb_tiles = offboard_tiles(&hex);
    let mut misc_tiles = miscellaneous_tiles(&hex);
    all_tiles.append(&mut town_tiles);
    all_tiles.append(&mut city_tiles);
    all_tiles.append(&mut offb_tiles);
    all_tiles.append(&mut misc_tiles);
    all_tiles
}

/// Tiles that are specific to 1867 and which cannot be placed by the player.
fn starting_city_tiles(hex: &Hex) -> Vec<Tile> {
    use rusty_hex::Direction::*;
    use rusty_hex::*;
    use rusty_tile::*;
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
        "Peterborough",
        "Kingston",
        "London",
    ];

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
            City::single(40).nudge(SWW, 0.4),
            City::single(40).nudge(N, 0.4),
            City::single(40).nudge(SEE, 0.4),
        ],
        hex,
    )
    .label(Label::City("M".to_string()), UpperLeft.to_centre(0.3))
    .label(Label::Revenue(0), UpperRight.to_centre(0.3));

    let toronto = Tile::new(
        Yellow,
        "Toronto",
        vec![
            Track::straight(LowerLeft).with_span(0.0, 0.3),
            Track::straight(UpperRight).with_span(0.0, 0.3),
        ],
        vec![
            City::single(30).nudge(SWW, 0.4),
            City::single(30).nudge(NEE, 0.4),
        ],
        hex,
    )
    .label(Label::City("T".to_string()), UpperLeft.to_centre(0.3))
    .label(Label::Revenue(0), BottomRight.to_centre(0.1));

    let timmins = Tile::new(
        Grey,
        "Timmins",
        vec![
            Track::straight(LowerLeft).with_span(0.0, 0.5),
            Track::straight(Bottom).with_span(0.0, 0.5),
            Track::straight(LowerRight).with_span(0.0, 0.5),
            Track::straight(UpperRight).with_span(0.0, 0.5),
        ],
        vec![City::single(40)],
        hex,
    )
    .label(Label::Revenue(0), BottomRight.to_centre(0.1));

    cities
        .into_iter()
        .map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex).label(
                Label::MapLocation(name.to_string()),
                Top.to_centre(0.5),
            )
        })
        .chain(cities_y.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::Y, Bottom.to_centre(0.3))
                .label(
                    Label::MapLocation(name.to_string()),
                    Top.to_centre(0.5),
                )
        }))
        .chain(cities_oy.into_iter().map(|name| {
            Tile::new(Empty, name, vec![], vec![City::single(0)], hex)
                .label(Label::Y, LowerLeft.to_centre(0.3))
                .label(
                    Label::City("O".to_string()),
                    LowerRight.to_centre(0.3),
                )
                .label(
                    Label::MapLocation(name.to_string()),
                    Top.to_centre(0.5),
                )
        }))
        .chain(vec![toronto, montreal, timmins].into_iter())
        .collect()
}

/// Tiles that are specific to 1867 and which cannot be placed by the player.
fn starting_town_tiles(hex: &Hex) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::DitShape::*;
    use rusty_tile::*;
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
            .label(Label::MapLocation(name.to_string()), Top.to_centre(0.5))
        })
        .collect()
}

fn sault_ste_marie(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

    let name = "Sault Ste Marie";
    vec![20, 30, 40, 40]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::straight(LowerRight).with_span(0.0, 0.5),
                    Track::straight(UpperRight).with_span(0.0, 0.5),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(Label::Revenue(0), Bottom.to_centre(0.25))
            .label(Label::MapLocation(name.to_string()), Top.to_centre(0.5))
        })
        .collect()
}

fn maritime_provinces(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

    let name = "Maritime Provinces";
    vec![30, 30, 40, 40]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::straight(LowerLeft).with_span(0.0, 0.5),
                    Track::straight(UpperLeft).with_span(0.0, 0.5),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(Label::Revenue(0), Bottom.to_centre(0.25))
            .label(Label::MapLocation(name.to_string()), Top.to_centre(0.5))
        })
        .collect()
}

fn maine(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

    let name = "Maine";
    vec![20, 30, 40, 40]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::straight(Top).with_span(0.0, 0.5),
                    Track::straight(UpperLeft).with_span(0.0, 0.5),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(Label::Revenue(0), UpperRight.to_centre(0.25))
            .label(
                Label::MapLocation(name.to_string()),
                Bottom.to_centre(0.5),
            )
        })
        .collect()
}

fn new_england(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

    let name = "New England";
    vec![30, 40, 50, 60]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![Track::straight(Top).with_span(0.0, 0.5)],
                vec![City::single(revenue)],
                hex,
            )
            .label(Label::Revenue(0), UpperLeft.to_centre(0.25))
            .label(
                Label::MapLocation(name.to_string()),
                Bottom.to_centre(0.5),
            )
        })
        .collect()
}

fn buffalo(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

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
            .label(Label::Revenue(0), Bottom.to_centre(0.25))
            .label(Label::MapLocation(name.to_string()), Top.to_centre(0.5))
        })
        .collect()
}

fn detroit(hex: &Hex, suffixes: &Vec<&str>) -> Vec<Tile> {
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

    let name = "Detroit";
    let mut tiles: Vec<Tile> = vec![30, 40, 50, 70]
        .iter()
        .enumerate()
        .map(|(ix, &revenue)| {
            Tile::new(
                Red,
                format!("{} {}", name, suffixes[ix]),
                vec![
                    Track::straight(Top).with_span(0.0, 0.5),
                    Track::straight(UpperRight).with_span(0.0, 0.5),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(Label::Revenue(0), UpperLeft.to_centre(0.25))
            .label(
                Label::MapLocation(name.to_string()),
                Bottom.to_centre(0.5),
            )
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
    use rusty_hex::*;
    use rusty_tile::DitShape::*;
    use rusty_tile::*;
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
        .label(Label::Revenue(0), UpperLeft.to_centre(0.5)),
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
        .label(Label::Revenue(0), Top.to_centre(0.5)),
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
    use rusty_hex::*;
    use rusty_tile::*;
    use HexColour::*;
    use HexFace::*;

    vec![
        Tile::new(Grey, "Grey1", vec![Track::hard_r(Bottom)], vec![], hex),
        Tile::new(Grey, "Grey2", vec![Track::gentle_r(Bottom)], vec![], hex),
    ]
}
