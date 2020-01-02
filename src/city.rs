use crate::coord::Coord;
use crate::draw::Draw;
use crate::hex::{Delta, Hex, HexCorner, HexFace, HexPosition};
use cairo::Context;
use std::f64::consts::PI;

/// Cities that are connected by track.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct City {
    // TODO: replace this with an enum that has a .count() method?
    num_tokens: usize,
    pub revenue: usize,
    pub position: HexPosition,
    pub angle: f64,
}

impl City {
    // TODO: allow a city to be "terminal" in that routes cannot leave it
    // (i.e., red off-map tiles) ... also need triangular track segments ...
    pub fn rotate(mut self, angle: f64) -> Self {
        self.angle += angle;
        self
    }

    fn delta_coords(
        &self,
        from: &Coord,
        delta: Option<Delta>,
        hex: &Hex,
    ) -> Coord {
        let radius = 0.5 * hex.max_d;
        match delta {
            Some(Delta::Nudge(angle, frac)) => Coord {
                x: frac * radius * angle.cos(),
                y: frac * radius * angle.sin(),
            },
            Some(Delta::ToCentre(frac)) => from * -frac,
            None => (0.0, 0.0).into(),
        }
    }

    fn translate_coords(&self, hex: &Hex) -> Coord {
        match self.position {
            HexPosition::Centre(delta) => {
                let from = (0.0, 0.0).into();
                let d = self.delta_coords(&from, delta, hex);
                &from + &d
            }
            HexPosition::Face(face, delta) => {
                let exit = hex.midpoint(&face);
                let centre = exit.scale_by(0.7);
                let d = self.delta_coords(&centre, delta, hex);
                &centre + &d
            }
            HexPosition::Corner(corner, delta) => {
                let c1 = hex.corner_coord(&corner.next());
                let c2 = hex.corner_coord(&corner.prev());
                let centre = c1.average(&c2);
                let d = self.delta_coords(&centre, delta, hex);
                &centre + &d
            }
        }
    }

    pub fn translate_begin(&self, hex: &Hex, ctx: &Context) {
        let coord = self.translate_coords(hex);
        ctx.translate(coord.x, coord.y);
    }

    pub fn translate_end(&self, hex: &Hex, ctx: &Context) {
        let coord = self.translate_coords(hex);
        ctx.translate(-coord.x, -coord.y);
    }

    pub fn nudge(mut self, angle: f64, frac: f64) -> Self {
        self.position = self.position.nudge(angle, frac);
        self
    }

    pub fn central_dit(revenue: usize) -> City {
        City {
            num_tokens: 0,
            revenue: revenue,
            position: HexPosition::Centre(None),
            angle: 0.0,
        }
    }

    pub fn single(revenue: usize) -> City {
        City {
            num_tokens: 1,
            revenue: revenue,
            position: HexPosition::Centre(None),
            angle: 0.0,
        }
    }

    pub fn single_at_face(revenue: usize, face: &HexFace) -> City {
        City {
            num_tokens: 1,
            revenue: revenue,
            position: HexPosition::Face(*face, None),
            angle: 0.0,
        }
    }

    pub fn single_at_corner(revenue: usize, corner: &HexCorner) -> City {
        City {
            num_tokens: 1,
            revenue: revenue,
            position: HexPosition::Corner(*corner, None),
            angle: 0.0,
        }
    }

    pub fn double(revenue: usize) -> City {
        City {
            num_tokens: 2,
            revenue: revenue,
            position: HexPosition::Centre(None),
            angle: 0.0,
        }
    }

    pub fn double_at_corner(revenue: usize, corner: &HexCorner) -> City {
        use HexCorner::*;

        let angle = match *corner {
            TopLeft => -PI / 6.0,
            TopRight => PI / 6.0,
            Right => 3.0 * PI / 6.0,
            BottomRight => 5.0 * PI / 6.0,
            BottomLeft => 7.0 * PI / 6.0,
            Left => 9.0 * PI / 6.0,
        };
        City {
            num_tokens: 2,
            revenue: revenue,
            position: HexPosition::Corner(*corner, None),
            angle: angle,
        }
    }

    // TODO: triple as a triangle or as a row of 3 tokens?
    pub fn triple(revenue: usize) -> City {
        City {
            num_tokens: 3,
            revenue: revenue,
            position: HexPosition::Centre(None),
            angle: 0.0,
        }
    }

