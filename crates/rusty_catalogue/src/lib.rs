use rusty_hex::*;
use rusty_tile::*;

/// Tiles as per the [18xx Tile Database](http://www.fwtwr.com/18xx/tiles/).
pub fn tile_catalogue(hex: &Hex) -> Vec<Tile> {
    use crate::hex::Direction::*;
    use crate::track::DitShape::*;
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;
    use HexPosition::*;
    use TrackEnd::*;

    // TODO: City/Y labels, revenue ...

    vec![
        Tile::new(
            Yellow,
            "3",
            vec![
                Track::hard_l(Bottom)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::hard_l(Bottom).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        // TODO: label at centre
        .label(Label::Revenue(0), Centre(None)),
        Tile::new(
            Yellow,
            "4",
            vec![
                Track::straight(Bottom)
                    .with_span(0.0, 0.25)
                    .with_dit(End, 10, Bar),
                Track::straight(Bottom).with_span(0.25, 1.0),
            ],
            vec![],
            hex,
        )
        // TODO: nudge towards centre
        .label(Label::Revenue(0), LowerLeft.to_centre(0.3)),
        Tile::new(
            Yellow,
            "5",
            vec![Track::mid(Bottom), Track::mid(LowerRight)],
            vec![City::single(20)],
            hex,
        )
        // TODO: nudge towards centre
        .label(Label::Revenue(0), TopLeft.to_centre(0.3)),
        Tile::new(
            Yellow,
            "6",
            vec![Track::mid(Bottom), Track::mid(UpperRight)],
            vec![City::single(20)],
            hex,
        )
        // TODO: nudge towards centre
        .label(Label::Revenue(0), Top.to_centre(0.3)),
        Tile::new(Yellow, "7", vec![Track::hard_r(Bottom)], vec![], hex),
        Tile::new(Yellow, "8", vec![Track::gentle_r(Bottom)], vec![], hex),
        Tile::new(Yellow, "9", vec![Track::straight(Bottom)], vec![], hex),
        Tile::new(
            Green,
            "14",
            vec![
                Track::mid(Bottom),
                Track::mid(Top),
                Track::mid(LowerLeft),
                Track::mid(UpperRight),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Green,
            "15",
            vec![
                Track::mid(Bottom),
                Track::mid(Top),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Green,
            "16",
            vec![Track::gentle_r(Bottom), Track::gentle_r(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "17",
            vec![Track::gentle_r(Bottom), Track::gentle_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "18",
            vec![Track::straight(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "19",
            vec![Track::gentle_r(LowerLeft), Track::straight(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "20",
            vec![Track::straight(LowerLeft), Track::straight(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "21",
            vec![Track::hard_l(Top), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "22",
            vec![Track::hard_r(Top), Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "23",
            vec![Track::straight(Bottom), Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "24",
            vec![Track::straight(Bottom), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "25",
            vec![Track::gentle_r(Bottom), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "26",
            vec![Track::straight(Bottom), Track::hard_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "27",
            vec![Track::straight(Bottom), Track::hard_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "28",
            vec![Track::gentle_r(Bottom), Track::hard_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "29",
            vec![Track::gentle_l(Bottom), Track::hard_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "30",
            vec![Track::hard_l(Bottom), Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "31",
            vec![Track::hard_r(Bottom), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "39",
            vec![
                Track::gentle_l(Bottom),
                Track::hard_l(Bottom),
                Track::hard_l(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "40",
            vec![
                Track::gentle_l(Bottom),
                Track::gentle_l(UpperLeft),
                Track::gentle_l(UpperRight),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "41",
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::hard_l(Top),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "42",
            vec![
                Track::straight(Bottom),
                Track::gentle_l(Bottom),
                Track::hard_r(Top),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "43",
            vec![
                Track::straight(Bottom),
                Track::gentle_l(Bottom),
                Track::hard_l(LowerLeft),
                Track::gentle_l(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "44",
            vec![
                Track::straight(Bottom),
                Track::hard_l(Bottom),
                Track::hard_l(Top),
                Track::straight(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "45",
            vec![
                Track::gentle_l(UpperLeft),
                Track::hard_r(Top),
                Track::gentle_r(Bottom),
                Track::straight(Bottom),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "46",
            vec![
                Track::gentle_l(UpperLeft),
                Track::hard_l(Top),
                Track::gentle_l(Bottom),
                Track::straight(Bottom),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "47",
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::gentle_l(LowerLeft),
                Track::straight(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Yellow,
            "57",
            vec![Track::mid(Bottom), Track::mid(Top)],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.4)),
        Tile::new(
            Yellow,
            "58",
            vec![
                Track::gentle_r(Bottom)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::gentle_r(Bottom).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.7)),
        Tile::new(
            Brown,
            "63",
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
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "70",
            vec![
                Track::gentle_l(Top),
                Track::hard_l(Top),
                Track::gentle_r(Bottom),
                Track::hard_r(Bottom),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "87",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), Right.to_centre(0.4)),
        Tile::new(
            Green,
            "88",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(LowerRight),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), UpperRight.to_centre(0.4)),
        Tile::new(
            Green,
            "120",
            vec![
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::hard_l(Top).with_span(0.0, 0.5),
                Track::hard_l(Top).with_span(0.5, 1.0),
            ],
            // TODO: Toronto label
            vec![
                City::single_at_corner(60, &Left),
                City::single_at_corner(60, &TopRight),
            ],
            hex,
        )
        .label(Label::City("T".to_string()), LowerRight.to_centre(0.3))
        .label(Label::Revenue(0), Bottom.to_centre(1.0)),
        Tile::new(
            Brown,
            "122",
            vec![
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::hard_l(Top).with_span(0.0, 0.5),
                Track::hard_l(Top).with_span(0.5, 1.0),
            ],
            // TODO: Toronto label
            vec![
                City::double_at_corner(80, &Left),
                City::double_at_corner(80, &TopRight),
            ],
            hex,
        )
        .label(
            Label::City("T".to_string()),
            BottomRight.nudge(Direction::N, 0.2),
        )
        .label(Label::Revenue(0), Bottom.to_centre(1.0)),
        Tile::new(
            Grey,
            "124",
            // NOTE: here we have a single track segment that *PASSES THROUGH*
            // a city/token space ...
            vec![
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            // TODO: Toronto label
            vec![City::quad(100)],
            hex,
        )
        .label(Label::City("T".to_string()), TopRight)
        .label(Label::Revenue(0), Right),
        Tile::new(
            Yellow,
            "201",
            vec![Track::mid(Bottom), Track::mid(LowerRight)],
            // TODO: Y label
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.3))
        .label(Label::Y, LowerLeft.to_centre(0.4)),
        Tile::new(
            Yellow,
            "202",
            vec![Track::mid(Bottom), Track::mid(UpperRight)],
            // TODO: Y label
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.3))
        .label(Label::Y, LowerLeft.to_centre(0.4)),
        Tile::new(
            Green,
            "204",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), LowerLeft.to_centre(0.5)),
        Tile::new(
            Green,
            "207",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            // TODO: Y label
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15))
        .label(Label::Y, TopRight.to_centre(0.1)),
        Tile::new(
            Green,
            "208",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperRight),
                Track::mid(Top),
            ],
            // TODO: Y label
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.15))
        .label(Label::Y, TopLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "611",
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
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Green,
            "619",
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Yellow,
            "621",
            vec![
                Track::straight(Bottom).with_span(0.0, 0.5),
                Track::straight(Bottom).with_span(0.5, 1.0),
            ],
            // TODO: Y label
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.3))
        .label(Label::Y, LowerLeft.to_centre(0.4)),
        Tile::new(
            Green,
            "622",
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            // TODO: Y label
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15))
        .label(Label::Y, BottomLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "623",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            // TODO: Y label
            vec![City::double(50)],
            hex,
        )
        .label(Label::Y, TopRight.to_centre(0.1))
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Green,
            "624",
            vec![Track::hard_l(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "625",
            vec![Track::hard_r(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "626",
            vec![Track::hard_r(LowerRight), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "637",
            vec![
                Track::hard_l(Bottom).with_span(0.0, 0.5),
                Track::hard_l(Bottom).with_span(0.5, 1.0),
                Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                Track::hard_l(UpperRight).with_span(0.0, 0.5),
                Track::hard_l(UpperRight).with_span(0.5, 1.0),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_corner(50, &BottomLeft),
                City::single_at_corner(50, &TopLeft),
                City::single_at_corner(50, &Right),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), Left.to_centre(0.1))
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Grey,
            "639",
            // NOTE: here we have a single track segment that *PASSES THROUGH*
            // a city/token space ...
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            // TODO: Montreal label
            vec![City::quad(100)],
            hex,
        )
        .label(Label::City("M".to_string()), TopRight)
        .label(Label::Revenue(0), Right),
        Tile::new(
            Brown,
            "801",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            // TODO: Y label
            vec![City::double(50)],
            hex,
        )
        .label(Label::Y, Right.to_centre(0.1))
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Brown,
            "911",
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
        .label(Label::Revenue(0), UpperLeft.to_centre(0.5)),
        Tile::new(
            Green,
            "X1",
            vec![
                Track::straight(Bottom).with_span(0.0, 0.9),
                Track::straight(Bottom).with_span(0.9, 1.0),
                Track::straight(LowerLeft).with_span(0.0, 0.1),
                Track::straight(LowerLeft).with_span(0.1, 1.0),
                Track::straight(LowerRight).with_span(0.0, 0.1),
                Track::straight(LowerRight).with_span(0.1, 1.0),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &LowerLeft),
                City::single_at_face(50, &LowerRight),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        // TODO: nudge isn't doing anything!!!
        // Need to scale by hex.max_d !!!
        .label(Label::Revenue(0), TopLeft.nudge(Direction::SSW, 0.16)),
        Tile::new(
            Green,
            "X2",
            vec![
                Track::gentle_r(LowerLeft).with_span(0.0, 0.9),
                Track::gentle_r(LowerLeft).with_span(0.9, 1.0),
                Track::gentle_l(UpperLeft).with_span(0.0, 0.1),
                Track::gentle_l(UpperLeft).with_span(0.1, 1.0),
                Track::straight(Bottom).with_span(0.0, 0.9),
                Track::straight(Bottom).with_span(0.9, 1.0),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &UpperLeft),
                City::single_at_face(50, &LowerRight),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        .label(Label::Revenue(0), Right.nudge(Direction::NW, 0.12)),
        Tile::new(
            Green,
            "X3",
            vec![
                Track::gentle_l(Top).with_span(0.0, 0.1),
                Track::gentle_l(Top).with_span(0.1, 1.0),
                Track::gentle_r(Bottom).with_span(0.0, 0.1),
                Track::gentle_r(Bottom).with_span(0.1, 1.0),
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &Bottom),
                City::single_at_corner(50, &Left),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomLeft.nudge(Direction::NW, 0.1),
        )
        // TODO: nudge isn't doing anything!!!
        // Need to scale by hex.max_d !!!
        .label(Label::Revenue(0), TopLeft.nudge(Direction::SSW, 0.16)),
        Tile::new(
            Green,
            "X4",
            vec![
                Track::straight(Top).with_span(0.0, 0.1),
                Track::straight(Top).with_span(0.1, 1.0),
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::hard_r(LowerRight).with_span(0.0, 0.5),
                Track::hard_r(LowerRight).with_span(0.5, 1.0),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_corner(50, &Left),
                City::single_at_corner(50, &Right),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomRight.nudge(Direction::N, 0.2),
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "X5",
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
            // TODO: Montreal label
            vec![
                City::single_at_face(70, &Top),
                City::double(70).nudge(Direction::S, 0.1),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        .label(Label::Revenue(0), Left.to_centre(0.1)),
        Tile::new(
            Brown,
            "X6",
            vec![
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::mid(Top),
                Track::mid(Bottom),
                Track::mid(LowerRight),
                Track::mid(UpperRight),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_corner(70, &Left),
                City::double(70)
                    .rotate(Rotation::Cw90)
                    .nudge(Direction::E, 0.1),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "X7",
            vec![
                Track::gentle_l(UpperLeft).with_span(0.0, 0.9),
                Track::gentle_l(UpperLeft).with_span(0.9, 1.0),
                Track::gentle_r(LowerLeft).with_span(0.0, 0.5),
                Track::gentle_l(LowerRight).with_span(0.0, 0.5),
                Track::straight(Top).with_span(0.0, 0.65),
                Track::straight(Bottom).with_span(0.0, 0.35),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(70, &UpperRight),
                City::double(70).nudge(Direction::S, 0.3),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), Left)
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Grey,
            "X8",
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
        // NOTE: add city and revenue labels.
        .label(Label::City("O".to_string()), Left)
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(
            Yellow,
            "IN10",
            vec![
                // TODO! dit is at the wrong location for gentle_l !!!
                // FIXED! But how to test?!?
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
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Green,
            "IN11",
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
                    .nudge(Direction::NEE, 0.2),
                City::single_at_face(30, &UpperRight)
                    .nudge(Direction::SWW, 0.2),
            ],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        // TODO: mark the following tiles in some way so that their names are
        // not shown on the map?
        Tile::new(
            Red,
            "Red1",
            vec![
                Track::straight(LowerRight).with_span(0.0, 0.5),
                Track::straight(UpperRight).with_span(0.0, 0.5),
            ],
            vec![City::single(50)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Red,
            "Red2",
            vec![Track::straight(LowerRight).with_span(0.0, 0.5)],
            vec![City::single(50)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Red,
            "Red3",
            vec![
                Track::straight(UpperLeft).with_span(0.0, 0.5),
                Track::straight(UpperRight).with_span(0.0, 0.5),
            ],
            vec![City::single(50)],
            hex,
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(Grey, "Grey1", vec![Track::hard_r(Bottom)], vec![], hex),
        Tile::new(Grey, "Grey2", vec![Track::gentle_r(Bottom)], vec![], hex),
        Tile::new(
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
        .label(Label::Revenue(0), BottomRight.to_centre(0.1)),
        Tile::new(
            Empty,
            "EmptyTown",
            // vec![],
            // vec![City::central_dit(0)],
            vec![Track::straight(Bottom)
                .with_span(0.5, 0.5)
                .with_dit(End, 10, Circle)],
            vec![],
            hex,
        ),
        Tile::new(Empty, "EmptyCity", vec![], vec![City::single(0)], hex),
        Tile::new(Empty, "EmptyCityY", vec![], vec![City::single(0)], hex)
            .label(Label::Y, Bottom.to_centre(0.3)),
        Tile::new(Empty, "Ottawa1", vec![], vec![City::single(0)], hex)
            .label(Label::City("O".to_string()), Right.to_centre(0.1))
            .label(Label::Y, Left.to_centre(0.1)),
        Tile::new(
            Yellow,
            "M0",
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
        .label(Label::Revenue(0), UpperRight.to_centre(0.3)),
        Tile::new(
            Yellow,
            "T0",
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
        .label(Label::Revenue(0), BottomRight.to_centre(0.1)),
    ]
}
