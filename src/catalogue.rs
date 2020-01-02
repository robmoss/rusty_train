use crate::prelude::*;
use cairo::Context;

/// Tiles as per the [18xx Tile Database](http://www.fwtwr.com/18xx/tiles/).
pub fn tile_catalogue(hex: &Hex, ctx: &Context) -> Tiles {
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;
    use HexPosition::*;

    // TODO: City/Y labels, revenue ...

    vec![
        Tile::new(
            Yellow,
            "3".to_string(),
            vec![Track::hard_l(Bottom).with_dit(0.5, 10)],
            vec![],
            ctx,
            hex,
        )
        // TODO: label at centre
        .label(Label::Revenue(0), Centre(None)),
        Tile::new(
            Yellow,
            "4".to_string(),
            vec![Track::straight(Bottom).with_dit(0.25, 10)],
            vec![],
            ctx,
            hex,
        )
        // TODO: nudge towards centre
        .label(Label::Revenue(0), LowerLeft.to_centre(0.3)),
        Tile::new(
            Yellow,
            "5".to_string(),
            vec![Track::mid(Bottom), Track::mid(LowerRight)],
            vec![City::single(20)],
            ctx,
            hex,
        )
        // TODO: nudge towards centre
        .label(Label::Revenue(0), TopLeft.to_centre(0.3)),
        Tile::new(
            Yellow,
            "6".to_string(),
            vec![Track::mid(Bottom), Track::mid(UpperRight)],
            vec![City::single(20)],
            ctx,
            hex,
        )
        // TODO: nudge towards centre
        .label(Label::Revenue(0), Top.to_centre(0.3)),
        Tile::new(
            Yellow,
            "7".to_string(),
            vec![Track::hard_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Yellow,
            "8".to_string(),
            vec![Track::gentle_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Yellow,
            "9".to_string(),
            vec![Track::straight(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "14".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(Top),
                Track::mid(LowerLeft),
                Track::mid(UpperRight),
            ],
            vec![City::double(30)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Green,
            "15".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(Top),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
            ],
            vec![City::double(30)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Green,
            "16".to_string(),
            vec![Track::gentle_r(Bottom), Track::gentle_r(LowerLeft)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "17".to_string(),
            vec![Track::gentle_r(Bottom), Track::gentle_l(LowerLeft)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "18".to_string(),
            vec![Track::straight(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "19".to_string(),
            vec![Track::gentle_r(LowerLeft), Track::straight(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "20".to_string(),
            vec![Track::straight(LowerLeft), Track::straight(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "21".to_string(),
            vec![Track::hard_l(Top), Track::gentle_l(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "22".to_string(),
            vec![Track::hard_r(Top), Track::gentle_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "23".to_string(),
            vec![Track::straight(Bottom), Track::gentle_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "24".to_string(),
            vec![Track::straight(Bottom), Track::gentle_l(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "25".to_string(),
            vec![Track::gentle_r(Bottom), Track::gentle_l(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "26".to_string(),
            vec![Track::straight(Bottom), Track::hard_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "27".to_string(),
            vec![Track::straight(Bottom), Track::hard_l(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "28".to_string(),
            vec![Track::gentle_r(Bottom), Track::hard_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "29".to_string(),
            vec![Track::gentle_l(Bottom), Track::hard_l(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "30".to_string(),
            vec![Track::hard_l(Bottom), Track::gentle_r(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "31".to_string(),
            vec![Track::hard_r(Bottom), Track::gentle_l(Bottom)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "39".to_string(),
            vec![
                Track::gentle_l(Bottom),
                Track::hard_l(Bottom),
                Track::hard_l(LowerLeft),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "40".to_string(),
            vec![
                Track::gentle_l(Bottom),
                Track::gentle_l(UpperLeft),
                Track::gentle_l(UpperRight),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "41".to_string(),
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::hard_l(Top),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "42".to_string(),
            vec![
                Track::straight(Bottom),
                Track::gentle_l(Bottom),
                Track::hard_r(Top),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "43".to_string(),
            vec![
                Track::straight(Bottom),
                Track::gentle_l(Bottom),
                Track::hard_l(LowerLeft),
                Track::gentle_l(LowerLeft),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "44".to_string(),
            vec![
                Track::straight(Bottom),
                Track::hard_l(Bottom),
                Track::hard_l(Top),
                Track::straight(LowerLeft),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "45".to_string(),
            vec![
                Track::gentle_l(UpperLeft),
                Track::hard_r(Top),
                Track::gentle_r(Bottom),
                Track::straight(Bottom),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "46".to_string(),
            vec![
                Track::gentle_l(UpperLeft),
                Track::hard_l(Top),
                Track::gentle_l(Bottom),
                Track::straight(Bottom),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Brown,
            "47".to_string(),
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::gentle_l(LowerLeft),
                Track::straight(LowerLeft),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Yellow,
            "57".to_string(),
            vec![Track::mid(Bottom), Track::mid(Top)],
            vec![City::single(20)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.4)),
        Tile::new(
            Yellow,
            "58".to_string(),
            vec![Track::gentle_r(Bottom).with_dit(0.5, 10)],
            vec![],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.7)),
        Tile::new(
            Brown,
            "63".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            vec![City::double(40)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "70".to_string(),
            vec![
                Track::gentle_l(Top),
                Track::hard_l(Top),
                Track::gentle_r(Bottom),
                Track::hard_r(Bottom),
            ],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "87".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![City::central_dit(10)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), Right.to_centre(0.4)),
        Tile::new(
            Green,
            "88".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerRight),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![City::central_dit(10)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), UpperRight.to_centre(0.4)),
        Tile::new(
            Green,
            "120".to_string(),
            // NOTE: here we have a single track segment that *PASSES THROUGH*
            // a city/token space ...
            vec![Track::hard_l(LowerLeft), Track::hard_l(Top)],
            // TODO: Toronto label
            vec![
                City::single_at_corner(60, &Left),
                City::single_at_corner(60, &TopRight),
            ],
            ctx,
            hex,
        )
        .label(Label::City("T".to_string()), LowerRight.to_centre(0.3))
        .label(Label::Revenue(0), Bottom.to_centre(1.0)),
        Tile::new(
            Brown,
            "122".to_string(),
            // NOTE: here we have a single track segment that *PASSES THROUGH*
            // a city/token space ...
            vec![Track::hard_l(LowerLeft), Track::hard_l(Top)],
            // TODO: Toronto label
            vec![
                City::double_at_corner(80, &Left),
                City::double_at_corner(80, &TopRight),
            ],
            ctx,
            hex,
        )
        .label(
            Label::City("T".to_string()),
            BottomRight.nudge(Direction::N, 0.2),
        )
        .label(Label::Revenue(0), Bottom.to_centre(1.0)),
        Tile::new(
            Grey,
            "124".to_string(),
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
            ctx,
            hex,
        )
        .label(Label::City("T".to_string()), TopRight)
        .label(Label::Revenue(0), Right),
        Tile::new(
            Yellow,
            "201".to_string(),
            vec![Track::mid(Bottom), Track::mid(LowerRight)],
            // TODO: Y label
            vec![City::single(30)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.3))
        .label(Label::Y, LowerLeft.to_centre(0.4)),
        Tile::new(
            Yellow,
            "202".to_string(),
            vec![Track::mid(Bottom), Track::mid(UpperRight)],
            // TODO: Y label
            vec![City::single(30)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.3))
        .label(Label::Y, LowerLeft.to_centre(0.4)),
        Tile::new(
            Green,
            "204".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::central_dit(10)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), LowerLeft.to_centre(0.5)),
        Tile::new(
            Green,
            "207".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            // TODO: Y label
            vec![City::double(40)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15))
        .label(Label::Y, TopRight.to_centre(0.1)),
        Tile::new(
            Green,
            "208".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperRight),
                Track::mid(Top),
            ],
            // TODO: Y label
            vec![City::double(40)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.15))
        .label(Label::Y, TopLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "611".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::double(40)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Green,
            "619".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::double(30)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Yellow,
            "621".to_string(),
            vec![Track::straight(Bottom)],
            // TODO: Y label
            vec![City::single(30)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.3))
        .label(Label::Y, LowerLeft.to_centre(0.4)),
        Tile::new(
            Green,
            "622".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            // TODO: Y label
            vec![City::double(40)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15))
        .label(Label::Y, BottomLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "623".to_string(),
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
            ctx,
            hex,
        )
        .label(Label::Y, TopRight.to_centre(0.1))
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Green,
            "624".to_string(),
            vec![Track::hard_l(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "625".to_string(),
            vec![Track::hard_r(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "626".to_string(),
            vec![Track::hard_r(LowerRight), Track::hard_l(LowerLeft)],
            vec![],
            ctx,
            hex,
        ),
        Tile::new(
            Green,
            "637".to_string(),
            vec![
                Track::hard_l(Bottom),
                Track::hard_l(UpperLeft),
                Track::hard_l(UpperRight),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_corner(50, &BottomLeft),
                City::single_at_corner(50, &TopLeft),
                City::single_at_corner(50, &Right),
            ],
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), Left.to_centre(0.1))
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Grey,
            "639".to_string(),
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
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), TopRight)
        .label(Label::Revenue(0), Right),
        Tile::new(
            Brown,
            "801".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            // TODO: Y label
            vec![City::double(50)],
            ctx,
            hex,
        )
        .label(Label::Y, Right.to_centre(0.1))
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Brown,
            "911".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            vec![City::central_dit(10)],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.5)),
        Tile::new(
            Green,
            "X1".to_string(),
            vec![
                Track::straight(Bottom),
                Track::straight(LowerLeft),
                Track::straight(LowerRight),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &LowerLeft),
                City::single_at_face(50, &LowerRight),
            ],
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        // TODO: nudge isn't doing anything!!!
        // Need to scale by hex.max_d !!!
        .label(Label::Revenue(0), TopLeft.nudge(Direction::SSW, 0.16)),
        Tile::new(
            Green,
            "X2".to_string(),
            vec![
                Track::gentle_r(LowerLeft),
                Track::gentle_l(UpperLeft),
                Track::straight(Bottom),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &UpperLeft),
                City::single_at_face(50, &LowerRight),
            ],
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        .label(Label::Revenue(0), Right.nudge(Direction::NW, 0.12)),
        Tile::new(
            Green,
            "X3".to_string(),
            vec![
                Track::gentle_l(Top),
                Track::gentle_r(Bottom),
                Track::hard_l(LowerLeft),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &Bottom),
                City::single_at_corner(50, &Left),
            ],
            ctx,
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
            "X4".to_string(),
            vec![
                Track::straight(Top),
                Track::hard_l(LowerLeft),
                Track::hard_r(LowerRight),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(50, &Top),
                City::single_at_corner(50, &Left),
                City::single_at_corner(50, &Right),
            ],
            ctx,
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomRight.nudge(Direction::N, 0.2),
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "X5".to_string(),
            vec![
                Track::straight(Top).with_clip(0.3625, 0.75),
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
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        .label(Label::Revenue(0), Left.to_centre(0.1)),
        Tile::new(
            Brown,
            "X6".to_string(),
            vec![
                Track::hard_l(LowerLeft),
                Track::mid(Top),
                Track::mid(Bottom),
                Track::mid(LowerRight),
                Track::mid(UpperRight),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_corner(70, &Left),
                City::double(70).rotate(PI / 2.0).nudge(Direction::E, 0.1),
            ],
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), BottomLeft)
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "X7".to_string(),
            vec![
                Track::gentle_l(UpperLeft),
                Track::gentle_r(LowerLeft).with_span(0.0, 0.5),
                Track::gentle_l(LowerRight).with_span(0.0, 0.5),
                Track::straight(Top).with_span(0.0, 0.6),
                Track::straight(Bottom).with_span(0.0, 0.4),
            ],
            // TODO: Montreal label
            vec![
                City::single_at_face(70, &UpperRight),
                City::double(70).nudge(Direction::S, 0.3),
            ],
            ctx,
            hex,
        )
        .label(Label::City("M".to_string()), Left)
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Grey,
            "X8".to_string(),
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(LowerRight),
                Track::mid(UpperRight),
            ],
            vec![City::triple(60).rotate(PI)],
            ctx,
            hex,
        )
        // NOTE: add city and revenue labels.
        .label(Label::City("O".to_string()), Left)
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(
            Yellow,
            "IN10".to_string(),
            vec![
                // TODO! dit is at the wrong location for gentle_l !!!
                // FIXED! But how to test?!?
                Track::gentle_l(Bottom).with_dit(0.85, 30),
                Track::gentle_r(Bottom).with_dit(0.85, 30),
                Track::straight(UpperLeft),
                Track::gentle_l(Top),
            ],
            vec![],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Green,
            "IN11".to_string(),
            vec![
                Track::straight(LowerRight),
                Track::gentle_r(LowerRight),
                Track::gentle_l(Bottom),
                Track::straight(Bottom),
            ],
            vec![
                City::single_at_face(30, &LowerLeft)
                    .nudge(Direction::NEE, 0.2),
                City::single_at_face(30, &UpperRight)
                    .nudge(Direction::SWW, 0.2),
            ],
            ctx,
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
    ]
}
