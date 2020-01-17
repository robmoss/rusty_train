use crate::coord::Coord;
use crate::draw::Draw;
use crate::hex::{Hex, HexCorner, HexFace};
use cairo::Context;
use std::f64::consts::PI;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

/// Track segments along which trains can run routes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Track {
    pub face: HexFace,
    pub curve: TrackCurve,
    pub x0: f64,
    pub x1: f64,
    pub clip: Option<(f64, f64)>,
    pub dit: Option<(TrackEnd, usize)>,
    // TODO: different gauges, double track, etc?
    // TODO: save track description?
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
        dit: Option<(TrackEnd, usize)>,
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
            face: face,
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
            face: face,
            curve: TrackCurve::Straight,
            x0: 0.0,
            x1: 0.5,
            clip: None,
            dit: None,
        }
    }

    pub fn gentle_l(face: HexFace) -> Self {
        Self {
            face: face,
            curve: TrackCurve::GentleL,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn gentle_r(face: HexFace) -> Self {
        Self {
            face: face,
            curve: TrackCurve::GentleR,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn hard_l(face: HexFace) -> Self {
        Self {
            face: face,
            curve: TrackCurve::HardL,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn hard_r(face: HexFace) -> Self {
        Self {
            face: face,
            curve: TrackCurve::HardR,
            x0: 0.0,
            x1: 1.0,
            clip: None,
            dit: None,
        }
    }

    pub fn with_dit(mut self, end: TrackEnd, revenue: usize) -> Self {
        // TODO: check location.
        self.dit = Some((end, revenue));
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
            if dt >= x0 && dt <= x1 {
                true
            } else {
                false
            }
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
                let centre = hex.corner_coord(&corner).clone();
                let radius = hex.max_d * 0.25;
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
                let centre = hex.corner_coord(&corner).clone();
                let radius = hex.max_d * 0.25;
                TrackPath::Curve {
                    centre: centre,
                    radius: radius,
                    angle_0: angle_0,
                    angle_1: angle_1,
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
                let radius = hex.max_d * 0.75;
                TrackPath::Curve {
                    centre: centre,
                    radius: radius,
                    angle_0: angle_0,
                    angle_1: angle_1,
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
                let radius = hex.max_d * 0.75;
                TrackPath::Curve {
                    centre: centre,
                    radius: radius,
                    angle_0: angle_0,
                    angle_1: angle_1,
                    clockwise: true,
                    // TODO: why doesn't make this false cause a test to fail?!?
                    // clockwise: false,
                }
            }
        }
    }

    pub fn draw_dit_ends(&self, line_width: f64, hex: &Hex, ctx: &Context) {
        use TrackPath::*;

        // let dit_width = hex.max_d * 0.10;
        let dit_width = hex.max_d * line_width;

        if let Some((dit_end, _revenue)) = self.dit {
            ctx.new_path();
            let dit_locn = self.end_coord(dit_end, hex);

            // let line_cap = ctx.get_line_cap();
            // // NOTE: Square doesn't work, Cairo can't figure out orientation.
            // // ctx.set_line_cap(cairo::LineCap::Square);
            // ctx.set_line_cap(cairo::LineCap::Round);

            // TODO: repeated use of self.describe_path() ...
            // TODO: can we call this once at construction time?
            match self.describe_path(hex) {
                Linear { start, end } => {
                    // TODO: lots of generic code here
                    // how much can we share with define_path()?

                    // NOTE: line needs to be perpendicular
                    let dit_dirn = Coord::unit_normal(&start, &end);
                    let dit_dirn = &dit_dirn * dit_width;
                    let dit_start = &dit_locn - &dit_dirn;
                    let dit_end = &dit_locn + &dit_dirn;

                    ctx.move_to(dit_start.x, dit_start.y);
                    // ctx.line_to(dit_start.x, dit_start.y);
                    // ctx.move_to(dit_end.x, dit_end.y);
                    ctx.line_to(dit_end.x, dit_end.y);
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
                    let dit_dirn = &dit_dirn * dit_width;
                    let dit_mid = Coord {
                        x: centre.x + dit_dx,
                        y: centre.y + dit_dy,
                    };
                    let dit_start = &dit_mid - &dit_dirn;
                    let dit_end = &dit_mid + &dit_dirn;
                    ctx.move_to(dit_start.x, dit_start.y);
                    // ctx.line_to(dit_start.x, dit_start.y);
                    // ctx.move_to(dit_end.x, dit_end.y);
                    ctx.line_to(dit_end.x, dit_end.y);
                }
            }

            ctx.stroke_preserve();
            // ctx.set_line_cap(line_cap);
        }
    }

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
                if let Some((dit_end, _revenue)) = self.dit {
                    // NOTE: line needs to be perpendicular
                    let dit_locn = self.end_coord(dit_end, hex);
                    let dit_width = hex.max_d * 0.10;
                    let dit_dirn = Coord::unit_normal(&start, &end);
                    let dit_dirn = &dit_dirn * dit_width;
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
                } else {
                    if clockwise {
                        ctx.arc(centre.x, centre.y, radius, a0, a1);
                    } else {
                        ctx.arc_negative(centre.x, centre.y, radius, a0, a1);
                    }
                }
                if let Some((dit_end, _revenue)) = self.dit {
                    // NOTE: line needs to be perpendicular
                    // Find the location along the arc, relative to centre.
                    let angle = match dit_end {
                        TrackEnd::Start => a0,
                        TrackEnd::End => a1,
                    };
                    let dit_dx = radius * angle.cos();
                    let dit_dy = radius * angle.sin();
                    let dit_width = hex.max_d * 0.10;
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
                    let dit_start = &dit_mid - &dit_dirn;
                    let dit_end = &dit_mid + &dit_dirn;
                    ctx.move_to(dit_start.x, dit_start.y);
                    ctx.line_to(dit_end.x, dit_end.y);
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
        self.dit.map(|(end, _revenue)| self.end_coord(end, hex))
    }

    pub fn get_coord(&self, hex: &Hex, x: f64) -> Option<Coord> {
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
        self.coords(hex, dt).any(|c| ctx.in_fill(c.x, c.y))
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
        ctx.in_fill(start.x, start.y) || ctx.in_fill(end.x, end.y)
    }

    pub fn connected(&self, other: &Self, hex: &Hex, ctx: &Context) -> bool {
        // NOTE: in_stroke() isn't sufficient here, we need to check whether
        // the track *ends* meet.
        // Mind you, we could check if the start/end of *each* track is in the
        // stroke of the other's boundary?
        other.define_boundary(hex, ctx);
        let c0 = self.start(hex);
        let c1 = self.end(hex);
        let self_in = ctx.in_stroke(c0.x, c0.y) || ctx.in_stroke(c1.x, c1.y);

        self.define_boundary(hex, ctx);
        let c0 = other.start(hex);
        let c1 = other.end(hex);
        let other_in = ctx.in_stroke(c0.x, c0.y) || ctx.in_stroke(c1.x, c1.y);

        self_in && other_in
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
        self.coords(hex, dt).any(|c| ctx.in_stroke(c.x, c.y))
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
        ctx.set_line_width(hex.max_d * 0.10);
    }

    fn draw_bg(&self, hex: &Hex, ctx: &Context) {
        // TODO: treat dits differently, so that their ends have white lines?
        self.define_path(hex, ctx);
        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.set_line_width(hex.max_d * 0.10);
        ctx.stroke_preserve();
        // TODO: for example, self.define_dits(), ctx.set_blah, ctx.stroke()
        // ctx.set_source_rgb(0.0, 1.0, 0.0);
        // NOTE: the width needs to be a bit LARGER than the path defined by
        // self.define_path() ... maybe define_path() should take dit_width
        // as an argument?!?
        self.draw_dit_ends(0.11, hex, ctx);
    }

    fn draw_fg(&self, hex: &Hex, ctx: &Context) {
        // TODO: treat dits differently, so that their ends have white lines?
        self.define_path(hex, ctx);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.08);
        ctx.stroke_preserve();
        // TODO: for example, self.define_dits(hex, ctx), ctx.set_blah, ctx.stroke()
        // ctx.set_source_rgb(0.0, 0.0, 1.0);
        self.draw_dit_ends(0.10, hex, ctx);
    }
}
