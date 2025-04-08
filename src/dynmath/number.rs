use std::ops::Add;
use crate::{float, Float};
use super::{DynMath, EvaluationError, Category};


impl DynMath for Float {

    fn category(&self) -> Category { Category::Number }

    fn shape(&self) -> &[usize] { &[] }

    // sclars match with anything
    fn shape_matches(&self, _: impl DynMath) -> bool { true }

    fn number(&self) -> Float {
        *self
    }



    // fn add(&self, other: Float) -> Result<Float, EvaluationError> 
    // {
    //     Ok(*self + other)
    // }
    

}