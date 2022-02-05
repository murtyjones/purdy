use crate::error::ParseError;
use crate::{dictionary::Dictionary, utils::strip_nom, ObjectId};
use anyhow::Result;
use lyon::geom::Vector;
use lyon::path::LineCap;
use shared::{ColorSpace, DashPattern, Height, LineWidth, Rgb, Width};

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
    MoveTo(Vector<f32>),
    LineTo(Vector<f32>),
    Rect(Vector<f32>, Width, Height),
    Stroke(bool),
    Fill,
    LineWidth(LineWidth),
    NonStrokeColor(Vec<f32>),
    StrokeColor(Vec<f32>),
    StrokeColorSpace(ColorSpace),
    NonStrokeColorSpace(ColorSpace),
    DashPattern(DashPattern),
}

impl<'a> Stream<'a> {
    pub fn get_content(&self) -> Result<Vec<StreamObject<'a>>> {
        let (rest, content) = stream_objects(self.content)?;
        if !rest.is_empty() {
            return Err(ParseError::FailedToParseAllStreamContent(String::from_utf8_lossy(rest).to_string()).into());
        }
        Ok(content)
    }
}


#[cfg(test)]
mod test {
    use crate::{document::Document, utils::{extend_lifetime, read_file_bytes}};

    #[test]
    fn test_sample_pdf_no_xref_objects() {
        let bytes = read_file_bytes(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/pdfs/sample-no-xref-entries/sample-no-xref-entries.pdf"
        ));
        let bytes: &[u8] = unsafe { extend_lifetime(&bytes) };
        let pdf = Document::from_bytes(bytes).expect("could not parse sample");
        assert!(pdf.get_object((5, 0)).unwrap().as_stream().unwrap().get_content().is_ok());
        assert!(pdf.get_object((11, 0)).unwrap().as_stream().unwrap().get_content().is_ok());
    }
}