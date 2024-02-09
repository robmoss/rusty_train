//! Defines the tiles for 1889: History of Shikoku Railways (Shikoku 1889).

use n18catalogue::{Builder, Catalogue, Kind};
use n18hex::{Direction::*, Hex, HexColour, HexCorner, HexFace, HexPosition};
use n18tile::{City, DitShape::*, Label, Tile, Track, TrackEnd::*};

use super::locns::Location;

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
        (Kind::_3, 2),
        (Kind::_5, 2),
        (Kind::_6, 2),
        (Kind::_7, 2),
        (Kind::_8, 5),
        (Kind::_9, 5),
        (Kind::_57, 2),
        (Kind::_58, 3),
        (Kind::_437, 1),
        (Kind::_438, 1),
        // Green tiles.
        (Kind::_12, 1),
        (Kind::_13, 1),
        (Kind::_14, 1),
        (Kind::_15, 3),
        (Kind::_16, 1),
        (Kind::_19, 1),
        (Kind::_20, 1),
        (Kind::_23, 2),
        (Kind::_24, 2),
        (Kind::_25, 1),
        (Kind::_26, 1),
        (Kind::_27, 1),
        (Kind::_28, 1),
        (Kind::_29, 1),
        (Kind::_205, 1),
        (Kind::_206, 1),
        (Kind::_439, 1),
        (Kind::_440, 1),
        // Brown tiles.
        (Kind::_39, 1),
        (Kind::_40, 1),
        (Kind::_41, 1),
        (Kind::_42, 1),
        (Kind::_45, 1),
        (Kind::_46, 1),
        (Kind::_47, 1),
        (Kind::_448, 4),
        (Kind::_465, 1),
        (Kind::_466, 1),
        (Kind::_492, 1),
        (Kind::_611, 2),
    ]
}

