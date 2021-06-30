use cairo::{Context, LineCap, LineJoin};

use crate::consts::PI;
use crate::consts::{PI_1_4, PI_3_4};
use crate::consts::{PI_1_6, PI_2_6, PI_3_6, PI_4_6, PI_5_6};
use crate::coord::Coord;

/// The tile background colours.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HexColour {
    Yellow,
    Green,
    Brown,
    Grey,
    Red,
    Blue,
    Empty,
}

impl HexColour {
    pub fn set_source_rgb(self: &Self, ctx: &Context) {
        match self {
            // #F3F013
            // HexColour::Yellow => ctx.set_source_rgb(0.953, 0.941, 0.075),
            // NOTE: Horrendous dark yellow to check track outlines.
            HexColour::Yellow => ctx.set_source_rgb(0.86, 0.75, 0.07),
            // #33B764
            HexColour::Green => ctx.set_source_rgb(0.20, 0.718, 0.392),
            // #AC6B3E
            HexColour::Brown => ctx.set_source_rgb(0.675, 0.420, 0.243),
            // #BCBCBC
            HexColour::Grey => ctx.set_source_rgb(0.741, 0.737, 0.737),
            // #BD5E64 -- Too similar to brown
            // HexColour::Red => ctx.set_source_rgb(0.741, 0.369, 0.392),
            HexColour::Red => ctx.set_source_rgb(0.86, 0.243, 0.243),
            HexColour::Blue => ctx.set_source_rgb(0.0, 0.5, 0.96),
            HexColour::Empty => ctx.set_source_rgb(0.741, 0.86, 0.741),
        }
    }

    pub fn next_phase(self: &Self) -> Option<Self> {
        match self {
            HexColour::Empty => Some(HexColour::Yellow),
            HexColour::Yellow => Some(HexColour::Green),
            HexColour::Green => Some(HexColour::Brown),
            HexColour::Brown => Some(HexColour::Grey),
            _ => None,
        }
    }

    pub fn next_colour(self: &Self) -> Self {
        match self {
            HexColour::Yellow => HexColour::Green,
            HexColour::Green => HexColour::Brown,
            HexColour::Brown => HexColour::Grey,
            HexColour::Grey => HexColour::Red,
            HexColour::Red => HexColour::Blue,
            HexColour::Blue => HexColour::Empty,
            HexColour::Empty => HexColour::Yellow,
        }
    }
}

/// The hexagon faces (edges).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HexFace {
    Top,
    UpperRight,
    LowerRight,
    Bottom,
    LowerLeft,
    UpperLeft,
}

impl HexFace {
    pub fn to_centre(self, frac: f64) -> HexPosition {
        let pos: HexPosition = self.into();
        pos.to_centre(frac)
    }

    pub fn nudge(self, dir: Direction, frac: f64) -> HexPosition {
        let pos: HexPosition = self.into();
        pos.nudge(dir, frac)
    }

    pub fn clockwise(&self) -> Self {
        match *self {
            HexFace::Top => HexFace::UpperRight,
            HexFace::UpperRight => HexFace::LowerRight,
            HexFace::LowerRight => HexFace::Bottom,
            HexFace::Bottom => HexFace::LowerLeft,
            HexFace::LowerLeft => HexFace::UpperLeft,
            HexFace::UpperLeft => HexFace::Top,
        }
    }

    pub fn anti_clockwise(&self) -> Self {
        match *self {
            HexFace::Top => HexFace::UpperLeft,
            HexFace::UpperRight => HexFace::Top,
            HexFace::LowerRight => HexFace::UpperRight,
            HexFace::Bottom => HexFace::LowerRight,
            HexFace::LowerLeft => HexFace::Bottom,
            HexFace::UpperLeft => HexFace::LowerLeft,
        }
    }

    pub fn opposite(self: &Self) -> Self {
        match *self {
            HexFace::Top => HexFace::Bottom,
            HexFace::UpperRight => HexFace::LowerLeft,
            HexFace::LowerRight => HexFace::UpperLeft,
            HexFace::Bottom => HexFace::Top,
            HexFace::LowerLeft => HexFace::UpperRight,
            HexFace::UpperLeft => HexFace::LowerRight,
        }
    }

