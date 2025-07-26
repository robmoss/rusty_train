//! Defines the tiles for 1830: Railways and Robber Barons.

use n18catalogue::{Builder, Catalogue, Kind};
use n18hex::{Direction::*, Hex, HexColour, HexCorner, HexFace, HexPosition};
use n18tile::{City, DitShape::*, Label, Tile, Track, TrackEnd::*};

/// Returns the tile catalogue, which contains all tiles needed to construct
/// the initial game map (including off-board locations) and the tiles that
/// are available for players to place on the map.
pub fn catalogue() -> Catalogue {
    let available_tiles = player_tiles();
    let mut builder = Builder::with_available_tiles(available_tiles).unwrap();
    let hex = builder.hex();
    let offboard_tiles = offboard_tiles(hex);
    let starting_tiles = initial_tiles(hex);
    builder.add_unavailable_tiles(offboard_tiles);
    builder.add_unavailable_tiles(starting_tiles);
    builder.build()
}

/// Returns the tiles that are available to players at the start of the game.
///
/// Each tile is represented as a `(Kind, count)` tuple, where `count` is the
/// number of copies that are available.
pub fn player_tiles() -> Vec<(Kind, usize)> {
    vec![
        // Yellow tiles.
        (Kind::_1, 1),
        (Kind::_2, 1),
        (Kind::_3, 2),
        (Kind::_4, 2),
        (Kind::_7, 4),
        (Kind::_8, 8),
        (Kind::_9, 7),
        (Kind::_55, 1),
        (Kind::_56, 1),
        (Kind::_57, 4),
        (Kind::_58, 2),
        (Kind::_69, 1),
        // Green tiles.
        (Kind::_14, 3),
        (Kind::_15, 2),
        (Kind::_16, 1),
        (Kind::_18, 1),
        (Kind::_19, 1),
        (Kind::_20, 1),
        (Kind::_23, 3),
        (Kind::_24, 3),
        (Kind::_25, 1),
        (Kind::_26, 1),
        (Kind::_27, 1),
        (Kind::_28, 1),
        (Kind::_29, 1),
        (Kind::_53, 2),
        (Kind::_54, 1),
        (Kind::_59, 2),
        // Brown tiles.
        (Kind::_39, 1),
        (Kind::_40, 1),
        (Kind::_41, 2),
        (Kind::_42, 2),
        (Kind::_43, 2),
        (Kind::_44, 1),
        (Kind::_45, 2),
        (Kind::_46, 2),
        (Kind::_47, 1),
        (Kind::_61, 2),
        (Kind::_62, 1),
        (Kind::_63, 3),
        (Kind::_64, 1),
        (Kind::_65, 1),
        (Kind::_66, 1),
        (Kind::_67, 1),
        (Kind::_68, 1),
        (Kind::_70, 1),
    ]
}

/// Returns the tiles that define the initial state of the map.
pub fn initial_tiles(hex: &Hex) -> Vec<Tile> {
    [
        yellow_oo_tiles(hex),
        yellow_city_tiles(hex),
        grey_city_tiles(hex),
        grey_town_tiles(hex),
        grey_track_tiles(hex),
        single_dit_tiles(hex),
        double_dit_tiles(hex),
        single_token_tiles(hex),
        single_token_cost_tiles(hex),
        empty_with_costs(hex),
    ]
    .concat()
    .into_iter()
    .map(|tile| tile.hide_tile_name())
    .collect()
}

