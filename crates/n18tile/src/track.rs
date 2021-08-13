use crate::draw::Draw;
use cairo::Context;
use n18hex::theme::Length;
use n18hex::{Coord, Hex, HexCorner, HexFace, PI};

/// The shapes that track segments may take.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TrackCurve {
    Straight,
    GentleL,
    HardL,
    GentleR,
    HardR,
}

impl Default for TrackCurve {
    fn default() -> Self {
        Self::Straight
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TrackPath {
    Linear {
        start: Coord,
        end: Coord,
    },
    Curve {
        centre: Coord,
        radius: f64,
        angle_0: f64,
        angle_1: f64,
        clockwise: bool,
    },
}

/// Each track segment has two ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum TrackEnd {
    Start,
    End,
}

impl TrackEnd {
    /// Returns the other end of the track segment.
    pub fn other_end(&self) -> Self {
        use TrackEnd::*;
        match self {
            Start => End,
            End => Start,
        }
    }
}

/// A dit may have one of several shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DitShape {
    Bar,
    Circle,
}

/// Track segments along which trains can run routes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Track {
    pub face: HexFace,
    pub curve: TrackCurve,
    pub x0: f64,
    pub x1: f64,
    pub clip: Option<(f64, f64)>,
    pub dit: Option<(TrackEnd, usize, DitShape)>,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            face: HexFace::Bottom,
            curve: TrackCurve::Straight,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }
}

impl Track {
    pub fn new(
        face: HexFace,
        curve: TrackCurve,
        x0: f64,
        x1: f64,
        clip: Option<(f64, f64)>,
        dit: Option<(TrackEnd, usize, DitShape)>,
    ) -> Self {
        // TODO: check x0, x1, clip, dit location.
        Self {
            face,
            curve,
            x0,
            x1,
            clip,
            dit,
        }
    }

