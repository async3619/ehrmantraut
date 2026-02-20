#![deny(clippy::all)]

pub mod cst;
pub mod lower;
pub mod parser;

pub use lower::{lower, LowerError};
pub use parser::{parse, Language, ParseError};