    pub fn quad(revenue: usize) -> City {
        City {
            num_tokens: 4,
            revenue: revenue,
            position: HexPosition::Centre(None),
            angle: 0.0,
        }
    }

    // TODO: pub fn sextuple(revenue: usize) -> City
    // See tiles 8887 and 8888 for the game 1880:
    // http://www.fwtwr.com/18xx/tiles/tiles.asp?xGame=1880

    fn define_fg_path(&self, hex: &Hex, ctx: &Context) {
        let radius = hex.max_d * 0.125;
        self.define_bg_path(hex, ctx);

        // TODO: if self.num_tokens == 0

        if self.num_tokens == 2 {
            // Define each token space.
            for x in vec![radius, -radius] {
                ctx.new_sub_path();
                ctx.arc(x, 0.0, radius, 0.0, 2.0 * PI);
            }
        } else if self.num_tokens == 3 {
            // Each circle is centred at the tip of an equilateral triangle
            // with side length 2 * radius; it has height radius * sqrt(3).
            let half_height = radius * (3.0 as f64).sqrt() / 2.0;
            let centres = vec![
                (-radius, half_height),
                (radius, half_height),
                (0.0, -half_height),
            ];
            // Define each token space.
            for (x, y) in &centres {
                ctx.new_sub_path();
                ctx.arc(*x, *y, radius, 0.0, 2.0 * PI);
            }
        } else if self.num_tokens == 4 {
            // Define each token space.
            for x in vec![radius, -radius] {
                for y in vec![radius, -radius] {
                    ctx.new_sub_path();
                    ctx.arc(x, y, radius, 0.0, 2.0 * PI);
                }
            }
        }
    }

    fn define_bg_path(&self, hex: &Hex, ctx: &Context) {
        let radius = hex.max_d * 0.125;
        ctx.new_path();

        if self.num_tokens == 0 {
            let radius = hex.max_d * 0.085;
            let (x, y) = (0.0, 0.0);
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
        } else if self.num_tokens == 1 {
            let (x, y) = (0.0, 0.0);
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
        } else if self.num_tokens == 2 {
            // Define the containing box.
            ctx.move_to(-radius, radius);
            ctx.line_to(radius, radius);
            ctx.arc_negative(radius, 0.0, radius, PI / 2.0, -PI / 2.0);
            ctx.line_to(radius, -radius);
            ctx.line_to(-radius, -radius);
            ctx.arc_negative(-radius, 0.0, radius, -PI / 2.0, PI / 2.0);
            ctx.close_path();
        } else if self.num_tokens == 3 {
            // Each circle is centred at the tip of an equilateral triangle
            // with side length 2 * radius; it has height radius * sqrt(3).
            let half_height = radius * (3.0 as f64).sqrt() / 2.0;
            let centres = vec![
                (-radius, half_height),
                (radius, half_height),
                (0.0, -half_height),
            ];
            // Define the containing box.
            // Want the middle half of each edge of an equilateral triangle
            // that is larger than that described by the circle centres.
            let scale = 2.0 / ((3.0 as f64).sqrt() - 1.0);
            let translate_y = -0.5 * radius;
            let pts: Vec<(f64, f64)> = centres
                .iter()
                .map(|(x, y)| (scale * x, scale * y + translate_y))
                .collect();
            ctx.move_to(
                pts[0].0 + 1.0 / 3.0 * (pts[1].0 - pts[0].0),
                pts[0].1 + 1.0 / 3.0 * (pts[1].1 - pts[0].1),
            );
            ctx.line_to(
                pts[0].0 + 2.0 / 3.0 * (pts[1].0 - pts[0].0),
                pts[0].1 + 2.0 / 3.0 * (pts[1].1 - pts[0].1),
            );
            ctx.arc_negative(
                centres[1].0,
                centres[1].1,
                radius,
                PI / 2.0,
                -PI / 6.0,
            );
            ctx.line_to(
                pts[1].0 + 2.0 / 3.0 * (pts[2].0 - pts[1].0),
                pts[1].1 + 2.0 / 3.0 * (pts[2].1 - pts[1].1),
            );
            ctx.arc_negative(
                centres[2].0,
                centres[2].1,
                radius,
                -PI / 6.0,
                -5.0 * PI / 6.0,
            );
            ctx.line_to(
                pts[2].0 + 2.0 / 3.0 * (pts[0].0 - pts[2].0),
                pts[2].1 + 2.0 / 3.0 * (pts[0].1 - pts[2].1),
            );
            ctx.arc_negative(
                centres[0].0,
                centres[0].1,
                radius,
                -5.0 * PI / 6.0,
                PI / 2.0,
            );
            ctx.close_path();
        } else if self.num_tokens == 4 {
            // Define the containing box.
            ctx.move_to(-radius, 2.0 * radius);
            ctx.line_to(radius, 2.0 * radius);
            ctx.arc_negative(radius, radius, radius, PI / 2.0, 0.0);
            ctx.line_to(2.0 * radius, radius);
            ctx.line_to(2.0 * radius, -radius);
            ctx.arc_negative(radius, -radius, radius, 0.0, -PI / 2.0);
            ctx.line_to(radius, -2.0 * radius);
            ctx.line_to(-radius, -2.0 * radius);
            ctx.arc_negative(-radius, -radius, radius, -PI / 2.0, -PI);
            ctx.line_to(-2.0 * radius, -radius);
            ctx.line_to(-2.0 * radius, radius);
            ctx.arc_negative(-radius, radius, radius, -PI, -3.0 * PI / 2.0);
            ctx.line_to(-radius, 2.0 * radius);
        }
    }