/// Returns the tiles that define the initial state of the map.
pub fn initial_tiles(hex: &Hex) -> Vec<Tile> {
    use Location::*;

    let city_tiles: Vec<Tile> = [
        Anan, Nahari, Kubokawa, Sukumo, Matsuyama, Saijou, Niihama, Kotohira,
        Marugame, Tokushima, Ikeda,
    ]
    .iter()
    .map(|locn| {
        let name = locn.as_str();
        let tile = Tile::new(
            HexColour::Empty,
            name,
            vec![],
            vec![City::single(0)],
            hex,
        )
        .label(
            Label::MapLocation(name.to_string()),
            HexPosition::centre().in_dir(N, 0.425),
        );
        if locn == &Kotohira {
            tile.label(
                Label::City("K".to_string()),
                HexPosition::centre().in_dir(S, 0.55),
            )
        } else {
            tile
        }
    })
    .collect();

    let town_tiles: Vec<Tile> =
        [Muki, Nangoku, Nakamura, RitsurinKouen, Komatsujima]
            .iter()
            .map(|locn| {
                let name = locn.as_str();
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
            .collect();

    let empty_tiles = vec![
        // Mountain
        Tile::new(HexColour::Empty, "Mountain", vec![], vec![], hex).label(
            Label::Note("Y80: Mountain".to_string()),
            HexPosition::Centre(None),
        ),
        // River
        Tile::new(HexColour::Empty, "River", vec![], vec![], hex).label(
            Label::Note("Y80: River".to_string()),
            HexPosition::Centre(None),
        ),
    ];

    let starting_tiles = vec![
        // Yellow: Ohzu, Takamatsu (T)
        Tile::new(
            HexColour::Yellow,
            "Ohzu",
            vec![Track::mid(HexFace::UpperLeft)],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), HexFace::Top.to_centre(0.15))
        .label(
            Label::MapLocation("Ohzu".to_string()),
            HexPosition::centre().in_dir(S, 0.55),
        ),
        Tile::new(
            HexColour::Yellow,
            "Takamatsu",
            vec![
                Track::mid(HexFace::UpperLeft),
                Track::mid(HexFace::LowerLeft),
                Track::mid(HexFace::Bottom),
            ],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), HexCorner::Right.to_centre(0.15))
        .label(
            Label::MapLocation("Takamatsu".to_string()),
            HexPosition::centre().in_dir(N, 0.6),
        )
        .label(
            Label::City("T".to_string()),
            HexCorner::BottomLeft.to_centre(0.08),
        ),
        // Green: Kouchi (K)
        Tile::new(
            HexColour::Green,
            "Kouchi",
            vec![
                Track::mid(HexFace::UpperLeft),
                Track::mid(HexFace::Top),
                Track::mid(HexFace::UpperRight),
                Track::mid(HexFace::LowerRight),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(
            Label::City("Ki".to_string()),
            HexCorner::BottomLeft.to_centre(0.08),
        )
        .label(
            Label::Note("Y80".to_string()),
            HexCorner::BottomRight.to_centre(0.125),
        )
        .label(Label::Revenue(0), HexCorner::TopRight.to_centre(0.15)),
        // Grey: Muroto, Uwajima, Yawatahama
        Tile::new(
            HexColour::Grey,
            "Muroto",
            vec![
                Track::hard_l(HexFace::Top)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 20, Bar),
                Track::hard_l(HexFace::Top).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(
            Label::MapLocation("Muroto".to_string()),
            HexPosition::centre(),
        )
        .label(Label::Revenue(0), HexCorner::TopLeft.to_centre(0.125)),
        Tile::new(
            HexColour::Grey,
            "Uwajima",
            vec![
                Track::mid(HexFace::Top),
                Track::mid(HexFace::LowerLeft),
                Track::mid(HexFace::LowerRight),
            ],
            vec![City::double(40)],
            hex,
        )
        .label(
            Label::MapLocation("Uwajima".to_string()),
            HexPosition::centre().in_dir(S, 0.55),
        )
        .label(Label::Revenue(0), HexCorner::TopLeft.to_centre(0.1)),
        Tile::new(
            HexColour::Grey,
            "Yawatahama",
            vec![
                Track::hard_l(HexFace::LowerRight)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 20, Bar),
                Track::hard_l(HexFace::LowerRight).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(
            Label::MapLocation("Yawatahama".to_string()),
            HexPosition::centre(),
        )
        .label(Label::Revenue(0), HexCorner::BottomLeft.to_centre(0.125)),
        // Grey: single-track tile
        Tile::new(
            HexColour::Grey,
            "Grey_Gentle",
            vec![Track::gentle_r(HexFace::LowerLeft)],
            vec![],
            hex,
        ),
    ];

    [city_tiles, town_tiles, empty_tiles, starting_tiles]
        .concat()
        .into_iter()
        .map(|tile| tile.hide_tile_name())
        .collect()
}

/// Returns the tiles for each off-board location.
pub fn offboard_tiles(hex: &Hex) -> Vec<Tile> {
    let suffixes = ["Yw", "Bn"];

    let imabari: Vec<Tile> = suffixes
        .iter()
        .map(|suffix| {
            let locn_name = Location::Imabari.as_str();
            let tile_name = format!("{}_{}", locn_name, suffix);
            let is_yellow = *suffix == "Yw";
            let revenue = if is_yellow { 30 } else { 60 };
            Tile::new(
                HexColour::Red,
                tile_name,
                vec![
                    Track::mid(HexFace::LowerLeft),
                    Track::mid(HexFace::Bottom),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenueVert(vec![
                    (HexColour::Yellow, 30, is_yellow),
                    (HexColour::Brown, 60, !is_yellow),
                    (HexColour::Grey, 100, false),
                ]),
                HexCorner::Right.to_centre(0.35),
            )
            .label(
                Label::MapLocation(locn_name.to_string()),
                HexFace::Top.to_centre(0.1),
            )
            .with_offboard_faces([HexFace::Bottom, HexFace::LowerLeft])
        })
        .collect();

    let sakaide: Vec<Tile> = suffixes
        .iter()
        .map(|suffix| {
            let locn_name = Location::SakaideAndOkoyama.as_str();
            let tile_name = format!("{}_{}", locn_name, suffix);
            let is_yellow = *suffix == "Yw";
            let revenue = if is_yellow { 20 } else { 40 };
            Tile::new(
                HexColour::Red,
                tile_name,
                vec![
                    Track::mid(HexFace::LowerLeft),
                    Track::mid(HexFace::Bottom),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenueVert(vec![
                    (HexColour::Yellow, 20, is_yellow),
                    (HexColour::Brown, 40, !is_yellow),
                    (HexColour::Grey, 80, false),
                ]),
                HexCorner::Right.in_dir(S60W, 0.5),
            )
            .label(
                Label::MapLocation(locn_name.to_string()),
                HexFace::Top.to_centre(0.1),
            )
            .with_offboard_faces([HexFace::Bottom, HexFace::LowerLeft])
        })
        .collect();

    let naruto: Vec<Tile> = suffixes
        .iter()
        .map(|suffix| {
            let locn_name = Location::NarutoAndAwaji.as_str();
            let tile_name = format!("{}_{}", locn_name, suffix);
            let is_yellow = *suffix == "Yw";
            let revenue = if is_yellow { 20 } else { 40 };
            Tile::new(
                HexColour::Red,
                tile_name,
                vec![
                    Track::mid(HexFace::LowerLeft),
                    Track::mid(HexFace::UpperLeft),
                ],
                vec![City::single(revenue)],
                hex,
            )
            .label(
                Label::PhaseRevenueVert(vec![
                    (HexColour::Yellow, 20, is_yellow),
                    (HexColour::Brown, 40, !is_yellow),
                    (HexColour::Grey, 80, false),
                ]),
                HexFace::Bottom.to_centre(0.1),
            )
            .label(
                Label::MapLocation(locn_name.to_string()),
                HexFace::Top.to_centre(0.1),
            )
            .with_offboard_faces([HexFace::UpperLeft, HexFace::LowerLeft])
        })
        .collect();

    [imabari, sakaide, naruto]
        .concat()
        .into_iter()
        .map(|tile| tile.hide_tile_name())
        .collect()
}