    pub fn is_adjacent(self: &Self, other: &Self) -> bool {
        match *self {
            HexFace::Top => {
                other == &HexFace::UpperLeft || other == &HexFace::UpperRight
            }
            HexFace::UpperRight => {
                other == &HexFace::Top || other == &HexFace::LowerRight
            }
            HexFace::LowerRight => {
                other == &HexFace::UpperRight || other == &HexFace::Bottom
            }
            HexFace::Bottom => {
                other == &HexFace::LowerRight || other == &HexFace::LowerLeft
            }
            HexFace::LowerLeft => {
                other == &HexFace::Bottom || other == &HexFace::UpperLeft
            }
            HexFace::UpperLeft => {
                other == &HexFace::LowerLeft || other == &HexFace::Top
            }
        }
    }

    /// Return the two corners that are connected by this hexagon face.
    pub fn corners(&self) -> (HexCorner, HexCorner) {
        match *self {
            HexFace::Top => (HexCorner::TopLeft, HexCorner::TopRight),
            HexFace::UpperRight => (HexCorner::TopRight, HexCorner::Right),
            HexFace::LowerRight => (HexCorner::Right, HexCorner::BottomRight),
            HexFace::Bottom => {
                (HexCorner::BottomRight, HexCorner::BottomLeft)
            }
            HexFace::LowerLeft => (HexCorner::BottomLeft, HexCorner::Left),
            HexFace::UpperLeft => (HexCorner::Left, HexCorner::TopLeft),
        }
    }
}

/// The hexagon corners.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HexCorner {
    TopLeft,
    TopRight,
    Left,
    Right,
    BottomLeft,
    BottomRight,
}

impl HexCorner {
    pub fn to_centre(self, frac: f64) -> HexPosition {
        let pos: HexPosition = self.into();
        pos.to_centre(frac)
    }

    pub fn nudge(self, dir: Direction, frac: f64) -> HexPosition {
        let pos: HexPosition = self.into();
        pos.nudge(dir, frac)
    }

    pub fn next(&self) -> Self {
        use HexCorner::*;

        match *self {
            TopLeft => TopRight,
            TopRight => Right,
            Right => BottomRight,
            BottomRight => BottomLeft,
            BottomLeft => Left,
            Left => TopLeft,
        }
    }

    pub fn prev(&self) -> Self {
        use HexCorner::*;

        match *self {
            TopLeft => Left,
            TopRight => TopLeft,
            Right => TopRight,
            BottomRight => Right,
            BottomLeft => BottomRight,
            Left => BottomLeft,
        }
    }
}

/// The different **absolute** directions in which a [HexPosition] can be
/// "nudged".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Up; from the hexagon centre, towards the top face.
    N,
    /// From the hexagon centre, towards the top-right corner.
    N30E,
    /// From the hexagon centre, towards the upper-right face.
    N60E,
    /// Right; from the hexagon centre, towards the right corner.
    E,
    /// From the hexagon centre, towards the lower-right face.
    S60E,
    /// From the hexagon centre, towards the bottom-right corner.
    S30E,
    /// Down; from the hexagon centre, towards the bottom face.
    S,
    /// From the hexagon centre, towards the bottom-left corner.
    S30W,
    /// From the hexagon centre, towards the lower-left face.
    S60W,
    /// Left; from the hexagon centre, towards the left corner.
    W,
    /// From the hexagon centre, towards the upper-left face.
    N60W,
    /// From the hexagon centre, towards the top-left corner.
    N30W,
    /// Up and right (45° clockwise of north).
    NE,
    /// Down and right (135° clockwise of north).
    SE,
    /// Down and left (225° clockwise of north).
    SW,
    /// Up and left (315° clockwise of north).
    NW,
}