/// Returns the yellow tiles with "OO" labels.
///
/// Note that the tile name labels are not hidden.
fn yellow_oo_tiles(hex: &Hex) -> Vec<Tile> {
    vec![
        Tile::new(
            HexColour::Yellow,
            "Detroit/Windsor",
            vec![],
            vec![
                City::single_at_corner(0, &HexCorner::Right).to_centre(0.1),
                City::single_at_face(0, &HexFace::LowerLeft).to_centre(0.2),
            ],
            hex,
        )
        .label(
            Label::MapLocation("Detroit".to_string()),
            HexFace::UpperLeft.in_dir(S30W, 0.2),
        )
        .label(
            Label::MapLocation("Windsor".to_string()),
            HexCorner::BottomRight.to_centre(0.25),
        )
        .label(Label::Note("$80".to_string()), HexPosition::centre())
        .label(
            Label::CityKind("OO".to_string()),
            HexCorner::TopRight.to_centre(0.1),
        ),
        Tile::new(
            HexColour::Yellow,
            "Hamilton/Toronto",
            vec![],
            vec![
                City::single_at_corner(0, &HexCorner::BottomLeft)
                    .in_dir(N60W, 0.1),
                City::single_at_face(0, &HexFace::UpperLeft).to_centre(0.1),
            ],
            hex,
        )
        .label(Label::Note("$80".to_string()), HexPosition::centre())
        .label(
            Label::MapLocation("Toronto".to_string()),
            HexCorner::TopRight.in_dir(S30W, 0.1),
        )
        .label(
            Label::MapLocation("Hamilton".to_string()),
            HexCorner::Right.in_dir(S60W, 0.15),
        )
        .label(
            Label::CityKind("OO".to_string()),
            HexCorner::BottomRight.to_centre(0.1),
        ),
        Tile::new(
            HexColour::Yellow,
            "Buffalo/Dunkirk",
            vec![],
            vec![
                City::single_at_corner(0, &HexCorner::BottomLeft)
                    .to_centre(0.1),
                City::single_at_face(0, &HexFace::Top).to_centre(0.2),
            ],
            hex,
        )
        .label(
            Label::MapLocation("Buffalo".to_string()),
            HexFace::UpperRight.to_centre(0.1),
        )
        .label(
            Label::MapLocation("Dunkirk".to_string()),
            HexCorner::BottomRight.to_centre(0.25),
        )
        .label(
            Label::CityKind("OO".to_string()),
            HexFace::UpperLeft.in_dir(S30W, 0.2),
        ),
        Tile::new(
            HexColour::Yellow,
            "Trenton/Philadelphia",
            vec![],
            vec![
                City::single_at_corner(0, &HexCorner::BottomLeft),
                City::single_at_face(0, &HexFace::Top).in_dir(S30E, 0.15),
            ],
            hex,
        )
        .label(
            Label::MapLocation("Trenton".to_string()),
            HexCorner::Left.to_centre(0.05),
        )
        .label(
            // NOTE: forcing the line-break ensures that the text displays
            // correctly at all zoom levels.
            Label::MapLocation("Phila-\ndelphia".to_string()),
            HexCorner::BottomRight.in_dir(N30E, 0.45),
        )
        .label(
            Label::CityKind("OO".to_string()),
            HexFace::UpperRight.to_centre(0.1),
        ),
    ]
}

/// Returns the yellow tiles with "B" and "NY" labels.
///
/// Note that the tile name labels are not hidden.
fn yellow_city_tiles(hex: &Hex) -> Vec<Tile> {
    vec![
        Tile::new(
            HexColour::Yellow,
            "Baltimore",
            vec![
                Track::straight(HexFace::UpperRight).with_span(0.0, 0.5),
                Track::straight(HexFace::Bottom).with_span(0.0, 0.5),
            ],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), HexFace::LowerRight.to_centre(0.15))
        .label(
            Label::CityKind("B".to_string()),
            HexFace::LowerLeft.to_centre(0.2),
        )
        .label(
            Label::MapLocation("Baltimore".to_string()),
            HexPosition::centre().in_dir(N, 0.425),
        ),
        Tile::new(
            HexColour::Yellow,
            "Boston",
            vec![
                Track::straight(HexFace::Top).with_span(0.0, 0.5),
                Track::straight(HexFace::LowerRight).with_span(0.0, 0.5),
            ],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), HexFace::Bottom.to_centre(0.15))
        .label(
            Label::CityKind("B".to_string()),
            HexFace::UpperRight.to_centre(0.2),
        )
        .label(
            Label::MapLocation("Boston".to_string()),
            HexCorner::Left.in_dir(E, 0.1),
        ),
        Tile::new(
            HexColour::Yellow,
            "New York",
            vec![
                Track::straight(HexFace::Top).with_span(0.0, 0.25),
                Track::straight(HexFace::Bottom).with_span(0.0, 0.25),
            ],
            vec![
                City::single_at_face(40, &HexFace::Top).to_centre(0.1),
                City::single_at_face(40, &HexFace::Bottom).to_centre(0.1),
            ],
            hex,
        )
        .label(Label::Revenue(0), HexFace::UpperLeft.to_centre(0.02))
        .label(Label::Revenue(0), HexFace::LowerRight.to_centre(0.02))
        .label(
            Label::Note("$80".to_string()),
            HexPosition::centre().in_dir(E, 0.1),
        )
        .label(
            Label::CityKind("NY".to_string()),
            HexFace::UpperRight.to_centre(0.2),
        )
        .label(
            Label::MapLocation("New\nYork".to_string()),
            HexCorner::Left.in_dir(E, 0.25),
        )
        .label(
            Label::MapLocation("Newark".to_string()),
            HexCorner::Right.to_centre(0.05),
        ),
    ]
}

