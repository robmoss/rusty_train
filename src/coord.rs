#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub fn average(self: &Self, other: &Self) -> Self {
        Coord {
            x: 0.5 * (self.x + other.x),
            y: 0.5 * (self.y + other.y),
        }
    }

    pub fn scale_by(self: &Self, scale: f64) -> Self {
        Coord {
            x: scale * self.x,
            y: scale * self.y,
        }
    }

    pub fn interpolate(self: &Self, other: &Self, frac: f64) -> Self {
        Coord {
            x: self.x + frac * (other.x - self.x),
            y: self.y + frac * (other.y - self.y),
        }
    }

    pub fn magnitude(self: &Self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalise(self: &Self) -> Self {
        self / self.magnitude()
    }

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
