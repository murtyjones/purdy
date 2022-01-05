use crate::error::{HandlingError, XrefError};
use crate::{
    dictionary::Dictionary,
    object::{Name, Object, StringFormat},
    stream::{Stream, TextContent},
    utils::{_name, _real, hex_char2, int1, take_until_unmatched, ws},
    xref::{Xref, XrefEntry},
    ObjectId, ObjectNumber, ParseResult,
};
use anyhow::Result;
use std::{collections::BTreeMap, str::from_utf8};
use std::{io::BufRead, str::FromStr};

use linked_hash_map::{LinkedHashMap, OccupiedEntry};
use nom::number::complete::le_u64;
use nom::{
    bitvec::vec,
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case, take, take_till, take_until, take_while},
    character::{
        complete::{alphanumeric1, anychar, char, digit0, digit1, newline, one_of},
        is_digit, is_hex_digit, is_oct_digit,
    },
    combinator::{map, map_opt, map_res, opt, verify},
    error::ParseError,
    error::{context, ErrorKind, VerboseError},
    multi::{count as nom_count, fold_many1, many0, many1},
    number::complete::{be_u16, be_u32, be_u64, double, f64, float, le_u16},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Err as NomErr, IResult,
};

use super::Document;

pub fn version(input: &[u8]) -> ParseResult<f64> {
    context(
        "PDF Version",
        tuple((tag("%"), tag("PDF"), tag("-"), double)),
    )(input)
    .map(|(rest, result)| (rest, result.3))
}

#[inline]
fn is_whitespace(c: u8) -> bool {
    b" \t\n\r\0\x0C".contains(&c)
}

#[inline]
fn is_delimiter(c: u8) -> bool {
    b"()<>[]{}/%".contains(&c)
}

#[inline]
fn is_regular(c: u8) -> bool {
    !is_whitespace(c) && !is_delimiter(c)
}

#[inline]
fn is_eol(input: &[u8]) -> bool {
    eol(input).map_or(false, |_| true)
}

#[inline]
fn minus(input: u8) -> bool {
    b"-".contains(&input)
}

fn eol(input: &[u8]) -> ParseResult<&[u8]> {
    alt((tag(b"\r\n"), tag(b"\n"), tag("\r")))(input)
}

#[inline]
fn to_nom<O, E>(
    result: std::result::Result<O, E>,
    input: &[u8],
    error_kind: ErrorKind,
) -> ParseResult<O> {
    result
        .map(|o| (input, o))
        .map_err(|_| nom::Err::Error(nom::error::ParseError::from_error_kind(input, error_kind)))
}

pub fn integer(input: &[u8]) -> ParseResult<Object> {
    map(_integer, Object::Integer)(input)
}

pub fn _integer(input: &[u8]) -> ParseResult<i64> {
    ws(int1::<i64>)(input)
}

fn real(input: &[u8]) -> ParseResult<Object> {
    map(ws(_real::<f64>), Object::Real)(input)
}

