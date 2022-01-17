mod test_utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cmyk(f32, f32, f32, f32);

impl Default for Cmyk {
    fn default() -> Self {
        Cmyk(0.0, 0.0, 0.0, 0.0)
    }
}

impl Cmyk {
    pub fn new(c: f32, m: f32, y: f32, k: f32) -> Self {
        let c = Cmyk::clamp(c);
        let m = Cmyk::clamp(m);
        let y = Cmyk::clamp(y);
        let k = Cmyk::clamp(k);
        Cmyk(c, m, y, k)
    }

    pub fn set(&mut self, c: Cmyk) {
        *self = c;
    }

    // TODO: What range is CMYK? Should RGB and CMYK be the same scale IE 0 to 1?
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
    use super::test_utils::assert_relative_eq_cmyk;
    use super::Cmyk;

    #[test]
    fn min_max() {
        let rgb = Cmyk::new(256.0, -1.0, 130.0, 0.0);
        assert_relative_eq_cmyk(rgb, Cmyk::new(255.0, 0.0, 130.0, 0.0))
    }
}
