//! 1. Only put small concepts here. Nothing major
//! 2. This crate *must* have no dependencies on other local crates in the project

mod dimensions;
mod error;

pub use dimensions::{Width, Height};
pub use error::NumberError;