    /// A straight track segments that crosses the tile.
    pub fn straight(face: HexFace) -> Self {
        Self {
            face,
            curve: TrackCurve::Straight,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    /// A straight track segment that runs to the centre of the tile.
    pub fn mid(face: HexFace) -> Self {
        Self {
            face,
            curve: TrackCurve::Straight,
            x0: 0.0,
            x1: 0.5,
            clip: None,
            dit: None,
        }
    }

    pub fn gentle_l(face: HexFace) -> Self {
        Self {
            face,
            curve: TrackCurve::GentleL,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn gentle_r(face: HexFace) -> Self {
        Self {
            face,
            curve: TrackCurve::GentleR,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn hard_l(face: HexFace) -> Self {
        Self {
            face,
            curve: TrackCurve::HardL,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn hard_r(face: HexFace) -> Self {
        Self {
            face,
            curve: TrackCurve::HardR,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn with_dit(
        mut self,
        end: TrackEnd,
        revenue: usize,
        shape: DitShape,
    ) -> Self {
        // TODO: check location.
        self.dit = Some((end, revenue, shape));
        self
    }

    pub fn with_clip(mut self, c0: f64, c1: f64) -> Self {
        // TODO: check c0 and c1.
        self.clip = Some((c0, c1));
        self
    }

    pub fn with_span(mut self, x0: f64, x1: f64) -> Self {
        // TODO: check x0 and x1.
        self.x0 = x0;
        self.x1 = x1;
        self
    }

    /// Check whether a point along this path is clipped.
    fn clipped(self, dt: f64) -> bool {
        if let Some((x0, x1)) = self.clip {
            dt >= x0 && dt <= x1
        } else {
            false
        }
    }

    fn describe_path(&self, hex: &Hex) -> TrackPath {
        use HexFace::*;
        use TrackCurve::*;

        // TODO: need to define the direction for curves, so that partial
        // track segments and coord iteration work!
        match self.curve {
            Straight => {
                let start = hex.midpoint(&self.face);
                let end = hex.midpoint(&self.face.opposite());
                TrackPath::Linear { start, end }
            }
            HardL => {
                // TODO: let corner = self.face.left()?
                let corner = match self.face {
                    Bottom => HexCorner::BottomLeft,
                    LowerLeft => HexCorner::Left,
                    UpperLeft => HexCorner::TopLeft,
                    Top => HexCorner::TopRight,
                    UpperRight => HexCorner::Right,
                    LowerRight => HexCorner::BottomRight,
                };
                let (angle_0, angle_1) = match self.face {
                    Bottom => (4.0 * PI / 3.0, 6.0 * PI / 3.0),
                    LowerLeft => (5.0 * PI / 3.0, 7.0 * PI / 3.0),
                    UpperLeft => (0.0 * PI / 3.0, 2.0 * PI / 3.0),
                    Top => (1.0 * PI / 3.0, 3.0 * PI / 3.0),
                    UpperRight => (2.0 * PI / 3.0, 4.0 * PI / 3.0),
                    LowerRight => (3.0 * PI / 3.0, 5.0 * PI / 3.0),
                };
                let centre = *hex.corner_coord(&corner);
                let radius = hex.theme.track_hard_radius.absolute(hex);
                TrackPath::Curve {
                    centre,
                    radius,
                    angle_0,
                    angle_1,
                    clockwise: false,
                }
            }
            HardR => {
                // TODO: let corner = self.face.right()?
                let corner = match self.face {
                    Bottom => HexCorner::BottomRight,
                    LowerLeft => HexCorner::BottomLeft,
                    UpperLeft => HexCorner::Left,
                    Top => HexCorner::TopLeft,
                    UpperRight => HexCorner::TopRight,
                    LowerRight => HexCorner::Right,
                };
                let (angle_0, angle_1) = match self.face {
                    Bottom => (3.0 * PI / 3.0, 5.0 * PI / 3.0),
                    LowerLeft => (4.0 * PI / 3.0, 6.0 * PI / 3.0),
                    UpperLeft => (5.0 * PI / 3.0, 7.0 * PI / 3.0),
                    Top => (0.0 * PI / 3.0, 2.0 * PI / 3.0),
                    UpperRight => (1.0 * PI / 3.0, 3.0 * PI / 3.0),
                    LowerRight => (2.0 * PI / 3.0, 4.0 * PI / 3.0),
                };
                let centre = *hex.corner_coord(&corner);
                let radius = hex.theme.track_hard_radius.absolute(hex);
                TrackPath::Curve {
                    centre,
                    radius,
                    angle_0,
                    angle_1,
                    clockwise: true,
                    // TODO: why doesn't make this false cause a test to fail?!?
                    // clockwise: false,
                }
            }
            GentleL => {
                let centre = match self.face {
                    Bottom => hex.midpoint(&HexFace::LowerLeft).scale_by(2.0),
                    LowerLeft => {
                        hex.midpoint(&HexFace::UpperLeft).scale_by(2.0)
                    }
                    UpperLeft => hex.midpoint(&HexFace::Top).scale_by(2.0),
                    Top => hex.midpoint(&HexFace::UpperRight).scale_by(2.0),
                    UpperRight => {
                        hex.midpoint(&HexFace::LowerRight).scale_by(2.0)
                    }
                    LowerRight => {
                        hex.midpoint(&HexFace::Bottom).scale_by(2.0)
                    }
                };
                // TODO: wrong angle order?
                let (angle_0, angle_1) = match self.face {
                    Bottom => (5.0 * PI / 3.0, 6.0 * PI / 3.0),
                    LowerLeft => (0.0 * PI / 3.0, 1.0 * PI / 3.0),
                    UpperLeft => (1.0 * PI / 3.0, 2.0 * PI / 3.0),
                    Top => (2.0 * PI / 3.0, 3.0 * PI / 3.0),
                    UpperRight => (3.0 * PI / 3.0, 4.0 * PI / 3.0),
                    LowerRight => (4.0 * PI / 3.0, 5.0 * PI / 3.0),
                };
                let radius = hex.theme.track_gentle_radius.absolute(hex);
                TrackPath::Curve {
                    centre,
                    radius,
                    angle_0,
                    angle_1,
                    clockwise: false,
                    // TODO: why doesn't make this false cause a test to fail?!?
                    // clockwise: true,
                }
            }
            GentleR => {
                let centre = match self.face {
                    Bottom => {
                        hex.midpoint(&HexFace::LowerRight).scale_by(2.0)
                    }
                    LowerLeft => hex.midpoint(&HexFace::Bottom).scale_by(2.0),
                    UpperLeft => {
                        hex.midpoint(&HexFace::LowerLeft).scale_by(2.0)
                    }
                    Top => hex.midpoint(&HexFace::UpperLeft).scale_by(2.0),
                    UpperRight => hex.midpoint(&HexFace::Top).scale_by(2.0),
                    LowerRight => {
                        hex.midpoint(&HexFace::UpperRight).scale_by(2.0)
                    }
                };
                let (angle_0, angle_1) = match self.face {
                    Bottom => (3.0 * PI / 3.0, 4.0 * PI / 3.0),
                    LowerLeft => (4.0 * PI / 3.0, 5.0 * PI / 3.0),
                    UpperLeft => (5.0 * PI / 3.0, 6.0 * PI / 3.0),
                    Top => (0.0 * PI / 3.0, 1.0 * PI / 3.0),
                    UpperRight => (1.0 * PI / 3.0, 2.0 * PI / 3.0),
                    LowerRight => (2.0 * PI / 3.0, 3.0 * PI / 3.0),
                };
                let radius = hex.theme.track_gentle_radius.absolute(hex);
                TrackPath::Curve {
                    centre,
                    radius,
                    angle_0,
                    angle_1,
                    clockwise: true,
                    // TODO: why doesn't make this false cause a test to fail?!?
                    // clockwise: false,
                }
            }
        }
    }

    pub fn draw_circle_dit(&self, hex: &Hex, ctx: &Context) -> bool {
        if let Some((dit_end, _revenue, DitShape::Circle)) = self.dit {
            let dit_locn = self.end_coord(dit_end, hex);
            let radius = hex.theme.dit_circle_radius.absolute(hex);
            ctx.new_path();
            hex.theme.dit_circle.apply_line(ctx, hex);
            let (x, y) = (dit_locn.x, dit_locn.y);
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
            hex.theme.dit_circle.apply_fill(ctx);
            ctx.fill_preserve().unwrap();
            hex.theme.dit_circle.apply_stroke(ctx);
            ctx.stroke_preserve().unwrap();
            true
        } else {
            false
        }
    }

    pub fn define_circle_dit(&self, hex: &Hex, ctx: &Context) -> bool {
        if let Some((dit_end, _revenue, DitShape::Circle)) = self.dit {
            let dit_locn = self.end_coord(dit_end, hex);
            let radius = hex.theme.dit_circle_radius.absolute(hex);
            ctx.new_path();
            let (x, y) = (dit_locn.x, dit_locn.y);
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
            true
        } else {
            false
        }
    }

    pub fn draw_dit_ends_fg(&self, hex: &Hex, ctx: &Context) {
        self.draw_dit_ends(hex.theme.dit_inner_length, hex, ctx)
    }

    pub fn draw_dit_ends_bg(&self, hex: &Hex, ctx: &Context) {
        self.draw_dit_ends(hex.theme.dit_outer_length, hex, ctx)
    }

    fn draw_dit_ends(&self, length: Length, hex: &Hex, ctx: &Context) {
        use TrackPath::*;

        let dit_length = length.absolute(hex);

        if let Some((dit_end, _revenue, shape)) = self.dit {
            ctx.new_path();
            let dit_locn = self.end_coord(dit_end, hex);

            // TODO: repeated use of self.describe_path() ...
            // TODO: can we call this once at construction time?
            match self.describe_path(hex) {
                Linear { start, end } => {
                    // TODO: lots of generic code here
                    // how much can we share with define_path()?

                    if shape == DitShape::Circle {
                        hex.theme.dit_circle.apply_line(ctx, hex);
                        let radius =
                            hex.theme.dit_circle_radius.absolute(hex);
                        let (x, y) = (dit_locn.x, dit_locn.y);
                        ctx.arc(x, y, radius, 0.0, 2.0 * PI);
                        hex.theme.dit_circle.apply_fill(ctx);
                        ctx.fill_preserve().unwrap();
                        hex.theme.dit_circle.apply_stroke(ctx);
                    } else {
                        // NOTE: line needs to be perpendicular
                        let dit_dirn = Coord::unit_normal(&start, &end);
                        let dit_dirn = &dit_dirn * dit_length;
                        let dit_start = &dit_locn - &dit_dirn;
                        let dit_end = &dit_locn + &dit_dirn;
                        ctx.move_to(dit_start.x, dit_start.y);
                        ctx.line_to(dit_end.x, dit_end.y);
                    }
                }
                Curve {
                    centre,
                    radius,
                    angle_0,
                    angle_1,
                    clockwise,
                } => {
                    // TODO: lots of generic code here
                    // how much can we share with define_path()?
                    let (x0, x1) = if clockwise {
                        (self.x0, self.x1)
                    } else {
                        (1.0 - self.x0, 1.0 - self.x1)
                    };
                    let a0 = angle_0 + x0 * (angle_1 - angle_0);
                    let a1 = angle_0 + x1 * (angle_1 - angle_0);

                    let angle = match dit_end {
                        TrackEnd::Start => a0,
                        TrackEnd::End => a1,
                    };
                    let dit_dx = radius * angle.cos();
                    let dit_dy = radius * angle.sin();
                    let dit_dirn = Coord {
                        x: dit_dx,
                        y: dit_dy,
                    }
                    .normalise();
                    let dit_dirn = &dit_dirn * dit_length;
                    let dit_mid = Coord {
                        x: centre.x + dit_dx,
                        y: centre.y + dit_dy,
                    };

                    if shape == DitShape::Circle {
                        hex.theme.dit_circle.apply_line(ctx, hex);
                        let radius =
                            hex.theme.dit_circle_radius.absolute(hex);
                        let (x, y) = (dit_mid.x, dit_mid.y);
                        ctx.arc(x, y, radius, 0.0, 2.0 * PI);
                        hex.theme.dit_circle.apply_fill(ctx);
                        ctx.fill_preserve().unwrap();
                        hex.theme.dit_circle.apply_stroke(ctx);
                    } else {
                        let dit_start = &dit_mid - &dit_dirn;
                        let dit_end = &dit_mid + &dit_dirn;
                        ctx.move_to(dit_start.x, dit_start.y);
                        ctx.line_to(dit_end.x, dit_end.y);
                    }
                }
            }

            ctx.stroke_preserve().unwrap();
        }
    }

    /// Defines the path for this track segment.
    ///
    /// Note that if this track segment includes a [DitShape::Bar] dit, the
    /// length of the dit is defined by [n18hex::Theme::dit_inner_length].
    /// This ensures that drawing the path will only cover the dit foreground,
    /// allowing the dit background to extend past each end of the foreground
    /// line and provide a complete border around the dit boundary, as long as
    /// [n18hex::Theme::dit_outer_length] is larger than
    /// [n18hex::Theme::dit_inner_length].
    pub fn define_path(&self, hex: &Hex, ctx: &Context) {
        use TrackPath::*;

        ctx.new_path();

        // TODO: draw dit
        // TODO: draw squares at the ends first, so that there's a full white
        // outline ??? No, the changed line_cap would have to apply to the
        // entire path ...
        // TODO: So what? Divide this up into separate methods.
        match self.describe_path(hex) {
            Linear { start, end } => {
                let c0 = start.interpolate(&end, self.x0);
                let c1 = start.interpolate(&end, self.x1);
                if let Some((t0, t1)) = self.clip {
                    let c0_end = start.interpolate(&end, t0);
                    let c1_start = start.interpolate(&end, t1);
                    ctx.move_to(c0.x, c0.y);
                    ctx.line_to(c0_end.x, c0_end.y);
                    ctx.move_to(c1_start.x, c1_start.y);
                    ctx.line_to(c1.x, c1.y);
                } else {
                    ctx.move_to(c0.x, c0.y);
                    ctx.line_to(c1.x, c1.y);
                }
                if let Some((dit_end, _revenue, shape)) = self.dit {
                    let dit_locn = self.end_coord(dit_end, hex);
                    if shape == DitShape::Circle {
                        // NOTE: don't draw the circle, it will produce an
                        // inappropriately-thick line.
                    } else {
                        // NOTE: line needs to be perpendicular
                        // NOTE: use dit_inner_length, as per the doc string.
                        let dit_width =
                            hex.theme.dit_inner_length.absolute(hex);
                        let dit_dirn = Coord::unit_normal(&start, &end);
                        let dit_dirn = &dit_dirn * dit_width;
                        let dit_start = &dit_locn - &dit_dirn;
                        let dit_end = &dit_locn + &dit_dirn;
                        ctx.move_to(dit_start.x, dit_start.y);
                        ctx.line_to(dit_end.x, dit_end.y);
                    }
                }
            }
            Curve {
                centre,
                radius,
                angle_0,
                angle_1,
                clockwise,
            } => {
                // TODO: may need to reverse x0 and x1 here !!!!
                // They're not always in the correct order.
                let (x0, x1) = if clockwise {
                    (self.x0, self.x1)
                } else {
                    (1.0 - self.x0, 1.0 - self.x1)
                };
                let a0 = angle_0 + x0 * (angle_1 - angle_0);
                let a1 = angle_0 + x1 * (angle_1 - angle_0);
                if let Some((t0, t1)) = self.clip {
                    let a0_end = angle_0 + t0 * (angle_1 - angle_0);
                    let a1_start = angle_0 + t1 * (angle_1 - angle_0);
                    if clockwise {
                        ctx.arc(centre.x, centre.y, radius, a0, a0_end);
                        ctx.arc(centre.x, centre.y, radius, a1_start, a1);
                    } else {
                        ctx.arc_negative(
                            centre.x, centre.y, radius, a0, a0_end,
                        );
                        ctx.arc_negative(
                            centre.x, centre.y, radius, a1_start, a1,
                        );
                    }
                } else if clockwise {
                    ctx.arc(centre.x, centre.y, radius, a0, a1);
                } else {
                    ctx.arc_negative(centre.x, centre.y, radius, a0, a1);
                }
                if let Some((dit_end, _revenue, shape)) = self.dit {
                    // NOTE: line needs to be perpendicular
                    // Find the location along the arc, relative to centre.
                    let angle = match dit_end {
                        TrackEnd::Start => a0,
                        TrackEnd::End => a1,
                    };
                    let dit_dx = radius * angle.cos();
                    let dit_dy = radius * angle.sin();
                    // NOTE: use dit_inner_length, as per the doc string.
                    let dit_width = hex.theme.dit_inner_length.absolute(hex);
                    let dit_dirn = Coord {
                        x: dit_dx,
                        y: dit_dy,
                    }
                    .normalise();
                    let dit_dirn = &dit_dirn * dit_width;
                    let dit_mid = Coord {
                        x: centre.x + dit_dx,
                        y: centre.y + dit_dy,
                    };
                    if shape == DitShape::Circle {
                        // NOTE: don't draw the circle, it will produce an
                        // inappropriately-thick line.
                    } else {
                        let dit_start = &dit_mid - &dit_dirn;
                        let dit_end = &dit_mid + &dit_dirn;
                        ctx.move_to(dit_start.x, dit_start.y);
                        ctx.line_to(dit_end.x, dit_end.y);
                    }
                }
            }
        }
    }

    pub fn coords(&self, hex: &Hex, dt: f64) -> TrackCoords {
        // TODO: test cases, ensure that all coords are in the stroke!
        // Also check that all strokes are within hex!
        TrackCoords::new(self, hex, dt)
    }

    pub fn dit_coord(&self, hex: &Hex) -> Option<Coord> {
        self.dit
            .map(|(end, _revenue, _shape)| self.end_coord(end, hex))
    }

    pub fn coord(&self, hex: &Hex, x: f64) -> Option<Coord> {
        use TrackPath::*;

        if self.clipped(x) {
            return None;
        }

        let descr = self.describe_path(hex);
        let coord = match descr {
            Linear { start, end } => start.interpolate(&end, x),
            Curve {
                centre,
                radius,
                angle_0,
                angle_1,
                clockwise,
            } => {
                let x = if clockwise { x } else { 1.0 - x };
                let angle = angle_0 + x * (angle_1 - angle_0);
                let x = centre.x + radius * angle.cos();
                let y = centre.y + radius * angle.sin();
                Coord { x, y }
            }
        };

        Some(coord)
    }

    pub fn start(&self, hex: &Hex) -> Coord {
        use TrackPath::*;

        match self.describe_path(hex) {
            Linear { start, end } => start.interpolate(&end, self.x0),
            Curve {
                centre,
                radius,
                angle_0,
                angle_1,
                clockwise,
            } => {
                // TODO: may need to reverse x0 and x1 here !!!!
                // They're not always in the correct order.
                let x0 = if clockwise { self.x0 } else { 1.0 - self.x0 };
                let angle = angle_0 + x0 * (angle_1 - angle_0);
                let x = centre.x + radius * angle.cos();
                let y = centre.y + radius * angle.sin();
                Coord { x, y }
            }
        }
    }

    pub fn end(&self, hex: &Hex) -> Coord {
        use TrackPath::*;

        match self.describe_path(hex) {
            Linear { start, end } => start.interpolate(&end, self.x1),
            Curve {
                centre,
                radius,
                angle_0,
                angle_1,
                clockwise,
            } => {
                // TODO: may need to reverse x0 and x1 here !!!!
                // They're not always in the correct order.
                let x1 = if clockwise { self.x1 } else { 1.0 - self.x1 };
                let angle = angle_0 + x1 * (angle_1 - angle_0);
                let x = centre.x + radius * angle.cos();
                let y = centre.y + radius * angle.sin();
                Coord { x, y }
            }
        }
    }

    pub fn end_coord(&self, end: TrackEnd, hex: &Hex) -> Coord {
        match end {
            TrackEnd::Start => self.start(hex),
            TrackEnd::End => self.end(hex),
        }
    }

    pub fn intersects_fill<D: Draw>(
        &self,
        obj: &D,
        hex: &Hex,
        dt: f64,
        ctx: &Context,
    ) -> bool {
        obj.define_boundary(hex, ctx);
        self.coords(hex, dt).any(|c| ctx.in_fill(c.x, c.y).unwrap())
    }

    pub fn connected_to_fill<D: Draw>(
        &self,
        obj: &D,
        hex: &Hex,
        ctx: &Context,
    ) -> bool {
        obj.define_boundary(hex, ctx);
        let start = self.start(hex);
        let end = self.end(hex);
        ctx.in_fill(start.x, start.y).unwrap()
            || ctx.in_fill(end.x, end.y).unwrap()
    }

    pub fn connected(&self, other: &Self, hex: &Hex, ctx: &Context) -> bool {
        // NOTE: in_stroke() isn't sufficient here, we need to check whether
        // the track *ends* meet.
        // Mind you, we could check if the start/end of *each* track is in the
        // stroke of the other's boundary?
        other.define_boundary(hex, ctx);
        let c0 = self.start(hex);
        let c1 = self.end(hex);
        let self_in = ctx.in_stroke(c0.x, c0.y).unwrap()
            || ctx.in_stroke(c1.x, c1.y).unwrap();

        self.define_boundary(hex, ctx);
        let c0 = other.start(hex);
        let c1 = other.end(hex);
        let other_in = ctx.in_stroke(c0.x, c0.y).unwrap()
            || ctx.in_stroke(c1.x, c1.y).unwrap();

        self_in && other_in
    }

    pub fn connected_to_faces(&self) -> Vec<(TrackEnd, HexFace)> {
        use TrackCurve::*;

        let mut faces = vec![];
        if self.x0 == 0.0 {
            faces.push((TrackEnd::Start, self.face))
        }
        if (self.x1 - 1.0).abs() < f64::EPSILON {
            let end_face = match self.curve {
                Straight => self.face.opposite(),
                HardL => self.face.clockwise(),
                HardR => self.face.anti_clockwise(),
                GentleL => self.face.clockwise().clockwise(),
                GentleR => self.face.anti_clockwise().anti_clockwise(),
            };
            faces.push((TrackEnd::End, end_face))
        }
        faces
    }

    pub fn connected_to_fill_at<D: Draw>(
        &self,
        obj: &D,
        hex: &Hex,
        ctx: &Context,
    ) -> Option<TrackEnd> {
        obj.define_boundary(hex, ctx);
        let start = self.start(hex);
        let end = self.end(hex);
        let conn_start = ctx.in_fill(start.x, start.y).unwrap();
        let conn_end = ctx.in_fill(end.x, end.y).unwrap();
        if conn_start && conn_end {
            panic!("Track connects at both ends")
        } else if conn_start {
            Some(TrackEnd::Start)
        } else if conn_end {
            Some(TrackEnd::End)
        } else {
            None
        }
    }

    pub fn connected_at(
        &self,
        other: &Self,
        hex: &Hex,
        ctx: &Context,
    ) -> Option<(TrackEnd, TrackEnd)> {
        use TrackEnd::*;

        other.define_boundary(hex, ctx);
        let c0 = self.start(hex);
        let c1 = self.end(hex);
        let self_c0 = ctx.in_stroke(c0.x, c0.y).unwrap();
        let self_c1 = ctx.in_stroke(c1.x, c1.y).unwrap();

        let self_conn = if self_c0 && self_c1 {
            panic!("Tracks connected at both ends")
        } else if self_c0 {
            Some(Start)
        } else if self_c1 {
            Some(End)
        } else {
            None
        };

        self.define_boundary(hex, ctx);
        let c0 = other.start(hex);
        let c1 = other.end(hex);
        let other_c0 = ctx.in_stroke(c0.x, c0.y).unwrap();
        let other_c1 = ctx.in_stroke(c1.x, c1.y).unwrap();

        let other_conn = if other_c0 && other_c1 {
            panic!("Tracks connected at both ends")
        } else if other_c0 {
            Some(Start)
        } else if other_c1 {
            Some(End)
        } else {
            None
        };

        match (self_conn, other_conn) {
            (None, None) => None,
            (Some(end_1), Some(end_2)) => Some((end_1, end_2)),
            _ => {
                // println!("{:?} and {:?}", self_conn, other_conn);
                // panic!("Inconsistent track connections")
                None
            }
        }
    }

    pub fn crosses(
        &self,
        other: &Self,
        hex: &Hex,
        dt: f64,
        ctx: &Context,
    ) -> bool {
        if self.connected(other, hex, ctx) {
            return false;
        }
        other.define_boundary(hex, ctx);
        self.coords(hex, dt)
            .any(|c| ctx.in_stroke(c.x, c.y).unwrap())
    }
}

/// Iterate along a track segment.
pub struct TrackCoords<'a> {
    track: &'a Track,
    descr: TrackPath,
    // start: Coord,
    // end: Coord,
    dt: f64,
    next_pos: f64,
}

impl<'a> TrackCoords<'a> {
    pub fn new(track: &'a Track, hex: &Hex, dt: f64) -> Self {
        let descr = track.describe_path(hex);
        // let start = track.start(hex);
        // let end = track.end(hex);
        let next_pos = track.x0;
        Self {
            track,
            descr,
            // start,
            // end,
            dt,
            next_pos,
        }
    }
}

impl<'a> Iterator for TrackCoords<'a> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        // NOTE: handle clipping
        while self.next_pos <= self.track.x1 {
            if !self.track.clipped(self.next_pos) {
                break;
            }
            // Skip over any clipped sections.
            self.next_pos += self.dt;
        }
        if self.next_pos <= self.track.x1 {
            // TODO: match on curve type
            // let coord = self.start.interpolate(&self.end, self.next_pos);

            use TrackPath::*;

            let coord = match self.descr {
                Linear { start, end } => {
                    start.interpolate(&end, self.next_pos)
                }
                Curve {
                    centre,
                    radius,
                    angle_0,
                    angle_1,
                    clockwise,
                } => {
                    let next_pos = if clockwise {
                        self.next_pos
                    } else {
                        1.0 - self.next_pos
                    };
                    let angle = angle_0 + next_pos * (angle_1 - angle_0);
                    let x = centre.x + radius * angle.cos();
                    let y = centre.y + radius * angle.sin();
                    Coord { x, y }
                }
            };

            self.next_pos += self.dt;
            Some(coord)
        } else {
            None
        }
    }
}

impl Draw for Track {
    fn define_boundary(&self, hex: &Hex, ctx: &Context) {
        self.define_path(hex, ctx);
        // NOTE: also set the line width so that ctx.in_stroke() will behave
        // as expected when trying to determine whether two track segments
        // cross or intersect.
        hex.theme.track_outer.apply_line(ctx, hex);
    }

    fn draw_bg(&self, hex: &Hex, ctx: &Context) {
        self.define_path(hex, ctx);
        hex.theme.track_outer.apply_line_and_stroke(ctx, hex);
        ctx.stroke_preserve().unwrap();
        // NOTE: the outer (background) dit length must be larger than the
        // inner (foreground) dit length, as used by self.define_path(), so
        // that the background stroke extends past the foreground stroke at
        // each end of the dit.
        // Alternatively, we could modify define_path() to take this length as
        // an additional argument.
        hex.theme.dit_outer.apply_line_and_stroke(ctx, hex);
        self.draw_dit_ends_bg(hex, ctx);
    }

    fn draw_fg(&self, hex: &Hex, ctx: &Context) {
        self.define_path(hex, ctx);
        hex.theme.track_inner.apply_line_and_stroke(ctx, hex);
        ctx.stroke_preserve().unwrap();
        hex.theme.dit_inner.apply_line_and_stroke(ctx, hex);
        self.draw_dit_ends_fg(hex, ctx);
    }
}
