use std::ops::{Deref, Sub, Div, Neg};

#[derive(Debug, Copy, Clone)]
pub struct PageWidth(f32);

impl PageWidth {
    pub fn new(v: f32) -> Self {
        PageWidth(v)
    }
}

impl Deref for PageWidth {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub<f32> for PageWidth {
    type Output = f32;

    fn sub(self, other: f32) -> Self::Output {
        self.0 - other
    }
}



impl Sub for PageWidth {
    type Output = PageWidth;

    fn sub(self, other: PageWidth) -> Self::Output {
        PageWidth(self.0 - other.0)
    }
}

impl Div<f32> for PageWidth {
    // The division of rational numbers is a closed operation.
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
    }
}

impl Neg for PageWidth {
    type Output = Self;

    fn neg(self) -> Self::Output {
        PageWidth(-self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PageHeight(f32);

impl PageHeight {
    pub fn new(v: f32) -> Self {
        PageHeight(v)
    }
}

impl Deref for PageHeight {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub<f32> for PageHeight {
    type Output = f32;

    fn sub(self, other: f32) -> Self::Output {
        self.0 - other
    }
}

impl Sub for PageHeight {
    type Output = PageHeight;

    fn sub(self, other: PageHeight) -> Self::Output {
        PageHeight(self.0 - other.0)
    }
}

impl Div<f32> for PageHeight {
    // The division of rational numbers is a closed operation.
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
    }
}

impl Neg for PageHeight {
    type Output = Self;

    fn neg(self) -> Self::Output {
        PageHeight(-self.0)
    }
}