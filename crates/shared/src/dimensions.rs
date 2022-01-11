use std::ops::{Deref, Sub, Div, Neg};

#[derive(Debug, Copy, Clone)]
pub struct Width(f32);

impl Width {
    pub fn new(v: f32) -> Self {
        Width(v)
    }
}

impl Deref for Width {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub<f32> for Width {
    type Output = f32;

    fn sub(self, other: f32) -> Self::Output {
        self.0 - other
    }
}



impl Sub for Width {
    type Output = Width;

    fn sub(self, other: Width) -> Self::Output {
        Width(self.0 - other.0)
    }
}

impl Div<f32> for Width {
    // The division of rational numbers is a closed operation.
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
    }
}

impl Neg for Width {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Width(-self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Height(f32);

impl Height {
    pub fn new(v: f32) -> Self {
        Height(v)
    }
}

impl Deref for Height {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub<f32> for Height {
    type Output = f32;

    fn sub(self, other: f32) -> Self::Output {
        self.0 - other
    }
}

impl Sub for Height {
    type Output = Height;

    fn sub(self, other: Height) -> Self::Output {
        Height(self.0 - other.0)
    }
}

impl Div<f32> for Height {
    // The division of rational numbers is a closed operation.
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
    }
}

impl Neg for Height {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Height(-self.0)
    }
}