fn maybe_parse_object(
    (input, (id, entry)): (&'static [u8], (&u32, &XrefEntry)),
) -> Option<(ObjectId, Object<'static>)> {
    match entry {
        XrefEntry::InUse { offset, .. } => object(&input[*offset..]).map(|(_, obj)| obj).ok(),
        XrefEntry::Free => None,
        XrefEntry::Compressed { .. } => unimplemented!(),
    }
}

pub fn all_objects(
    input: &'static [u8],
    mut document: Document<'static>,
) -> Result<Document<'static>> {
    let objects = document
        .xref
        .entries
        .iter()
        .map(|entry| (input, entry))
        .filter_map(maybe_parse_object)
        .collect::<Vec<(ObjectId, Object<'static>)>>();

    let mut tree = BTreeMap::new();

    tree.extend(objects);

    document.objects = tree;

    Ok(document)
}

pub fn make_xref_table(input: &'static [u8]) -> Result<(Xref, Dictionary)> {
    make_xref_table_from_end_of_file(input).or_else(|_| make_xref_table_manually(input))
}

pub fn make_xref_table_from_end_of_file(input: &'static [u8]) -> Result<(Xref, Dictionary)> {
    // The final `startxref` in the file should appear in the final 1024 bytes, by
    // convention. Include extra 1 because we need this position to be an index and
    // it's starting as a length
    let starting_search_pos_startxref = input.len().checked_sub(1024).unwrap_or(0);
    let final_xref_offset = final_xref_offset(&input[starting_search_pos_startxref..])?.1;
    let (rest, xref_table) = xref(&input[final_xref_offset..])?;
    let (_, trailer) = trailer(rest)?;
    assert_all_xref_entry_offsets_are_accurate(input, &xref_table)?;
    Ok((xref_table, trailer))
}

pub fn make_xref_table_manually(input: &'static [u8]) -> Result<(Xref, Dictionary)> {
    let mut xref_table = Xref::new();
    find_all_object_ids(input)?
        .into_iter()
        .for_each(|((id, generation), offset)| {
            xref_table.insert(id, XrefEntry::InUse { offset, generation });
        });

    // The final `trailer` in the file should appear in the final 1024 bytes, by
    // convention. Include extra 1 because we need this position to be an index and
    // it's starting as a length
    let starting_search_pos_startxref = input.len().checked_sub(1024).unwrap_or(0);
    let (input_from_final_trailer_onwards, _) =
        input_from_final_trailer_onwards(&input[starting_search_pos_startxref..])?;
    let (_, trailer) = _dictionary(input_from_final_trailer_onwards)?;
    Ok((xref_table, trailer))
}

fn find_all_object_ids(input: &'static [u8]) -> Result<Vec<(ObjectId, usize)>> {
    let starting_len = input.len();
    let mut objects = Vec::new();
    let mut needles = vec![input];

    while needles.len() > 0 {
        let input = needles.pop().unwrap();
        if input.len() == 0 {
            continue;
        }
        if let Ok((rest, (id, gen))) = object_beginning(input) {
            objects.push(((id, gen), starting_len - input.len()));
            needles.push(rest);
        } else {
            let rest = take_till_digit(input)?.0;
            let rest = if rest == input { &rest[1..] } else { rest };
            needles.push(rest);
        }
    }

    Ok(objects)
}

fn take_till_digit(input: &'static [u8]) -> ParseResult<&[u8]> {
    take_till(is_digit)(input)
}

fn assert_all_xref_entry_offsets_are_accurate<'a, 'b>(
    input: &'a [u8],
    xref: &'b Xref,
) -> Result<()> {
    for (id, xref_entry) in xref.entries.iter() {
        match xref_entry {
            &XrefEntry::Free => continue,
            &XrefEntry::Compressed { .. } => unimplemented!(),
            &XrefEntry::InUse { offset, generation } => match object_beginning(&input[offset..]) {
                Ok((_, (found_id, found_gen))) => {
                    if found_id != *id || found_gen != generation {
                        let expected = (*id, generation);
                        let found = (found_id, found_gen);
                        return Err(
                            XrefError::FoundWrongObjectForExpectedEntry((expected, found)).into(),
                        );
                    }
                    continue;
                }
                Err(_) => {
                    return Err(XrefError::XrefTableInvalidEntry((*id, generation)).into());
                }
            },
        }
    }
    Ok(())
}

fn object_beginning(input: &[u8]) -> ParseResult<ObjectId> {
    terminated(pair(ws(int1::<u32>), ws(int1::<u16>)), ws(tag(b"obj")))(input)
}

fn final_xref_offset(input: &[u8]) -> ParseResult<usize> {
    let go_to_startxref = take_until("startxref");
    let startxref = ws(tag("startxref"));
    // Depending how early in the file we are, there may be multiple `startxref`
    // entries. We will only take the last one
    let (input, startxrefs) = many1(tuple((go_to_startxref, startxref, int1::<usize>)))(input)?;
    let final_startxref = startxrefs.last().unwrap().2;
    Ok((input, final_startxref))
}

fn input_from_final_trailer_onwards(input: &[u8]) -> ParseResult<()> {
    // Depending how early in the file we are, there may be multiple `trailer`
    // entries. We will only take the last one
    map(
        many1(tuple((take_until("trailer"), tag("trailer")))),
        |_| (),
    )(input)
}

fn xref(input: &[u8]) -> ParseResult<Xref> {
    let mut xref = Xref::new();
    let (input, _) = ws(tag("xref"))(input)?;
    let (input, starting_object_number) = ws(int1::<u32>)(input)?;
    let (input, number_of_objects) = ws(int1::<u32>)(input)?;
    let (input, references) =
        many1(tuple((ws(int1::<usize>), ws(int1::<u16>), one_of("fn"))))(input)?;
    let entries = references
        .iter()
        .enumerate()
        .map(|(i, (offset, generation, status))| {
            let id = starting_object_number + (i as u32);
            (
                id,
                match status {
                    'f' => XrefEntry::Free,
                    'n' => XrefEntry::InUse {
                        offset: *offset,
                        generation: *generation,
                    },
                    _ => unreachable!(),
                },
            )
        })
        .collect::<Vec<(ObjectNumber, XrefEntry)>>();
    xref.extend(entries);
    Ok((input, xref))
}

fn trailer(input: &'static [u8]) -> ParseResult<Dictionary<'_>> {
    let (input, _) = ws(tag("trailer"))(input)?;
    _dictionary(input)
}

fn object(input: &'static [u8]) -> ParseResult<(ObjectId, Object<'_>)> {
    map(
        tuple((object_id, ws(tag("obj")), object_body)),
        |(id, _, object)| (id, object),
    )(input)
}

fn object_body(input: &'static [u8]) -> ParseResult<Object<'_>> {
    alt((
        reference, null, real, integer, name, boolean, stream, dictionary, array, string,
    ))(input)
}

fn array(input: &'static [u8]) -> ParseResult<Object<'_>> {
    map(_array, Object::Array)(input)
}

fn _array(input: &'static [u8]) -> ParseResult<Vec<Object<'_>>> {
    delimited(ws(char('[')), many0(object_body), ws(char(']')))(input)
}

fn string(input: &[u8]) -> ParseResult<Object<'_>> {
    map(_string, |(a, b)| Object::String(a, b))(input)
}

fn _string(input: &[u8]) -> ParseResult<(Vec<u8>, StringFormat)> {
    alt((
        map(ws(string_literal), |s| (s, StringFormat::Literal)),
        map(ws(string_hex), |s| (s, StringFormat::Hexadecimal)),
    ))(input)
}

fn string_literal(input: &[u8]) -> ParseResult<Vec<u8>> {
    let (rest, raw) = delimited(
        ws(char('(')),
        take_until_unmatched(b"(", b")"),
        ws(char(')')),
    )(input)?;
    let (rest_inner, result) = map(
        many0(alt((
            preceded(char('\\'), char('(')),
            preceded(char('\\'), char(')')),
            preceded(char('\\'), char('\\')),
            // The reverse solidus at the end of a line indicates that the
            // string continues on the next line
            preceded(tag("\\n"), anychar),
            preceded(tag("\\r\\n"), anychar),
            preceded(tag("\\r"), anychar),
            preceded(char('\\'), octal_char3),
            preceded(char('\\'), octal_char2),
            preceded(char('\\'), octal_char1),
            anychar,
        ))),
        |c| c.into_iter().map(|e| e as u8).collect(),
    )(raw)?;
    if cfg!(debug_assertions) {
        // There should be nothing leftover inside of the << >> brackets
        assert!(
            rest_inner.len() == 0,
            "Leftover stuff in string literal, {:?}",
            rest_inner
        );
    }
    Ok((rest, result))
}

fn string_hex(input: &[u8]) -> ParseResult<Vec<u8>> {
    let (rest, raw) = delimited(
        ws(char('<')),
        take_until_unmatched(b"<", b">"),
        ws(char('>')),
    )(input)?;
    // The hex string may have an odd number of characters, handled via hex_char1 at
    // the end of the string
    let (rest_inner, result) = map(many0(alt((hex_char2, hex_char1))), |c| {
        c.into_iter().map(|e| e as u8).collect()
    })(raw)?;
    if cfg!(debug_assertions) {
        // There should be nothing leftover inside of the << >> brackets
        assert!(
            rest_inner.len() == 0,
            "Leftover stuff in string literal, {:?}",
            rest_inner
        );
    }
    Ok((rest, result))
}

fn stream(input: &'static [u8]) -> ParseResult<Object<'_>> {
    map(_stream, Object::Stream)(input)
}

fn _stream(input: &'static [u8]) -> ParseResult<Stream<'_>> {
    map(tuple((_dictionary, stream_body)), |(dict, content)| {
        Stream {
            dict,
            content,
            // TODO:
            allows_compression: false,
            start_position: None,
        }
    })(input)
}

fn stream_body(input: &[u8]) -> ParseResult<&[u8]> {
    delimited(
        ws(tag("stream")),
        take_until("endstream"),
        ws(tag("endstream")),
    )(input)
}

fn dictionary(input: &'static [u8]) -> ParseResult<Object<'_>> {
    map(_dictionary, Object::Dictionary)(input)
}

fn _dictionary(input: &'static [u8]) -> ParseResult<Dictionary<'_>> {
    let (rest_outer, raw_dict) = delimited(
        ws(tag("<<")),
        take_until_unmatched(b"<<", b">>"),
        ws(tag(">>")),
    )(input)?;
    let (rest_inner, items) = many0(tuple((_name, object_body)))(raw_dict)?;
    if cfg!(debug_assertions) {
        // There should be nothing leftover inside of the << >> brackets
        assert!(
            rest_inner.len() == 0,
            "Leftover stuff in object, {:?}",
            rest_inner
        );
    }
    let mut dictionary = Dictionary::new();
    items.into_iter().for_each(|(name, object)| {
        dictionary.insert(name, object);
    });
    Ok((rest_outer, dictionary))
}

fn hex_char1(input: &[u8]) -> ParseResult<u8> {
    map_res(
        verify(take(1usize), |h: &[u8]| h.iter().cloned().all(is_hex_digit)),
        // According to the PDF spec, "If the final digit of a hexidecimal string is missing–that
        // is, is there is an odd number of digits–the final digit shall be assumed to be zero" and
        // 0 in ASCII is 48 in decimal
        |x: &[u8]| u8::from_str_radix(from_utf8(&vec![x[0], 48]).unwrap(), 16),
    )(input)
}

fn octal_char3(input: &[u8]) -> ParseResult<char> {
    map(
        verify(take(3usize), |h: &[u8]| h.iter().cloned().all(is_oct_digit)),
        |x| u8::from_str_radix(from_utf8(x).unwrap(), 8).unwrap() as char,
    )(input)
}

fn octal_char2(input: &[u8]) -> ParseResult<char> {
    map(
        verify(take(2usize), |h: &[u8]| h.iter().cloned().all(is_oct_digit)),
        |x| u8::from_str_radix(from_utf8(x).unwrap(), 8).unwrap() as char,
    )(input)
}

fn octal_char1(input: &[u8]) -> ParseResult<char> {
    map(
        verify(take(1usize), |h: &[u8]| h.iter().cloned().all(is_oct_digit)),
        |x| u8::from_str_radix(from_utf8(x).unwrap(), 8).unwrap() as char,
    )(input)
}

fn name(input: &[u8]) -> ParseResult<Object<'_>> {
    map(_name, Object::Name)(input)
}

fn object_id(input: &[u8]) -> ParseResult<ObjectId> {
    pair(ws(int1), ws(int1))(input)
}

fn reference(input: &[u8]) -> ParseResult<Object> {
    map(terminated(object_id, ws(tag(b"R"))), Object::Reference)(input)
}

fn null(input: &[u8]) -> ParseResult<Object> {
    let parser = ws(tag_no_case(b"null"));
    map(parser, |_| Object::Null)(input)
}

fn boolean(input: &[u8]) -> ParseResult<Object> {
    alt((
        map(ws(tag_no_case(b"true")), |_| Object::Boolean(true)),
        map(ws(tag_no_case(b"false")), |_| Object::Boolean(false)),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::utils::strip_nom;
    use crate::{dictionary_struct, int, real, string_hex, string_lit};

    use super::*;

    fn strip<T>(result: ParseResult<'static, T>) -> T {
        strip_nom(result).unwrap()
    }

    #[test]
    fn test_version() {
        assert_eq!(
            version(b"%PDF-1.3").expect("could not parse version").1,
            1.3
        )
    }

    macro_rules! assert_name_eq {
        ($input:expr, $expected:expr) => {
            assert_eq!(strip(name($input)), Object::Name($expected.to_vec()));
        };
    }

    #[test]
    fn test_name() {
        assert_name_eq!(b"/", b"");
        // These names come from the PDF 1.7 spec, Page 17
        assert_name_eq!(b"/Name1", b"Name1");
        assert_name_eq!(b"/ASomewhatLongerName", b"ASomewhatLongerName");
        assert_name_eq!(
            b"/A;Name_With-Various***Characters?",
            b"A;Name_With-Various***Characters?"
        );
        assert_name_eq!(b"/1.2", b"1.2");
        assert_name_eq!(b"/$$", b"$$");
        assert_name_eq!(b"/@pattern", b"@pattern");
        assert_name_eq!(b"/.notdef", b".notdef");
        assert_name_eq!(b"/lime#20Green", b"lime Green");
        assert_name_eq!(b"/paired#28#29parentheses", b"paired()parentheses");
        assert_name_eq!(b"/The_Key_of_F#23_Minor", b"The_Key_of_F#_Minor");
        assert_name_eq!(b"/A#42", b"AB");
    }

    macro_rules! assert_object_id_eq {
        ($input:expr, $expected:expr) => {
            assert_eq!(strip(object_id($input)), $expected);
        };
    }

    #[test]
    fn test_object_id() {
        assert_object_id_eq!(b"255 10001", (255, 10001));
        assert_object_id_eq!(b"1 0", (1, 0));
        assert_object_id_eq!(b"1 0 R", (1, 0));
        assert_object_id_eq!(b"4294967295 0", (u32::MAX, 0));
        assert_object_id_eq!(b"0000000001 0", (1, 0));
        assert_object_id_eq!(b"1 0000000000", (1, 0));
    }

    macro_rules! assert_reference_eq {
        ($input:expr, $expected:expr) => {
            assert_eq!(strip(reference($input)), Object::Reference($expected));
        };
    }

    #[test]
    fn test_reference() {
        assert_reference_eq!(b"255 10001 R", (255, 10001));
        assert_reference_eq!(b"1 0 R", (1, 0));
        assert_reference_eq!(b"4294967295 0 R", (u32::MAX, 0));
        assert_reference_eq!(b"0000000001 0 R", (1, 0));
        assert_reference_eq!(b"1 0000000000 R", (1, 0));
    }

    macro_rules! assert_null {
        ($input:expr) => {
            assert_eq!(strip(null($input)), Object::Null);
        };
    }

    #[test]
    fn test_null() {
        assert_null!(b"null");
        assert_null!(b"NULL");
        assert_null!(b"Null");
        assert_null!(b"  Null  ");
    }

    macro_rules! assert_boolean {
        ($input:expr, $expected:expr) => {
            assert_eq!(strip(boolean($input)), Object::Boolean($expected));
        };
    }

    #[test]
    fn test_boolean() {
        assert_boolean!(b"true", true);
        assert_boolean!(b"false", false);
        assert_boolean!(b"FALSE", false);
        assert_boolean!(b"False", false);
        assert_boolean!(b"  True ", true);
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(is_whitespace(b' '), true);
    }

    #[test]
    fn test_eol() {
        assert_eq!(is_eol(b"\n"), true);
        assert_eq!(is_eol(b"\r\n"), true);
        assert_eq!(is_eol(b"\r"), true);
        assert_eq!(is_eol(b""), false);
        assert_eq!(is_eol(b" "), false);
        assert_eq!(is_eol(b"something"), false);
    }

    #[test]
    fn test_integer() {
        assert_eq!(strip(int1::<i64>(b"-10")), -10);
        assert_eq!(strip(int1::<i64>(b"100000")), 100000);
    }

    #[test]
    fn test_real() {
        assert_eq!(strip(_real::<f64>(b"100.4292498")), 100.4292498);
        assert_eq!(strip(_real::<f64>(b"+100.4292498")), 100.4292498);
        assert_eq!(strip(_real::<f64>(b"-100.4292498")), -100.4292498);
        assert_eq!(strip(_real::<f64>(b"-.4292498")), -0.4292498);
        assert_eq!(strip(_real::<f64>(b"-0.4292498")), -0.4292498);
        assert_eq!(strip(_real::<f64>(b"-0.4292498")), -0.4292498);
        assert_eq!(strip(_real::<f64>(b"34.5")), 34.5);
        assert_eq!(strip(_real::<f64>(b"-3.62")), -3.62);
        assert_eq!(strip(_real::<f64>(b"-123.6")), -123.6);
        assert_eq!(strip(_real::<f64>(b"4.")), 4.0);
        assert_eq!(strip(_real::<f64>(b"-.002")), -0.002);
        assert_eq!(strip(_real::<f64>(b"0.0")), 0.0);
    }

    #[test]
    fn test_array() {
        assert_eq!(strip(_array(b"[]")), vec![]);
        assert_eq!(strip(_array(b"[68 69]")), vec![int!(68), int!(69)]);
        assert_eq!(
            strip(_array(b"[0 0 612.0000 792.0000]")),
            vec![int!(0), int!(0), real!(612.0), real!(792.0)]
        );
    }

    macro_rules! assert_string_eq {
        ($input:expr, $expected:expr) => {
            assert!(
                strip(string($input)) == string_lit!($expected)
                    || strip(string($input)) == string_hex!($expected)
            );
        };
    }

    #[test]
    fn test_string() {
        assert_eq!(_string(b"(this is a string) Tj").unwrap().0, b"Tj");
        assert_string_eq!(b"(This is a string) and this is junk", b"This is a string");
        assert_string_eq!(
            b"(Strings may contain\nnewlines and such.) and this is junk",
            b"Strings may contain\nnewlines and such."
        );
        assert_string_eq!(
				b"(Strings may contain (balanced parenthesis) as well as special chars (e.g. $%^&)) and this is junk",
				b"Strings may contain (balanced parenthesis) as well as special chars (e.g. $%^&)"
		);
        assert_string_eq!(
            b"(Strings may have escaped unbalanced \\( in addition to (balanced)))",
            b"Strings may have escaped unbalanced ( in addition to (balanced))"
        );
        assert_string_eq!(b"(literal backslack \\\\)", b"literal backslack \\");
        assert_string_eq!(b"(this is all \\none line)", b"this is all one line");
        assert_string_eq!(b"(\\032)", &[26]);
        assert_string_eq!(b"(printable octal \\122\\123\\124)", b"printable octal RST");
        assert_string_eq!(b"(\\53)", b"+");
        assert_string_eq!(b"(\\7)", &[7]);
        assert_string_eq!(b"(\\0053)", &[5, 51]);
        assert_string_eq!(b"(\\\\7)", b"\\7");
        assert_string_eq!(b"<68656C6C6F>", b"hello");
        assert_string_eq!(b"<901FA>", &[144, 31, 160]);
    }

    #[test]
    fn test_stream() {
        let input = "<< /Length 1074 >> stream wow endstream";
        assert_eq!(
            _stream(&input.as_bytes()).unwrap().1,
            Stream {
                dict: dictionary_struct! {
                    "Length" => int!(1074),
                },
                content: &[119, 111, 119, 32],
                allows_compression: false,
                start_position: None,
            }
        )
    }

    #[test]
    fn test_find_object_ids() {
        let input = "
1 0 obj
some junk
000002 00000 obj
"
        .as_bytes();
        let objects = find_all_object_ids(input).unwrap();
        assert_eq!(objects, vec![((1, 0), 0), ((2, 0), 19)]);
    }
}
