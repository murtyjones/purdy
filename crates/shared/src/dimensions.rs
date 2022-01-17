use std::ops::{Div, Sub};

use derive_more::{Add, Deref, Display, From, Into, Mul, Neg};

#[derive(Debug, Copy, Clone, Add, Display, From, Into, PartialEq, Mul, Deref, Neg)]
pub struct Width(f32);

impl Width {
    pub fn new(v: f32) -> Self {
        Width(v)
    }
}

impl Sub<f32> for Width {
    type Output = f32;

    fn sub(self, other: f32) -> Self::Output {
        self.0 - other
    }
}

impl Div<f32> for Width {
    // The division of rational numbers is a closed operation.
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
    }
}

#[derive(Debug, Copy, Clone, Add, Display, From, Into, PartialEq, Mul, Deref, Neg)]
pub struct Height(f32);

impl Height {
    pub fn new(v: f32) -> Self {
        Height(v)
    }
}

impl Sub<f32> for Height {
    type Output = f32;

    fn sub(self, other: f32) -> Self::Output {
        self.0 - other
    }
}

impl Div<f32> for Height {
    // The division of rational numbers is a closed operation.
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
    }
}
