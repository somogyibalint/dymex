#![allow(dead_code)] //! FIXME

#[cfg(feature = "default")]
use std::f64 as float;
#[cfg(feature = "default")]
type Float = f64;

#[cfg(feature = "single_precision")]
use std::f32 as float;
#[cfg(feature = "single_precision")]
type Float = f32;


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

mod mermaid;
pub use crate::mermaid::*;

