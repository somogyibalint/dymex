#![allow(dead_code)] //! FIXME

#[cfg(test)] #[macro_use]
extern crate assert_matches;

mod lexer;
#[allow(unused_imports)]
pub use crate::lexer::*;

mod parser;
#[allow(unused_imports)]
pub use crate::parser::*;

mod dynmath;
pub use crate::dynmath::*;