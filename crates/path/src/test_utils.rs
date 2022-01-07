use approx::{AbsDiffEq, assert_relative_eq, RelativeEq};
use lyon_geom::euclid::{Point2D, UnknownUnit};

pub fn assert_relative_eq_boxed_pt_slice(left: Box<[Point2D<f32, UnknownUnit>]>, right: Box<[Point2D<f32, UnknownUnit>]>) {
    assert_relative_eq!(
        AssertableBoxedPointSlice(left),
        AssertableBoxedPointSlice(right),
    )
}

#[derive(PartialEq, Debug)]
pub(crate) struct AssertableBoxedPointSlice(pub Box<[Point2D<f32, UnknownUnit>]>);

impl AbsDiffEq for AssertableBoxedPointSlice {
    type Epsilon = f32;

    fn default_epsilon() -> f32 {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: f32) -> bool {
        let me = &self.0;
        let other = &other.0;
        let l = usize::max(me.len(), other.len());
        for i in 0..l {
            let me = me.get(i);
            let other = other.get(i);
            if me.is_none() || other.is_none() {
                return false;
            }
            let me = me.unwrap();
            let other = other.unwrap();
            if !f32::abs_diff_eq(&me.x, &other.x, epsilon) {
                return false;
            }
            if !f32::abs_diff_eq(&me.y, &other.y, epsilon) {
                return false;
            }
        }
        true
    }
}

impl RelativeEq for AssertableBoxedPointSlice {
    fn default_max_relative() -> f32 {
        f32::default_max_relative()
    }

    fn relative_eq(&self, other: &Self, epsilon: f32, max_relative: f32) -> bool {
        let me = &self.0;
        let other = &other.0;
        let l = usize::max(me.len(), other.len());
        for i in 0..l {
            let me = me.get(i);
            let other = other.get(i);
            if me.is_none() || other.is_none() {
                return false;
            }
            let me = me.unwrap();
            let other = other.unwrap();
            if !f32::relative_eq(&me.x, &other.x, epsilon, max_relative) {
                return false;
            }
            if !f32::relative_eq(&me.y, &other.y, epsilon, max_relative) {
                return false;
            }
        }
        true
    }
}
