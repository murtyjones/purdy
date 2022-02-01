use std::str::from_utf8;
use std::{fs::File, path::PathBuf};
use std::{io::Read, str::FromStr};

use crate::{object::Name, NomResult, ObjectId};
use anyhow::Result;
use nom::bitvec::view::AsBits;
use nom::bytes::complete::take_while1;
use nom::multi::fold_many0;
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
use nom::{AsChar, InputTakeAtPosition};

pub fn relative_path_to_absolute(path: &str) -> PathBuf {
    let relative_path = PathBuf::from(path);
    let mut absolute_path = std::env::current_dir().unwrap();
    absolute_path.push(relative_path);
    absolute_path
}

pub fn read_file_bytes(path: &str) -> Vec<u8> {
    let mut file = File::open(path).expect("could not read file");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("could not read file");
    data
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Marks a var as having a static lifetime
/// # Safety
/// Only use for things coming from the other side of WASM or similar places
/// where they have an incorrect lifetime applied. In other words, be sure
/// it actually has a static lifetime.
pub unsafe fn extend_lifetime<'a, T: ?Sized>(input: &'a T) -> &'static T {
    std::mem::transmute::<&'a T, &'static T>(input)
}

pub fn object_id_to_filename(page_id: ObjectId) -> String {
    format!("{}-{}", page_id.0, page_id.1)
}

#[inline]
pub fn strip_nom<T>(result: NomResult<'static, T>) -> Result<T> {
    Ok(result?.1)
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

fn eol(input: &[u8]) -> NomResult<&[u8]> {
    alt((tag(b"\r\n"), tag(b"\n"), tag("\r")))(input)
}

#[inline]
fn to_nom<O, E>(
    result: std::result::Result<O, E>,
    input: &[u8],
    error_kind: ErrorKind,
) -> NomResult<O> {
    result.map(|o| (input, o)).map_err(|_| {
        let err = nom::error::ParseError::from_error_kind(input, error_kind);
        nom::Err::Error(err)
    })
}

pub fn hex_char2(input: &[u8]) -> NomResult<u8> {
    map_res(
        verify(take(2usize), |h: &[u8]| h.iter().cloned().all(is_hex_digit)),
        |x| u8::from_str_radix(from_utf8(x).unwrap(), 16),
    )(input)
}

fn comment(input: &[u8]) -> NomResult<()> {
    map(
        tuple((char('%'), take_while(|c: u8| !b"\r\n".contains(&c)), eol)),
        |_| (),
    )(input)
}

#[test]
fn test_comment() {
    assert_eq!(comment("%hi\n".as_bytes()).unwrap(), ("".as_bytes(), ()));
}

fn space_or_comment<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], &'a [u8], E> {
    Ok(fold_many0(
        alt((map(take_while1(is_whitespace), |_| ()), comment)),
        input,
        |_, _| "".as_bytes(),
    )(input)
    // TODO: There's an issue with the error type in this funciton, so ? doesn't work. This is fine
    // because this function should never return an error (because of fold_many0, no matches is
    // still a pass). But it'd be nice to fix the error and use ?
    .unwrap())
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes both leading and trailing whitespace, returning the output of
/// `inner`.
#[inline]
pub fn ws<'a, F: 'a, O, E: ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(space_or_comment, inner, space_or_comment)
}

#[inline]
pub fn int1<I: FromStr>(input: &[u8]) -> NomResult<I> {
    map_res(tuple((opt(char('-')), digit1)), |(minus, digits)| {
        let digit_str = from_utf8(digits).unwrap();
        if minus.is_none() {
            return I::from_str(digit_str);
        }
        let digit_str = format!("-{}", digit_str);
        I::from_str(&digit_str)
    })(input)
}

pub fn _name(input: &[u8]) -> NomResult<Name> {
    delimited(
        tuple((space_or_comment, tag(b"/"))),
        many0(alt((
            preceded(tag(b"#"), hex_char2),
            map_opt(take(1usize), |c: &[u8]| {
                if c[0] != b'#' && is_regular(c[0]) {
                    Some(c[0])
                } else {
                    None
                }
            }),
        ))),
        space_or_comment,
    )(input)
}

