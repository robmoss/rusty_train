use crate::catalogue::Catalogue;
/// Load tile catalogues from disk.
use crate::hex::Hex;

use cairo::Context;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum TrackType {
    Mid(HexFace),
    Straight(HexFace),
    GentleL(HexFace),
    GentleR(HexFace),
    HardL(HexFace),
    HardR(HexFace),
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
pub enum CityType {
    CentralDit,
    Single(Location),
    Double(CornerLocation),
    Triple,
    Quad,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct City {
    #[serde(flatten)]
    pub city_type: CityType,
    pub revenue: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LabelType {
    City(String),
    Y,
    TileName,
    Revenue(usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Label {
    #[serde(flatten)]
    pub label_type: LabelType,
    pub location: Location,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_centre: Option<f64>,
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
            self.cities.iter().map(|c| c.build(hex)).collect(),
            ctx,
            hex,
        );
        let tile = self.labels.iter().fold(tile, |tile, label| {
            let posn = label.position(hex);
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
            LabelType::Y => crate::label::Label::Y,
            LabelType::TileName => crate::label::Label::TileName,
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
    pub fn position(&self, hex: &Hex) -> crate::hex::HexPosition {
        let position: crate::hex::HexPosition = (&self.location).into();
        let position = if let Some((angle, distance)) = self.nudge {
            position.nudge(angle, distance)
        } else {
            position
        };
        let position = if let Some(frac) = self.to_centre {
            position.to_centre(frac, hex)
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
    pub fn build(&self, revenue: usize, hex: &Hex) -> crate::city::City {
        use CityType::*;

        match self {
            CentralDit => crate::city::City::central_dit(revenue),
            Single(location) => {
                use crate::city::City;
                use crate::hex::HexCorner::*;
                use crate::hex::HexFace::*;
                use Location::*;

                match location {
                    Centre => City::single(revenue),
                    TopLeftCorner => {
                        City::single_at_corner(revenue, hex, &TopLeft)
                    }
                    TopRightCorner => {
                        City::single_at_corner(revenue, hex, &TopRight)
                    }
                    LeftCorner => City::single_at_corner(revenue, hex, &Left),
                    RightCorner => {
                        City::single_at_corner(revenue, hex, &Right)
                    }
                    BottomLeftCorner => {
                        City::single_at_corner(revenue, hex, &BottomLeft)
                    }
                    BottomRightCorner => {
                        City::single_at_corner(revenue, hex, &BottomRight)
                    }
                    TopFace => City::single_at_face(revenue, hex, &Top),
                    UpperRightFace => {
                        City::single_at_face(revenue, hex, &UpperRight)
                    }
                    LowerRightFace => {
                        City::single_at_face(revenue, hex, &LowerRight)
                    }
                    BottomFace => City::single_at_face(revenue, hex, &Bottom),
                    LowerLeftFace => {
                        City::single_at_face(revenue, hex, &LowerLeft)
                    }
                    UpperLeftFace => {
                        City::single_at_face(revenue, hex, &UpperLeft)
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
                        City::double_at_corner(revenue, hex, &TopLeft)
                    }
                    TopRightCorner => {
                        City::double_at_corner(revenue, hex, &TopRight)
                    }
                    LeftCorner => City::double_at_corner(revenue, hex, &Left),
                    RightCorner => {
                        City::double_at_corner(revenue, hex, &Right)
                    }
                    BottomLeftCorner => {
                        City::double_at_corner(revenue, hex, &BottomLeft)
                    }
                    BottomRightCorner => {
                        City::double_at_corner(revenue, hex, &BottomRight)
                    }
                }
            }
            Triple => crate::city::City::triple(revenue),
            Quad => crate::city::City::quad(revenue),
        }
    }
}

impl City {
    pub fn build(&self, hex: &Hex) -> crate::city::City {
        let city = self.city_type.build(self.revenue, hex);
        let city = if let Some((angle, radius)) = self.nudge {
            city.nudge(hex, angle, radius)
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
    use LabelType::*;
    use TrackType::*;

    // TODO: define all of the tiles in crate::catalogue::tile_catalogue().

    Tiles {
        tiles: vec![Tile {
            name: "3".to_string(),
            colour: Yellow,
            track: vec![Track {
                track_type: HardL(HexFace::Bottom),
                dit: Some((0.5, 10)),
                clip: None,
            }],
            cities: vec![],
            labels: vec![Label {
                label_type: Revenue(0),
                location: Location::Centre,
                nudge: None,
                to_centre: None,
            }],
        }],
    }
}