/// Returns the grey city tiles.
///
/// Note that the tile name labels are not hidden.
fn grey_city_tiles(hex: &Hex) -> Vec<Tile> {
    vec![
        Tile::new(
            HexColour::Grey,
            "Lansing",
            vec![
                Track::straight(HexFace::UpperRight).with_span(0.0, 0.5),
                Track::straight(HexFace::LowerRight).with_span(0.0, 0.5),
            ],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::BottomLeft.to_centre(0.2))
        .label(
            Label::MapLocation("Lansing".to_string()),
            HexPosition::centre().in_dir(N, 0.425),
        ),
        Tile::new(
            HexColour::Grey,
            "Cleveland",
            vec![
                Track::straight(HexFace::LowerRight).with_span(0.0, 0.5),
                Track::straight(HexFace::Bottom).with_span(0.0, 0.5),
            ],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::BottomLeft.to_centre(0.2))
        .label(
            Label::MapLocation("Cleveland".to_string()),
            HexPosition::centre().in_dir(N, 0.425),
        ),
        Tile::new(
            HexColour::Grey,
            "Altoona",
            vec![
                Track::straight(HexFace::LowerLeft),
                Track::gentle_r(HexFace::LowerLeft).with_span(0.0, 0.6),
                Track::gentle_l(HexFace::UpperRight).with_span(0.0, 0.6),
            ],
            vec![City::single(30).in_dir(S, 0.4)],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::TopLeft.to_centre(0.15))
        .label(
            Label::MapLocation("Altoona".to_string()),
            HexPosition::centre().in_dir(N, 0.35),
        ),
        Tile::new(
            HexColour::Grey,
            "Rochester",
            vec![
                Track::straight(HexFace::UpperRight).with_span(0.0, 0.5),
                Track::straight(HexFace::Bottom).with_span(0.0, 0.5),
                Track::straight(HexFace::LowerLeft).with_span(0.0, 0.5),
            ],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), HexFace::LowerRight.to_centre(0.15))
        .label(
            Label::MapLocation("Rochester".to_string()),
            HexPosition::centre().in_dir(N, 0.425),
        ),
        Tile::new(
            HexColour::Grey,
            "Richmond",
            vec![Track::straight(HexFace::UpperLeft).with_span(0.0, 0.5)],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::TopLeft.to_centre(0.2))
        .label(
            Label::MapLocation("Richmond".to_string()),
            HexPosition::centre().in_dir(S, 0.425),
        ),
        Tile::new(
            HexColour::Grey,
            "Montreal",
            vec![
                Track::straight(HexFace::LowerRight).with_span(0.0, 0.5),
                Track::straight(HexFace::Bottom).with_span(0.0, 0.5),
            ],
            vec![City::single(40)],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::BottomLeft.to_centre(0.2))
        .label(
            Label::MapLocation("Montreal".to_string()),
            HexPosition::centre().in_dir(N, 0.425),
        ),
    ]
}