impl Direction {
    pub fn radians(&self) -> f64 {
        use Direction::*;

        match self {
            N => -PI_3_6,    // - PI / 2     Top face
            N30E => -PI_2_6, // - 2 PI / 6   Top-right corner
            NE => -PI_1_4,   // - PI / 4     North-east
            N60E => -PI_1_6, // - PI / 6     Upper-right face
            E => 0.0,        //   0 radians  Right corner
            S60E => PI_1_6,  //   PI / 6     Lower-right face
            SE => PI_1_4,    //   PI / 4     South-east
            S30E => PI_2_6,  //   2 PI / 6   Bottom-right corner
            S => PI_3_6,     //   PI / 2     Bottom face
            S30W => PI_4_6,  //   4 PI / 6   Bottom-left corner
            SW => PI_3_4,    //   3 PI / 4   South-west
            S60W => PI_5_6,  //   5 PI / 6   Lower-left face
            W => PI,         //   PI         Left corner
            N60W => -PI_5_6, // - 5 PI / 6   Upper-left face
            NW => -PI_3_4,   // - 3 PI / 4   North-west
            N30W => -PI_4_6, // - 4 PI / 6   Top-left corner
        }
    }
}

/// The direction and distance in which to "nudge" a [HexPosition], relative
/// to a reference point.
///
/// Distances are represented as multiples of the hexagon's maximum radius
/// (which is one half of the hexagon's maximum diameter, `hex.max_d`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Delta {
    /// Nudge towards the centre of the hexagon.
    ToCentre(f64),
    /// Nudge in an absolute direction.
    Nudge(Direction, f64),
}

/// Define specific positions within a hexagon, based on a reference point
/// and an optional "nudge" in some direction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HexPosition {
    /// Define positions relative to the centre of the hexagon.
    Centre(Option<Delta>),
    /// Define positions relative to a specific hexagon face.
    Face(HexFace, Option<Delta>),
    /// Define positions relative to a specific hexagon corner.
    Corner(HexCorner, Option<Delta>),
}

impl std::convert::From<HexFace> for HexPosition {
    fn from(face: HexFace) -> Self {
        Self::Face(face, None)
    }
}

impl std::convert::From<HexCorner> for HexPosition {
    fn from(corner: HexCorner) -> Self {
        Self::Corner(corner, None)
    }
}

impl std::default::Default for HexPosition {
    fn default() -> Self {
        Self::Centre(None)
    }
}

impl HexPosition {
    pub fn nudge(self, dir: Direction, frac: f64) -> Self {
        use HexPosition::*;

        let delta = Some(Delta::Nudge(dir, frac));
        match self {
            Centre(_) => Centre(delta),
            Face(face, _) => Face(face, delta),
            Corner(corner, _) => Corner(corner, delta),
        }
    }

    pub fn to_centre(self, frac: f64) -> Self {
        use HexPosition::*;

        match self {
            Centre(_) => self,
            Face(face, _) => Face(face, Some(Delta::ToCentre(frac))),
            Corner(corner, _) => Corner(corner, Some(Delta::ToCentre(frac))),
        }
    }

    pub fn coord(&self, hex: &Hex) -> Coord {
        use HexPosition::*;

        let radius = 0.5 * hex.max_d;

        match self {
            Centre(delta) => {
                if let Some(Delta::Nudge(angle, frac)) = delta {
                    let angle = angle.radians();
                    Coord {
                        x: frac * radius * angle.cos(),
                        y: frac * radius * angle.sin(),
                    }
                } else {
                    (0.0, 0.0).into()
                }
            }
            Face(face, delta) => {
                let coord = &hex.midpoint(&face);
                let shift = match delta {
                    Some(Delta::Nudge(angle, frac)) => {
                        let angle = angle.radians();
                        Coord {
                            x: frac * radius * angle.cos(),
                            y: frac * radius * angle.sin(),
                        }
                    }
                    Some(Delta::ToCentre(frac)) => coord * -frac,
                    None => (0.0, 0.0).into(),
                };
                coord + &shift
            }
            Corner(corner, delta) => {
                let coord = hex.corner_coord(&corner);
                let shift = match delta {
                    Some(Delta::Nudge(angle, frac)) => {
                        let angle = angle.radians();
                        Coord {
                            x: frac * radius * angle.cos(),
                            y: frac * radius * angle.sin(),
                        }
                    }
                    Some(Delta::ToCentre(frac)) => coord * -frac,
                    None => (0.0, 0.0).into(),
                };
                coord + &shift
            }
        }
    }