    pub fn token_ixs(&self) -> Vec<usize> {
        (0..self.num_tokens).collect()
    }

    pub fn define_token_path(
        &self,
        ix: usize,
        hex: &Hex,
        ctx: &Context,
    ) -> bool {
        if ix >= self.num_tokens {
            return false;
        }

        self.translate_begin(hex, ctx);
        ctx.rotate(self.angle);
        let radius = hex.max_d * 0.125;
        ctx.new_path();

        if self.num_tokens == 1 {
            let (x, y) = (0.0, 0.0);
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
        } else if self.num_tokens == 2 {
            let x = vec![radius, -radius][ix];
            ctx.arc(x, 0.0, radius, 0.0, 2.0 * PI);
        } else if self.num_tokens == 3 {
            let half_height = radius * (3.0 as f64).sqrt() / 2.0;
            let (x, y) = vec![
                (-radius, half_height),
                (radius, half_height),
                (0.0, -half_height),
            ][ix];
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
        } else if self.num_tokens == 4 {
            let (x, y) = vec![
                (radius, radius),
                (radius, -radius),
                (-radius, radius),
                (-radius, -radius),
            ][ix];
            ctx.arc(x, y, radius, 0.0, 2.0 * PI);
        } else {
            // NOTE: will have to extend this for 6-token cities.
            unreachable!()
        }

        ctx.rotate(-self.angle);
        self.translate_end(hex, ctx);

        return true;
    }
}

impl Draw for City {
    fn define_boundary(&self, hex: &Hex, ctx: &Context) {
        self.translate_begin(hex, ctx);
        ctx.rotate(self.angle);

        self.define_bg_path(hex, ctx);

        ctx.rotate(-self.angle);
        self.translate_end(hex, ctx);
    }

    fn draw_bg(&self, hex: &Hex, ctx: &Context) {
        self.translate_begin(hex, ctx);
        ctx.rotate(self.angle);

        self.define_bg_path(hex, ctx);
        if self.num_tokens == 0 {
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.fill_preserve();
        } else {
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.set_line_width(hex.max_d * 0.03);
            ctx.stroke_preserve();
            ctx.fill_preserve();
        }

        ctx.rotate(-self.angle);
        self.translate_end(hex, ctx);
    }

    fn draw_fg(&self, hex: &Hex, ctx: &Context) {
        self.translate_begin(hex, ctx);
        ctx.rotate(self.angle);

        // TODO: if self.num_tokens == 0
        self.define_bg_path(hex, ctx);
        if self.num_tokens == 0 {
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.set_line_width(hex.max_d * 0.01);
            ctx.stroke_preserve();
        } else {
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.fill_preserve();
            self.define_fg_path(hex, ctx);
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.set_line_width(hex.max_d * 0.01);
            ctx.stroke();
        }

        ctx.rotate(-self.angle);
        self.translate_end(hex, ctx);
    }
}