/// Returns the grey town tiles.
///
/// Note that the tile name labels are not hidden.
fn grey_town_tiles(hex: &Hex) -> Vec<Tile> {
    vec![
        Tile::new(
            HexColour::Grey,
            "Atlantic City",
            vec![
                Track::hard_r(HexFace::UpperLeft)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::hard_r(HexFace::UpperLeft).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), HexFace::Top.to_centre(0.2))
        .label(
            Label::Note("Atlantic City".to_string()),
            HexPosition::centre().in_dir(S, 0.2),
        ),
        Tile::new(
            HexColour::Grey,
            "Fall River",
            vec![
                Track::hard_r(HexFace::UpperLeft)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::hard_r(HexFace::UpperLeft).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), HexFace::Top.to_centre(0.2))
        .label(
            Label::Note("Fall River".to_string()),
            HexPosition::centre().in_dir(S, 0.2),
        ),
        Tile::new(
            HexColour::Grey,
            "Kingston",
            vec![
                Track::gentle_r(HexFace::Top)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::gentle_r(HexFace::Top).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::TopLeft.to_centre(0.15))
        .label(
            Label::Note("Kingston".to_string()),
            HexPosition::centre().in_dir(S, 0.2),
        ),
    ]
}

/// Returns the grey track tiles.
///
/// Note that the tile name labels are not hidden.
fn grey_track_tiles(hex: &Hex) -> Vec<Tile> {
    vec![Tile::new(
        HexColour::Grey,
        "Grey1",
        vec![Track::hard_l(HexFace::Bottom)],
        vec![],
        hex,
    )]
}

/// Returns the city tiles with building costs.
///
/// Note that the tile name labels are not hidden.
fn single_token_cost_tiles(hex: &Hex) -> Vec<Tile> {
    [
        ("Toledo", 80),
        ("Washington", 80),
        ("Providence", 80),
        ("Scranton", 120),
    ]
    .iter()
    .map(|(name, cost)| {
        Tile::new(HexColour::Empty, *name, vec![], vec![City::single(0)], hex)
            .label(
                Label::MapLocation(name.to_string()),
                HexPosition::centre().in_dir(N, 0.425),
            )
            .label(
                Label::Note(format!("${cost}")),
                HexPosition::centre().in_dir(S, 0.425),
            )
    })
    .collect()
}

/// Returns the city tiles without building costs.
///
/// Note that the tile name labels are not hidden.
fn single_token_tiles(hex: &Hex) -> Vec<Tile> {
    [
        "Columbus",
        "Barre",
        "Pittsburgh",
        "Ottawa",
        "Lancaster",
        "Albany",
    ]
    .iter()
    .map(|&name| {
        Tile::new(HexColour::Empty, name, vec![], vec![City::single(0)], hex)
            .label(
                Label::MapLocation(name.to_string()),
                HexPosition::centre().in_dir(N, 0.425),
            )
    })
    .collect()
}

/// Returns the single-dit tiles.
///
/// Note that the tile name labels are not hidden.
fn single_dit_tiles(hex: &Hex) -> Vec<Tile> {
    ["Flint", "London", "Erie", "Burlington"]
        .iter()
        .map(|&name| {
            Tile::new(
                HexColour::Empty,
                name,
                vec![Track::straight(HexFace::Bottom)
                    .with_span(0.5, 0.5)
                    .with_dit(End, 10, Circle)],
                vec![],
                hex,
            )
            .label(
                Label::MapLocation(name.to_string()),
                HexPosition::centre().in_dir(N, 0.425),
            )
        })
        .collect()
}

/// Returns the double-dit tiles.
///
/// Note that the tile name labels are not hidden.
fn double_dit_tiles(hex: &Hex) -> Vec<Tile> {
    [
        ("Akron", "Canton"),
        ("Allentown", "Reading"),
        ("Hartfort", "New Haven"),
    ]
    .iter()
    .map(|&(a, b)| {
        Tile::new(
            HexColour::Empty,
            format!("{a}/{b}"),
            vec![
                Track::straight(HexFace::Bottom)
                    .with_span(0.3, 0.3)
                    .with_dit(End, 10, Circle),
                Track::straight(HexFace::Top)
                    .with_span(0.3, 0.3)
                    .with_dit(End, 10, Circle),
            ],
            vec![],
            hex,
        )
        .label(
            Label::Note(a.to_string()),
            HexCorner::TopLeft.to_centre(0.25),
        )
        .label(
            Label::Note(b.to_string()),
            HexCorner::BottomRight.to_centre(0.25),
        )
    })
    .collect()
}

/// Returns the empty-hex tiles with building costs.
///
/// Note that the tile name labels are not hidden.
fn empty_with_costs(hex: &Hex) -> Vec<Tile> {
    vec![
        // Empty hexes with building costs ($80 or $120).
        Tile::new(HexColour::Empty, "Cost_80", vec![], vec![], hex)
            .label(Label::Note("$80".to_string()), HexPosition::Centre(None)),
        Tile::new(HexColour::Empty, "Cost_120", vec![], vec![], hex).label(
            Label::Note("$120".to_string()),
            HexPosition::Centre(None),
        ),
    ]
}

/// Returns the tiles for each off-board location.
pub fn offboard_tiles(hex: &Hex) -> Vec<Tile> {
    let suffixes = ["Yw", "Gn", "Bn"];
    [
        offboard_chicago(hex, &suffixes),
        offboard_deep_south(hex, &suffixes),
        offboard_maritime(hex, &suffixes),
        offboard_mexico_1(hex, &suffixes),
        offboard_canada_1(hex, &suffixes),
        vec![offboard_mexico_2(hex), offboard_canada_2(hex)],
    ]
    .concat()
}

/// The off-board tiles for Chicago.
fn offboard_chicago(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    let city = "Chicago";
    let revenues = [40, 40, 70];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, revenue)| {
            let suffix = suffixes[ix];
            let name = format!("{city}_{suffix}");
            Tile::new(
                HexColour::Red,
                name,
                vec![
                    Track::straight(HexFace::Top).with_span(0.0, 0.5),
                    Track::straight(HexFace::UpperRight).with_span(0.0, 0.5),
                    Track::straight(HexFace::LowerRight).with_span(0.0, 0.5),
                ],
                vec![City::single(*revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, revenues[0], ix == 0),
                    (HexColour::Green, revenues[1], ix == 1),
                    (HexColour::Brown, revenues[2], ix > 1),
                ]),
                HexCorner::BottomLeft.to_centre(0.1),
            )
            .label(
                Label::MapLocation(city.to_string()),
                HexFace::LowerLeft.to_centre(0.1),
            )
            .with_offboard_faces([
                HexFace::Top,
                HexFace::UpperRight,
                HexFace::LowerRight,
            ])
            .hide_tile_name()
        })
        .collect()
}

