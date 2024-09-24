use lopdf::Object;
use std::ops::Add;

#[derive(Copy, Clone, Debug)]
struct Coordinate(f32, f32);

impl Add<Coordinate> for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Coordinate) -> Coordinate {
        Coordinate(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BBox {
    lower_left: Coordinate,
    upper_right: Coordinate,
}

impl BBox {
    fn height(&self) -> f32 {
        self.upper_right.1 - self.lower_left.1
    }

    pub fn from_llur(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let lower_left = Coordinate(x0, y0);
        let upper_right = Coordinate(x1, y1);
        Self {
            lower_left,
            upper_right,
        }
    }

    // Construct a BBox from (left, top, width, height)
    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        let lower_left = Coordinate(left, top - height);
        let upper_right = Coordinate(left + width, top);
        Self {
            lower_left,
            upper_right,
        }
    }

    pub fn as_vec(&self) -> Vec<Object> {
        vec![
            Object::Real(self.lower_left.0),
            Object::Real(self.lower_left.1),
            Object::Real(self.upper_right.0),
            Object::Real(self.upper_right.1),
        ]
    }

    pub fn as_quad_vec(&self) -> Vec<Object> {
        vec![
            Object::Real(self.lower_left.0),
            Object::Real(self.lower_left.1),
            Object::Real(self.upper_right.0),
            Object::Real(self.lower_left.1),
            Object::Real(self.upper_right.0),
            Object::Real(self.upper_right.1),
            Object::Real(self.lower_left.0),
            Object::Real(self.upper_right.1),
        ]
    }

    pub fn offset_within(self, other: BBox) -> BBox {
        let lower_left = Coordinate(
            self.lower_left.0 + other.lower_left.0,
            other.upper_right.1 - self.upper_right.1 - self.height(),
        );
        let upper_right = Coordinate(
            self.upper_right.0 + other.lower_left.0,
            other.upper_right.1 - self.upper_right.1,
        );
        BBox {
            lower_left,
            upper_right,
        }
    }
}
