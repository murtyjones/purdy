use anyhow::Result;

use crate::color::{ColorSpace, Color, ColorError};
use crate::{Rgb, Cmyk, Gray};

#[derive(Debug, Clone)]
pub struct StrokeColor {
    pub color_space: ColorSpace,
    pub rgb: Rgb,
    pub cmyk: Cmyk,
    pub gray: Gray,
}

impl Default for StrokeColor {
    fn default() -> Self {
        StrokeColor { 
            color_space: ColorSpace::DeviceGray,
            // TODO: Confirm these values with the PDF spec
            rgb: Rgb::new(0.0, 0.0, 0.0),
            gray: Gray::new(0.0),
            cmyk: Cmyk::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl Color for StrokeColor {
    fn set_color_space(&mut self, c: ColorSpace) {
        self.color_space = c;
    }

    fn set_color(&mut self, c: Vec<f32>) -> Result<()> {
        match c.len() {
            0 => Err(ColorError::TooFewParams.into()),
            1 => {
                // see how PDF spec handles it
                unimplemented!()
            },
            3 => {
                // see how PDF spec handles it
                unimplemented!()
            },
            4 => {
                // see how PDF spec handles it
                unimplemented!()
            },
            other => Err(ColorError::TooManyParams(other).into()),
        }
    }
}