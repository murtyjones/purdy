use lyon_geom::Point;
use lyon_path::LineCap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_until,
    character::complete::char,
    character::streaming::multispace0,
    combinator::{map, map_opt, map_res, opt},
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, terminated, tuple},
};

use crate::{
    error::ParseError as PdfParseError,
    rgb::Rgb,
    utils::{_name, _real, int1, take_until_unmatched, ws},
    ParseResult,
};

use super::{StreamObject, TextContent};

fn rg(input: &[u8]) -> ParseResult<Rgb> {
    map(
        terminated(
            tuple((ws(int1::<u8>), ws(int1::<u8>), ws(int1::<u8>))),
            ws(tag("rg")),
        ),
        |(r, g, b)| Rgb { r, g, b },
    )(input)
}

fn font_family_and_size(input: &[u8]) -> ParseResult<(Vec<u8>, u32)> {
    terminated(tuple((ws(_name), ws(int1::<u32>))), tag("Tf"))(input)
}

fn location(input: &[u8]) -> ParseResult<(f32, f32)> {
    terminated(tuple((ws(_real::<f32>), ws(_real::<f32>))), tag("Td"))(input)
}

fn text_content(input: &[u8]) -> ParseResult<&[u8]> {
    terminated(
        delimited(
            tuple((multispace0, char('('))),
            take_until_unmatched(b"(", b")"),
            tuple((char(')'), multispace0)),
        ),
        tag("Tj"),
    )(input)
}

fn text(input: &[u8]) -> ParseResult<TextContent<'_>> {
    map(
        delimited(
            ws(tag("BT")),
            tuple((opt(rg), font_family_and_size, location, text_content)),
            ws(tag("ET")),
        ),
        |(rgb, (font_family, font_size), l_r, contents)| TextContent {
            font_family,
            rgb,
            font_size,
            l_r,
            contents,
        },
    )(input)
}

fn cap_style(input: &[u8]) -> ParseResult<LineCap> {
    map_res(terminated(ws(int1::<u8>), ws(char('J'))), |e| match e {
        0 => Ok(LineCap::Butt),
        1 => Ok(LineCap::Round),
        2 => Ok(LineCap::Square),
        // TODO: Shouldn't need e.into()...
        _ => {
            let e: PdfParseError = PdfParseError::InvalidCapStyle(e.into()).into();
            Err(e)
        }
    })(input)
}

fn move_to(input: &[u8]) -> ParseResult<Point<f32>> {
    map(
        terminated(tuple((ws(int1::<u32>), ws(int1::<u32>))), ws(char('m'))),
        // TODO: any issue with number overflow?
        |(x, y)| Point::from((x as f32, y as f32)),
    )(input)
}

fn line_to(input: &[u8]) -> ParseResult<Point<f32>> {
    map(
        terminated(tuple((ws(int1::<u32>), ws(int1::<u32>))), ws(char('l'))),
        // TODO: any issue with number overflow?
        |(x, y)| Point::from((x as f32, y as f32)),
    )(input)
}

fn stroke(input: &[u8]) -> ParseResult<()> {
    map(ws(char('S')), |_| ())(input)
}

pub fn stream_objects(input: &[u8]) -> ParseResult<Vec<StreamObject<'_>>> {
    many0(alt((
        map(text, StreamObject::Text),
        map(cap_style, StreamObject::CapStyle),
        map(move_to, StreamObject::MoveTo),
        map(line_to, StreamObject::LineTo),
        map(stroke, |_| StreamObject::Stroke),
    )))(input)
}

#[cfg(test)]
mod test {
    use lyon_path::LineCap;

    use super::stream_objects;
    use crate::stream::{Rgb, StreamObject, TextContent};

    #[test]
    fn test_stream() {
        let input = b"2 J
BT
0 0 0 rg
/F1 0027 Tf
57.3750 722.2800 Td
( Simple PDF File 2 ) Tj
ET
BT
/F1 0010 Tf
69.2500 688.6080 Td
( ...continued from page 1. Yet more text. And more text. And more text. ) Tj
ET";
        assert_eq!(
            stream_objects(input).unwrap().1,
            vec![
                StreamObject::CapStyle(LineCap::Square),
                StreamObject::Text(TextContent {
                    font_family: b"F1".to_vec(),
                    rgb: Some(Rgb { r: 0, g: 0, b: 0 }),
                    font_size: 27,
                    l_r: (57.375, 722.28),
                    contents: b" Simple PDF File 2 ",
                },),
                StreamObject::Text(TextContent {
                    font_family: b"F1".to_vec(),
                    rgb: None,
                    font_size: 10,
                    l_r: (69.25, 688.608),
                    contents:
                        b" ...continued from page 1. Yet more text. And more text. And more text. "
                })
            ]
        )
    }
}
