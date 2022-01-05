use lyon_geom::Point;
use lyon_path::LineCap;
use anyhow::Result;
use crate::{
    dictionary::Dictionary,
    rgb::Rgb,
    utils::strip_nom,
};

use self::parsers::stream_objects;

mod parsers;

/// Stream object
/// Warning - all streams must be indirect objects, while
/// the stream dictionary may be a direct object
#[derive(PartialEq, Debug, Clone)]
pub struct Stream<'a> {
    /// Associated stream dictionary
    pub dict: Dictionary<'a>,
    /// Contents of the stream in bytes
    pub content: &'static [u8],
    /// Can the stream be compressed by the `Document::compress()` function?
    /// Font streams may not be compressed, for example
    pub allows_compression: bool,
    /// Stream data's position in PDF file.
    pub start_position: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub struct TextContent<'a> {
    pub font_family: Vec<u8>,
    pub rgb: Option<Rgb>,
    pub font_size: u32,
    pub l_r: (f32, f32),
    pub contents: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub enum StreamObject<'a> {
    Text(TextContent<'a>),
    CapStyle(LineCap),
    MoveTo(Point<f32>),
    LineTo(Point<f32>),
    Stroke,
}

impl<'a> Stream<'a> {
    pub fn get_content(&self) -> Result<Vec<StreamObject<'a>>> {
        strip_nom(stream_objects(self.content))
    }
}
