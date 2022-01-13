mod test_utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gray(f32);

impl Default for Gray {
    fn default() -> Self {
        Gray(0.0)
    }
}

impl Gray {
    pub fn new(c: f32) -> Self {
        let c = Gray::clamp(c);
        Gray(c)
    }

    pub fn set(&mut self, c: Gray) {
        *self = c;
    }

    fn clamp(v: f32) -> f32 {
        if v > 255.0 {
            255.0
        } else if v < 0.0 {
            0.0
        } else {
            v
        }
    }
}

#[cfg(test)]
mod test {
    use super::test_utils::assert_relative_eq_gray;
    use super::Gray;

    #[test]
    fn min_max() {
        let rgb = Gray::new(256.0);
        assert_relative_eq_gray(rgb, Gray::new(255.0))
    }
}
