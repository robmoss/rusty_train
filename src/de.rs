/// Load tile catalogues from disk.
use crate::catalogue::Catalogue;
use crate::hex::Hex;
use crate::prelude::PI;

use cairo::Context;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Tiles {
    tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HexColour {
    Yellow,
    Green,
    Brown,
    Grey,
    Red,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HexFace {
    Top,
    UpperRight,
    LowerRight,
    Bottom,
    LowerLeft,
    UpperLeft,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    pub name: String,
    pub colour: HexColour,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub track: Vec<Track>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cities: Vec<City>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<Label>,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum TrackType {
    Mid(HexFace),
    Straight(HexFace),
    GentleL(HexFace),
    GentleR(HexFace),
    HardL(HexFace),
    HardR(HexFace),
    Frac((HexFace, f64)),
    GentleLHalf(HexFace),
    GentleRHalf(HexFace),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    #[serde(flatten)]
    pub track_type: TrackType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dit: Option<(f64, usize)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<(f64, f64)>,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            track_type: TrackType::Straight(HexFace::Bottom),
            dit: None,
            clip: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Location {
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

#[derive(Serialize, Deserialize, Debug)]
pub enum CornerLocation {
    Centre,
    TopLeftCorner,
    TopRightCorner,
    LeftCorner,
    RightCorner,
    BottomLeftCorner,
    BottomRightCorner,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CentreLocation {
    Centre,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CityType {
    Dit(CentreLocation),
    Single(Location),
    Double(CornerLocation),
    Triple(CentreLocation),
    Quad(CentreLocation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct City {
    #[serde(flatten)]
    pub city_type: CityType,
    pub revenue: usize,
    /// An optional nudge `(angle, frac)` where `frac` is relative to the
    /// maximal radius of the tile (i.e., from the centre to any corner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<f64>,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum LabelType {
    City(String),
    Y(()),
    TileName(()),
    Revenue(usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Label {
    #[serde(flatten)]
    pub label_type: LabelType,
    pub location: Location,
    /// An optional nudge `(angle, frac)` where `frac` is relative to the
    /// maximal radius of the tile (i.e., from the centre to any corner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_centre: Option<f64>,
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

pub fn read<P: AsRef<Path>>(path: P) -> Result<Tiles, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tiles = serde_json::from_reader(reader)?;
    Ok(tiles)
}

pub fn write<P: AsRef<Path>>(
    path: P,
    tiles: &Tiles,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    serde_json::to_writer(file, tiles)?;
    Ok(())
}

pub fn write_pretty<P: AsRef<Path>>(
    path: P,
    tiles: &Tiles,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, tiles)?;
    Ok(())
}

/// Load a tile catalogue from disk.
pub fn load<P: AsRef<Path>>(
    path: P,
    hex: &Hex,
    ctx: &Context,
) -> Result<Catalogue, Box<dyn Error>> {
    let tiles = read(path)?;
    Ok(tiles.catalogue(hex, ctx))
}

// NOTE: need hex and ctx to construct tiles!

impl Tiles {
    pub fn catalogue(&self, hex: &Hex, ctx: &Context) -> Catalogue {
        self.tiles.iter().map(|t| t.build(hex, ctx)).collect()
    }
}

impl Tile {
    pub fn build(&self, hex: &Hex, ctx: &Context) -> crate::tile::Tile {
        let tile = crate::tile::Tile::new(
            (&self.colour).into(),
            self.name.clone(),
            self.track.iter().map(|t| t.into()).collect(),
            self.cities.iter().map(|c| c.build()).collect(),
            ctx,
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
        let position = if let Some((angle, frac)) = self.nudge {
            // NOTE: retain fractional unit of distance.
            position.nudge(angle, frac)
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
            TrackType::Frac((ref face, frac)) => {
                crate::track::Track::straight(face.into())
                    .with_span(0.0, frac)
            }
            TrackType::GentleLHalf(ref face) => {
                crate::track::Track::gentle_l(face.into()).with_span(0.0, 0.5)
            }
            TrackType::GentleRHalf(ref face) => {
                crate::track::Track::gentle_r(face.into()).with_span(0.0, 0.5)
            }
        };
        let track = if let Some((posn, revenue)) = t.dit {
            track.with_dit(posn, revenue)
        } else {
            track
        };
        let track = if let Some((lower, upper)) = t.clip {
            track.with_clip(lower, upper)
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
        let city = if let Some((angle, radius)) = self.nudge {
            city.nudge(angle, radius)
        } else {
            city
        };
        let city = if let Some(angle) = self.rotate {
            city.rotate(angle)
        } else {
            city
        };
        city
    }
}

/// Should yield the same tiles as `crate::catalogue::tile_catalogue()`.
pub fn test_tiles() -> Tiles {
    use HexColour::*;
    use TrackType::*;

    // TODO: define all of the tiles in crate::catalogue::tile_catalogue().

    Tiles {
        tiles: vec![
            Tile {
                name: "3".to_string(),
                colour: Yellow,
                track: vec![Track {
                    track_type: HardL(HexFace::Bottom),
                    dit: Some((0.5, 10)),
                    ..Default::default()
                }],
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
                track: vec![Track {
                    track_type: Straight(HexFace::Bottom),
                    dit: Some((0.25, 10)),
                    ..Default::default()
                }],
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
                track: vec![Track {
                    track_type: GentleR(HexFace::Bottom),
                    dit: Some((0.5, 10)),
                    ..Default::default()
                }],
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
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
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
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::Top),
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
                        nudge: Some((-PI / 2.0, 0.2)),
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
                track: vec![Track {
                    track_type: Straight(HexFace::Bottom),
                    ..Default::default()
                }],
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
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::UpperRight),
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
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::LowerRight),
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
                        nudge: Some((1.3 * PI / 2.0, 0.16)),
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
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleL(HexFace::UpperLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::Bottom),
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
                        nudge: Some((-3.0 * PI / 4.0, 0.12)),
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
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
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
                        nudge: Some((-3.0 * PI / 4.0, 0.1)),
                        ..Default::default()
                    },
                    Label {
                        label_type: LabelType::Revenue(0),
                        location: Location::TopLeftCorner,
                        nudge: Some((1.3 * PI / 2.0, 0.16)),
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
                        ..Default::default()
                    },
                    Track {
                        track_type: HardL(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: HardR(HexFace::LowerRight),
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
                        nudge: Some((-PI / 2.0, 0.2)),
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
                        clip: Some((0.3625, 0.75)),
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
                        nudge: Some((PI / 2.0, 0.1)),
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
                        rotate: Some(PI / 2.0),
                        nudge: Some((0.0, 0.1)),
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
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleRHalf(HexFace::LowerLeft),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleLHalf(HexFace::LowerRight),
                        ..Default::default()
                    },
                    Track {
                        track_type: Frac((HexFace::Top, 0.6)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Frac((HexFace::Bottom, 0.4)),
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
                        nudge: Some((PI / 2.0, 0.3)),
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
                    rotate: Some(PI),
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
                        dit: Some((0.85, 30)),
                        ..Default::default()
                    },
                    Track {
                        track_type: GentleR(HexFace::Bottom),
                        dit: Some((0.85, 30)),
                        ..Default::default()
                    },
                    Track {
                        track_type: Straight(HexFace::UpperLeft),
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
                cities: vec![
                    City {
                        city_type: CityType::Single(Location::LowerLeftFace),
                        revenue: 30,
                        nudge: Some((-PI / 6.0, 0.2)),
                        ..Default::default()
                    },
                    City {
                        city_type: CityType::Single(Location::UpperRightFace),
                        revenue: 30,
                        nudge: Some((5.0 * PI / 6.0, 0.2)),
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
