/// Load tile catalogues from disk.
use crate::hex::Hex;

use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Tiles {
    pub tiles: Vec<Tile>,
}

impl std::convert::From<&[crate::tile::Tile]> for Tiles {
    fn from(src: &[crate::tile::Tile]) -> Self {
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
}

impl std::convert::From<crate::hex::HexColour> for HexColour {
    fn from(src: crate::hex::HexColour) -> Self {
        use crate::hex::HexColour::*;

        match src {
            Yellow => HexColour::Yellow,
            Green => HexColour::Green,
            Brown => HexColour::Brown,
            Grey => HexColour::Grey,
            Red => HexColour::Red,
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

impl std::convert::From<crate::hex::HexFace> for HexFace {
    fn from(src: crate::hex::HexFace) -> Self {
        use crate::hex::HexFace::*;

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

impl std::convert::From<&crate::tile::Tile> for Tile {
    fn from(src: &crate::tile::Tile) -> Self {
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

impl std::convert::From<crate::track::TrackEnd> for TrackEnd {
    fn from(src: crate::track::TrackEnd) -> Self {
        use crate::track::TrackEnd::*;

        match src {
            Start => TrackEnd::Start,
            End => TrackEnd::End,
        }
    }
}

impl std::convert::From<TrackEnd> for crate::track::TrackEnd {
    fn from(src: TrackEnd) -> Self {
        use crate::track::TrackEnd::*;

        match src {
            TrackEnd::Start => Start,
            TrackEnd::End => End,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Track {
    #[serde(flatten)]
    pub track_type: TrackType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dit: Option<(TrackEnd, usize)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<(f64, f64)>,
}

impl std::convert::From<&crate::track::Track> for Track {
    fn from(src: &crate::track::Track) -> Self {
        use crate::track::TrackCurve::*;

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
            dit: src.dit.map(|(end, revenue)| (end.into(), revenue)),
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

impl std::convert::From<&crate::hex::HexPosition> for Location {
    fn from(src: &crate::hex::HexPosition) -> Self {
        use crate::hex::HexPosition::*;

        match src {
            Centre(_delta) => Location::Centre,
            Face(face, _delta) => face.into(),
            Corner(corner, _delta) => corner.into(),
        }
    }
}

impl std::convert::From<&crate::hex::HexFace> for Location {
    fn from(src: &crate::hex::HexFace) -> Self {
        use crate::hex::HexFace::*;

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

impl std::convert::From<&crate::hex::HexCorner> for Location {
    fn from(src: &crate::hex::HexCorner) -> Self {
        use crate::hex::HexCorner::*;

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

impl std::convert::From<&crate::hex::HexPosition> for CornerLocation {
    fn from(src: &crate::hex::HexPosition) -> Self {
        use crate::hex::HexPosition::*;

        match src {
            Centre(_delta) => CornerLocation::Centre,
            Face(_face, _delta) => panic!("Cannot convert Face into Corner"),
            Corner(corner, _delta) => corner.into(),
        }
    }
}

impl std::convert::From<&crate::hex::HexCorner> for CornerLocation {
    fn from(src: &crate::hex::HexCorner) -> Self {
        use crate::hex::HexCorner::*;

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
    Dit(CentreLocation),
    Single(Location),
    Double(CornerLocation),
    Triple(CentreLocation),
    Quad(CentreLocation),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Rotation {
    Zero,
    Cw90,
    Acw90,
    HalfTurn,
}

impl std::convert::From<crate::city::Rotation> for Option<Rotation> {
    fn from(src: crate::city::Rotation) -> Self {
        use crate::city::Rotation::*;

        match src {
            Zero => None,
            Cw90 => Some(Rotation::Cw90),
            Acw90 => Some(Rotation::Acw90),
            HalfTurn => Some(Rotation::HalfTurn),
        }
    }
}

impl std::convert::From<Option<&Rotation>> for crate::city::Rotation {
    fn from(src: Option<&Rotation>) -> Self {
        use crate::city::Rotation::*;

        match src {
            None => Zero,
            Some(ref rot) => match rot {
                Rotation::Zero => Zero,
                Rotation::Cw90 => Cw90,
                Rotation::Acw90 => Acw90,
                Rotation::HalfTurn => HalfTurn,
            },
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
    pub rotate: Option<Rotation>,
}

impl std::convert::From<&crate::city::City> for City {
    fn from(src: &crate::city::City) -> Self {
        use crate::city::Tokens;
        use crate::hex::Delta;
        use crate::hex::HexPosition::*;

        let revenue = src.revenue;
        let position = &src.position;
        let city_type = match src.tokens {
            Tokens::Dit => CityType::Dit(CentreLocation::Centre),
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
        let rotate = src.angle.into();
        Self {
            city_type,
            revenue,
            nudge,
            rotate,
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum LabelType {
    City(String),
    Y(()),
    TileName(()),
    Revenue(usize),
}

impl std::convert::From<&crate::label::Label> for LabelType {
    fn from(src: &crate::label::Label) -> Self {
        use crate::label::Label as L;

        match src {
            L::City(ref name) => LabelType::City(name.clone()),
            L::Y => LabelType::Y(()),
            L::TileName => LabelType::TileName(()),
            L::Revenue(revenue) => LabelType::Revenue(*revenue),
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

impl std::convert::From<&crate::tile::LabelAndPos> for Label {
    fn from(src: &crate::tile::LabelAndPos) -> Self {
        use crate::hex::Delta::*;
        use crate::hex::HexPosition::*;

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

impl std::convert::From<&crate::hex::Direction> for Direction {
    fn from(src: &crate::hex::Direction) -> Self {
        use crate::hex::Direction::*;

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

impl std::convert::From<&Direction> for crate::hex::Direction {
    fn from(src: &Direction) -> Self {
        use crate::hex::Direction::*;

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
) -> Result<crate::tile::Tile, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tile: Tile = serde_json::from_reader(reader)?;
    Ok(tile.build(hex))
}

/// Reads multiple tiles from disk.
pub fn read_tiles<P: AsRef<Path>>(
    path: P,
    hex: &Hex,
) -> Result<Vec<crate::tile::Tile>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tiles: Tiles = serde_json::from_reader(reader)?;
    Ok(tiles.build(hex))
}

/// Writes a single tile to disk.
pub fn write_tile<P: AsRef<Path>>(
    path: P,
    tile: &crate::tile::Tile,
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
    tiles: &[crate::tile::Tile],
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
    pub fn build(&self, hex: &Hex) -> crate::tile::Tiles {
        self.tiles.iter().map(|t| t.build(hex)).collect()
    }
}

impl Tile {
    pub fn build(&self, hex: &Hex) -> crate::tile::Tile {
        let tile = crate::tile::Tile::new(
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

impl From<&LabelType> for crate::label::Label {
    fn from(lt: &LabelType) -> Self {
        match lt {
            LabelType::City(ref name) => {
                crate::label::Label::City(name.clone())
            }
            LabelType::Y(()) => crate::label::Label::Y,
            LabelType::TileName(()) => crate::label::Label::TileName,
            LabelType::Revenue(ix) => crate::label::Label::Revenue(*ix),
        }
    }
}

impl From<&CornerLocation> for crate::hex::HexPosition {
    fn from(locn: &CornerLocation) -> Self {
        use CornerLocation::*;
        match locn {
            Centre => crate::hex::HexPosition::Centre(None),
            TopLeftCorner => crate::hex::HexCorner::TopLeft.into(),
            TopRightCorner => crate::hex::HexCorner::TopRight.into(),
            LeftCorner => crate::hex::HexCorner::Left.into(),
            RightCorner => crate::hex::HexCorner::Right.into(),
            BottomLeftCorner => crate::hex::HexCorner::BottomLeft.into(),
            BottomRightCorner => crate::hex::HexCorner::BottomRight.into(),
        }
    }
}

impl From<&Location> for crate::hex::HexPosition {
    fn from(locn: &Location) -> Self {
        use Location::*;
        match locn {
            Centre => crate::hex::HexPosition::Centre(None),
            TopLeftCorner => crate::hex::HexCorner::TopLeft.into(),
            TopRightCorner => crate::hex::HexCorner::TopRight.into(),
            LeftCorner => crate::hex::HexCorner::Left.into(),
            RightCorner => crate::hex::HexCorner::Right.into(),
            BottomLeftCorner => crate::hex::HexCorner::BottomLeft.into(),
            BottomRightCorner => crate::hex::HexCorner::BottomRight.into(),
            TopFace => crate::hex::HexFace::Top.into(),
            UpperRightFace => crate::hex::HexFace::UpperRight.into(),
            LowerRightFace => crate::hex::HexFace::LowerRight.into(),
            BottomFace => crate::hex::HexFace::Bottom.into(),
            LowerLeftFace => crate::hex::HexFace::LowerLeft.into(),
            UpperLeftFace => crate::hex::HexFace::UpperLeft.into(),
        }
    }
}

impl Label {
    pub fn position(&self) -> crate::hex::HexPosition {
        let position: crate::hex::HexPosition = (&self.location).into();
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

impl From<&HexColour> for crate::hex::HexColour {
    fn from(c: &HexColour) -> crate::hex::HexColour {
        match c {
            HexColour::Yellow => crate::hex::HexColour::Yellow,
            HexColour::Green => crate::hex::HexColour::Green,
            HexColour::Brown => crate::hex::HexColour::Brown,
            HexColour::Grey => crate::hex::HexColour::Grey,
            HexColour::Red => crate::hex::HexColour::Red,
        }
    }
}

impl From<&HexFace> for crate::hex::HexFace {
    fn from(c: &HexFace) -> crate::hex::HexFace {
        match c {
            HexFace::Top => crate::hex::HexFace::Top,
            HexFace::UpperRight => crate::hex::HexFace::UpperRight,
            HexFace::LowerRight => crate::hex::HexFace::LowerRight,
            HexFace::Bottom => crate::hex::HexFace::Bottom,
            HexFace::LowerLeft => crate::hex::HexFace::LowerLeft,
            HexFace::UpperLeft => crate::hex::HexFace::UpperLeft,
        }
    }
}

impl From<&Track> for crate::track::Track {
    fn from(t: &Track) -> crate::track::Track {
        let track = match t.track_type {
            TrackType::Mid(ref face) => crate::track::Track::mid(face.into()),
            TrackType::Straight(ref face) => {
                crate::track::Track::straight(face.into())
            }
            TrackType::GentleL(ref face) => {
                crate::track::Track::gentle_l(face.into())
            }
            TrackType::GentleR(ref face) => {
                crate::track::Track::gentle_r(face.into())
            }
            TrackType::HardL(ref face) => {
                crate::track::Track::hard_l(face.into())
            }
            TrackType::HardR(ref face) => {
                crate::track::Track::hard_r(face.into())
            }
        };
        let track = if let Some((posn, revenue)) = t.dit {
            track.with_dit(posn.into(), revenue)
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
    pub fn build(&self, revenue: usize) -> crate::city::City {
        use CityType::*;

        match self {
            Dit(_centre) => crate::city::City::central_dit(revenue),
            Single(location) => {
                use crate::city::City;
                use crate::hex::HexCorner::*;
                use crate::hex::HexFace::*;
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
                use crate::city::City;
                use crate::hex::HexCorner::*;
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
            Triple(_centre) => crate::city::City::triple(revenue),
            Quad(_centre) => crate::city::City::quad(revenue),
        }
    }
}

impl City {
    pub fn build(&self) -> crate::city::City {
        let city = self.city_type.build(self.revenue);
        let city = if let Some((ref angle, radius)) = self.nudge {
            city.nudge(angle.into(), radius)
        } else {
            city
        };
        let city = city.rotate(self.rotate.as_ref().into());
        city
    }
}

/// Should yield the same tiles as `crate::catalogue::tile_catalogue()`.
#[allow(dead_code)]
fn test_tiles() -> Tiles {
    use HexColour::*;
    use TrackEnd::*;
    use TrackType::*;

    // TODO: define all of the tiles in crate::catalogue::tile_catalogue().

    Tiles {
        tiles: vec![
            Tile {
                name: "3".to_string(),
                colour: Yellow,
                track: vec![
                    Track {
                        track_type: HardL(HexFace::Bottom),
                        dit: Some((End, 10)),
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
                        dit: Some((End, 10)),
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
                        dit: Some((End, 10)),
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
                    city_type: CityType::Dit(CentreLocation::Centre),
                    revenue: 10,
                    ..Default::default()
                }],
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
                cities: vec![City {
                    city_type: CityType::Dit(CentreLocation::Centre),
                    revenue: 10,
                    ..Default::default()
                }],
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
                    city_type: CityType::Dit(CentreLocation::Centre),
                    revenue: 10,
                    ..Default::default()
                }],
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
                cities: vec![City {
                    city_type: CityType::Dit(CentreLocation::Centre),
                    revenue: 10,
                    ..Default::default()
                }],
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
                        rotate: Some(Rotation::Cw90),
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
                    rotate: Some(Rotation::HalfTurn),
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
                        dit: Some((End, 30)),
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
                        dit: Some((End, 30)),
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
        let catalogue = crate::catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;
        assert_eq!(catalogue.len(), de_tiles.len());
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
        let cat_in = crate::catalogue::tile_catalogue(&hex);
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
        let catalogue = crate::catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;

        for (cat_tile, de_descr) in catalogue.iter().zip(&de_tiles) {
            let de_tile: crate::tile::Tile = de_descr.build(&hex);
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
        let catalogue = crate::catalogue::tile_catalogue(&hex);
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
