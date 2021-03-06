use n18hex::{Direction, Hex};
use n18tile::{City, Label, Rotation, Tile, Track};

/// Predefined tiles, named as per the [18xx Tile
/// Database](http://www.fwtwr.com/18xx/tiles/).
///
/// This can be used to build individual tiles:
///
/// ```rust
/// # use n18catalogue::Kind;
/// # use n18hex::Hex;
/// # use n18tile::Tile;
/// let hex = Hex::default();
/// let tile: Tile = Kind::_3.build(&hex);
/// ```
///
/// An iterator over all predefined tiles is also provided:
///
/// ```rust
/// # use n18catalogue::Kind;
/// let kinds: Vec<Kind> = Kind::iter().collect();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _18,
    _19,
    _20,
    _21,
    _22,
    _23,
    _24,
    _25,
    _26,
    _27,
    _28,
    _29,
    _30,
    _31,
    _39,
    _40,
    _41,
    _42,
    _43,
    _44,
    _45,
    _46,
    _47,
    _53,
    _54,
    _55,
    _56,
    _57,
    _58,
    _59,
    _61,
    _62,
    _63,
    _64,
    _65,
    _66,
    _67,
    _68,
    _69,
    _70,
    _87,
    _88,
    _120,
    _122,
    _124,
    _201,
    _202,
    _204,
    _205,
    _206,
    _207,
    _208,
    _437,
    _438,
    _439,
    _440,
    _448,
    _465,
    _466,
    _492,
    _611,
    _619,
    _621,
    _622,
    _623,
    _624,
    _625,
    _626,
    _635,
    _636,
    _637,
    _638,
    _639,
    _640,
    _641,
    _642,
    _801,
    _911,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    IN10,
    IN11,
}

impl Kind {
    pub fn iter() -> impl Iterator<Item = Kind> {
        static KINDS: &[Kind] = &[
            Kind::_1,
            Kind::_2,
            Kind::_3,
            Kind::_4,
            Kind::_5,
            Kind::_6,
            Kind::_7,
            Kind::_8,
            Kind::_9,
            Kind::_12,
            Kind::_13,
            Kind::_14,
            Kind::_15,
            Kind::_16,
            Kind::_17,
            Kind::_18,
            Kind::_19,
            Kind::_20,
            Kind::_21,
            Kind::_22,
            Kind::_23,
            Kind::_24,
            Kind::_25,
            Kind::_26,
            Kind::_27,
            Kind::_28,
            Kind::_29,
            Kind::_30,
            Kind::_31,
            Kind::_39,
            Kind::_40,
            Kind::_41,
            Kind::_42,
            Kind::_43,
            Kind::_44,
            Kind::_45,
            Kind::_46,
            Kind::_47,
            Kind::_53,
            Kind::_54,
            Kind::_55,
            Kind::_56,
            Kind::_57,
            Kind::_58,
            Kind::_59,
            Kind::_61,
            Kind::_62,
            Kind::_63,
            Kind::_64,
            Kind::_65,
            Kind::_66,
            Kind::_67,
            Kind::_68,
            Kind::_69,
            Kind::_70,
            Kind::_87,
            Kind::_88,
            Kind::_120,
            Kind::_122,
            Kind::_124,
            Kind::_201,
            Kind::_202,
            Kind::_204,
            Kind::_205,
            Kind::_206,
            Kind::_207,
            Kind::_208,
            Kind::_437,
            Kind::_438,
            Kind::_439,
            Kind::_440,
            Kind::_448,
            Kind::_465,
            Kind::_466,
            Kind::_492,
            Kind::_611,
            Kind::_619,
            Kind::_621,
            Kind::_622,
            Kind::_623,
            Kind::_624,
            Kind::_625,
            Kind::_626,
            Kind::_635,
            Kind::_636,
            Kind::_637,
            Kind::_638,
            Kind::_639,
            Kind::_640,
            Kind::_641,
            Kind::_642,
            Kind::_801,
            Kind::_911,
            Kind::X1,
            Kind::X2,
            Kind::X3,
            Kind::X4,
            Kind::X5,
            Kind::X6,
            Kind::X7,
            Kind::X8,
            Kind::IN10,
            Kind::IN11,
        ];
        KINDS.iter().copied()
    }

