//! 1. Only put small concepts here. Nothing major
//! 2. This crate *must* have no dependencies on other local crates in the project

mod page_dimensions;
mod error;

pub use page_dimensions::{PageWidth, PageHeight};
pub use error::NumberError;