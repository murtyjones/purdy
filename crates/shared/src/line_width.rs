use derive_more::{Add, Deref, Display, Div, From, Into, Mul, Neg};

#[derive(Debug, Copy, Clone, Add, Display, From, Into, PartialEq, Mul, Div, Deref, Neg)]
pub struct LineWidth(f32);

impl LineWidth {
    pub fn new(v: f32) -> Self {
        LineWidth(v)
    }

    pub fn set(&mut self, v: LineWidth) {
        self.0 = *v;
    }
}

impl Default for LineWidth {
    fn default() -> Self {
        LineWidth::new(1.0)
    }
}
