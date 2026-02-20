#![deny(clippy::all)]

pub mod cst;
pub mod parser;

pub use parser::{parse, Language, ParseError};