/// The off-board tiles for the Gulf of Mexico that have phase-specific
/// content.
fn offboard_mexico_1(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    let name = "Gulf of Mexico";
    let text = "Gulf of\nMexico";
    let revenues = [30, 30, 60];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, revenue)| {
            let suffix = suffixes[ix];
            let tile_name = format!("{name}_{suffix}");
            Tile::new(
                HexColour::Red,
                tile_name,
                vec![
                    Track::straight(HexFace::UpperLeft).with_span(0.0, 0.5),
                    Track::straight(HexFace::Top).with_span(0.0, 0.5),
                    Track::straight(HexFace::UpperRight).with_span(0.0, 0.5),
                ],
                vec![City::single(*revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, revenues[0], ix == 0),
                    (HexColour::Green, revenues[1], ix == 1),
                    (HexColour::Brown, revenues[2], ix > 1),
                ]),
                HexCorner::Left.to_centre(0.1),
            )
            .label(
                Label::MapLocation(text.to_string()),
                HexFace::Bottom.to_centre(0.1),
            )
            .with_offboard_faces([HexFace::Top, HexFace::UpperRight])
            .hide_tile_name()
        })
        .collect()
}

/// The off-board tiles for the Gulf of Mexico that do not have phase-specific
/// content.
fn offboard_mexico_2(hex: &Hex) -> Tile {
    let name = "Gulf of Mexico 2";
    Tile::new(
        HexColour::Red,
        name,
        vec![Track::hard_l(HexFace::UpperRight)],
        vec![],
        hex,
    )
    .with_offboard_faces([HexFace::UpperRight])
    .hide_tile_name()
}

