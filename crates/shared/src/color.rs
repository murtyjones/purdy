use anyhow::{Error, Result};
use std::str::FromStr;
use thiserror::Error;

use crate::{Cmyk, Gray, Rgb};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    DeviceRGB,
    DeviceGray,
    DeviceCMYK,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpaceWithColor {
    DeviceRGB(Rgb),
    DeviceGray(Gray),
    DeviceCMYK(Cmyk),
}

#[derive(Error, Debug)]
pub enum ColorSpaceError {
    #[error("Unrecognized color space: {0}")]
    UnrecognizedColorSpace(String),
}

impl FromStr for ColorSpace {
    type Err = Error;

    fn from_str(input: &str) -> Result<ColorSpace, Self::Err> {
        match input {
            "DeviceRGB" => Ok(ColorSpace::DeviceRGB),
            "DeviceGray" => Ok(ColorSpace::DeviceGray),
            "DeviceCMYK" => Ok(ColorSpace::DeviceCMYK),
            s => Err(ColorSpaceError::UnrecognizedColorSpace(s.to_owned()).into()),
        }
    }
}

#[derive(Error, Debug)]
pub enum ColorError {
    #[error("Received {0} params for color. Max is four")]
    TooManyParams(usize),
    #[error("Received 0 params for color. At least one is required.")]
    TooFewParams,
}

pub trait Color {
    fn set_color_space(&mut self, c: ColorSpace);

    fn set_color(&mut self, c: Vec<f32>) -> Result<()>;

    fn get_current_color(&self) -> ColorSpaceWithColor;
}