    pub fn get_corner(&self) -> Option<HexCorner> {
        match self {
            Self::Corner(corner, _nudge) => Some(*corner),
            _ => None,
        }
    }

    pub fn get_face(&self) -> Option<HexFace> {
        match self {
            Self::Face(face, _nudge) => Some(*face),
            _ => None,
        }
    }

    pub fn is_centre(&self) -> bool {
        match self {
            Self::Centre(_nudge) => true,
            _ => false,
        }
    }

    pub fn is_corner(&self) -> bool {
        self.get_corner().is_some()
    }

    pub fn is_face(&self) -> bool {
        self.get_face().is_some()
    }
}

/// The geometry of hexagonal tiles.
pub struct Hex {
    pub max_d: f64,
    pub min_d: f64,
    // alpha: f64,
    // beta: f64,
    corners: Vec<Coord>,
    #[allow(dead_code)]
    surface: cairo::ImageSurface,
    context: cairo::Context,
}

impl From<f64> for Hex {
    fn from(max_d: f64) -> Self {
        Self::new(max_d)
    }
}

impl Hex {
    pub fn new(max_d: f64) -> Self {
        let min_d = (3.0 as f64).sqrt() * max_d / 2.0;
        let alpha = max_d / 4.0;
        let beta = alpha * (3.0 as f64).sqrt();
        let corners = vec![
            (-2.0 * alpha, 0.0), // Middle left
            (-alpha, beta),      // Upper left
            (alpha, beta),       // Upper right
            (2.0 * alpha, 0.0),  // Middle right
            (alpha, -beta),      // Lower right
            (-alpha, -beta),     // Lower left
        ];

        let corner_coords = corners.iter().map(|c| c.into()).collect();

        let dim = (max_d * 2.0) as i32;
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, dim, dim)
                .expect("Can't create cairo::ImageSurface");
        let context = cairo::Context::new(&surface);
        // Move the origin to the centre of this surface.
        context.translate(max_d, max_d);

        Self {
            max_d: max_d,
            min_d: min_d,
            // alpha: alpha,
            // beta: beta,
            corners: corner_coords,
            surface,
            context,
        }
    }

    pub fn context(&self) -> &cairo::Context {
        &self.context
    }

    pub fn corner_coord(self: &Self, corner: &HexCorner) -> &Coord {
        use HexCorner::*;

        match corner {
            TopLeft => &self.corners[5],
            TopRight => &self.corners[4],
            Right => &self.corners[3],
            BottomRight => &self.corners[2],
            BottomLeft => &self.corners[1],
            Left => &self.corners[0],
        }
    }

    pub fn define_boundary(&self, ctx: &Context) {
        ctx.set_line_cap(LineCap::Butt);
        ctx.set_line_join(LineJoin::Round);

        ctx.new_path();
        for coord in &self.corners {
            ctx.line_to(coord.x, coord.y);
        }

        ctx.close_path();
    }

    pub fn draw_background(self: &Self, colour: HexColour, ctx: &Context) {
        self.define_boundary(ctx);
        colour.set_source_rgb(ctx);
        ctx.set_line_width(self.max_d * 0.01);
        ctx.fill_preserve();
        ctx.set_source_rgb(0.7, 0.7, 0.7);
        ctx.stroke();
    }

    pub fn midpoint(self: &Self, face: &HexFace) -> Coord {
        match face {
            HexFace::UpperLeft => self.corners[5].average(&self.corners[0]),
            HexFace::Top => self.corners[4].average(&self.corners[5]),
            HexFace::UpperRight => self.corners[3].average(&self.corners[4]),
            HexFace::LowerRight => self.corners[2].average(&self.corners[3]),
            HexFace::Bottom => self.corners[1].average(&self.corners[2]),
            HexFace::LowerLeft => self.corners[0].average(&self.corners[1]),
        }
    }
}
