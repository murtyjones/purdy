use approx::{AbsDiffEq, assert_relative_eq, RelativeEq};
use crate::Cmyk;

pub fn assert_relative_eq_cmyk(left: Cmyk, right: Cmyk) {
    assert_relative_eq!(
        AssertableCmyk(left),
        AssertableCmyk(right),
    )
}

#[derive(PartialEq, Debug)]
pub(crate) struct AssertableCmyk(pub Cmyk);

impl AbsDiffEq for AssertableCmyk {
    type Epsilon = f32;

    fn default_epsilon() -> f32 {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: f32) -> bool {
        f32::abs_diff_eq(&self.0.0, &other.0.0, epsilon) &&
        f32::abs_diff_eq(&self.0.1, &other.0.1, epsilon) &&
        f32::abs_diff_eq(&self.0.2, &other.0.2, epsilon) &&
        f32::abs_diff_eq(&self.0.3, &other.0.3, epsilon)
    }
}

impl RelativeEq for AssertableCmyk {
    fn default_max_relative() -> f32 {
        f32::default_max_relative()
    }

    fn relative_eq(&self, other: &Self, epsilon: f32, max_relative: f32) -> bool {
        f32::relative_eq(&self.0.0, &other.0.0, epsilon, max_relative) &&
        f32::relative_eq(&self.0.1, &other.0.1, epsilon, max_relative) &&
        f32::relative_eq(&self.0.2, &other.0.2, epsilon, max_relative) &&
        f32::relative_eq(&self.0.3, &other.0.3, epsilon, max_relative)
    }
}
