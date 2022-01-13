use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum ColorSpace {
    DeviceRGB,
    DeviceGray,
    DeviceCMYK,
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
}