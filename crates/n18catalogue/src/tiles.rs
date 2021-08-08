use n18hex::*;
use n18tile::*;

use super::TileFn;

/// Returns a tile-creation closure for each tile in the master catalogue.
pub(crate) fn all_tile_fns() -> Vec<(&'static str, Box<TileFn>)> {
    use n18tile::DitShape::*;
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;
    use HexPosition::*;
    use TrackEnd::*;

    vec![
        (
            "3",
            Box::new(|hex: &Hex, name: &str| {
                Tile::new(
                    Yellow,
                    name,
                    vec![
                        Track::hard_l(Bottom)
                            .with_span(0.0, 0.5)
                            .with_dit(End, 10, Bar),
                        Track::hard_l(Bottom).with_span(0.5, 1.0),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), Centre(None))
            }) as Box<TileFn>,
        ),
        (
            "4",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![
                        Track::straight(Bottom)
                            .with_span(0.0, 0.25)
                            .with_dit(End, 10, Bar),
                        Track::straight(Bottom).with_span(0.25, 1.0),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), LowerLeft.to_centre(0.3))
            }),
        ),
        (
            "5",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::mid(Bottom), Track::mid(LowerRight)],
                    vec![City::single(20)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.3))
            }),
        ),
        (
            "6",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::mid(Bottom), Track::mid(UpperRight)],
                    vec![City::single(20)],
                    hex,
                )
                .label(Label::Revenue(0), Top.to_centre(0.2))
            }),
        ),
        (
            "7",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::hard_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "8",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::gentle_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "9",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::straight(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "14",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(Top),
                        Track::mid(LowerLeft),
                        Track::mid(UpperRight),
                    ],
                    vec![City::double(30)],
                    hex,
                )
                .label(Label::Revenue(0), TopRight.to_centre(0.15))
            }),
        ),
        (
            "15",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(Top),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                    ],
                    vec![City::double(30)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.15))
            }),
        ),
        (
            "16",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::gentle_r(Bottom), Track::gentle_r(LowerLeft)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "17",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::gentle_r(Bottom), Track::gentle_l(LowerLeft)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "18",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::straight(Bottom), Track::hard_l(LowerLeft)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "19",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::gentle_r(LowerLeft), Track::straight(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "20",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::straight(LowerLeft), Track::straight(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "21",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_l(Top), Track::gentle_l(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "22",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_r(Top), Track::gentle_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "23",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::straight(Bottom), Track::gentle_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "24",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::straight(Bottom), Track::gentle_l(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "25",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::gentle_r(Bottom), Track::gentle_l(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "26",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::straight(Bottom), Track::hard_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "27",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::straight(Bottom), Track::hard_l(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "28",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::gentle_r(Bottom), Track::hard_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "29",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::gentle_l(Bottom), Track::hard_l(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "30",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_l(Bottom), Track::gentle_r(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "31",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_r(Bottom), Track::gentle_l(Bottom)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "39",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::gentle_l(Bottom),
                        Track::hard_l(Bottom),
                        Track::hard_l(LowerLeft),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "40",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::gentle_l(Bottom),
                        Track::gentle_l(UpperLeft),
                        Track::gentle_l(UpperRight),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "41",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::straight(Bottom),
                        Track::gentle_r(Bottom),
                        Track::hard_l(Top),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "42",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::straight(Bottom),
                        Track::gentle_l(Bottom),
                        Track::hard_r(Top),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "43",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::straight(Bottom),
                        Track::gentle_l(Bottom),
                        Track::hard_l(LowerLeft),
                        Track::gentle_l(LowerLeft),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "44",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::straight(Bottom),
                        Track::hard_l(Bottom),
                        Track::hard_l(Top),
                        Track::straight(LowerLeft),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "45",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::gentle_l(UpperLeft),
                        Track::hard_r(Top),
                        Track::gentle_r(Bottom),
                        Track::straight(Bottom),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "46",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::gentle_l(UpperLeft),
                        Track::hard_l(Top),
                        Track::gentle_l(Bottom),
                        Track::straight(Bottom),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "47",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::straight(Bottom),
                        Track::gentle_r(Bottom),
                        Track::gentle_l(LowerLeft),
                        Track::straight(LowerLeft),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "57",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::mid(Bottom), Track::mid(Top)],
                    vec![City::single(20)],
                    hex,
                )
                .label(Label::Revenue(0), UpperLeft.to_centre(0.2))
            }),
        ),
        (
            "58",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![
                        Track::gentle_r(Bottom)
                            .with_span(0.0, 0.5)
                            .with_dit(End, 10, Bar),
                        Track::gentle_r(Bottom).with_span(0.5, 1.0),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), UpperLeft.to_centre(0.5))
            }),
        ),
        (
            "63",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                        Track::mid(LowerRight),
                    ],
                    vec![City::double(40)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.1))
            }),
        ),
        (
            "70",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::gentle_l(Top),
                        Track::hard_l(Top),
                        Track::gentle_r(Bottom),
                        Track::hard_r(Bottom),
                    ],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "87",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom).with_dit(End, 10, Circle),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), Right.to_centre(0.4))
            }),
        ),
        (
            "88",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom).with_dit(End, 10, Circle),
                        Track::mid(LowerRight),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), UpperRight.to_centre(0.2))
            }),
        ),
        (
            "120",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                        Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                        Track::hard_l(Top).with_span(0.0, 0.5),
                        Track::hard_l(Top).with_span(0.5, 1.0),
                    ],
                    vec![
                        City::single_at_corner(60, &Left),
                        City::single_at_corner(60, &TopRight),
                    ],
                    hex,
                )
                .label(
                    Label::City("T".to_string()),
                    LowerRight.in_dir(Direction::W, 0.15),
                )
                .label(Label::Revenue(0), Centre(None))
            }),
        ),
        (
            "122",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                        Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                        Track::hard_l(Top).with_span(0.0, 0.5),
                        Track::hard_l(Top).with_span(0.5, 1.0),
                    ],
                    vec![
                        City::double_at_corner(80, &Left),
                        City::double_at_corner(80, &TopRight),
                    ],
                    hex,
                )
                .label(
                    Label::City("T".to_string()),
                    BottomRight.in_dir(Direction::N, 0.2),
                )
                .label(Label::Revenue(0), Centre(None))
            }),
        ),
        (
            "124",
            Box::new(|hex, name| {
                Tile::new(
                    Grey,
                    name,
                    vec![
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                    ],
                    vec![City::quad(100)],
                    hex,
                )
                .label(Label::City("T".to_string()), TopRight.to_centre(0.05))
                .label(Label::Revenue(0), Right.to_centre(0.08))
            }),
        ),
        (
            "201",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::mid(Bottom), Track::mid(LowerRight)],
                    vec![City::single(30)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.25))
                .label(Label::Y, LowerLeft.to_centre(0.2))
            }),
        ),
        (
            "202",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![Track::mid(Bottom), Track::mid(UpperRight)],
                    vec![City::single(30)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.25))
                .label(Label::Y, LowerLeft.to_centre(0.2))
            }),
        ),
        (
            "204",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom).with_dit(End, 10, Circle),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), LowerLeft.to_centre(0.25))
            }),
        ),
        (
            "207",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                    ],
                    vec![City::double(40)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.15))
                .label(Label::Y, TopRight.to_centre(0.15))
            }),
        ),
        (
            "208",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperRight),
                        Track::mid(Top),
                    ],
                    vec![City::double(40)],
                    hex,
                )
                .label(Label::Revenue(0), BottomLeft.to_centre(0.15))
                .label(Label::Y, TopLeft.to_centre(0.15))
            }),
        ),
        (
            "611",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                    ],
                    vec![City::double(40)],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.125))
            }),
        ),
        (
            "619",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                    ],
                    vec![City::double(30)],
                    hex,
                )
                .label(Label::Revenue(0), TopRight.to_centre(0.15))
            }),
        ),
        (
            "621",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![
                        Track::straight(Bottom).with_span(0.0, 0.5),
                        Track::straight(Bottom).with_span(0.5, 1.0),
                    ],
                    vec![City::single(30)],
                    hex,
                )
                .label(Label::Revenue(0), UpperLeft.to_centre(0.1))
                .label(Label::Y, LowerLeft.to_centre(0.2))
            }),
        ),
        (
            "622",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                    ],
                    vec![City::double(40)],
                    hex,
                )
                .label(Label::Revenue(0), TopRight.to_centre(0.15))
                .label(Label::Y, BottomLeft.to_centre(0.15))
            }),
        ),
        (
            "623",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                        Track::mid(LowerRight),
                    ],
                    vec![City::double(50)],
                    hex,
                )
                .label(Label::Y, TopRight.to_centre(0.15))
                .label(Label::Revenue(0), TopLeft.to_centre(0.15))
            }),
        ),
        (
            "624",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_l(Bottom), Track::hard_l(LowerLeft)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "625",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_r(Bottom), Track::hard_l(LowerLeft)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "626",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![Track::hard_r(LowerRight), Track::hard_l(LowerLeft)],
                    vec![],
                    hex,
                )
            }),
        ),
        (
            "637",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::hard_l(Bottom).with_span(0.0, 0.5),
                        Track::hard_l(Bottom).with_span(0.5, 1.0),
                        Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                        Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                        Track::hard_l(UpperRight).with_span(0.0, 0.5),
                        Track::hard_l(UpperRight).with_span(0.5, 1.0),
                    ],
                    vec![
                        City::single_at_corner(50, &BottomLeft),
                        City::single_at_corner(50, &TopLeft),
                        City::single_at_corner(50, &Right),
                    ],
                    hex,
                )
                .label(Label::City("M".to_string()), Left.to_centre(0.25))
                .label(Label::Revenue(0), TopRight.to_centre(0.15))
            }),
        ),
        (
            "639",
            Box::new(|hex, name| {
                Tile::new(
                    Grey,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                        Track::mid(LowerRight),
                    ],
                    vec![City::quad(100)],
                    hex,
                )
                .label(Label::City("M".to_string()), TopRight.to_centre(0.05))
                .label(Label::Revenue(0), Right.to_centre(0.08))
            }),
        ),
        (
            "801",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                    ],
                    vec![City::double(50)],
                    hex,
                )
                .label(Label::Y, Right.to_centre(0.2))
                .label(Label::Revenue(0), TopRight.to_centre(0.15))
            }),
        ),
        (
            "911",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::mid(Bottom).with_dit(End, 10, Circle),
                        Track::mid(LowerLeft),
                        Track::mid(Top),
                        Track::mid(UpperRight),
                        Track::mid(LowerRight),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), UpperLeft.to_centre(0.25))
            }),
        ),
        (
            "X1",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::straight(Bottom).with_span(0.0, 0.9),
                        Track::straight(Bottom).with_span(0.9, 1.0),
                        Track::straight(LowerLeft).with_span(0.0, 0.1),
                        Track::straight(LowerLeft).with_span(0.1, 1.0),
                        Track::straight(LowerRight).with_span(0.0, 0.1),
                        Track::straight(LowerRight).with_span(0.1, 1.0),
                    ],
                    vec![
                        City::single_at_face(50, &Top),
                        City::single_at_face(50, &LowerLeft),
                        City::single_at_face(50, &LowerRight),
                    ],
                    hex,
                )
                .label(
                    Label::City("M".to_string()),
                    BottomLeft.in_dir(Direction::E, 0.05),
                )
                .label(
                    Label::Revenue(0),
                    TopLeft.in_dir(Direction::S30W, 0.16),
                )
            }),
        ),
        (
            "X2",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::gentle_r(LowerLeft).with_span(0.0, 0.9),
                        Track::gentle_r(LowerLeft).with_span(0.9, 1.0),
                        Track::gentle_l(UpperLeft).with_span(0.0, 0.1),
                        Track::gentle_l(UpperLeft).with_span(0.1, 1.0),
                        Track::straight(Bottom).with_span(0.0, 0.9),
                        Track::straight(Bottom).with_span(0.9, 1.0),
                    ],
                    vec![
                        City::single_at_face(50, &Top),
                        City::single_at_face(50, &UpperLeft),
                        City::single_at_face(50, &LowerRight),
                    ],
                    hex,
                )
                .label(
                    Label::City("M".to_string()),
                    BottomLeft.in_dir(Direction::E, 0.05),
                )
                .label(Label::Revenue(0), Right.in_dir(Direction::N60W, 0.15))
            }),
        ),
        (
            "X3",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::gentle_l(Top).with_span(0.0, 0.1),
                        Track::gentle_l(Top).with_span(0.1, 1.0),
                        Track::gentle_r(Bottom).with_span(0.0, 0.1),
                        Track::gentle_r(Bottom).with_span(0.1, 1.0),
                        Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                        Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                    ],
                    vec![
                        City::single_at_face(50, &Top),
                        City::single_at_face(50, &Bottom),
                        City::single_at_corner(50, &Left),
                    ],
                    hex,
                )
                .label(
                    Label::City("M".to_string()),
                    BottomLeft.in_dir(Direction::N30W, 0.1),
                )
                .label(
                    Label::Revenue(0),
                    TopLeft.in_dir(Direction::S30W, 0.16),
                )
            }),
        ),
        (
            "X4",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::straight(Top).with_span(0.0, 0.1),
                        Track::straight(Top).with_span(0.1, 1.0),
                        Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                        Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                        Track::hard_r(LowerRight).with_span(0.0, 0.5),
                        Track::hard_r(LowerRight).with_span(0.5, 1.0),
                    ],
                    vec![
                        City::single_at_face(50, &Top),
                        City::single_at_corner(50, &Left),
                        City::single_at_corner(50, &Right),
                    ],
                    hex,
                )
                .label(
                    Label::City("M".to_string()),
                    BottomRight.in_dir(Direction::N, 0.2),
                )
                .label(Label::Revenue(0), BottomLeft.to_centre(0.1))
            }),
        ),
        (
            "X5",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::straight(Top).with_span(0.0, 0.1),
                        Track::straight(Top)
                            .with_span(0.1, 1.0)
                            .with_clip(0.3625, 0.75),
                        Track::mid(UpperLeft),
                        Track::mid(LowerLeft),
                        Track::mid(LowerRight),
                        Track::mid(UpperRight),
                    ],
                    vec![
                        City::single_at_face(70, &Top),
                        City::double(70).in_dir(Direction::S, 0.1),
                    ],
                    hex,
                )
                .label(
                    Label::City("M".to_string()),
                    BottomLeft.in_dir(Direction::E, 0.05),
                )
                .label(Label::Revenue(0), Left.to_centre(0.1))
            }),
        ),
        (
            "X6",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                        Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                        Track::mid(Top),
                        Track::mid(Bottom),
                        Track::mid(LowerRight),
                        Track::mid(UpperRight),
                    ],
                    vec![
                        City::single_at_corner(70, &Left),
                        City::double(70)
                            .rotate(Rotation::Cw90)
                            .in_dir(Direction::E, 0.1),
                    ],
                    hex,
                )
                .label(
                    Label::City("M".to_string()),
                    BottomLeft.in_dir(Direction::E, 0.05),
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.15))
            }),
        ),
        (
            "X7",
            Box::new(|hex, name| {
                Tile::new(
                    Brown,
                    name,
                    vec![
                        Track::gentle_l(UpperLeft).with_span(0.0, 0.9),
                        Track::gentle_l(UpperLeft).with_span(0.9, 1.0),
                        Track::gentle_r(LowerLeft).with_span(0.0, 0.5),
                        Track::gentle_l(LowerRight).with_span(0.0, 0.5),
                        Track::straight(Top).with_span(0.0, 0.65),
                        Track::straight(Bottom).with_span(0.0, 0.35),
                    ],
                    vec![
                        City::single_at_face(70, &UpperRight),
                        City::double(70).in_dir(Direction::S, 0.3),
                    ],
                    hex,
                )
                .label(Label::City("M".to_string()), Left.to_centre(0.15))
                .label(Label::Revenue(0), TopLeft.to_centre(0.15))
            }),
        ),
        (
            "X8",
            Box::new(|hex, name| {
                Tile::new(
                    Grey,
                    name,
                    vec![
                        Track::mid(Bottom),
                        Track::mid(LowerLeft),
                        Track::mid(UpperLeft),
                        Track::mid(Top),
                        Track::mid(LowerRight),
                        Track::mid(UpperRight),
                    ],
                    vec![City::triple(60).rotate(Rotation::HalfTurn)],
                    hex,
                )
                .label(Label::City("O".to_string()), Left.to_centre(0.15))
                .label(Label::Revenue(0), BottomLeft.to_centre(0.1))
            }),
        ),
        (
            "IN10",
            Box::new(|hex, name| {
                Tile::new(
                    Yellow,
                    name,
                    vec![
                        Track::gentle_l(Bottom)
                            .with_span(0.0, 0.85)
                            .with_dit(End, 30, Bar),
                        Track::gentle_l(Bottom).with_span(0.85, 1.0),
                        Track::gentle_r(Bottom)
                            .with_span(0.0, 0.85)
                            .with_dit(End, 30, Bar),
                        Track::gentle_r(Bottom).with_span(0.85, 1.0),
                        Track::straight(UpperLeft).with_span(0.125, 1.0),
                        Track::gentle_l(Top),
                    ],
                    vec![],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.1))
            }),
        ),
        (
            "IN11",
            Box::new(|hex, name| {
                Tile::new(
                    Green,
                    name,
                    vec![
                        Track::straight(LowerRight),
                        Track::gentle_r(LowerRight).with_span(0.0, 0.5),
                        Track::gentle_r(LowerRight).with_span(0.5, 1.0),
                        Track::gentle_l(Bottom).with_span(0.0, 0.5),
                        Track::gentle_l(Bottom).with_span(0.5, 1.0),
                        Track::straight(Bottom),
                    ],
                    vec![
                        City::single_at_face(30, &LowerLeft)
                            .in_dir(Direction::N60E, 0.2),
                        City::single_at_face(30, &UpperRight)
                            .in_dir(Direction::S60W, 0.2),
                    ],
                    hex,
                )
                .label(Label::Revenue(0), TopLeft.to_centre(0.1))
            }),
        ),
    ]
}