/// Returns the first occurence of any of the given needles in the given byte
/// slice, or `None` if no needles found.
fn find_subsequence(haystack: &[u8], needles: Vec<&[u8]>) -> Option<usize> {
    let mut final_result = None;
    for needle in needles.into_iter() {
        if let Some(x) = haystack
            .windows(needle.len())
            .position(|window| window == needle)
        {
            // We want to always return the first occurrence of any of the needles. For
            // example, if we call this function with `find_subsequence(b"ab", vec![b"b",
            // b"a"])`, we want to return `Some(0)` even though "b" is the first given
            // needle
            if final_result.is_none() || final_result.unwrap() > x {
                final_result = Some(x)
            }
        }
    }
    final_result
}

/// This parser is designed to work inside the `nom::sequence::delimited`
/// parser, e.g.: `nom::sequence::delimited(tag("<<"),
/// take_until_unmatched(b"<<", b">>"), tag("<<"))(i)` It skips nested brackets
/// until it finds an extra closing bracket. This function is very similar to
/// `nom::bytes::complete::take_until(")")`, except it also takes nested
/// brackets. Escaped brackets e.g. `\(` and `\)` are not considered as brackets
/// and are taken by default.
pub fn take_until_unmatched<'a>(
    opening: &'a [u8],
    closing: &'a [u8],
) -> impl Fn(&'a [u8]) -> NomResult<&[u8]> {
    move |i: &[u8]| {
        // the index to cut the beginning of the string
        let mut index = 0;
        // The number of unmatched opening brackets
        let mut bracket_counter = 0;

        while let Some(n) = find_subsequence(&i[index..], vec![opening, closing]) {
            index += n;
            let l = opening.len();
            let it = &i[index..(index + l)];
            match it {
                c if c == opening => {
                    bracket_counter += 1;
                    index += opening.len();
                }
                c if c == closing => {
                    bracket_counter -= 1;
                    index += closing.len();
                }
                _ => unreachable!(),
            }
            // We found the unmatched closing bracket.
            if bracket_counter == -1 {
                // We do not consume it.
                index -= closing.len();
                return Ok((&i[index..], &i[0..index]));
            };
        }

        if bracket_counter == 0 {
            Ok((&[], i))
        } else {
            let err = ParseError::from_error_kind(i, ErrorKind::TakeUntil);
            let err = NomErr::Error(err);
            Err(err)
        }
    }
}

pub fn _real<I: FromStr>(input: &[u8]) -> NomResult<I> {
    let (i, _) = pair(
        opt(one_of("+-")),
        alt((
            map(tuple((digit1, tag(b"."), digit0)), |_| ()),
            map(pair(tag(b"."), digit1), |_| ()),
        )),
    )(input)?;

    let float_input = &input[..input.len() - i.len()];
    to_nom(
        I::from_str(std::str::from_utf8(float_input).unwrap()),
        i,
        ErrorKind::Digit,
    )
}

#[cfg(test)]
mod test {
    use nom::error::VerboseError;

    use super::space_or_comment;

    #[test]
    fn test_space_never_fails() {
        let empty = "".as_bytes();
        let input = "".as_bytes();
        assert_eq!(
            space_or_comment::<VerboseError<&[u8]>>(input).unwrap(),
            (empty, input)
        );
        let input = "blah".as_bytes();
        assert_eq!(
            space_or_comment::<VerboseError<&[u8]>>(input).unwrap(),
            (input, input)
        );
        let input = " ".as_bytes();
        assert_eq!(
            space_or_comment::<VerboseError<&[u8]>>(input).unwrap(),
            (empty, empty)
        );
        let input = "%comment\n".as_bytes();
        assert_eq!(
            space_or_comment::<VerboseError<&[u8]>>(input).unwrap(),
            (empty, empty)
        );
        let i = "%doublecomment\n  ".as_bytes();
        assert_eq!(
            space_or_comment::<VerboseError<&[u8]>>(i).unwrap(),
            (empty, empty)
        );
    }
}
