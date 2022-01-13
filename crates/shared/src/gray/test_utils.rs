use approx::{AbsDiffEq, assert_relative_eq, RelativeEq};
use crate::Gray;

pub fn assert_relative_eq_gray(left: Gray, right: Gray) {
    assert_relative_eq!(
        AssertableGray(left),
        AssertableGray(right),
    )
}

#[derive(PartialEq, Debug)]
pub(crate) struct AssertableGray(pub Gray);

impl AbsDiffEq for AssertableGray {
    type Epsilon = f32;

    fn default_epsilon() -> f32 {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: f32) -> bool {
        f32::abs_diff_eq(&self.0.0, &other.0.0, epsilon)
    }
}

impl RelativeEq for AssertableGray {
    fn default_max_relative() -> f32 {
        f32::default_max_relative()
    }

    fn relative_eq(&self, other: &Self, epsilon: f32, max_relative: f32) -> bool {
        f32::relative_eq(&self.0.0, &other.0.0, epsilon, max_relative)
    }
}
