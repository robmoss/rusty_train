/// Represents Cartesian coordinates as a struct with named fields.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl std::ops::Add for &Coord {
    type Output = Coord;

    fn add(self, other: &Coord) -> Coord {
        Coord {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for &Coord {
    type Output = Coord;

    fn sub(self, other: &Coord) -> Coord {
        Coord {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Div<f64> for &Coord {
    type Output = Coord;

    fn div(self, rhs: f64) -> Coord {
        Coord {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::Mul<f64> for &Coord {
    type Output = Coord;

    fn mul(self, rhs: f64) -> Coord {
        Coord {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Coord {
    /// Returns the midpoint of this point and the `other` point.
    pub fn average(&self, other: &Self) -> Self {
        Coord {
            x: 0.5 * (self.x + other.x),
            y: 0.5 * (self.y + other.y),
        }
    }

    /// Multiplies the `x` and `y` values by `scale`.
    pub fn scale_by(&self, scale: f64) -> Self {
        Coord {
            x: scale * self.x,
            y: scale * self.y,
        }
    }

    /// Returns the point at some fraction between this point (`frac = 0`) and
    /// the `other` point (`frac = 1`).
    pub fn interpolate(&self, other: &Self, frac: f64) -> Self {
        Coord {
            x: self.x + frac * (other.x - self.x),
            y: self.y + frac * (other.y - self.y),
        }
    }

    /// Returns the magnitude (Euclidean norm) of this coordinate.
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns a scaled version of the point that has a magnitude of `1`
    /// (i.e., a unit vector).
    pub fn normalise(&self) -> Self {
        self / self.magnitude()
    }

    /// Returns a unit vector parallel to the line between `from` and `to`.
    pub fn unit_normal(from: &Self, to: &Self) -> Self {
        let vec = (to - from).normalise();
        Self { x: vec.y, y: vec.x }
    }
}

impl From<(f64, f64)> for Coord {
    fn from(tuple: (f64, f64)) -> Self {
        Coord {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<&(f64, f64)> for Coord {
    fn from(tuple: &(f64, f64)) -> Self {
        Coord {
            x: tuple.0,
            y: tuple.1,
        }
    }
}
