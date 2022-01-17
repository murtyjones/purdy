//! 1. Only put small concepts here. Nothing major
//! 2. This crate *must* have no dependencies on other local crates in the project

mod cmyk;
mod color;
mod dimensions;
mod error;
mod gray;
mod line_width;
mod non_stroke_color;
mod rgb;
mod stroke_color;

pub use cmyk::Cmyk;
pub use color::{Color, ColorError, ColorSpace, ColorSpaceWithColor};
pub use dimensions::{Height, Width};
pub use error::NumberError;
pub use gray::Gray;
pub use line_width::LineWidth;
pub use non_stroke_color::NonStrokeColor;
pub use rgb::Rgb;
pub use stroke_color::StrokeColor;
