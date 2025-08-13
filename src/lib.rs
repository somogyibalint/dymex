#![allow(dead_code)] //! FIXME

#[cfg(feature = "default")]
pub use std::f64 as float;
#[cfg(feature = "default")]
pub type Float = f64;

#[cfg(feature = "single_precision")]
use std::f32 as float;
#[cfg(feature = "single_precision")]
type Float = f32;


#[cfg(test)] #[macro_use]
extern crate assert_matches;

mod tokenizer;
#[allow(unused_imports)]
pub use crate::tokenizer::*;

mod parser;
#[allow(unused_imports)]
pub use crate::parser::*;

mod error;
pub use crate::error::*;

mod dynmath;
pub use crate::dynmath::*;

mod mermaid;
pub use crate::mermaid::*;