/// The off-board tiles for the Canadian West that have phase-specific
/// content.
fn offboard_canada_1(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    let city = "Canadian West";
    let revenues = [30, 30, 50];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, revenue)| {
            let suffix = suffixes[ix];
            let name = format!("{city}_{suffix}");
            Tile::new(
                HexColour::Red,
                name,
                vec![
                    Track::straight(HexFace::LowerLeft).with_span(0.0, 0.5),
                    Track::straight(HexFace::Bottom).with_span(0.0, 0.5),
                    Track::straight(HexFace::LowerRight).with_span(0.0, 0.5),
                ],
                vec![City::single(*revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, revenues[0], ix == 0),
                    (HexColour::Green, revenues[1], ix == 1),
                    (HexColour::Brown, revenues[2], ix > 1),
                ]),
                HexFace::UpperRight.to_centre(0.1),
            )
            .label(
                Label::MapLocation(city.to_string()),
                HexCorner::TopLeft.to_centre(0.3),
            )
            .with_offboard_faces([HexFace::Bottom, HexFace::LowerRight])
            .hide_tile_name()
        })
        .collect()
}

/// The off-board tiles for the Canadian West that do not have phase-specific
/// content.
fn offboard_canada_2(hex: &Hex) -> Tile {
    let name = "Canadian West 2";
    Tile::new(
        HexColour::Red,
        name,
        vec![Track::hard_r(HexFace::LowerRight)],
        vec![],
        hex,
    )
    .with_offboard_faces([HexFace::LowerRight])
    .hide_tile_name()
}

/// The off-board tiles for the Deep South.
fn offboard_deep_south(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    let city = "Deep South";
    let revenues = [30, 30, 40];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, revenue)| {
            let suffix = suffixes[ix];
            let name = format!("{city}_{suffix}");
            Tile::new(
                HexColour::Red,
                name,
                vec![
                    Track::straight(HexFace::Top).with_span(0.0, 0.5),
                    Track::straight(HexFace::UpperLeft).with_span(0.0, 0.5),
                ],
                vec![City::single(*revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, revenues[0], ix == 0),
                    (HexColour::Green, revenues[1], ix == 1),
                    (HexColour::Brown, revenues[2], ix > 1),
                ]),
                HexCorner::BottomRight.to_centre(0.5),
            )
            .label(
                Label::MapLocation(city.to_string()),
                HexPosition::centre(),
            )
            .with_offboard_faces([HexFace::Top, HexFace::UpperLeft])
            .hide_tile_name()
        })
        .collect()
}

/// The off-board tiles for the Maritime Provinces.
fn offboard_maritime(hex: &Hex, suffixes: &[&str]) -> Vec<Tile> {
    let name = "Maritime Provinces";
    let text = "Maritime\nProvinces";
    let revenues = [20, 20, 30];
    revenues
        .iter()
        .enumerate()
        .map(|(ix, revenue)| {
            let suffix = suffixes[ix];
            let tile_name = format!("{name}_{suffix}");
            Tile::new(
                HexColour::Red,
                tile_name,
                vec![
                    Track::straight(HexFace::Bottom).with_span(0.0, 0.5),
                    Track::straight(HexFace::LowerLeft).with_span(0.0, 0.5),
                ],
                vec![City::single(*revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenue(vec![
                    (HexColour::Yellow, revenues[0], ix == 0),
                    (HexColour::Green, revenues[1], ix == 1),
                    (HexColour::Brown, revenues[2], ix > 1),
                ]),
                HexCorner::Right.to_centre(0.1),
            )
            .label(
                Label::MapLocation(text.to_string()),
                HexCorner::TopRight.in_dir(W, 0.1),
            )
            .with_offboard_faces([HexFace::Bottom, HexFace::LowerLeft])
            .hide_tile_name()
        })
        .collect()
}
