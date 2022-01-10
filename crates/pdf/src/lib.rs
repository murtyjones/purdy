// For use in development only:
#![allow(dead_code, unused_imports)]

use nom::{error::VerboseError, IResult};

mod dictionary;
mod document;
mod encodings;
mod error;
mod macros;
mod object;
mod pdf;
mod rgb;
mod stream;
// public for the window binary
pub mod utils;
mod xref;

#[cfg(test)]
mod known;

#[macro_use]
extern crate maplit;

pub use crate::pdf::Pdf;
pub use crate::stream::StreamObject;

type NomError = ();
type NomResult<'a, O, E = NomError> = IResult<&'a [u8], O, E>;

type ObjectNumber = u32;

type GenerationNumber = u16;

type ObjectId = (ObjectNumber, GenerationNumber);
