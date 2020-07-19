/// Load tile catalogues from disk.
use rusty_hex::Hex;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Tiles {
    pub tiles: Vec<Tile>,
}

impl std::convert::From<&[rusty_tile::Tile]> for Tiles {
    fn from(src: &[rusty_tile::Tile]) -> Self {
        Self {
            tiles: src.iter().map(|t| t.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum HexColour {
    Yellow,
    Green,
    Brown,
    Grey,
    Red,
    Blue,
    Empty,
}

impl std::convert::From<rusty_hex::HexColour> for HexColour {
    fn from(src: rusty_hex::HexColour) -> Self {
        use rusty_hex::HexColour::*;

        match src {
            Yellow => HexColour::Yellow,
            Green => HexColour::Green,
            Brown => HexColour::Brown,
            Grey => HexColour::Grey,
            Red => HexColour::Red,
            Blue => HexColour::Blue,
            Empty => HexColour::Empty,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum HexFace {
    Top,
    UpperRight,
    LowerRight,
    Bottom,
    LowerLeft,
    UpperLeft,
}

impl std::convert::From<rusty_hex::HexFace> for HexFace {
    fn from(src: rusty_hex::HexFace) -> Self {
        use rusty_hex::HexFace::*;

        match src {
            Top => HexFace::Top,
            UpperRight => HexFace::UpperRight,
            LowerRight => HexFace::LowerRight,
            Bottom => HexFace::Bottom,
            LowerLeft => HexFace::LowerLeft,
            UpperLeft => HexFace::UpperLeft,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Tile {
    pub name: String,
    pub colour: HexColour,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub track: Vec<Track>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cities: Vec<City>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<Label>,
}

impl std::convert::From<&rusty_tile::Tile> for Tile {
    fn from(src: &rusty_tile::Tile) -> Self {
        Self {
            name: src.name.clone(),
            colour: src.colour.into(),
            track: src.tracks().iter().map(|track| track.into()).collect(),
            cities: src.cities().iter().map(|city| city.into()).collect(),
            labels: src.labels().iter().map(|lnp| lnp.into()).collect(),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            colour: HexColour::Yellow,
            track: vec![],
            cities: vec![],
            labels: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TrackType {
    Mid(HexFace),
    Straight(HexFace),
    GentleL(HexFace),
    GentleR(HexFace),
    HardL(HexFace),
    HardR(HexFace),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
enum TrackEnd {
    Start,
    End,
}

impl std::convert::From<rusty_tile::TrackEnd> for TrackEnd {
    fn from(src: rusty_tile::TrackEnd) -> Self {
        use rusty_tile::TrackEnd::*;

        match src {
            Start => TrackEnd::Start,
            End => TrackEnd::End,
        }
    }
}

impl std::convert::From<TrackEnd> for rusty_tile::TrackEnd {
    fn from(src: TrackEnd) -> Self {
        use rusty_tile::TrackEnd::*;

        match src {
            TrackEnd::Start => Start,
            TrackEnd::End => End,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
enum DitShape {
    Bar,
    Circle,
}

impl std::convert::From<rusty_tile::DitShape> for DitShape {
    fn from(src: rusty_tile::DitShape) -> Self {
        use rusty_tile::DitShape::*;

        match src {
            Bar => DitShape::Bar,
            Circle => DitShape::Circle,
        }
    }
}

impl std::convert::From<DitShape> for rusty_tile::DitShape {
    fn from(src: DitShape) -> Self {
        use rusty_tile::DitShape::*;

        match src {
            DitShape::Bar => Bar,
            DitShape::Circle => Circle,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Track {
    #[serde(flatten)]
    pub track_type: TrackType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dit: Option<(TrackEnd, usize, DitShape)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<(f64, f64)>,
}

impl std::convert::From<&rusty_tile::Track> for Track {
    fn from(src: &rusty_tile::Track) -> Self {
        use rusty_tile::TrackCurve::*;

        let span = if src.x0 == 0.0 && src.x1 == 1.0 {
            None
        } else if src.x0 >= 0.0 && src.x1 <= 1.0 {
            Some((src.x0, src.x1))
        } else {
            panic!("Invalid track span: [{}, {}]", src.x0, src.x1)
        };

        let (track_type, span) = match src.curve {
            Straight => {
                if src.x0 == 0.0 && src.x1 == 0.5 {
                    (TrackType::Mid(src.face.into()), None)
                } else {
                    (TrackType::Straight(src.face.into()), span)
                }
            }
            GentleL => (TrackType::GentleL(src.face.into()), span),
            HardL => (TrackType::HardL(src.face.into()), span),
            GentleR => (TrackType::GentleR(src.face.into()), span),
            HardR => (TrackType::HardR(src.face.into()), span),
        };
        Self {
            track_type: track_type,
            dit: src.dit.map(|(end, revenue, shape)| {
                (end.into(), revenue, shape.into())
            }),
            clip: src.clip,
            span: span,
        }
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            track_type: TrackType::Straight(HexFace::Bottom),
            dit: None,
            clip: None,
            span: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Location {
    Centre,
    TopLeftCorner,
    TopRightCorner,
    LeftCorner,
    RightCorner,
    BottomLeftCorner,
    BottomRightCorner,
    TopFace,
    UpperRightFace,
    LowerRightFace,
    BottomFace,
    LowerLeftFace,
    UpperLeftFace,
}

impl std::convert::From<&rusty_hex::HexPosition> for Location {
    fn from(src: &rusty_hex::HexPosition) -> Self {
        use rusty_hex::HexPosition::*;

        match src {
            Centre(_delta) => Location::Centre,
            Face(face, _delta) => face.into(),
            Corner(corner, _delta) => corner.into(),
        }
    }
}

impl std::convert::From<&rusty_hex::HexFace> for Location {
    fn from(src: &rusty_hex::HexFace) -> Self {
        use rusty_hex::HexFace::*;

        match src {
            Top => Location::TopFace,
            UpperRight => Location::UpperRightFace,
            LowerRight => Location::LowerRightFace,
            Bottom => Location::BottomFace,
            LowerLeft => Location::LowerLeftFace,
            UpperLeft => Location::UpperLeftFace,
        }
    }
}

impl std::convert::From<&rusty_hex::HexCorner> for Location {
    fn from(src: &rusty_hex::HexCorner) -> Self {
        use rusty_hex::HexCorner::*;

        match src {
            TopLeft => Location::TopLeftCorner,
            TopRight => Location::TopRightCorner,
            Left => Location::LeftCorner,
            Right => Location::RightCorner,
            BottomLeft => Location::BottomLeftCorner,
            BottomRight => Location::BottomRightCorner,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum CornerLocation {
    Centre,
    TopLeftCorner,
    TopRightCorner,
    LeftCorner,
    RightCorner,
    BottomLeftCorner,
    BottomRightCorner,
}

impl std::convert::From<&rusty_hex::HexPosition> for CornerLocation {
    fn from(src: &rusty_hex::HexPosition) -> Self {
        use rusty_hex::HexPosition::*;

        match src {
            Centre(_delta) => CornerLocation::Centre,
            Face(_face, _delta) => panic!("Cannot convert Face into Corner"),
            Corner(corner, _delta) => corner.into(),
        }
    }
}

impl std::convert::From<&rusty_hex::HexCorner> for CornerLocation {
    fn from(src: &rusty_hex::HexCorner) -> Self {
        use rusty_hex::HexCorner::*;

        match src {
            TopLeft => CornerLocation::TopLeftCorner,
            TopRight => CornerLocation::TopRightCorner,
            Left => CornerLocation::LeftCorner,
            Right => CornerLocation::RightCorner,
            BottomLeft => CornerLocation::BottomLeftCorner,
            BottomRight => CornerLocation::BottomRightCorner,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum CentreLocation {
    Centre,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum CityType {
    Single(Location),
    Double(CornerLocation),
    Triple(CentreLocation),
    Quad(CentreLocation),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum CityRotation {
    Zero,
    Cw90,
    Acw90,
    HalfTurn,
}

impl CityRotation {
    fn from_rot(src: rusty_tile::Rotation) -> Option<Self> {
        use rusty_tile::Rotation::*;

        match src {
            Zero => None,
            Cw90 => Some(CityRotation::Cw90),
            Acw90 => Some(CityRotation::Acw90),
            HalfTurn => Some(CityRotation::HalfTurn),
        }
    }

    fn into_rot(&self) -> rusty_tile::Rotation {
        use rusty_tile::Rotation::*;

        match self {
            CityRotation::Zero => Zero,
            CityRotation::Cw90 => Cw90,
            CityRotation::Acw90 => Acw90,
            CityRotation::HalfTurn => HalfTurn,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct City {
    #[serde(flatten)]
    pub city_type: CityType,
    pub revenue: usize,
    /// An optional nudge `(angle, frac)` where `frac` is relative to the
    /// maximal radius of the tile (i.e., from the centre to any corner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(Direction, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<CityRotation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_colour: Option<HexColour>,
}

impl std::convert::From<&rusty_tile::City> for City {
    fn from(src: &rusty_tile::City) -> Self {
        use rusty_hex::Delta;
        use rusty_hex::HexPosition::*;
        use rusty_tile::Tokens;

        let revenue = src.revenue;
        let position = &src.position;
        let city_type = match src.tokens {
            Tokens::Single => CityType::Single(position.into()),
            Tokens::Double => CityType::Double(position.into()),
            Tokens::Triple => CityType::Triple(CentreLocation::Centre),
            Tokens::Quadruple => CityType::Quad(CentreLocation::Centre),
        };
        let nudge = match position {
            Centre(delta) | Face(_, delta) | Corner(_, delta) => {
                if let Some(Delta::Nudge(angle, frac)) = delta {
                    Some((angle.into(), *frac))
                } else {
                    None
                }
            }
        };
        let rotate = CityRotation::from_rot(src.angle);
        let fill_colour = src.fill_colour.map(|colour| colour.into());
        Self {
            city_type,
            revenue,
            nudge,
            rotate,
            fill_colour,
        }
    }
}

impl Default for City {
    fn default() -> Self {
        Self {
            city_type: CityType::Single(Location::Centre),
            revenue: 10,
            nudge: None,
            rotate: None,
            fill_colour: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum LabelType {
    City(String),
    Y(()),
    TileName(()),
    MapLocation(String),
    Revenue(usize),
    PhaseRevenue(Vec<(HexColour, usize, bool)>),
}

impl std::convert::From<&rusty_tile::Label> for LabelType {
    fn from(src: &rusty_tile::Label) -> Self {
        use rusty_tile::Label as L;

        match src {
            L::City(ref name) => LabelType::City(name.clone()),
            L::Y => LabelType::Y(()),
            L::TileName => LabelType::TileName(()),
            L::MapLocation(ref name) => LabelType::MapLocation(name.clone()),
            L::Revenue(revenue) => LabelType::Revenue(*revenue),
            L::PhaseRevenue(revenues) => {
                let revs = revenues
                    .iter()
                    .map(|(colour, revenue, active)| {
                        ((*colour).into(), *revenue, *active)
                    })
                    .collect();
                LabelType::PhaseRevenue(revs)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Label {
    #[serde(flatten)]
    pub label_type: LabelType,
    pub location: Location,
    /// An optional nudge `(angle, frac)` where `frac` is relative to the
    /// maximal radius of the tile (i.e., from the centre to any corner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(Direction, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_centre: Option<f64>,
}

impl std::convert::From<&rusty_tile::LabelAndPos> for Label {
    fn from(src: &rusty_tile::LabelAndPos) -> Self {
        use rusty_hex::Delta::*;
        use rusty_hex::HexPosition::*;

        let label = &src.0;
        let posn = &src.1;
        let nudge = match posn {
            Centre(delta) => {
                if let Some(Nudge(angle, frac)) = delta {
                    Some((angle.into(), *frac))
                } else {
                    None
                }
            }
            Face(_face, delta) => {
                if let Some(Nudge(angle, frac)) = delta {
                    Some((angle.into(), *frac))
                } else {
                    None
                }
            }
            Corner(_corner, delta) => {
                if let Some(Nudge(angle, frac)) = delta {
                    Some((angle.into(), *frac))
                } else {
                    None
                }
            }
        };
        let to_centre = match posn {
            Centre(delta) => {
                if let Some(ToCentre(frac)) = delta {
                    Some(*frac)
                } else {
                    None
                }
            }
            Face(_face, delta) => {
                if let Some(ToCentre(frac)) = delta {
                    Some(*frac)
                } else {
                    None
                }
            }
            Corner(_corner, delta) => {
                if let Some(ToCentre(frac)) = delta {
                    Some(*frac)
                } else {
                    None
                }
            }
        };
        Self {
            label_type: label.into(),
            location: posn.into(),
            nudge: nudge,
            to_centre: to_centre,
        }
    }
}

impl Default for Label {
    fn default() -> Self {
        Self {
            label_type: LabelType::TileName(()),
            location: Location::BottomRightCorner,
            nudge: None,
            to_centre: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Direction {
    N,
    NNE,
    NE,
    NEE,
    E,
    SEE,
    SE,
    SSE,
    S,
    SSW,
    SW,
    SWW,
    W,
    NWW,
    NW,
    NNW,
}

impl std::convert::From<&rusty_hex::Direction> for Direction {
    fn from(src: &rusty_hex::Direction) -> Self {
        use rusty_hex::Direction::*;

        match src {
            N => Self::N,
            NNE => Self::NNE,
            NE => Self::NE,
            NEE => Self::NEE,
            E => Self::E,
            SEE => Self::SEE,
            SE => Self::SE,
            SSE => Self::SSE,
            S => Self::S,
            SSW => Self::SSW,
            SW => Self::SW,
            SWW => Self::SWW,
            W => Self::W,
            NWW => Self::NWW,
            NW => Self::NW,
            NNW => Self::NNW,
        }
    }
}

impl std::convert::From<&Direction> for rusty_hex::Direction {
    fn from(src: &Direction) -> Self {
        use rusty_hex::Direction::*;

        match src {
            Direction::N => N,
            Direction::NNE => NNE,
            Direction::NE => NE,
            Direction::NEE => NEE,
            Direction::E => E,
            Direction::SEE => SEE,
            Direction::SE => SE,
            Direction::SSE => SSE,
            Direction::S => S,
            Direction::SSW => SSW,
            Direction::SW => SW,
            Direction::SWW => SWW,
            Direction::W => W,
            Direction::NWW => NWW,
            Direction::NW => NW,
            Direction::NNW => NNW,
        }
    }
}

/// Reads a single tile from disk.
pub fn read_tile<P: AsRef<Path>>(
    path: P,
    hex: &Hex,
) -> Result<rusty_tile::Tile, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tile: Tile = serde_json::from_reader(reader)?;
    Ok(tile.build(hex))
}

/// Reads multiple tiles from disk.
pub fn read_tiles<P: AsRef<Path>>(
    path: P,
    hex: &Hex,
) -> Result<Vec<rusty_tile::Tile>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tiles: Tiles = serde_json::from_reader(reader)?;
    Ok(tiles.build(hex))
}

/// Writes a single tile to disk.
pub fn write_tile<P: AsRef<Path>>(
    path: P,
    tile: &rusty_tile::Tile,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let tile: Tile = tile.into();
    if pretty {
        serde_json::to_writer_pretty(file, &tile)?;
    } else {
        serde_json::to_writer(file, &tile)?;
    }
    Ok(())
}

/// Writes multiple tiles to disk.
pub fn write_tiles<P: AsRef<Path>>(
    path: P,
    tiles: &[rusty_tile::Tile],
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let tiles: Tiles = tiles.into();
    if pretty {
        serde_json::to_writer_pretty(file, &tiles)?;
    } else {
        serde_json::to_writer(file, &tiles)?;
    }
    Ok(())
}

// NOTE: need hex and ctx to construct tiles!

impl Tiles {
    pub fn build(&self, hex: &Hex) -> Vec<rusty_tile::Tile> {
        self.tiles.iter().map(|t| t.build(hex)).collect()
    }
}

impl Tile {
    pub fn build(&self, hex: &Hex) -> rusty_tile::Tile {
        let tile = rusty_tile::Tile::new(
            (&self.colour).into(),
            self.name.clone(),
            self.track.iter().map(|t| t.into()).collect(),
            self.cities.iter().map(|c| c.build()).collect(),
            hex,
        );
        let tile = self.labels.iter().fold(tile, |tile, label| {
            let posn = label.position();
            tile.label((&label.label_type).into(), posn)
        });
        tile
    }
}

impl From<&LabelType> for rusty_tile::Label {
    fn from(lt: &LabelType) -> Self {
        match lt {
            LabelType::City(ref name) => {
                rusty_tile::Label::City(name.clone())
            }
            LabelType::Y(()) => rusty_tile::Label::Y,
            LabelType::TileName(()) => rusty_tile::Label::TileName,
            LabelType::MapLocation(ref name) => {
                rusty_tile::Label::MapLocation(name.clone())
            }
            LabelType::Revenue(ix) => rusty_tile::Label::Revenue(*ix),
            LabelType::PhaseRevenue(revenues) => {
                let revs = revenues
                    .iter()
                    .map(|(colour, revenue, active)| {
                        (colour.into(), *revenue, *active)
                    })
                    .collect();
                rusty_tile::Label::PhaseRevenue(revs)
            }
        }
    }
}

impl From<&CornerLocation> for rusty_hex::HexPosition {
    fn from(locn: &CornerLocation) -> Self {
        use CornerLocation::*;
        match locn {
            Centre => rusty_hex::HexPosition::Centre(None),
            TopLeftCorner => rusty_hex::HexCorner::TopLeft.into(),
            TopRightCorner => rusty_hex::HexCorner::TopRight.into(),
            LeftCorner => rusty_hex::HexCorner::Left.into(),
            RightCorner => rusty_hex::HexCorner::Right.into(),
            BottomLeftCorner => rusty_hex::HexCorner::BottomLeft.into(),
            BottomRightCorner => rusty_hex::HexCorner::BottomRight.into(),
        }
    }
}

impl From<&Location> for rusty_hex::HexPosition {
    fn from(locn: &Location) -> Self {
        use Location::*;
        match locn {
            Centre => rusty_hex::HexPosition::Centre(None),
            TopLeftCorner => rusty_hex::HexCorner::TopLeft.into(),
            TopRightCorner => rusty_hex::HexCorner::TopRight.into(),
            LeftCorner => rusty_hex::HexCorner::Left.into(),
            RightCorner => rusty_hex::HexCorner::Right.into(),
            BottomLeftCorner => rusty_hex::HexCorner::BottomLeft.into(),
            BottomRightCorner => rusty_hex::HexCorner::BottomRight.into(),
            TopFace => rusty_hex::HexFace::Top.into(),
            UpperRightFace => rusty_hex::HexFace::UpperRight.into(),
            LowerRightFace => rusty_hex::HexFace::LowerRight.into(),
            BottomFace => rusty_hex::HexFace::Bottom.into(),
            LowerLeftFace => rusty_hex::HexFace::LowerLeft.into(),
            UpperLeftFace => rusty_hex::HexFace::UpperLeft.into(),
        }
    }
}

impl Label {
    pub fn position(&self) -> rusty_hex::HexPosition {
        let position: rusty_hex::HexPosition = (&self.location).into();
        let position = if let Some((ref angle, frac)) = self.nudge {
            // NOTE: retain fractional unit of distance.
            position.nudge(angle.into(), frac)
        } else {
            position
        };
        let position = if let Some(frac) = self.to_centre {
            position.to_centre(frac)
        } else {
            position
        };
        position
    }
}

impl From<&HexColour> for rusty_hex::HexColour {
    fn from(c: &HexColour) -> rusty_hex::HexColour {
        match c {
            HexColour::Yellow => rusty_hex::HexColour::Yellow,
            HexColour::Green => rusty_hex::HexColour::Green,
            HexColour::Brown => rusty_hex::HexColour::Brown,
            HexColour::Grey => rusty_hex::HexColour::Grey,
            HexColour::Red => rusty_hex::HexColour::Red,
            HexColour::Blue => rusty_hex::HexColour::Blue,
            HexColour::Empty => rusty_hex::HexColour::Empty,
        }
    }
}

impl From<&HexFace> for rusty_hex::HexFace {
    fn from(c: &HexFace) -> rusty_hex::HexFace {
        match c {
            HexFace::Top => rusty_hex::HexFace::Top,
            HexFace::UpperRight => rusty_hex::HexFace::UpperRight,
            HexFace::LowerRight => rusty_hex::HexFace::LowerRight,
            HexFace::Bottom => rusty_hex::HexFace::Bottom,
            HexFace::LowerLeft => rusty_hex::HexFace::LowerLeft,
            HexFace::UpperLeft => rusty_hex::HexFace::UpperLeft,
        }
    }
}

impl From<&Track> for rusty_tile::Track {
    fn from(t: &Track) -> rusty_tile::Track {
        let track = match t.track_type {
            TrackType::Mid(ref face) => rusty_tile::Track::mid(face.into()),
            TrackType::Straight(ref face) => {
                rusty_tile::Track::straight(face.into())
            }
            TrackType::GentleL(ref face) => {
                rusty_tile::Track::gentle_l(face.into())
            }
            TrackType::GentleR(ref face) => {
                rusty_tile::Track::gentle_r(face.into())
            }
            TrackType::HardL(ref face) => {
                rusty_tile::Track::hard_l(face.into())
            }
            TrackType::HardR(ref face) => {
                rusty_tile::Track::hard_r(face.into())
            }
        };
        let track = if let Some((posn, revenue, shape)) = t.dit {
            track.with_dit(posn.into(), revenue, shape.into())
        } else {
            track
        };
        let track = if let Some((lower, upper)) = t.clip {
            track.with_clip(lower, upper)
        } else {
            track
        };
        let track = if let Some((x0, x1)) = t.span {
            track.with_span(x0, x1)
        } else {
            track
        };
        track
    }
}

impl CityType {
    pub fn build(&self, revenue: usize) -> rusty_tile::City {
        use CityType::*;

        match self {
            Single(location) => {
                use rusty_hex::HexCorner::*;
                use rusty_hex::HexFace::*;
                use rusty_tile::City;
                use Location::*;

                match location {
                    Centre => City::single(revenue),
                    TopLeftCorner => {
                        City::single_at_corner(revenue, &TopLeft)
                    }
                    TopRightCorner => {
                        City::single_at_corner(revenue, &TopRight)
                    }
                    LeftCorner => City::single_at_corner(revenue, &Left),
                    RightCorner => City::single_at_corner(revenue, &Right),
                    BottomLeftCorner => {
                        City::single_at_corner(revenue, &BottomLeft)
                    }
                    BottomRightCorner => {
                        City::single_at_corner(revenue, &BottomRight)
                    }
                    TopFace => City::single_at_face(revenue, &Top),
                    UpperRightFace => {
                        City::single_at_face(revenue, &UpperRight)
                    }
                    LowerRightFace => {
                        City::single_at_face(revenue, &LowerRight)
                    }
                    BottomFace => City::single_at_face(revenue, &Bottom),
                    LowerLeftFace => {
                        City::single_at_face(revenue, &LowerLeft)
                    }
                    UpperLeftFace => {
                        City::single_at_face(revenue, &UpperLeft)
                    }
                }
            }
            Double(location) => {
                use rusty_hex::HexCorner::*;
                use rusty_tile::City;
                use CornerLocation::*;

                match location {
                    Centre => City::double(revenue),
                    TopLeftCorner => {
                        City::double_at_corner(revenue, &TopLeft)
                    }
                    TopRightCorner => {
                        City::double_at_corner(revenue, &TopRight)
                    }
                    LeftCorner => City::double_at_corner(revenue, &Left),
                    RightCorner => City::double_at_corner(revenue, &Right),
                    BottomLeftCorner => {
                        City::double_at_corner(revenue, &BottomLeft)
                    }
                    BottomRightCorner => {
                        City::double_at_corner(revenue, &BottomRight)
                    }
                }
            }
            Triple(_centre) => rusty_tile::City::triple(revenue),
            Quad(_centre) => rusty_tile::City::quad(revenue),
        }
    }
}

impl City {
    pub fn build(&self) -> rusty_tile::City {
        let city = self.city_type.build(self.revenue);
        let city = if let Some((ref angle, radius)) = self.nudge {
            city.nudge(angle.into(), radius)
        } else {
            city
        };
        let city = city.rotate(
            self.rotate
                .as_ref()
                .map(|r| r.into_rot())
                .unwrap_or(rusty_tile::Rotation::Zero),
        );
        city
    }
}

/// Should yield the same tiles as `rusty_catalogue::tile_catalogue()`.
#[allow(dead_code)]
fn test_tiles() -> Tiles {
    use DitShape::*;
    use HexColour::*;
    use TrackEnd::*;
    use TrackType::*;

    // TODO: define all of the tiles in rusty_catalogue::tile_catalogue().

    Tiles {
        tiles: vec![
            Tile {
                name: "3".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        dit: Some((End, 10, Bar)),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::Centre,
                    ..Default::default()
                }],
            },
            Tile {
                name: "4".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        dit: Some((End, 10, Bar)),
                        span: Some((0.0, 0.25)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.25, 1.0)),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::LowerLeftFace,
                    to_centre: Some(0.3),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "5".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Single(Location::Centre),
                    revenue: 20,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopLeftCorner,
                    to_centre: Some(0.3),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "6".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Single(Location::Centre),
                    revenue: 20,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopFace,
                    to_centre: Some(0.3),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "7".to_string(),
                colour: Yellow,
                track: vec![Track {
                    track_type: HardR(HexFace::Bottom),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "8".to_string(),
                colour: Yellow,
                track: vec![Track {
                    track_type: GentleR(HexFace::Bottom),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "9".to_string(),
                colour: Yellow,
                track: vec![Track {
                    track_type: Straight(HexFace::Bottom),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "14".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 30,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopRightCorner,
                    to_centre: Some(0.15),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "15".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 30,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopLeftCorner,
                    to_centre: Some(0.15),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "16".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "17".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "18".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "19".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "20".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "21".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "22".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardR(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "23".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "24".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "25".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "26".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "27".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "28".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "29".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "30".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "31".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "39".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "40".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "41".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "42".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::Top),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "43".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "44".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "45".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "46".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "47".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "57".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Single(Location::Centre),
                    revenue: 20,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::UpperLeftFace,
                    to_centre: Some(0.4),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "58".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        dit: Some((End, 10, Bar)),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::UpperLeftFace,
                    to_centre: Some(0.7),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "63".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 40,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopLeftCorner,
                    to_centre: Some(0.1),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "70".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "87".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        dit: Some((End, 10, Circle)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::RightCorner,
                    to_centre: Some(0.4),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "88".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        dit: Some((End, 10, Circle)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::UpperRightFace,
                    to_centre: Some(0.4),
                    ..Default::default()
                }],
                ..Default::default()
            },
            // NOTE: in this tile there are two separate track segments, each
            // of which *passes through* a city rather than being divided into
            // two separate track segments.
            Tile {
                name: "120".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::LeftCorner),
                        revenue: 60,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::TopRightCorner),
                        revenue: 60,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("T".to_string()),
                        location: Location::LowerRightFace,
                        to_centre: Some(0.3),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::BottomFace,
                        to_centre: Some(1.0),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "122".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Double(
                            CornerLocation::LeftCorner,
                        ),
                        revenue: 80,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Double(
                            CornerLocation::TopRightCorner,
                        ),
                        revenue: 80,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("T".to_string()),
                        location: Location::BottomRightCorner,
                        nudge: Some((Direction::N, 0.2)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::BottomFace,
                        to_centre: Some(1.0),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "124".to_string(),
                colour: Grey,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Quad(CentreLocation::Centre),
                    revenue: 100,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::City("T".to_string()),
                        location: Location::TopRightCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::RightCorner,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "201".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Single(Location::Centre),
                    revenue: 30,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.3),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::LowerLeftFace,
                        to_centre: Some(0.4),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "202".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Single(Location::Centre),
                    revenue: 30,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.3),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::LowerLeftFace,
                        to_centre: Some(0.4),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "204".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        dit: Some((End, 10, Circle)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::LowerLeftFace,
                    to_centre: Some(0.5),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "207".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 40,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::TopRightCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "208".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 40,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::BottomLeftCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "611".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 40,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopLeftCorner,
                    to_centre: Some(0.1),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "619".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 30,
                    ..Default::default()
                }],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopRightCorner,
                    to_centre: Some(0.15),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "621".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Single(Location::Centre),
                    revenue: 30,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::UpperLeftFace,
                        to_centre: Some(0.3),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::LowerLeftFace,
                        to_centre: Some(0.4),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "622".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 40,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopRightCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::BottomLeftCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "623".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 50,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::TopRightCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "624".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "625".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "626".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardR(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "637".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::UpperLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::UpperLeft),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::UpperRight),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::UpperRight),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(
                            Location::BottomLeftCorner,
                        ),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::TopLeftCorner),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::RightCorner),
                        revenue: 50,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::LeftCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopRightCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "639".to_string(),
                colour: Grey,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Quad(CentreLocation::Centre),
                    revenue: 100,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::TopRightCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::RightCorner,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "801".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Double(CornerLocation::Centre),
                    revenue: 50,
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::RightCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopRightCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "911".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        dit: Some((End, 10, Circle)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::UpperLeftFace,
                    to_centre: Some(0.5),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "X1".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.0, 0.9)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.9, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerLeft),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerLeft),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerRight),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerRight),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::TopFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::LowerLeftFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::LowerRightFace),
                        revenue: 50,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::BottomLeftCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        nudge: Some((Direction::SSW, 0.16)),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X2".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleR(HexFace::LowerLeft),
                        span: Some((0.0, 0.9)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::LowerLeft),
                        span: Some((0.9, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.0, 0.9)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.9, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::TopFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::UpperLeftFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::LowerRightFace),
                        revenue: 50,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::BottomLeftCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::RightCorner,
                        nudge: Some((Direction::NW, 0.12)),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X3".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::Top),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Top),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::TopFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::BottomFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::LeftCorner),
                        revenue: 50,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::BottomLeftCorner,
                        nudge: Some((Direction::NW, 0.1)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        nudge: Some((Direction::SSW, 0.16)),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X4".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Top),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Top),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::LowerRight),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::LowerRight),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::TopFace),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::LeftCorner),
                        revenue: 50,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::RightCorner),
                        revenue: 50,
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::BottomRightCorner,
                        nudge: Some((Direction::N, 0.2)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::BottomLeftCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X5".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::Top),
                        span: Some((0.0, 0.1)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Top),
                        clip: Some((0.3625, 0.75)),
                        span: Some((0.1, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::TopFace),
                        revenue: 70,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Double(CornerLocation::Centre),
                        revenue: 70,
                        nudge: Some((Direction::S, 0.1)),
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::BottomLeftCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::LeftCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X6".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::LeftCorner),
                        revenue: 70,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Double(CornerLocation::Centre),
                        revenue: 70,
                        rotate: Some(CityRotation::Cw90),
                        nudge: Some((Direction::E, 0.1)),
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::BottomLeftCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X7".to_string(),
                colour: Brown,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        span: Some((0.0, 0.9)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        span: Some((0.9, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::LowerLeft),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::LowerRight),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Top),
                        span: Some((0.0, 0.65)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        span: Some((0.0, 0.35)),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::UpperRightFace),
                        revenue: 70,
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Double(CornerLocation::Centre),
                        revenue: 70,
                        nudge: Some((Direction::S, 0.3)),
                        ..Default::default()
                    },
                ],
                labels: vec![
                    Label {
                        label_type: LabelType::City("M".to_string()),
                        location: Location::LeftCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        to_centre: Some(0.15),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "X8".to_string(),
                colour: Grey,
                track: vec![
                    Track {
                        track_type: Mid(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::Top),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Mid(HexFace::UpperRight),
                        ..Default::default()
                    },
                ],
                cities: vec![City {
                    city_type: CityType::Triple(CentreLocation::Centre),
                    revenue: 60,
                    rotate: Some(CityRotation::HalfTurn),
                    ..Default::default()
                }],
                labels: vec![
                    Label {
                        label_type: LabelType::City("O".to_string()),
                        location: Location::LeftCorner,
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::BottomLeftCorner,
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Tile {
                name: "IN10".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        dit: Some((End, 30, Bar)),
                        span: Some((0.0, 0.85)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        span: Some((0.85, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        dit: Some((End, 30, Bar)),
                        span: Some((0.0, 0.85)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        span: Some((0.85, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::UpperLeft),
                        span: Some((0.125, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Top),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopLeftCorner,
                    to_centre: Some(0.1),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Tile {
                name: "IN11".to_string(),
                colour: Green,
                track: vec![
                    Track {
                        track_type: Straight(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::LowerRight),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::LowerRight),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        span: Some((0.0, 0.5)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::Bottom),
                        span: Some((0.5, 1.0)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
                        ..Default::default()
                    },
                ],
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::LowerLeftFace),
                        revenue: 30,
                        nudge: Some((Direction::NEE, 0.2)),
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::UpperRightFace),
                        revenue: 30,
                        nudge: Some((Direction::SWW, 0.2)),
                        ..Default::default()
                    },
                ],
                labels: vec![Label {
                    label_type: LabelType::Revenue(0),
                    location: Location::TopLeftCorner,
                    to_centre: Some(0.1),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum TileRotation {
    Zero,
    Cw1,
    Cw2,
    Half,
    Acw2,
    Acw1,
}

impl TileRotation {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

impl Default for TileRotation {
    fn default() -> Self {
        TileRotation::Zero
    }
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub style: TokenStyle,
    pub x_pcnt: usize,
    pub y_pcnt: usize,
}

#[derive(Serialize, Deserialize)]
pub enum TokenStyle {
    SideArcs {
        bg: TokenColour,
        fg: TokenColour,
        text: TokenColour,
    },
    TopArcs {
        bg: TokenColour,
        fg: TokenColour,
        text: TokenColour,
    },
    TopSquares {
        bg: TokenColour,
        fg: TokenColour,
        text: TokenColour,
    },
    TopLines {
        bg: TokenColour,
        fg: TokenColour,
        text: TokenColour,
    },
    TopTriangles {
        bg: TokenColour,
        fg: TokenColour,
        text: TokenColour,
    },
    TripleTriangles {
        bg: TokenColour,
        fg: TokenColour,
        text: TokenColour,
    },
}

#[derive(Serialize, Deserialize)]
pub struct TokenColour {
    pub red: usize,
    pub green: usize,
    pub blue: usize,
    pub alpha: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct TileDescr {
    pub tile: String,
    #[serde(default, skip_serializing_if = "TileRotation::is_default")]
    pub rotation: TileRotation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<(usize, String)>,
}

#[derive(Serialize, Deserialize)]
struct HexAddress {
    row: usize,
    col: usize,
    #[serde(flatten)]
    tile: Option<TileDescr>,
}

impl HexAddress {
    fn with_tile(mut self, tile: Option<TileDescr>) -> Self {
        self.tile = tile;
        self
    }
}

#[derive(Serialize, Deserialize)]
struct Descr {
    tiles: Vec<HexAddress>,
}

impl std::convert::From<&rusty_map::RotateCW> for TileRotation {
    fn from(src: &rusty_map::RotateCW) -> Self {
        use rusty_map::RotateCW::*;

        match src {
            Zero => TileRotation::Zero,
            One => TileRotation::Cw1,
            Two => TileRotation::Cw2,
            Three => TileRotation::Half,
            Four => TileRotation::Acw2,
            Five => TileRotation::Acw1,
        }
    }
}

impl std::convert::From<&TileRotation> for rusty_map::RotateCW {
    fn from(src: &TileRotation) -> Self {
        use self::TileRotation::*;
        use rusty_map::RotateCW;

        match src {
            Zero => RotateCW::Zero,
            Cw1 => RotateCW::One,
            Cw2 => RotateCW::Two,
            Half => RotateCW::Three,
            Acw2 => RotateCW::Four,
            Acw1 => RotateCW::Five,
        }
    }
}

impl std::convert::From<&rusty_token::Colour> for TokenColour {
    fn from(src: &rusty_token::Colour) -> Self {
        Self {
            red: src.red,
            blue: src.blue,
            green: src.green,
            alpha: src.alpha,
        }
    }
}

impl std::convert::From<&TokenColour> for rusty_token::Colour {
    fn from(src: &TokenColour) -> Self {
        Self {
            red: src.red,
            blue: src.blue,
            green: src.green,
            alpha: src.alpha,
        }
    }
}

impl std::convert::From<&rusty_token::TokenStyle> for TokenStyle {
    fn from(src: &rusty_token::TokenStyle) -> Self {
        use rusty_token::TokenStyle::*;

        match src {
            SideArcs { bg, fg, text } => Self::SideArcs {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopArcs { bg, fg, text } => Self::TopArcs {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopSquares { bg, fg, text } => Self::TopSquares {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopLines { bg, fg, text } => Self::TopLines {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopTriangles { bg, fg, text } => Self::TopTriangles {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TripleTriangles { bg, fg, text } => Self::TripleTriangles {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
        }
    }
}

impl std::convert::From<&TokenStyle> for rusty_token::TokenStyle {
    fn from(src: &TokenStyle) -> Self {
        use TokenStyle::*;

        match src {
            SideArcs { bg, fg, text } => Self::SideArcs {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopArcs { bg, fg, text } => Self::TopArcs {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopSquares { bg, fg, text } => Self::TopSquares {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopLines { bg, fg, text } => Self::TopLines {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TopTriangles { bg, fg, text } => Self::TopTriangles {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
            TripleTriangles { bg, fg, text } => Self::TripleTriangles {
                bg: bg.into(),
                fg: fg.into(),
                text: text.into(),
            },
        }
    }
}

impl std::convert::From<&rusty_token::Token> for Token {
    fn from(src: &rusty_token::Token) -> Self {
        Token {
            style: (&src.style).into(),
            x_pcnt: src.x_pcnt,
            y_pcnt: src.y_pcnt,
        }
    }
}

impl std::convert::From<&Token> for rusty_token::Token {
    fn from(src: &Token) -> Self {
        Self {
            style: (&src.style).into(),
            x_pcnt: src.x_pcnt,
            y_pcnt: src.y_pcnt,
        }
    }
}

impl std::convert::From<&rusty_map::HexAddress> for HexAddress {
    fn from(src: &rusty_map::HexAddress) -> Self {
        let (row, col) = src.into();
        HexAddress {
            row: row,
            col: col,
            tile: None,
        }
    }
}

impl std::convert::From<&HexAddress> for rusty_map::HexAddress {
    fn from(src: &HexAddress) -> Self {
        (src.row, src.col).into()
    }
}

impl std::convert::From<&rusty_map::descr::TileDescr> for TileDescr {
    fn from(src: &rusty_map::descr::TileDescr) -> Self {
        TileDescr {
            tile: src.tile.clone(),
            rotation: (&src.rotation).into(),
            tokens: src
                .tokens
                .iter()
                .map(|(ix, tok)| (*ix, tok.into()))
                .collect(),
        }
    }
}

// NOTE: cannot implement From<(&HexAddress, &TileDescr)> for TileDescr.
fn tile_descr(addr: &HexAddress, descr: &TileDescr) -> rusty_map::TileDescr {
    rusty_map::TileDescr {
        row: addr.row,
        col: addr.col,
        tile: descr.tile.clone(),
        rotation: (&descr.rotation).into(),
        tokens: descr
            .tokens
            .iter()
            .map(|(ix, tok)| (*ix, tok.into()))
            .collect(),
    }
}

impl std::convert::From<&rusty_map::descr::Descr> for Descr {
    fn from(src: &rusty_map::descr::Descr) -> Self {
        let tiles: &HashMap<_, _> = src.into();
        let tiles: Vec<HexAddress> = tiles
            .iter()
            .map(|(k, v)| {
                HexAddress::from(k).with_tile(v.as_ref().map(|td| td.into()))
            })
            .collect();
        Descr { tiles: tiles }
    }
}

impl std::convert::From<&Descr> for rusty_map::descr::Descr {
    fn from(src: &Descr) -> Self {
        let tiles: HashMap<_, _> = src
            .tiles
            .iter()
            .map(|addr| {
                (
                    addr.into(),
                    addr.tile.as_ref().map(|td| tile_descr(addr, td)),
                )
            })
            .collect();
        tiles.into()
    }
}

/// Reads a map configuration from disk.
pub fn read_map_descr<P: AsRef<Path>>(
    path: P,
) -> Result<rusty_map::descr::Descr, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let descr: Descr = serde_json::from_reader(reader)?;
    Ok((&descr).into())
}

/// Writes a map configuration to disk.
pub fn write_map_descr<P: AsRef<Path>>(
    path: P,
    descr: &rusty_map::descr::Descr,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let descr: Descr = descr.into();
    if pretty {
        serde_json::to_writer_pretty(file, &descr)?;
    } else {
        serde_json::to_writer(file, &descr)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read<P: AsRef<Path>>(path: P) -> Result<Tiles, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let tiles = serde_json::from_reader(reader)?;
        Ok(tiles)
    }

    fn write<P: AsRef<Path>>(
        path: P,
        tiles: &Tiles,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer(file, tiles)?;
        Ok(())
    }

    fn init_hex() -> Hex {
        let hex_max_diameter = 125.0;
        Hex::new(hex_max_diameter)
    }

    #[test]
    fn compare_catalogues() {
        let hex = init_hex();
        let catalogue = rusty_catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;
        // NOTE: have added new tiles to the catalogue for 1867 map.
        // assert_eq!(catalogue.len(), de_tiles.len());
        assert!(catalogue.len() >= de_tiles.len());
    }

    #[test]
    fn json_round_trip_1() {
        let filename = "test-json_round_trip_1.json";
        let de_in = super::test_tiles();
        let write_res = write(filename, &de_in);
        assert!(write_res.is_ok(), "Could not write {}", filename);
        let read_res = read(filename);
        assert!(read_res.is_ok(), "Could not read {}", filename);
        let de_out = read_res.unwrap();
        assert_eq!(de_in.tiles, de_out.tiles);
    }

    #[test]
    fn json_round_trip_2() {
        let hex = init_hex();
        let cat_in = rusty_catalogue::tile_catalogue(&hex);
        let filename = "test-json_round_trip_2.json";
        let pretty = false;

        let write_res = super::write_tiles(filename, &cat_in, pretty);
        assert!(write_res.is_ok(), "Could not write {}", filename);
        let read_res = super::read_tiles(filename, &hex);
        assert!(read_res.is_ok(), "Could not read {}", filename);
        let cat_out = read_res.unwrap();
        assert_eq!(cat_in, cat_out);
    }

    #[test]
    fn compare_to_catalogue_de() {
        let hex = init_hex();
        let catalogue = rusty_catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;

        for (cat_tile, de_descr) in catalogue.iter().zip(&de_tiles) {
            let de_tile: rusty_tile::Tile = de_descr.build(&hex);
            assert_eq!(
                cat_tile, &de_tile,
                "Tiles differ: '{}' and '{}'",
                cat_tile.name, de_tile.name
            )
        }
    }

    #[test]
    fn compare_to_catalogue_ser() {
        let hex = init_hex();
        let catalogue = rusty_catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;

        for (cat_tile, de_descr) in catalogue.iter().zip(&de_tiles) {
            let cat_descr: Tile = cat_tile.into();
            assert_eq!(
                &cat_descr, de_descr,
                "Tiles differ: '{}' and '{}'",
                cat_descr.name, de_descr.name
            )
        }
    }
}
