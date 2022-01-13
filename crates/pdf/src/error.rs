use crate::{object::Object, ObjectId};
use anyhow::Error;
use core::panic;
use nom::{error::VerboseError, Needed};
use std::str::Utf8Error;
use strum_macros::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlingError {
    #[error("invalid attempt to cast object to wrong type")]
    // TODO: Support actual and intended object type
    ObjectCast,
    #[error("object not found")]
    // TODO: Support showing attempt object to be found
    ObjectNotFound,
}

#[derive(Error, Debug)]
pub enum XrefError {
    #[error("Found wrong object for expected xref entry")]
    FoundWrongObjectForExpectedEntry((ObjectId, ObjectId)),
    #[error("Invalid entry found in XREF table")]
    XrefTableInvalidEntry(ObjectId),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Cap style should be 0, 1, 2 but was {0}")]
    InvalidCapStyle(i32),
    #[error("Failed to parse entire object stream for object")]
    FailedToParseAllStreamContent,
}

