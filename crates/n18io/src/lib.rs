/// Load tile catalogues from disk.
use n18hex::Hex;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod routes;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Tiles {
    pub tiles: Vec<Tile>,
}

impl std::convert::From<&[n18tile::Tile]> for Tiles {
    fn from(src: &[n18tile::Tile]) -> Self {
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

impl std::convert::From<n18hex::HexColour> for HexColour {
    fn from(src: n18hex::HexColour) -> Self {
        use n18hex::HexColour::*;

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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
enum HexFace {
    Top,
    UpperRight,
    LowerRight,
    Bottom,
    LowerLeft,
    UpperLeft,
}

impl std::convert::From<n18hex::HexFace> for HexFace {
    fn from(src: n18hex::HexFace) -> Self {
        use n18hex::HexFace::*;

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
    #[serde(
        default = "show_tile_name_default",
        skip_serializing_if = "show_tile_name_skip"
    )]
    pub show_tile_name: bool,
}

/// By default, show tile names on the tile.
fn show_tile_name_default() -> bool {
    true
}

/// Only serialise 'show_tile_name' when its value is `false`.
fn show_tile_name_skip(show: &bool) -> bool {
    *show
}

impl std::convert::From<&n18tile::Tile> for Tile {
    fn from(src: &n18tile::Tile) -> Self {
        Self {
            name: src.name.clone(),
            colour: src.colour.into(),
            track: src.tracks().iter().map(|track| track.into()).collect(),
            cities: src.cities().iter().map(|city| city.into()).collect(),
            labels: src.labels().iter().map(|lnp| lnp.into()).collect(),
            show_tile_name: src.is_tile_name_visible(),
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
            show_tile_name: true,
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

impl std::convert::From<n18tile::TrackEnd> for TrackEnd {
    fn from(src: n18tile::TrackEnd) -> Self {
        use n18tile::TrackEnd::*;

        match src {
            Start => TrackEnd::Start,
            End => TrackEnd::End,
        }
    }
}

impl std::convert::From<TrackEnd> for n18tile::TrackEnd {
    fn from(src: TrackEnd) -> Self {
        use n18tile::TrackEnd::*;

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

impl std::convert::From<n18tile::DitShape> for DitShape {
    fn from(src: n18tile::DitShape) -> Self {
        use n18tile::DitShape::*;

        match src {
            Bar => DitShape::Bar,
            Circle => DitShape::Circle,
        }
    }
}

impl std::convert::From<DitShape> for n18tile::DitShape {
    fn from(src: DitShape) -> Self {
        use n18tile::DitShape::*;

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

impl std::convert::From<&n18tile::Track> for Track {
    fn from(src: &n18tile::Track) -> Self {
        use n18tile::TrackCurve::*;

        let eps = std::f64::EPSILON;
        let span = if src.x0 == 0.0 && (src.x1 - 1.0).abs() < eps {
            None
        } else if src.x0 >= 0.0 && src.x1 <= 1.0 {
            Some((src.x0, src.x1))
        } else {
            panic!("Invalid track span: [{}, {}]", src.x0, src.x1)
        };

        let (track_type, span) = match src.curve {
            Straight => {
                if src.x0 == 0.0 && (src.x1 - 0.5).abs() < eps {
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
            track_type,
            dit: src.dit.map(|(end, revenue, shape)| {
                (end.into(), revenue, shape.into())
            }),
            clip: src.clip,
            span,
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

impl std::convert::From<&n18hex::HexPosition> for Location {
    fn from(src: &n18hex::HexPosition) -> Self {
        use n18hex::HexPosition::*;

        match src {
            Centre(_delta) => Location::Centre,
            Face(face, _delta) => face.into(),
            Corner(corner, _delta) => corner.into(),
        }
    }
}

impl std::convert::From<&n18hex::HexFace> for Location {
    fn from(src: &n18hex::HexFace) -> Self {
        use n18hex::HexFace::*;

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

impl std::convert::From<&n18hex::HexCorner> for Location {
    fn from(src: &n18hex::HexCorner) -> Self {
        use n18hex::HexCorner::*;

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

impl std::convert::From<&n18hex::HexPosition> for CornerLocation {
    fn from(src: &n18hex::HexPosition) -> Self {
        use n18hex::HexPosition::*;

        match src {
            Centre(_delta) => CornerLocation::Centre,
            Face(_face, _delta) => panic!("Cannot convert Face into Corner"),
            Corner(corner, _delta) => corner.into(),
        }
    }
}

impl std::convert::From<&n18hex::HexCorner> for CornerLocation {
    fn from(src: &n18hex::HexCorner) -> Self {
        use n18hex::HexCorner::*;

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
    fn from_rot(src: n18tile::Rotation) -> Option<Self> {
        use n18tile::Rotation::*;

        match src {
            Zero => None,
            Cw90 => Some(CityRotation::Cw90),
            Acw90 => Some(CityRotation::Acw90),
            HalfTurn => Some(CityRotation::HalfTurn),
        }
    }

    fn to_rot(&self) -> n18tile::Rotation {
        use n18tile::Rotation::*;

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
    /// An optional translation `(angle, frac)` where `frac` is relative to
    /// the maximal radius of the tile (i.e., from the centre to any corner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(Direction, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<CityRotation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_colour: Option<HexColour>,
}

impl std::convert::From<&n18tile::City> for City {
    fn from(src: &n18tile::City) -> Self {
        use n18hex::Delta;
        use n18hex::HexPosition::*;
        use n18tile::Tokens;

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
                if let Some(Delta::InDir(angle, frac)) = delta {
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

impl std::convert::From<&n18tile::Label> for LabelType {
    fn from(src: &n18tile::Label) -> Self {
        use n18tile::Label as L;

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

impl std::convert::From<&n18tile::LabelAndPos> for Label {
    fn from(src: &n18tile::LabelAndPos) -> Self {
        use n18hex::Delta::*;
        use n18hex::HexPosition::*;

        let label = &src.0;
        let posn = &src.1;
        let nudge = match posn {
            Centre(delta) => {
                if let Some(InDir(angle, frac)) = delta {
                    Some((angle.into(), *frac))
                } else {
                    None
                }
            }
            Face(_face, delta) => {
                if let Some(InDir(angle, frac)) = delta {
                    Some((angle.into(), *frac))
                } else {
                    None
                }
            }
            Corner(_corner, delta) => {
                if let Some(InDir(angle, frac)) = delta {
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
            nudge,
            to_centre,
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
    N30E,
    NE,
    N60E,
    E,
    S60E,
    SE,
    S30E,
    S,
    S30W,
    SW,
    S60W,
    W,
    N60W,
    NW,
    N30W,
}

impl std::convert::From<&n18hex::Direction> for Direction {
    fn from(src: &n18hex::Direction) -> Self {
        use n18hex::Direction::*;

        match src {
            N => Self::N,
            N30E => Self::N30E,
            NE => Self::NE,
            N60E => Self::N60E,
            E => Self::E,
            S60E => Self::S60E,
            SE => Self::SE,
            S30E => Self::S30E,
            S => Self::S,
            S30W => Self::S30W,
            SW => Self::SW,
            S60W => Self::S60W,
            W => Self::W,
            N60W => Self::N60W,
            NW => Self::NW,
            N30W => Self::N30W,
        }
    }
}

impl std::convert::From<&Direction> for n18hex::Direction {
    fn from(src: &Direction) -> Self {
        use n18hex::Direction::*;

        match src {
            Direction::N => N,
            Direction::N30E => N30E,
            Direction::NE => NE,
            Direction::N60E => N60E,
            Direction::E => E,
            Direction::S60E => S60E,
            Direction::SE => SE,
            Direction::S30E => S30E,
            Direction::S => S,
            Direction::S30W => S30W,
            Direction::SW => SW,
            Direction::S60W => S60W,
            Direction::W => W,
            Direction::N60W => N60W,
            Direction::NW => NW,
            Direction::N30W => N30W,
        }
    }
}

/// Reads a single tile from disk.
pub fn read_tile<P: AsRef<Path>>(
    path: P,
    hex: &Hex,
) -> Result<n18tile::Tile, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tile: Tile = serde_json::from_reader(reader)?;
    Ok(tile.build(hex))
}

/// Reads multiple tiles from disk.
pub fn read_tiles<P: AsRef<Path>>(
    path: P,
    hex: &Hex,
) -> Result<Vec<n18tile::Tile>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tiles: Tiles = serde_json::from_reader(reader)?;
    Ok(tiles.build(hex))
}

/// Writes a single tile to disk.
pub fn write_tile<P: AsRef<Path>>(
    path: P,
    tile: &n18tile::Tile,
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
    tiles: &[n18tile::Tile],
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

/// Reads train routes from disk.
pub fn read_routes<P: AsRef<Path>>(
    path: P,
) -> Result<n18route::Routes, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let routes: routes::Routes = serde_json::from_reader(reader)?;
    Ok(routes.into())
}

/// Writes train routes to disk.
pub fn write_routes<P: AsRef<Path>>(
    path: P,
    routes: &n18route::Routes,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let routes: routes::Routes = routes.into();
    if pretty {
        serde_json::to_writer_pretty(file, &routes)?;
    } else {
        serde_json::to_writer(file, &routes)?;
    }
    Ok(())
}

// NOTE: need hex and ctx to construct tiles!

impl Tiles {
    pub fn build(&self, hex: &Hex) -> Vec<n18tile::Tile> {
        self.tiles.iter().map(|t| t.build(hex)).collect()
    }
}

impl Tile {
    pub fn build(&self, hex: &Hex) -> n18tile::Tile {
        let tile = n18tile::Tile::new(
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
        // Hide the tile name label if it should not be displayed.
        if !self.show_tile_name {
            tile.hide_tile_name()
        } else {
            tile
        }
    }
}

impl From<&LabelType> for n18tile::Label {
    fn from(lt: &LabelType) -> Self {
        match lt {
            LabelType::City(ref name) => n18tile::Label::City(name.clone()),
            LabelType::Y(()) => n18tile::Label::Y,
            LabelType::TileName(()) => n18tile::Label::TileName,
            LabelType::MapLocation(ref name) => {
                n18tile::Label::MapLocation(name.clone())
            }
            LabelType::Revenue(ix) => n18tile::Label::Revenue(*ix),
            LabelType::PhaseRevenue(revenues) => {
                let revs = revenues
                    .iter()
                    .map(|(colour, revenue, active)| {
                        (colour.into(), *revenue, *active)
                    })
                    .collect();
                n18tile::Label::PhaseRevenue(revs)
            }
        }
    }
}

impl From<&CornerLocation> for n18hex::HexPosition {
    fn from(locn: &CornerLocation) -> Self {
        use CornerLocation::*;
        match locn {
            Centre => n18hex::HexPosition::Centre(None),
            TopLeftCorner => n18hex::HexCorner::TopLeft.into(),
            TopRightCorner => n18hex::HexCorner::TopRight.into(),
            LeftCorner => n18hex::HexCorner::Left.into(),
            RightCorner => n18hex::HexCorner::Right.into(),
            BottomLeftCorner => n18hex::HexCorner::BottomLeft.into(),
            BottomRightCorner => n18hex::HexCorner::BottomRight.into(),
        }
    }
}

impl From<&Location> for n18hex::HexPosition {
    fn from(locn: &Location) -> Self {
        use Location::*;
        match locn {
            Centre => n18hex::HexPosition::Centre(None),
            TopLeftCorner => n18hex::HexCorner::TopLeft.into(),
            TopRightCorner => n18hex::HexCorner::TopRight.into(),
            LeftCorner => n18hex::HexCorner::Left.into(),
            RightCorner => n18hex::HexCorner::Right.into(),
            BottomLeftCorner => n18hex::HexCorner::BottomLeft.into(),
            BottomRightCorner => n18hex::HexCorner::BottomRight.into(),
            TopFace => n18hex::HexFace::Top.into(),
            UpperRightFace => n18hex::HexFace::UpperRight.into(),
            LowerRightFace => n18hex::HexFace::LowerRight.into(),
            BottomFace => n18hex::HexFace::Bottom.into(),
            LowerLeftFace => n18hex::HexFace::LowerLeft.into(),
            UpperLeftFace => n18hex::HexFace::UpperLeft.into(),
        }
    }
}

impl Label {
    pub fn position(&self) -> n18hex::HexPosition {
        let position: n18hex::HexPosition = (&self.location).into();
        let position = if let Some((ref angle, frac)) = self.nudge {
            // NOTE: retain fractional unit of distance.
            position.in_dir(angle.into(), frac)
        } else {
            position
        };
        if let Some(frac) = self.to_centre {
            position.to_centre(frac)
        } else {
            position
        }
    }
}

impl From<&HexColour> for n18hex::HexColour {
    fn from(c: &HexColour) -> n18hex::HexColour {
        match c {
            HexColour::Yellow => n18hex::HexColour::Yellow,
            HexColour::Green => n18hex::HexColour::Green,
            HexColour::Brown => n18hex::HexColour::Brown,
            HexColour::Grey => n18hex::HexColour::Grey,
            HexColour::Red => n18hex::HexColour::Red,
            HexColour::Blue => n18hex::HexColour::Blue,
            HexColour::Empty => n18hex::HexColour::Empty,
        }
    }
}

impl From<&HexFace> for n18hex::HexFace {
    fn from(c: &HexFace) -> n18hex::HexFace {
        match c {
            HexFace::Top => n18hex::HexFace::Top,
            HexFace::UpperRight => n18hex::HexFace::UpperRight,
            HexFace::LowerRight => n18hex::HexFace::LowerRight,
            HexFace::Bottom => n18hex::HexFace::Bottom,
            HexFace::LowerLeft => n18hex::HexFace::LowerLeft,
            HexFace::UpperLeft => n18hex::HexFace::UpperLeft,
        }
    }
}

impl From<&Track> for n18tile::Track {
    fn from(t: &Track) -> n18tile::Track {
        let track = match t.track_type {
            TrackType::Mid(ref face) => n18tile::Track::mid(face.into()),
            TrackType::Straight(ref face) => {
                n18tile::Track::straight(face.into())
            }
            TrackType::GentleL(ref face) => {
                n18tile::Track::gentle_l(face.into())
            }
            TrackType::GentleR(ref face) => {
                n18tile::Track::gentle_r(face.into())
            }
            TrackType::HardL(ref face) => n18tile::Track::hard_l(face.into()),
            TrackType::HardR(ref face) => n18tile::Track::hard_r(face.into()),
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
        if let Some((x0, x1)) = t.span {
            track.with_span(x0, x1)
        } else {
            track
        }
    }
}

impl CityType {
    pub fn build(&self, revenue: usize) -> n18tile::City {
        use CityType::*;

        match self {
            Single(location) => {
                use n18hex::HexCorner::*;
                use n18hex::HexFace::*;
                use n18tile::City;
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
                use n18hex::HexCorner::*;
                use n18tile::City;
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
            Triple(_centre) => n18tile::City::triple(revenue),
            Quad(_centre) => n18tile::City::quad(revenue),
        }
    }
}

impl City {
    pub fn build(&self) -> n18tile::City {
        let city = self.city_type.build(self.revenue);
        let city = if let Some((ref angle, radius)) = self.nudge {
            city.in_dir(angle.into(), radius)
        } else {
            city
        };
        let city = city.rotate(
            self.rotate
                .as_ref()
                .map(|r| r.to_rot())
                .unwrap_or(n18tile::Rotation::Zero),
        );
        // Apply the optional fill colour.
        if let Some(ref colour) = self.fill_colour {
            city.with_fill(colour.into())
        } else {
            city
        }
    }
}

/// Should yield the same tiles as `n18catalogue::tile_catalogue()`.
#[allow(dead_code)]
fn test_tiles() -> Tiles {
    use DitShape::*;
    use HexColour::*;
    use TrackEnd::*;
    use TrackType::*;

    // TODO: define all of the tiles in n18catalogue::tile_catalogue().

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
                ..Default::default()
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
                    to_centre: Some(0.2),
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
                    to_centre: Some(0.2),
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
                    to_centre: Some(0.5),
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
                    to_centre: Some(0.2),
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
                        nudge: Some((Direction::W, 0.15)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::Centre,
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
                        location: Location::Centre,
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
                        to_centre: Some(0.05),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::RightCorner,
                        to_centre: Some(0.08),
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
                        to_centre: Some(0.25),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::LowerLeftFace,
                        to_centre: Some(0.2),
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
                        to_centre: Some(0.25),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::LowerLeftFace,
                        to_centre: Some(0.2),
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
                    to_centre: Some(0.25),
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
                        to_centre: Some(0.15),
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
                        to_centre: Some(0.15),
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
                    to_centre: Some(0.125),
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
                        to_centre: Some(0.1),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Y(()),
                        location: Location::LowerLeftFace,
                        to_centre: Some(0.2),
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
                        to_centre: Some(0.15),
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
                        to_centre: Some(0.25),
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
                        to_centre: Some(0.05),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::RightCorner,
                        to_centre: Some(0.08),
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
                        to_centre: Some(0.2),
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
                    to_centre: Some(0.25),
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
                        nudge: Some((Direction::E, 0.05)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        nudge: Some((Direction::S30W, 0.16)),
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
                        nudge: Some((Direction::E, 0.05)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::RightCorner,
                        nudge: Some((Direction::N60W, 0.15)),
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
                        nudge: Some((Direction::N30W, 0.1)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        nudge: Some((Direction::S30W, 0.16)),
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
                        nudge: Some((Direction::E, 0.05)),
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
                        nudge: Some((Direction::E, 0.05)),
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
                        to_centre: Some(0.15),
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
                        to_centre: Some(0.15),
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
                        nudge: Some((Direction::N60E, 0.2)),
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::UpperRightFace),
                        revenue: 30,
                        nudge: Some((Direction::S60W, 0.2)),
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
struct Token {
    pub style: TokenStyle,
    pub x_pcnt: usize,
    pub y_pcnt: usize,
}

#[derive(Serialize, Deserialize)]
enum TokenStyle {
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
    TribandV {
        sides: TokenColour,
        middle: TokenColour,
        text: TokenColour,
    },
    TribandH {
        sides: TokenColour,
        middle: TokenColour,
        text: TokenColour,
    },
    TricolourV {
        left: TokenColour,
        middle: TokenColour,
        right: TokenColour,
        text: TokenColour,
    },
    TricolourH {
        top: TokenColour,
        middle: TokenColour,
        bottom: TokenColour,
        text: TokenColour,
    },
}

#[derive(Serialize, Deserialize)]
struct TokenColour {
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

impl std::convert::From<&n18map::RotateCW> for TileRotation {
    fn from(src: &n18map::RotateCW) -> Self {
        use n18map::RotateCW::*;

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

impl std::convert::From<&TileRotation> for n18map::RotateCW {
    fn from(src: &TileRotation) -> Self {
        use self::TileRotation::*;
        use n18map::RotateCW;

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

impl std::convert::From<&n18token::Colour> for TokenColour {
    fn from(src: &n18token::Colour) -> Self {
        Self {
            red: src.red,
            blue: src.blue,
            green: src.green,
            alpha: src.alpha,
        }
    }
}

impl std::convert::From<&TokenColour> for n18token::Colour {
    fn from(src: &TokenColour) -> Self {
        Self {
            red: src.red,
            blue: src.blue,
            green: src.green,
            alpha: src.alpha,
        }
    }
}

impl std::convert::From<&n18token::TokenStyle> for TokenStyle {
    fn from(src: &n18token::TokenStyle) -> Self {
        use n18token::TokenStyle::*;

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
            TribandV {
                sides,
                middle,
                text,
            } => Self::TribandV {
                sides: sides.into(),
                middle: middle.into(),
                text: text.into(),
            },
            TribandH {
                sides,
                middle,
                text,
            } => Self::TribandH {
                sides: sides.into(),
                middle: middle.into(),
                text: text.into(),
            },
            TricolourV {
                left,
                middle,
                right,
                text,
            } => Self::TricolourV {
                left: left.into(),
                middle: middle.into(),
                right: right.into(),
                text: text.into(),
            },
            TricolourH {
                top,
                middle,
                bottom,
                text,
            } => Self::TricolourH {
                top: top.into(),
                middle: middle.into(),
                bottom: bottom.into(),
                text: text.into(),
            },
        }
    }
}

impl std::convert::From<&TokenStyle> for n18token::TokenStyle {
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
            TribandV {
                sides,
                middle,
                text,
            } => Self::TribandV {
                sides: sides.into(),
                middle: middle.into(),
                text: text.into(),
            },
            TribandH {
                sides,
                middle,
                text,
            } => Self::TribandH {
                sides: sides.into(),
                middle: middle.into(),
                text: text.into(),
            },
            TricolourV {
                left,
                middle,
                right,
                text,
            } => Self::TricolourV {
                left: left.into(),
                middle: middle.into(),
                right: right.into(),
                text: text.into(),
            },
            TricolourH {
                top,
                middle,
                bottom,
                text,
            } => Self::TricolourH {
                top: top.into(),
                middle: middle.into(),
                bottom: bottom.into(),
                text: text.into(),
            },
        }
    }
}

impl std::convert::From<&n18token::Token> for Token {
    fn from(src: &n18token::Token) -> Self {
        Token {
            style: (&src.style).into(),
            x_pcnt: src.x_pcnt,
            y_pcnt: src.y_pcnt,
        }
    }
}

impl std::convert::From<&Token> for n18token::Token {
    fn from(src: &Token) -> Self {
        Self {
            style: (&src.style).into(),
            x_pcnt: src.x_pcnt,
            y_pcnt: src.y_pcnt,
        }
    }
}

impl std::convert::From<&n18map::HexAddress> for HexAddress {
    fn from(src: &n18map::HexAddress) -> Self {
        let (row, col) = src.into();
        HexAddress {
            row,
            col,
            tile: None,
        }
    }
}

impl std::convert::From<&HexAddress> for n18map::HexAddress {
    fn from(src: &HexAddress) -> Self {
        (src.row, src.col).into()
    }
}

impl std::convert::From<&n18map::descr::TileDescr> for TileDescr {
    fn from(src: &n18map::descr::TileDescr) -> Self {
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
fn tile_descr(addr: &HexAddress, descr: &TileDescr) -> n18map::TileDescr {
    n18map::TileDescr {
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

impl std::convert::From<&n18map::descr::Descr> for Descr {
    fn from(src: &n18map::descr::Descr) -> Self {
        let tiles: &BTreeMap<_, _> = src.into();
        let tiles: Vec<HexAddress> = tiles
            .iter()
            .map(|(k, v)| {
                HexAddress::from(k).with_tile(v.as_ref().map(|td| td.into()))
            })
            .collect();
        Descr { tiles }
    }
}

impl std::convert::From<&Descr> for n18map::descr::Descr {
    fn from(src: &Descr) -> Self {
        let tiles: BTreeMap<_, _> = src
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
) -> Result<n18map::descr::Descr, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let descr: Descr = serde_json::from_reader(reader)?;
    Ok((&descr).into())
}

/// Writes a map configuration to disk.
pub fn write_map_descr<P: AsRef<Path>>(
    path: P,
    descr: &n18map::descr::Descr,
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

    static OUT_DIR: &str = "../../tests/output";

    fn output_path(file: &'static str) -> std::path::PathBuf {
        std::path::Path::new(OUT_DIR).join(file)
    }

    #[test]
    fn compare_catalogues() {
        let hex = init_hex();
        let catalogue = n18catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;
        // NOTE: have added new tiles to the catalogue for 1867 map.
        // assert_eq!(catalogue.len(), de_tiles.len());
        assert!(catalogue.len() >= de_tiles.len());
    }

    #[test]
    fn json_round_trip_1() {
        let filename = output_path("test-json_round_trip_1.json");
        let de_in = super::test_tiles();
        let write_res = write(&filename, &de_in);
        assert!(write_res.is_ok(), "Could not write {}", filename.display());
        let read_res = read(&filename);
        assert!(read_res.is_ok(), "Could not read {}", filename.display());
        let de_out = read_res.unwrap();
        assert_eq!(de_in.tiles, de_out.tiles);
    }

    #[test]
    fn json_round_trip_2() {
        let hex = init_hex();
        let cat_in = n18catalogue::tile_catalogue(&hex);
        let filename = output_path("test-json_round_trip_2.json");
        let pretty = false;

        let write_res = super::write_tiles(&filename, &cat_in, pretty);
        assert!(write_res.is_ok(), "Could not write {}", filename.display());
        let read_res = super::read_tiles(&filename, &hex);
        assert!(read_res.is_ok(), "Could not read {}", filename.display());
        let cat_out = read_res.unwrap();
        assert_eq!(cat_in, cat_out);
    }

    #[test]
    fn json_round_trip_1867() {
        // The 1867 game includes starting tiles (part of the map) and
        // off-board tiles, which make use of features such as hiding the tile
        // names and marking unavailable token spaces with fill colours, that
        // are not used by any of the tiles in n18catalogue.
        // This test case ensures these features are correctly (de)serialised.
        use n18game::Game;
        let hex = init_hex();
        let game = n18game::_1867::Game::new(&hex);
        let cat_in = game.player_tiles();
        let filename = output_path("test-json_round_trip_1867.json");
        let pretty = false;

        let write_res = super::write_tiles(&filename, &cat_in, pretty);
        assert!(write_res.is_ok(), "Could not write {}", filename.display());
        let read_res = super::read_tiles(&filename, &hex);
        assert!(read_res.is_ok(), "Could not read {}", filename.display());
        let cat_out = read_res.unwrap();
        assert_eq!(cat_in, cat_out);
    }

    #[test]
    fn compare_to_catalogue_de() {
        let hex = init_hex();
        let catalogue = n18catalogue::tile_catalogue(&hex);
        let de_tiles = super::test_tiles().tiles;

        for (cat_tile, de_descr) in catalogue.iter().zip(&de_tiles) {
            let de_tile: n18tile::Tile = de_descr.build(&hex);
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
        let catalogue = n18catalogue::tile_catalogue(&hex);
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
