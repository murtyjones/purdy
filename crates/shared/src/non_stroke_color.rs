use anyhow::Result;

use crate::{Cmyk, Color, ColorError, ColorSpace, ColorSpaceWithColor, Gray, Rgb};

#[derive(Debug, Clone, Copy)]
pub struct NonStrokeColor {
    pub color_space: ColorSpace,
    pub rgb: Rgb,
    pub cmyk: Cmyk,
    pub gray: Gray,
}

impl Default for NonStrokeColor {
    fn default() -> Self {
        NonStrokeColor {
            // TODO: Confirm these values with the PDF spec
            color_space: ColorSpace::DeviceRGB,
            rgb: Rgb::new(0.0, 0.0, 0.0),
            gray: Gray::new(0.0),
            cmyk: Cmyk::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl Color for NonStrokeColor {
    fn set_color_space(&mut self, c: ColorSpace) {
        self.color_space = c;
    }

    fn set_color(&mut self, c: Vec<f32>) -> Result<()> {
        match c.len() {
            0 => Err(ColorError::TooFewParams.into()),
            1 => {
                // TODO: see how PDF spec handles it
                match self.color_space {
                    ColorSpace::DeviceCMYK => unimplemented!(),
                    ColorSpace::DeviceGray => unimplemented!(),
                    ColorSpace::DeviceRGB => unimplemented!(),
                }
            }
            3 => {
                // see how PDF spec handles it
                match self.color_space {
                    ColorSpace::DeviceCMYK => unimplemented!(),
                    ColorSpace::DeviceGray => unimplemented!(),
                    ColorSpace::DeviceRGB => {
                        use std::convert::TryInto;
                        let [r, g, b]: [f32; 3] = c.try_into().unwrap();
                        self.rgb = Rgb::new(r, g, b);
                        Ok(())
                    }
                }
            }
            4 => {
                // see how PDF spec handles it
                match self.color_space {
                    ColorSpace::DeviceCMYK => unimplemented!(),
                    ColorSpace::DeviceGray => unimplemented!(),
                    ColorSpace::DeviceRGB => unimplemented!(),
                }
            }
            other => Err(ColorError::TooManyParams(other).into()),
        }
    }

    fn get_current_color(&self) -> ColorSpaceWithColor {
        match self.color_space {
            ColorSpace::DeviceCMYK => ColorSpaceWithColor::DeviceCMYK(self.cmyk),
            ColorSpace::DeviceRGB => ColorSpaceWithColor::DeviceRGB(self.rgb),
            ColorSpace::DeviceGray => ColorSpaceWithColor::DeviceGray(self.gray),
        }
    }
}
