use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
    pub const RIGHT: Self = Self { x: 1, y: 0, z: 0 };
    pub const LEFT: Self = Self { x: -1, y: 0, z: 0 };
    pub const FORTH: Self = Self { x: 0, y: 1, z: 0 };
    pub const BACK: Self = Self { x: 0, y: -1, z: 0 };
    pub const UP: Self = Self { x: 0, y: 0, z: 1 };
    pub const DOWN: Self = Self { x: 0, y: 0, z: -1 };

    #[inline]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<i32> for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: i32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl MulAssign<i32> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, other: i32) {
        *self = *self * other;
    }
}
