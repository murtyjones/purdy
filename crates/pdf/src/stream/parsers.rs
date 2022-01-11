use lyon_geom::{vector, Vector};
use lyon_path::LineCap;
use shared::{NumberError, Height, Width};
use anyhow::Error;
use std::str::FromStr;
use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_until,
    character::complete::{char, one_of, digit0, digit1},
    character::streaming::multispace0,
    combinator::{map, map_opt, map_res, opt},
    error::{ErrorKind, ParseError},
    multi::{many0, many1},
    sequence::{delimited, pair, terminated, tuple},
};
use num::ToPrimitive;

use crate::{
    error::ParseError as PdfParseError,
    rgb::Rgb,
    utils::{_name, _real, int1, take_until_unmatched, ws},
    NomResult, NomError,
};

use super::{StreamObject, TextContent};

#[inline]
fn convert_result<O, E>(result: Result<O, E>, input: &[u8], error_kind: ErrorKind) -> NomResult<O> {
    result
        .map(|o| (input, o))
        .map_err(|_| nom::Err::Error(NomError::from_error_kind(input, error_kind)))
}

fn rg(input: &[u8]) -> NomResult<Rgb> {
    map(
        terminated(
            tuple((ws(int1::<u8>), ws(int1::<u8>), ws(int1::<u8>))),
            ws(tag("rg")),
        ),
        |(r, g, b)| Rgb { r, g, b },
    )(input)
}

fn font_family_and_size(input: &[u8]) -> NomResult<(Vec<u8>, u32)> {
    terminated(tuple((ws(_name), ws(int1::<u32>))), tag("Tf"))(input)
}

fn location(input: &[u8]) -> NomResult<(f32, f32)> {
    terminated(tuple((ws(_real::<f32>), ws(_real::<f32>))), tag("Td"))(input)
}

fn text_content(input: &[u8]) -> NomResult<&[u8]> {
    terminated(
        delimited(
            tuple((multispace0, char('('))),
            take_until_unmatched(b"(", b")"),
            tuple((char(')'), multispace0)),
        ),
        tag("Tj"),
    )(input)
}

fn text(input: &[u8]) -> NomResult<TextContent<'_>> {
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

fn cap_style(input: &[u8]) -> NomResult<LineCap> {
    map_res(terminated(ws(int1::<u8>), ws(char('J'))), |cap| match cap {
        0 => Ok(LineCap::Butt),
        1 => Ok(LineCap::Round),
        2 => Ok(LineCap::Square),
        // TODO: Shouldn't need e.into()...
        _ => {
            let e: PdfParseError = PdfParseError::InvalidCapStyle(cap.into()).into();
            Err(e)
        }
    })(input)
}

fn move_to(input: &[u8]) -> NomResult<Vector<f32>> {
    map(
        terminated(tuple((ws(number_forced_to_f32), ws(number_forced_to_f32))), ws(char('m'))),
        |(x, y)| vector(x, y),
    )(input)
}

#[test]
fn test_move_to() {
    assert_eq!(vector(1.23, 1.23), move_to(&"1.23 1.23 m".as_bytes()).unwrap().1);
    assert_eq!(vector(1.00, 1.23), move_to(&"1 +1.23 m".as_bytes()).unwrap().1);
    assert_eq!(vector(1.00, -1.23), move_to(&"1 -1.23 m".as_bytes()).unwrap().1);
    assert_eq!(vector(-1.23, -1.24), move_to(&"+-+1.23 --1.24 m".as_bytes()).unwrap().1);
    assert_eq!(vector(-10.0, 1.24), move_to(&"+-+10 ++1.24 m".as_bytes()).unwrap().1);
    assert_eq!(vector(-10.0, -1.0), move_to(&"-----10 +-+1 m".as_bytes()).unwrap().1);
}

fn number_forced_to_f32(input: &[u8]) -> NomResult<f32> {
    alt((
        real,
        map_res::<_, _, _, _, Error, _, _>(integer, |num| {
            num.to_f32().ok_or::<Error>(NumberError::InvalidNumberConversion.into())
        })
    ))(input)
}