    pub fn build(&self, hex: &Hex) -> Tile {
        use n18hex::HexColour::*;
        use n18hex::HexCorner::*;
        use n18hex::HexFace::*;
        use n18hex::HexPosition::*;
        use n18tile::DitShape::*;
        use n18tile::TrackEnd::*;

        match self {
            Kind::_1 => Tile::new(
                Yellow,
                "1",
                vec![
                    Track::gentle_l(UpperLeft)
                        .with_span(0.0, 0.5)
                        .with_dit(End, 10, Bar),
                    Track::gentle_l(UpperLeft).with_span(0.5, 1.0),
                    Track::gentle_r(LowerLeft)
                        .with_span(0.0, 0.5)
                        .with_dit(End, 10, Bar),
                    Track::gentle_r(LowerLeft).with_span(0.5, 1.0),
                ],
                vec![],
                hex,
            )
            .label(Label::Revenue(0), Left.to_centre(0.2)),
            Kind::_2 => Tile::new(
                Yellow,
                "2",
                vec![
                    Track::straight(UpperLeft)
                        .with_span(0.0, 0.5)
                        .with_dit(End, 10, Bar),
                    Track::straight(UpperLeft).with_span(0.5, 1.0),
                    Track::hard_l(Top)
                        .with_span(0.0, 0.5)
                        .with_dit(End, 10, Bar),
                    Track::hard_l(Top).with_span(0.5, 1.0),
                ],
                vec![],
                hex,
            )
            .label(Label::Revenue(0), LowerLeft.to_centre(0.2)),
            Kind::_3 => Tile::new(
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
            .label(Label::Revenue(0), Centre(None)),
            Kind::_4 => Tile::new(
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
            .label(Label::Revenue(0), LowerLeft.to_centre(0.3)),
            Kind::_5 => Tile::new(
                Yellow,
                "5",
                vec![Track::mid(Bottom), Track::mid(LowerRight)],
                vec![City::single(20)],
                hex,
            )
            .label(Label::Revenue(0), TopLeft.to_centre(0.3)),
            Kind::_6 => Tile::new(
                Yellow,
                "6",
                vec![Track::mid(Bottom), Track::mid(UpperRight)],
                vec![City::single(20)],
                hex,
            )
            .label(Label::Revenue(0), Top.to_centre(0.2)),
            Kind::_7 => Tile::new(
                Yellow,
                "7",
                vec![Track::hard_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_8 => Tile::new(
                Yellow,
                "8",
                vec![Track::gentle_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_9 => Tile::new(
                Yellow,
                "9",
                vec![Track::straight(Bottom)],
                vec![],
                hex,
            ),
            Kind::_12 => Tile::new(
                Green,
                "12",
                vec![
                    Track::mid(Top),
                    Track::mid(UpperRight),
                    Track::mid(LowerRight),
                ],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), Left.to_centre(0.25)),
            Kind::_13 => Tile::new(
                Green,
                "13",
                vec![
                    Track::mid(Top),
                    Track::mid(LowerLeft),
                    Track::mid(LowerRight),
                ],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), UpperLeft.to_centre(0.2)),
            Kind::_14 => Tile::new(
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
            Kind::_15 => Tile::new(
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
            Kind::_16 => Tile::new(
                Green,
                "16",
                vec![Track::gentle_r(Bottom), Track::gentle_r(LowerLeft)],
                vec![],
                hex,
            ),
            Kind::_17 => Tile::new(
                Green,
                "17",
                vec![Track::gentle_r(Bottom), Track::gentle_l(LowerLeft)],
                vec![],
                hex,
            ),
            Kind::_18 => Tile::new(
                Green,
                "18",
                vec![Track::straight(Bottom), Track::hard_l(LowerLeft)],
                vec![],
                hex,
            ),
            Kind::_19 => Tile::new(
                Green,
                "19",
                vec![Track::gentle_r(LowerLeft), Track::straight(Bottom)],
                vec![],
                hex,
            ),
            Kind::_20 => Tile::new(
                Green,
                "20",
                vec![Track::straight(LowerLeft), Track::straight(Bottom)],
                vec![],
                hex,
            ),
            Kind::_21 => Tile::new(
                Green,
                "21",
                vec![Track::hard_l(Top), Track::gentle_l(Bottom)],
                vec![],
                hex,
            ),
            Kind::_22 => Tile::new(
                Green,
                "22",
                vec![Track::hard_r(Top), Track::gentle_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_23 => Tile::new(
                Green,
                "23",
                vec![Track::straight(Bottom), Track::gentle_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_24 => Tile::new(
                Green,
                "24",
                vec![Track::straight(Bottom), Track::gentle_l(Bottom)],
                vec![],
                hex,
            ),
            Kind::_25 => Tile::new(
                Green,
                "25",
                vec![Track::gentle_r(Bottom), Track::gentle_l(Bottom)],
                vec![],
                hex,
            ),
            Kind::_26 => Tile::new(
                Green,
                "26",
                vec![Track::straight(Bottom), Track::hard_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_27 => Tile::new(
                Green,
                "27",
                vec![Track::straight(Bottom), Track::hard_l(Bottom)],
                vec![],
                hex,
            ),
            Kind::_28 => Tile::new(
                Green,
                "28",
                vec![Track::gentle_r(Bottom), Track::hard_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_29 => Tile::new(
                Green,
                "29",
                vec![Track::gentle_l(Bottom), Track::hard_l(Bottom)],
                vec![],
                hex,
            ),
            Kind::_30 => Tile::new(
                Green,
                "30",
                vec![Track::hard_l(Bottom), Track::gentle_r(Bottom)],
                vec![],
                hex,
            ),
            Kind::_31 => Tile::new(
                Green,
                "31",
                vec![Track::hard_r(Bottom), Track::gentle_l(Bottom)],
                vec![],
                hex,
            ),
            Kind::_39 => Tile::new(
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
            Kind::_40 => Tile::new(
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
            Kind::_41 => Tile::new(
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
            Kind::_42 => Tile::new(
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
            Kind::_43 => Tile::new(
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
            Kind::_44 => Tile::new(
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
            Kind::_45 => Tile::new(
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
            Kind::_46 => Tile::new(
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
            Kind::_47 => Tile::new(
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
            Kind::_53 => Tile::new(
                Green,
                "53",
                vec![
                    Track::straight(Bottom).with_span(0.0, 0.5),
                    Track::straight(UpperLeft).with_span(0.0, 0.5),
                    Track::straight(UpperRight).with_span(0.0, 0.5),
                ],
                vec![City::single(50)],
                hex,
            )
            .label(Label::City("B".to_string()), LowerRight.to_centre(0.15))
            .label(Label::Revenue(0), LowerLeft.to_centre(0.15)),
            Kind::_54 => Tile::new(
                Green,
                "54",
                vec![
                    Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                    Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                    Track::hard_l(Bottom).with_span(0.0, 0.5),
                    Track::hard_l(Bottom).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(60, &TopLeft),
                    City::single_at_corner(60, &BottomLeft),
                ],
                hex,
            )
            .label(Label::City("NY".to_string()), Right.to_centre(0.25))
            .label(Label::Revenue(0), Centre(None)),
            Kind::_55 => Tile::new(
                Yellow,
                "55",
                vec![
                    Track::straight(UpperLeft)
                        .with_span(0.0, 0.8)
                        .with_dit(End, 10, Bar),
                    Track::straight(UpperLeft).with_span(0.8, 1.0),
                    Track::straight(LowerLeft)
                        .with_span(0.0, 0.2)
                        .with_dit(End, 10, Bar),
                    Track::straight(LowerLeft).with_span(0.2, 1.0),
                ],
                vec![],
                hex,
            )
            .label(Label::Revenue(0), Bottom.to_centre(0.2)),
            Kind::_56 => Tile::new(
                Yellow,
                "56",
                vec![
                    Track::gentle_l(UpperLeft)
                        .with_span(0.0, 0.2)
                        .with_dit(End, 10, Bar),
                    Track::gentle_l(UpperLeft).with_span(0.2, 1.0),
                    Track::gentle_r(LowerRight)
                        .with_span(0.0, 0.2)
                        .with_dit(End, 10, Bar),
                    Track::gentle_r(LowerRight).with_span(0.2, 1.0),
                ],
                vec![],
                hex,
            )
            .label(Label::Revenue(0), BottomLeft.to_centre(0.4)),
            Kind::_57 => Tile::new(
                Yellow,
                "57",
                vec![Track::mid(Bottom), Track::mid(Top)],
                vec![City::single(20)],
                hex,
            )
            .label(Label::Revenue(0), UpperLeft.to_centre(0.2)),
            Kind::_58 => Tile::new(
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
            .label(Label::Revenue(0), UpperLeft.to_centre(0.5)),
            Kind::_59 => Tile::new(
                Green,
                "59",
                vec![
                    Track::hard_r(UpperLeft).with_span(0.0, 0.5),
                    Track::hard_l(UpperRight).with_span(0.0, 0.5),
                ],
                vec![
                    City::single_at_corner(40, &Left),
                    City::single_at_corner(40, &Right),
                ],
                hex,
            )
            .label(Label::CityKind("OO".to_string()), Top.to_centre(0.2))
            .label(Label::Revenue(0), Centre(None)),
            Kind::_61 => Tile::new(
                Brown,
                "61",
                vec![
                    Track::straight(Bottom).with_span(0.0, 0.5),
                    Track::straight(UpperLeft).with_span(0.0, 0.5),
                    Track::straight(Top).with_span(0.0, 0.5),
                    Track::straight(UpperRight).with_span(0.0, 0.5),
                ],
                vec![City::single(60)],
                hex,
            )
            .label(Label::City("B".to_string()), LowerRight.to_centre(0.2))
            .label(Label::Revenue(0), LowerLeft.to_centre(0.2)),
            Kind::_62 => Tile::new(
                Brown,
                "62",
                vec![
                    Track::hard_l(Bottom).with_span(0.0, 0.5),
                    Track::hard_l(Bottom).with_span(0.5, 1.0),
                    Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                    Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                ],
                vec![
                    City::double_at_corner(60, &TopLeft),
                    City::double_at_corner(60, &BottomLeft),
                ],
                hex,
            )
            .label(Label::City("NY".to_string()), Right.to_centre(0.2))
            .label(Label::Revenue(0), Centre(None)),
            Kind::_63 => Tile::new(
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
            Kind::_64 => Tile::new(
                Brown,
                "64",
                vec![
                    Track::hard_r(Bottom).with_span(0.0, 0.5),
                    Track::hard_r(Bottom).with_span(0.5, 1.0),
                    Track::gentle_l(UpperLeft).with_span(0.0, 0.5),
                    Track::gentle_l(UpperLeft).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(50, &BottomRight),
                    City::single_at_face(50, &Top).to_centre(0.6),
                ],
                hex,
            )
            .label(
                Label::CityKind("OO".to_string()),
                LowerLeft.to_centre(0.2),
            )
            .label(Label::Revenue(0), Right.to_centre(0.2)),

            Kind::_65 => Tile::new(
                Brown,
                "65",
                vec![
                    Track::hard_r(LowerLeft).with_span(0.0, 0.5),
                    Track::hard_r(LowerLeft).with_span(0.5, 1.0),
                    Track::gentle_l(UpperLeft).with_span(0.0, 0.5),
                    Track::gentle_l(UpperLeft).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(50, &BottomLeft),
                    City::single_at_face(50, &Top).to_centre(0.6),
                ],
                hex,
            )
            .label(
                Label::CityKind("OO".to_string()),
                LowerRight.to_centre(0.2),
            )
            .label(Label::Revenue(0), Left.to_centre(0.2)),
            Kind::_66 => Tile::new(
                Brown,
                "66",
                vec![
                    Track::straight(LowerRight).with_span(0.0, 0.2),
                    Track::straight(LowerRight).with_span(0.2, 1.0),
                    Track::hard_l(Top).with_span(0.0, 0.5),
                    Track::hard_l(Top).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_face(50, &LowerRight).to_centre(0.2),
                    City::single_at_corner(50, &TopRight),
                ],
                hex,
            )
            .label(Label::CityKind("OO".to_string()), Left.to_centre(0.2))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.2)),
            Kind::_67 => Tile::new(
                Brown,
                "67",
                vec![
                    Track::straight(Bottom).with_span(0.0, 0.2),
                    Track::straight(Bottom).with_span(0.2, 1.0),
                    Track::gentle_l(UpperLeft).with_span(0.0, 0.15),
                    Track::gentle_l(UpperLeft).with_span(0.15, 1.0),
                ],
                vec![
                    City::single_at_face(50, &Bottom).to_centre(0.2),
                    City::single_at_face(50, &UpperLeft).to_centre(0.1),
                ],
                hex,
            )
            .label(Label::CityKind("OO".to_string()), Right.to_centre(0.2))
            .label(Label::Revenue(0), LowerLeft.to_centre(0.2)),
            Kind::_68 => Tile::new(
                Brown,
                "68",
                vec![
                    Track::straight(LowerLeft).with_span(0.0, 0.9),
                    Track::straight(LowerLeft).with_span(0.9, 1.0),
                    Track::straight(UpperLeft).with_span(0.0, 0.1),
                    Track::straight(UpperLeft).with_span(0.1, 1.0),
                ],
                vec![
                    City::single_at_face(50, &UpperRight).to_centre(0.1),
                    City::single_at_face(50, &UpperLeft).to_centre(0.1),
                ],
                hex,
            )
            .label(Label::CityKind("OO".to_string()), Bottom.to_centre(0.2))
            .label(Label::Revenue(0), Top.to_centre(0.2)),
            Kind::_69 => Tile::new(
                Yellow,
                "69",
                vec![
                    Track::straight(Bottom)
                        .with_span(0.0, 0.2)
                        .with_dit(End, 10, Bar),
                    Track::straight(Bottom).with_span(0.2, 1.0),
                    Track::gentle_l(UpperLeft)
                        .with_span(0.0, 0.8)
                        .with_dit(End, 10, Bar),
                    Track::gentle_l(UpperLeft).with_span(0.8, 1.0),
                ],
                vec![],
                hex,
            )
            .label(Label::Revenue(0), LowerRight.to_centre(0.2)),
            Kind::_70 => Tile::new(
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
            Kind::_87 => Tile::new(
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
            Kind::_88 => Tile::new(
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
            .label(Label::Revenue(0), UpperRight.to_centre(0.2)),
            Kind::_120 => Tile::new(
                Green,
                "120",
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
            .label(Label::Revenue(0), Centre(None)),
            Kind::_122 => Tile::new(
                Brown,
                "122",
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
            .label(Label::Revenue(0), Centre(None)),
            Kind::_124 => Tile::new(
                Grey,
                "124",
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
            .label(Label::Revenue(0), Right.to_centre(0.08)),
            Kind::_201 => Tile::new(
                Yellow,
                "201",
                vec![Track::mid(Bottom), Track::mid(LowerRight)],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), TopLeft.to_centre(0.25))
            .label(Label::y(), LowerLeft.to_centre(0.2)),
            Kind::_202 => Tile::new(
                Yellow,
                "202",
                vec![Track::mid(Bottom), Track::mid(UpperRight)],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), TopLeft.to_centre(0.25))
            .label(Label::y(), LowerLeft.to_centre(0.2)),
            Kind::_204 => Tile::new(
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
            .label(Label::Revenue(0), LowerLeft.to_centre(0.25)),
            Kind::_205 => Tile::new(
                Green,
                "205",
                vec![
                    Track::mid(Top),
                    Track::mid(UpperRight),
                    Track::mid(Bottom),
                ],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), Left.to_centre(0.25)),
            Kind::_206 => Tile::new(
                Green,
                "206",
                vec![
                    Track::mid(Top),
                    Track::mid(LowerRight),
                    Track::mid(Bottom),
                ],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), Left.to_centre(0.25)),
            Kind::_207 => Tile::new(
                Green,
                "207",
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
            .label(Label::y(), TopRight.to_centre(0.15)),
            Kind::_208 => Tile::new(
                Green,
                "208",
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
            .label(Label::y(), TopLeft.to_centre(0.15)),
            Kind::_437 => Tile::new(
                Yellow,
                "437",
                vec![
                    Track::mid(Top).with_dit(End, 30, Circle),
                    Track::mid(LowerRight),
                ],
                vec![],
                hex,
            )
            .label(Label::Revenue(0), LowerLeft.to_centre(0.2)),
            Kind::_438 => Tile::new(
                Yellow,
                "438",
                vec![Track::mid(Top), Track::mid(LowerRight)],
                vec![City::single(40)],
                hex,
            )
            .label(Label::Note("Y80".to_string()), TopLeft.to_centre(0.125))
            .label(Label::City("K".to_string()), UpperRight.to_centre(0.2))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.2)),
            Kind::_439 => Tile::new(
                Green,
                "439",
                vec![
                    Track::mid(UpperLeft),
                    Track::mid(UpperRight),
                    Track::mid(Bottom),
                ],
                vec![City::double(60)],
                hex,
            )
            .label(
                Label::Note("Y80".to_string()),
                BottomLeft.to_centre(0.125),
            )
            .label(Label::City("K".to_string()), BottomRight.to_centre(0.25))
            .label(Label::Revenue(0), Top.to_centre(0.2)),
            Kind::_440 => Tile::new(
                Green,
                "440",
                vec![
                    Track::mid(UpperLeft),
                    Track::mid(LowerLeft),
                    Track::mid(Bottom),
                ],
                vec![City::double(40)],
                hex,
            )
            .label(Label::City("T".to_string()), BottomRight.to_centre(0.25))
            .label(Label::Revenue(0), Top.to_centre(0.2)),
            Kind::_448 => Tile::new(
                Brown,
                "448",
                vec![
                    Track::mid(Top),
                    Track::mid(UpperRight),
                    Track::mid(LowerRight),
                    Track::mid(Bottom),
                ],
                vec![City::double(40)],
                hex,
            )
            .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
            Kind::_465 => Tile::new(
                Brown,
                "465",
                vec![
                    Track::mid(UpperLeft),
                    Track::mid(Top),
                    Track::mid(UpperRight),
                    Track::mid(LowerRight),
                ],
                vec![City::triple(60)],
                hex,
            )
            .label(Label::City("Ki".to_string()), BottomLeft.to_centre(0.08))
            .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
            Kind::_466 => Tile::new(
                Brown,
                "466",
                vec![
                    Track::mid(UpperLeft),
                    Track::mid(LowerLeft),
                    Track::mid(Bottom),
                ],
                vec![City::double(60)],
                hex,
            )
            .label(Label::City("T".to_string()), BottomRight.to_centre(0.25))
            .label(Label::Revenue(0), Top.to_centre(0.2)),
            Kind::_492 => Tile::new(
                Brown,
                "492",
                vec![
                    Track::mid(LowerLeft),
                    Track::mid(UpperLeft),
                    Track::mid(Top),
                    Track::mid(UpperRight),
                    Track::mid(LowerRight),
                    Track::mid(Bottom),
                ],
                vec![City::triple(80)],
                hex,
            )
            .label(Label::City("K".to_string()), TopRight.to_centre(0.08))
            .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
            Kind::_611 => Tile::new(
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
            .label(Label::Revenue(0), TopLeft.to_centre(0.125)),
            Kind::_619 => Tile::new(
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
            Kind::_621 => Tile::new(
                Yellow,
                "621",
                vec![
                    Track::straight(Bottom).with_span(0.0, 0.5),
                    Track::straight(Bottom).with_span(0.5, 1.0),
                ],
                vec![City::single(30)],
                hex,
            )
            .label(Label::Revenue(0), UpperLeft.to_centre(0.1))
            .label(Label::y(), LowerLeft.to_centre(0.2)),
            Kind::_622 => Tile::new(
                Green,
                "622",
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
            .label(Label::y(), BottomLeft.to_centre(0.15)),
            Kind::_623 => Tile::new(
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
                vec![City::double(50)],
                hex,
            )
            .label(Label::y(), TopRight.to_centre(0.15))
            .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
            Kind::_624 => Tile::new(
                Green,
                "624",
                vec![Track::hard_l(Bottom), Track::hard_l(LowerLeft)],
                vec![],
                hex,
            ),
            Kind::_625 => Tile::new(
                Green,
                "625",
                vec![Track::hard_r(Bottom), Track::hard_l(LowerLeft)],
                vec![],
                hex,
            ),
            Kind::_626 => Tile::new(
                Green,
                "626",
                vec![Track::hard_r(LowerRight), Track::hard_l(LowerLeft)],
                vec![],
                hex,
            ),
            Kind::_635 => Tile::new(
                Green,
                "635",
                vec![
                    Track::hard_l(Bottom).with_span(0.0, 0.5),
                    Track::hard_l(Bottom).with_span(0.5, 1.0),
                    Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                    Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                    Track::hard_l(UpperRight).with_span(0.0, 0.5),
                    Track::hard_l(UpperRight).with_span(0.5, 1.0),
                ],
                vec![
                    City::single_at_corner(40, &BottomLeft),
                    City::single_at_corner(40, &TopLeft),
                    City::single_at_corner(40, &Right),
                ],
                hex,
            )
            .label(Label::City("K".to_string()), Left.to_centre(0.25))
            .label(Label::Note("R40".to_string()), Centre(None))
            .label(Label::Revenue(0), TopRight.to_centre(0.15)),
            Kind::_636 => Tile::new(
                Brown,
                "636",
                vec![
                    Track::mid(Bottom),
                    Track::mid(LowerLeft),
                    Track::mid(UpperLeft),
                    Track::mid(Top),
                    Track::mid(LowerRight),
                    Track::mid(UpperRight),
                ],
                vec![City::triple(50).rotate(Rotation::HalfTurn)],
                hex,
            )
            .label(Label::City("K".to_string()), TopRight.to_centre(0.08))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::_637 => Tile::new(
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
                vec![
                    City::single_at_corner(50, &BottomLeft),
                    City::single_at_corner(50, &TopLeft),
                    City::single_at_corner(50, &Right),
                ],
                hex,
            )
            .label(Label::City("M".to_string()), Left.to_centre(0.25))
            .label(Label::Revenue(0), TopRight.to_centre(0.15)),
            Kind::_638 => Tile::new(
                Brown,
                "638",
                vec![
                    Track::mid(Bottom),
                    Track::mid(LowerLeft),
                    Track::mid(UpperLeft),
                    Track::mid(Top),
                    Track::mid(LowerRight),
                    Track::mid(UpperRight),
                ],
                vec![City::triple(70).rotate(Rotation::HalfTurn)],
                hex,
            )
            .label(Label::City("M".to_string()), TopRight.to_centre(0.08))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::_639 => Tile::new(
                Grey,
                "639",
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
            .label(Label::Revenue(0), Right.to_centre(0.08)),
            Kind::_640 => Tile::new(
                Grey,
                "640",
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
            .label(Label::City("Kh".to_string()), TopRight.to_centre(0.08))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::_641 => Tile::new(
                Brown,
                "641",
                vec![
                    Track::mid(Bottom),
                    Track::mid(LowerLeft),
                    Track::mid(LowerRight),
                ],
                vec![City::triple(50).rotate(Rotation::HalfTurn)],
                hex,
            )
            .label(Label::City("S".to_string()), Top.to_centre(0.08))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::_642 => Tile::new(
                Grey,
                "642",
                vec![
                    Track::mid(Bottom),
                    Track::mid(LowerLeft),
                    Track::mid(LowerRight),
                ],
                vec![City::triple(70).rotate(Rotation::HalfTurn)],
                hex,
            )
            .label(Label::City("S".to_string()), Top.to_centre(0.08))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::_801 => Tile::new(
                Brown,
                "801",
                vec![
                    Track::mid(Bottom),
                    Track::mid(LowerLeft),
                    Track::mid(UpperLeft),
                    Track::mid(Top),
                ],
                vec![City::double(50)],
                hex,
            )
            .label(Label::y(), Right.to_centre(0.2))
            .label(Label::Revenue(0), TopRight.to_centre(0.15)),
            Kind::_911 => Tile::new(
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
            .label(Label::Revenue(0), UpperLeft.to_centre(0.25)),
            Kind::X1 => Tile::new(
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
            .label(Label::Revenue(0), TopLeft.in_dir(Direction::S30W, 0.16)),
            Kind::X2 => Tile::new(
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
            .label(Label::Revenue(0), Right.in_dir(Direction::N60W, 0.15)),
            Kind::X3 => Tile::new(
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
            .label(Label::Revenue(0), TopLeft.in_dir(Direction::S30W, 0.16)),
            Kind::X4 => Tile::new(
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
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::X5 => Tile::new(
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
            .label(Label::Revenue(0), Left.to_centre(0.1)),
            Kind::X6 => Tile::new(
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
            .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
            Kind::X7 => Tile::new(
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
                vec![
                    City::single_at_face(70, &UpperRight),
                    City::double(70).in_dir(Direction::S, 0.3),
                ],
                hex,
            )
            .label(Label::City("M".to_string()), Left.to_centre(0.15))
            .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
            Kind::X8 => Tile::new(
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
            .label(Label::City("O".to_string()), Left.to_centre(0.15))
            .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
            Kind::IN10 => Tile::new(
                Yellow,
                "IN10",
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
            .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
            Kind::IN11 => Tile::new(
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
                        .in_dir(Direction::N60E, 0.2),
                    City::single_at_face(30, &UpperRight)
                        .in_dir(Direction::S60W, 0.2),
                ],
                hex,
            )
            .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        }
    }
}
