use std::ops::Add;
use crate::{float, Float};
use super::{DynMath, EvaluationError};


impl DynMath for Float {

    fn type_name(&self) -> String { "Number".into() }

    fn is_scalar(&self) -> bool { true }

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