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
        // Red off-board tiles.
        ((3, 0).into(), ("Red1", RotateCW::Zero)),
        ((3, 15).into(), ("Red1", RotateCW::Three)),
        ((6, 14).into(), ("Red1", RotateCW::Four)),
        ((7, 7).into(), ("Red3", RotateCW::Zero)),
        ((7, 12).into(), ("Red2", RotateCW::Four)),
        ((8, 0).into(), ("Red2", RotateCW::Zero)),
        ((8, 5).into(), ("Red2", RotateCW::Three)),
        ((9, 0).into(), ("Red2", RotateCW::Five)),
        ((9, 4).into(), ("Red2", RotateCW::Four)),
        // Grey (fixed) tiles.
        ((0, 3).into(), ("Timmins", RotateCW::Zero)),
        ((0, 4).into(), ("Grey2", RotateCW::One)),
        ((1, 2).into(), ("Grey2", RotateCW::Zero)),
        ((6, 11).into(), ("Grey1", RotateCW::Three)),
        ((7, 1).into(), ("Grey1", RotateCW::Zero)),
        ((7, 1).into(), ("Grey1", RotateCW::Zero)),
        // Towns without track.
        ((4, 7).into(), ("EmptyTown", RotateCW::Zero)),
        ((4, 11).into(), ("EmptyTown", RotateCW::Zero)),
        ((6, 7).into(), ("EmptyTown", RotateCW::Zero)),
        ((6, 10).into(), ("EmptyTown", RotateCW::Zero)),
        ((6, 12).into(), ("EmptyTown", RotateCW::Zero)),
        ((7, 2).into(), ("EmptyTown", RotateCW::Zero)),
        ((8, 1).into(), ("EmptyTown", RotateCW::Zero)),
        // Cities without track.
        ((3, 3).into(), ("EmptyCity", RotateCW::Zero)),
        ((3, 5).into(), ("EmptyCity", RotateCW::Zero)),
        ((4, 12).into(), ("EmptyCity", RotateCW::Zero)),
        ((5, 13).into(), ("EmptyCity", RotateCW::Zero)),
        ((6, 4).into(), ("EmptyCity", RotateCW::Zero)),
        ((7, 4).into(), ("EmptyCity", RotateCW::Zero)),
        ((7, 6).into(), ("EmptyCity", RotateCW::Zero)),
        ((7, 8).into(), ("EmptyCity", RotateCW::Zero)),
        ((8, 2).into(), ("EmptyCity", RotateCW::Zero)),
        // Y Cities without track.
        ((3, 14).into(), ("EmptyCityY", RotateCW::Zero)),
        ((5, 9).into(), ("Ottawa1", RotateCW::Zero)),
        ((7, 3).into(), ("EmptyCityY", RotateCW::Zero)),
        ((8, 4).into(), ("EmptyCityY", RotateCW::Zero)),
        // Cities with initial track.
        ((5, 11).into(), ("M0", RotateCW::Zero)),
        ((7, 5).into(), ("T0", RotateCW::Zero)),
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
        let all_tiles = tile_catalogue(&hex);
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
    fn create_map(&self, hex: &Hex) -> Map {
        let tiles = tile_catalogue(&hex);
        let hexes: Vec<HexAddress> =
            addrs().iter().map(|coords| coords.into()).collect();
        let mut map = Map::new(tiles, hexes);
        for (addr, (tile_name, rotn)) in initial_tiles() {
            map.place_tile(addr, tile_name, rotn);
        }
        for (addr, label) in hex_labels() {
            map.add_label_at(addr, label);
        }
        // TODO: mark tiles that are not modifiable.
        map
    }

    /// Return the tiles that players are allowed to place on the map.
    fn player_tiles(&self) -> &[Tile] {
        // TODO: this also returns special map tiles.
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
    fn set_phase(&mut self, _map: &mut Map, phase: usize) {
        if phase > 6 {
            return;
        }
        self.phase = phase;
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