fn integer(input: &[u8]) -> NomResult<i64> {
    let (rest, (pluses_minuses, _)) = pair(opt(many1(one_of("+-"))), digit1)(input)?;
    let number_of_pluses_minuses = pluses_minuses.as_ref().unwrap_or(&vec![]).len();
    let contains_minus = pluses_minuses.as_ref().unwrap_or(&vec![]).contains(&'-');

    let unsigned_int = &input[number_of_pluses_minuses..input.len() - rest.len()];
    let plus_minus = &[(if contains_minus { b'-' } else { b'+' })];
    let final_number: Vec<u8> = [plus_minus, unsigned_int].concat();
    convert_result(i64::from_str(std::str::from_utf8(&final_number).unwrap()), rest, ErrorKind::Digit)
}

fn real(input: &[u8]) -> NomResult<f32> {
    let (rest, (pluses_minuses, _)) = pair(
        opt(many1(one_of("+-"))),
        alt((
            map(tuple((digit1, tag(b"."), digit0)), |_| ()),
            map(pair(tag(b"."), digit1), |_| ()),
        )),
    )(input)?;
    let number_of_pluses_minuses = pluses_minuses.as_ref().unwrap_or(&vec![]).len();
    let contains_minus = pluses_minuses.as_ref().unwrap_or(&vec![]).contains(&'-');
    
    let unsigned_float = &input[number_of_pluses_minuses..input.len() - rest.len()];
    let plus_minus = &[(if contains_minus { b'-' } else { b'+' })];
    let final_number: Vec<u8> = [plus_minus, unsigned_float].concat();
    convert_result(f32::from_str(std::str::from_utf8(&final_number).unwrap()), rest, ErrorKind::Digit)
}

fn line_to(input: &[u8]) -> NomResult<Vector<f32>> {
    map(
        terminated(tuple((ws(number_forced_to_f32), ws(number_forced_to_f32))), ws(char('l'))),
        |(x, y)| vector(x, y),
    )(input)
}

#[test]
fn test_line_to() {
    assert_eq!(vector(1.23, 1.23), line_to(&"1.23 1.23 l".as_bytes()).unwrap().1);
    assert_eq!(vector(1.00, 1.23), line_to(&"1 +1.23 l".as_bytes()).unwrap().1);
    assert_eq!(vector(1.00, -1.23), line_to(&"1 -1.23 l".as_bytes()).unwrap().1);
    assert_eq!(vector(-1.23, -1.24), line_to(&"+-+1.23 --1.24 l".as_bytes()).unwrap().1);
    assert_eq!(vector(-10.0, 1.24), line_to(&"+-+10 ++1.24 l".as_bytes()).unwrap().1);
    assert_eq!(vector(-10.0, -1.0), line_to(&"-----10 +-+1 l".as_bytes()).unwrap().1);
}

fn rect(input: &[u8]) -> NomResult<(Vector<f32>, Width, Height)> {
    map(
        terminated(tuple((
            ws(number_forced_to_f32), 
            ws(number_forced_to_f32),
            ws(number_forced_to_f32), 
            ws(number_forced_to_f32)
        )), ws(tag("re"))),
        |(x, y, w, h)| {
            (vector(x, y), Width::new(w), Height::new(h))
        },
    )(input)
}

#[test]
fn test_rect() {
    assert_eq!((vector(100.0, 101.0), 102.0, 0.0), rect(&"100 101 102 0 re".as_bytes()).unwrap().1);
}

fn stroke(input: &[u8]) -> NomResult<()> {
    map(ws(char('S')), |_| ())(input)
}

fn fill(input: &[u8]) -> NomResult<()> {
    map(ws(char('f')), |_| ())(input)
}

pub fn stream_objects(input: &[u8]) -> NomResult<Vec<StreamObject<'_>>> {
    many0(alt((
        map(text, StreamObject::Text),
        map(cap_style, StreamObject::CapStyle),
        map(move_to, StreamObject::MoveTo),
        map(line_to, StreamObject::LineTo),
        map(rect, |(low_left, width, height)| StreamObject::Rect(low_left, width, height)),
        map(stroke, |_| StreamObject::Stroke),
        map(fill, |_| StreamObject::Fill),
    )))(input)
}

#[cfg(test)]
mod test {
    use super::stream_objects;
    use crate::stream::{Rgb, StreamObject, TextContent};
    use lyon_geom::vector;
    use lyon_path::LineCap;

    #[test]
    fn test_text_stream() {
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

    #[test]
    fn test_drawing_stream() {
        let input = b"500 500 m
600 600 l
f";

        assert_eq!(
            stream_objects(input).unwrap().1,
            vec![
                StreamObject::MoveTo(vector(500.0, 500.0)),
                StreamObject::LineTo(vector(600.0, 600.0)),
                StreamObject::Fill,
            ]
        )
    }
}
