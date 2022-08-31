use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[macro_export]
macro_rules! pos {
    ($x:expr, $y:expr) => {
        Pos { x: $x, y: $y }
    };
}

impl Add for Pos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        pos!(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Pos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        pos!(self.x - rhs.x, self.y - rhs.y)
    }
}
