//! 1. Only put small concepts here. Nothing major
//! 2. This crate *must* have no dependencies on other local crates in the project

mod dimensions;
mod error;
mod line_width;
mod rgb;
mod cmyk;
mod gray;
mod color;
mod stroke_color;
mod non_stroke_color;

pub use dimensions::{Width, Height};
pub use error::NumberError;
pub use line_width::LineWidth;
pub use rgb::Rgb;
pub use cmyk::Cmyk;
pub use gray::Gray;
pub use stroke_color::StrokeColor;
pub use non_stroke_color::NonStrokeColor;
pub use color::{Color, ColorError, ColorSpace, ColorSpaceWithColor};
