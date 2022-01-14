use approx::{AbsDiffEq, assert_relative_eq, RelativeEq};
use crate::Rgb;

pub fn assert_relative_eq_rgb(left: Rgb, right: Rgb) {
    assert_relative_eq!(
        AssertableRgb(left),
        AssertableRgb(right),
    )
}

#[derive(PartialEq, Debug)]
pub(crate) struct AssertableRgb(pub Rgb);

impl AbsDiffEq for AssertableRgb {
    type Epsilon = f32;

    fn default_epsilon() -> f32 {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: f32) -> bool {
        f32::abs_diff_eq(&self.0.red, &other.0.red, epsilon) &&
        f32::abs_diff_eq(&self.0.green, &other.0.green, epsilon) &&
        f32::abs_diff_eq(&self.0.blue, &other.0.blue, epsilon)
    }
}

impl RelativeEq for AssertableRgb {
    fn default_max_relative() -> f32 {
        f32::default_max_relative()
    }

    fn relative_eq(&self, other: &Self, epsilon: f32, max_relative: f32) -> bool {
        f32::relative_eq(&self.0.red, &other.0.red, epsilon, max_relative) &&
        f32::relative_eq(&self.0.green, &other.0.green, epsilon, max_relative) &&
        f32::relative_eq(&self.0.blue, &other.0.blue, epsilon, max_relative)
    }
}
