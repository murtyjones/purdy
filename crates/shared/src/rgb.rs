mod test_utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb(f32, f32, f32);

impl Default for Rgb {
    fn default() -> Self {
        Rgb(0.0, 0.0, 0.0)
    }
}

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        let r = Rgb::clamp(r);
        let g = Rgb::clamp(g);
        let b = Rgb::clamp(b);
        Rgb(r,g,b)
    }

    pub fn set(&mut self, c: Rgb) {
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
    use super::test_utils::assert_relative_eq_rgb;
    use super::Rgb;

    #[test]
    fn min_max() {
        let rgb = Rgb::new(256.0, -1.0, 130.0);
        assert_relative_eq_rgb(rgb, Rgb::new(255.0, 0.0, 130.0))
    }
}
