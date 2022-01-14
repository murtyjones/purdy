mod test_utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    red: f32,
    green: f32,
    blue: f32,
}

impl Default for Rgb {
    fn default() -> Self {
        Rgb {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        }
    }
}

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        let red = Rgb::normalize(Rgb::clamp(r));
        let green = Rgb::normalize(Rgb::clamp(g));
        let blue = Rgb::normalize(Rgb::clamp(b));
        Rgb { red, green, blue }
    }

    pub fn set(&mut self, c: Rgb) {
        *self = c;
    }

    pub fn red(&self) -> f32 {
        self.red
    }

    pub fn blue(&self) -> f32 {
        self.blue
    }
    
    pub fn green(&self) -> f32 {
        self.green
    }

    fn normalize(v: f32) -> f32 {
        if v <= 1.0 {
            return v;
        }
        return v / 255.0;
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
