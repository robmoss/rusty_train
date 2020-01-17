use cairo::{Context, LineCap, LineJoin, TextExtents};

use crate::coord::Coord;
use crate::prelude::PI;
use crate::prelude::{PI_1_4, PI_3_4};
use crate::prelude::{PI_1_6, PI_2_6, PI_3_6, PI_4_6, PI_5_6};

/// The tile background colours.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HexColour {
    Yellow,
    Green,
    Brown,
    Grey,
    Red,
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
            // #BD5E64
            HexColour::Red => ctx.set_source_rgb(0.741, 0.369, 0.392),
        }
    }

    pub fn next_phase(self: &Self) -> Option<Self> {
        match self {
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
            HexColour::Red => HexColour::Yellow,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
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

impl Direction {
    pub fn radians(&self) -> f64 {
        use Direction::*;

        match self {
            N => -PI_3_6,   // - PI / 2
            NNE => -PI_2_6, // - 2 PI / 6
            NE => -PI_1_4,  // - PI / 4
            NEE => -PI_1_6, // - PI / 6
            E => 0.0,       //   0 radians
            SEE => PI_1_6,  //   PI / 6
            SE => PI_1_4,   //   PI / 4
            SSE => PI_2_6,  //   2 PI / 6
            S => PI_3_6,    //   PI / 2
            SSW => PI_4_6,  //   4 PI / 6
            SW => PI_3_4,   //   3 PI / 4
            SWW => PI_5_6,  //   5 PI / 6
            W => PI,        //   PI
            NWW => -PI_5_6, // - 5 PI / 6
            NW => -PI_3_4,  // - 3 PI / 4
            NNW => -PI_4_6, // - 4 PI / 6
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Delta {
    ToCentre(f64),
    Nudge(Direction, f64),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HexPosition {
    Centre(Option<Delta>),
    Face(HexFace, Option<Delta>),
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

    pub fn draw_tile_name(self: &Self, name: &str, ctx: &Context) {
        let locn = &self.corners[2];
        let exts = ctx.text_extents(name);
        ctx.move_to(locn.x - exts.width - 1.5, locn.y - 3.0);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(name);
        ctx.new_path();
    }

    pub fn circ_text<P>(self: &Self, text: &str, pos: P, ctx: &Context)
    where
        P: Into<HexPosition>,
    {
        let pos: HexPosition = pos.into();
        let exts = ctx.text_extents(text);
        let mut coord = pos.coord(self);
        // TODO: margins and scaling ...
        // Need to increase the nudge because of the circle
        // ... multiple exts by scale
        // ... hmm ... maybe not ...?
        let nudge = self.text_nudge(&pos, &exts);
        let ratio = exts.width / exts.height;
        let mut circ_nudge = nudge.clone();

        let scale = 1.5;
        let radius = scale * (0.5 * exts.width).max(0.5 * exts.height);

        let matrix = ctx.get_matrix();
        if ratio >= 2.0 {
            ctx.scale(1.0, 1.5 / ratio);
            coord.y *= ratio / 1.5;
            circ_nudge.y *= 1.0 + 0.8 * (ratio / 1.5 - 1.0);
        }

        ctx.new_path();
        ctx.arc(
            coord.x + circ_nudge.x + 0.7 * radius,
            coord.y + circ_nudge.y - 0.5 * radius,
            radius,
            0.0,
            2.0 * PI,
        );
        if ratio >= 2.0 {
            ctx.set_matrix(matrix);
            coord.y *= 1.5 / ratio;
        }

        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.fill_preserve();
        ctx.set_line_width(self.max_d * 0.01);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.stroke_preserve();

        ctx.move_to(coord.x + nudge.x, coord.y + nudge.y);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(text);
        ctx.new_path();
    }

    fn text_nudge(
        self: &Self,
        pos: &HexPosition,
        exts: &TextExtents,
    ) -> Coord {
        use HexCorner::*;
        use HexFace::*;

        if pos.is_centre() {
            Coord::from((-0.5 * exts.width, -0.5 * exts.height))
        } else if let Some(corner) = pos.get_corner() {
            // TODO: improve this spacing?
            match corner {
                TopLeft => (0.1 * exts.width, 1.4 * exts.height).into(),
                TopRight => (-1.2 * exts.width, 1.4 * exts.height).into(),
                Left => (0.5 * exts.width, 0.5 * exts.height).into(),
                Right => (-1.5 * exts.width, 0.5 * exts.height).into(),
                BottomLeft => (0.1 * exts.width, -0.4 * exts.height).into(),
                BottomRight => (-1.2 * exts.width, -0.4 * exts.height).into(),
            }
        } else if let Some(face) = pos.get_face() {
            // TODO: improve this spacing?
            match face {
                Top => (-0.5 * exts.width, 0.5 * exts.height).into(),
                UpperRight => (-0.5 * exts.width, 0.5 * exts.height).into(),
                LowerRight => (-0.5 * exts.width, 0.5 * exts.height).into(),
                Bottom => (-0.5 * exts.width, 0.5 * exts.height).into(),
                LowerLeft => (-0.5 * exts.width, 0.5 * exts.height).into(),
                UpperLeft => (-0.5 * exts.width, 0.5 * exts.height).into(),
            }
        } else {
            // NOTE: it should not be possible to reach this branch/
            unreachable!()
        }
    }

    pub fn text<P>(self: &Self, text: &str, pos: P, ctx: &Context)
    where
        P: Into<HexPosition>,
    {
        let pos: HexPosition = pos.into();
        let exts = ctx.text_extents(text);
        let coord = pos.coord(self);
        let nudge = self.text_nudge(&pos, &exts);
        ctx.move_to(coord.x + nudge.x, coord.y + nudge.y);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(text);
        ctx.new_path();
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
