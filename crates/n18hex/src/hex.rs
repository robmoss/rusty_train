use cairo::Context;

use crate::consts::PI;
use crate::consts::{PI_1_4, PI_3_4};
use crate::consts::{PI_1_6, PI_2_6, PI_3_6, PI_4_6, PI_5_6};
use crate::coord::Coord;
use crate::theme::Theme;

/// The tile background colours for [Hex].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    /// Returns the colour associated with the next phase of tiles, if any.
    pub fn next_phase(&self) -> Option<Self> {
        match self {
            HexColour::Empty => Some(HexColour::Yellow),
            HexColour::Yellow => Some(HexColour::Green),
            HexColour::Green => Some(HexColour::Brown),
            HexColour::Brown => Some(HexColour::Grey),
            _ => None,
        }
    }

    /// Returns the next colour, in the order that the enum variants are
    ///defined, and cycling back to the start.
    pub fn next_colour(&self) -> Self {
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
    /// Returns a [HexPosition] that corresponds to the middle of this hexagon
    /// face, with an optional translation towards the hexagon centre.
    /// The value of `frac` should be between `0` (the hexagon face) and `1`
    /// (the hexagon centre), although values outside of this range are
    /// accepted.
    ///
    /// See [HexPosition] and [Delta] for further details.
    pub fn to_centre(self, frac: f64) -> HexPosition {
        HexPosition::from(self).to_centre(frac)
    }

    /// Returns a [HexPosition] that corresponds to the middle of this hexagon
    /// face, with an optional translation `frac` in some direction `dir`.
    /// The unit of `frac` is the maximal radius (the length between the
    /// hexagon corners and the hexagon centre).
    ///
    /// See [HexPosition] and [Delta] for further details.
    pub fn in_dir(self, dir: Direction, frac: f64) -> HexPosition {
        HexPosition::from(self).in_dir(dir, frac)
    }

    /// Returns the next hexagon face in the clockwise direction.
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

    /// Returns the next hexagon face in the anti-clockwise direction.
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

    /// Returns the opposite hexagon face.
    pub fn opposite(&self) -> Self {
        match *self {
            HexFace::Top => HexFace::Bottom,
            HexFace::UpperRight => HexFace::LowerLeft,
            HexFace::LowerRight => HexFace::UpperLeft,
            HexFace::Bottom => HexFace::Top,
            HexFace::LowerLeft => HexFace::UpperRight,
            HexFace::UpperLeft => HexFace::LowerRight,
        }
    }

    /// Returns whether two faces are adjacent (i.e., share a corner).
    pub fn is_adjacent(&self, other: &Self) -> bool {
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

    /// Returns the two corners that are connected by this hexagon face.
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
    /// Returns a [HexPosition] that corresponds to this hexagon corner, with
    /// an optional translation towards the hexagon centre.
    /// The value of `frac` should be between `0` (the hexagon corner) and `1`
    /// (the hexagon centre), although values outside of this range are
    /// accepted.
    ///
    /// See [HexPosition] and [Delta] for further details.
    pub fn to_centre(self, frac: f64) -> HexPosition {
        HexPosition::from(self).to_centre(frac)
    }

    /// Returns a [HexPosition] that corresponds to this hexagon corner, with
    /// an optional translation `frac` in some direction `dir`.
    /// The unit of `frac` is the maximal radius (the length between the
    /// hexagon corners and the hexagon centre).
    ///
    /// See [HexPosition] and [Delta] for further details.
    pub fn in_dir(self, dir: Direction, frac: f64) -> HexPosition {
        HexPosition::from(self).in_dir(dir, frac)
    }

    /// Returns the next hexagon corner in the clockwise direction.
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

    /// Returns the previous hexagon corner in the clockwise direction.
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
/// translated.
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
    /// Returns the value of this direction in radians.
    ///
    /// East is defined to have an angle of `0`, and angles increase in the
    /// clockwise direction.
    ///
    /// This function returns values between  `-π` (exclusive) to `π`
    /// (inclusive).
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

/// The direction and distance in which to translate a [HexPosition], relative
/// to a reference point.
///
/// Distances are represented as multiples of the hexagon's maximum radius
/// (which is one half of the hexagon's maximum diameter, `hex.max_d`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Delta {
    /// Translate towards the centre of the hexagon.
    ToCentre(f64),
    /// Translate in an absolute direction.
    InDir(Direction, f64),
}

/// Define specific positions within a hexagon, based on a reference point
/// and an optional translation in some direction.
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

/// The default position is the hexagon centre.
impl std::default::Default for HexPosition {
    fn default() -> Self {
        Self::Centre(None)
    }
}

impl HexPosition {
    /// Replaces the existing translation, if any, with a translation `frac`
    /// in some direction `dir`.
    /// The unit of `frac` is the maximal radius (the length between the
    /// hexagon corners and the hexagon centre).
    pub fn in_dir(self, dir: Direction, frac: f64) -> Self {
        use HexPosition::*;

        let delta = Some(Delta::InDir(dir, frac));
        match self {
            Centre(_) => Centre(delta),
            Face(face, _) => Face(face, delta),
            Corner(corner, _) => Corner(corner, delta),
        }
    }

    /// Replaces the existing translation, if any, with a translation `frac`
    /// towards the hexagon centre.
    /// The unit of `frac` is the distance between the reference point and the
    /// hexagon centre.
    /// Accordingly, a value of `0` corresponds to the reference point, and a
    /// value of `1` corresponds to the hexagon centre.
    pub fn to_centre(self, frac: f64) -> Self {
        use HexPosition::*;

        match self {
            Centre(_) => self,
            Face(face, _) => Face(face, Some(Delta::ToCentre(frac))),
            Corner(corner, _) => Corner(corner, Some(Delta::ToCentre(frac))),
        }
    }

    /// Returns the Cartesian coordinates that correspond to this position,
    /// according to the geometry of the provided hexagon `hex`.
    pub fn coord(&self, hex: &Hex) -> Coord {
        use HexPosition::*;

        let radius = 0.5 * hex.max_d;

        match self {
            Centre(delta) => {
                if let Some(Delta::InDir(angle, frac)) = delta {
                    let angle = angle.radians();
                    Coord {
                        x: frac * radius * angle.cos(),
                        y: frac * radius * angle.sin(),
                    }
                } else {
                    // NOTE: can ignore Delta::ToCentre, the result will
                    // always be the same.
                    (0.0, 0.0).into()
                }
            }
            Face(face, delta) => {
                let coord = &hex.midpoint(&face);
                let shift = match delta {
                    Some(Delta::InDir(angle, frac)) => {
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
                    Some(Delta::InDir(angle, frac)) => {
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

    /// Returns the hexagon corner associated with this position, if the
    /// position is defined relative to a corner.
    pub fn corner(&self) -> Option<HexCorner> {
        match self {
            Self::Corner(corner, _delta) => Some(*corner),
            _ => None,
        }
    }

    /// Returns the hexagon face associated with this position, if the
    /// position is defined relative to a face.
    pub fn face(&self) -> Option<HexFace> {
        match self {
            Self::Face(face, _delta) => Some(*face),
            _ => None,
        }
    }

    /// Returns whether this position is defined relative to the hexagon
    /// centre.
    pub fn is_centre(&self) -> bool {
        matches!(self, Self::Centre(_delta))
    }

    /// Returns whether this position is defined relative to a hexagon corner.
    pub fn is_corner(&self) -> bool {
        self.corner().is_some()
    }

    /// Returns whether this position is defined relative to a hexagon face.
    pub fn is_face(&self) -> bool {
        self.face().is_some()
    }
}

/// The geometry of hexagonal tiles.
///
/// The origin is defined to be the centre of the hexagon.
pub struct Hex {
    /// The colours and drawing styles for this hexagon.
    pub theme: Theme,
    /// The maximal diameter (the length between opposite corners).
    pub max_d: f64,
    /// The minimal diameter (the length between opposite faces).
    pub min_d: f64,
    corners: Vec<Coord>,
    #[allow(dead_code)]
    surface: cairo::ImageSurface,
    context: cairo::Context,
}

/// Constructs a hexagon for the given maximal diameter.
impl From<f64> for Hex {
    fn from(max_d: f64) -> Self {
        Self::new(max_d)
    }
}

impl Hex {
    /// Constructs a hexagon for the given maximal diameter.
    pub fn new(max_d: f64) -> Self {
        let theme = Theme::default();
        Self::with_theme(max_d, theme)
    }

    /// Returns the coordinates of each hexagon corner, relative to the
    /// hexagon centre.
    fn corner_coords(alpha: f64, beta: f64) -> Vec<Coord> {
        vec![
            Coord::from((-2.0 * alpha, 0.0)), // Middle left
            Coord::from((-alpha, beta)),      // Upper left
            Coord::from((alpha, beta)),       // Upper right
            Coord::from((2.0 * alpha, 0.0)),  // Middle right
            Coord::from((alpha, -beta)),      // Lower right
            Coord::from((-alpha, -beta)),     // Lower left
        ]
    }

    /// Constructs a hexagon for the given maximal diameter and drawing theme.
    pub fn with_theme(max_d: f64, theme: Theme) -> Self {
        let min_d = (3.0_f64).sqrt() * max_d / 2.0;
        let alpha = max_d / 4.0;
        let beta = alpha * (3.0_f64).sqrt();
        let corners = Self::corner_coords(alpha, beta);

        let dim = (max_d * 2.0) as i32;
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, dim, dim)
                .expect("Can't create cairo::ImageSurface");
        let context = cairo::Context::new(&surface)
            .expect("Can't create cairo::Context");
        // Move the origin to the centre of this surface.
        context.translate(max_d, max_d);

        Self {
            theme,
            max_d,
            min_d,
            corners,
            surface,
            context,
        }
    }

    /// Resizes the hexagon to have the specified maximal diameter.
    pub fn resize(&mut self, max_d: f64) {
        let min_d = (3.0_f64).sqrt() * max_d / 2.0;
        let alpha = max_d / 4.0;
        let beta = alpha * (3.0_f64).sqrt();
        let corners = Self::corner_coords(alpha, beta);

        self.max_d = max_d;
        self.min_d = min_d;
        self.corners = corners;

        let dim = (max_d * 2.0) as i32;
        let resize_surface =
            self.surface.width() < dim || self.surface.height() < dim;
        if resize_surface {
            self.surface =
                cairo::ImageSurface::create(cairo::Format::ARgb32, dim, dim)
                    .expect("Can't create cairo::ImageSurface");
            self.context = cairo::Context::new(&self.surface)
                .expect("Can't create cairo::Context");
        }
        // Move the origin to the centre of this hexagon.
        self.context.translate(max_d, max_d);
    }

    /// Returns the ratio of the minimal diameter to the maximal diameter:
    /// `sqrt(3) / 2`.
    pub fn ratio_min_d() -> f64 {
        (3.0_f64).sqrt() / 2.0
    }

    /// Returns the ratio of the maximal diameter to the minimal diameter:
    /// `2 / sqrt(3)`.
    pub fn ratio_max_d() -> f64 {
        2.0 / (3.0_f64).sqrt()
    }

    /// Returns the context associated with a private surface with sufficient
    /// dimensions for drawing the hexagon.
    ///
    /// This context is intended for checking properties such as whether a
    /// specific coordinate is inside an area that would be affected by a
    /// stroke or fill operation.
    pub fn context(&self) -> &cairo::Context {
        &self.context
    }

    /// Returns the Cartesian coordinates for the given hexagon corner, where
    /// the origin is the hexagon centre.
    pub fn corner_coord(&self, corner: &HexCorner) -> &Coord {
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

    /// Defines the hexagon boundary as a path on the provided context.
    pub fn define_boundary(&self, ctx: &Context) {
        self.theme.hex_border.apply_line(ctx, self);
        ctx.new_path();
        for coord in &self.corners {
            ctx.line_to(coord.x, coord.y);
        }

        ctx.close_path();
    }

    /// Fills the hexagon with a specific colour on the provided context.
    pub fn draw_background(&self, colour: HexColour, ctx: &Context) {
        self.define_boundary(ctx);
        self.theme.apply_hex_colour(ctx, colour);
        ctx.fill_preserve().unwrap();
        self.theme.hex_border.apply_line_and_stroke(ctx, self);
        ctx.stroke().unwrap();
    }

    /// Returns the Cartesian coordinates for the middle of the given hexagon
    /// face, where the origin is the hexagon centre.
    pub fn midpoint(&self, face: &HexFace) -> Coord {
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    /// Tests that adding a translation to a HexPosition has the intended
    /// behaviour, and that scaling by Hex::ratio_min_d() behaves as expected.
    fn translate_to_boundary() {
        // The threshold for coordinates to be considered equal.
        let epsilon = 1e-10;
        let hex = Hex::new(125.0);

        // Translate from the centre to the top-left corner, and check that
        // the result is consistent with the top-left corner.
        let expect = HexPosition::from(HexCorner::TopLeft).coord(&hex);
        let result = HexPosition::Centre(None)
            .in_dir(Direction::N30W, 1.0)
            .coord(&hex);
        let diff = (&expect - &result).magnitude();
        assert!(diff < epsilon);

        // Translate from the centre to the top face, and check that the
        // result is consistent with the top face.
        // NOTE: faces are closer to the centre than are corners, so we must
        // use a smaller translation; Hex::ratio_min_d() provides the
        // appropriate scaling factor.
        let expect = HexPosition::from(HexFace::Top).coord(&hex);
        let result = HexPosition::Centre(None)
            .in_dir(Direction::N, Hex::ratio_min_d())
            .coord(&hex);
        let diff = (&expect - &result).magnitude();
        assert!(diff < epsilon);
    }
}
