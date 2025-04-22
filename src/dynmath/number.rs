use crate::{float, Float};
use super::{DynMath, EvaluationError, Category, unimpl_binary};


impl DynMath for Float {

    fn category(&self) -> Category { Category::Number }

    fn shape(&self) -> &[usize] { &[] }

    fn as_number(&self) -> Float {
        *self
    }


    fn add(&self, other: Box<dyn DynMath>) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self + other.as_number())),
            Category::Array => (*other).add(Box::new(*self)),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    fn sub(&self, other: Box<dyn DynMath>) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self - other.as_number())),
            Category::Array => (*other).sub_inv(Box::new(*self)),
            _ => unimpl_binary(self.type_name(), other.type_name(), "-")             
        }
    }

    fn mul(&self, other: Box<dyn DynMath>) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self * other.as_number())),
            Category::Array => (*other).mul(Box::new(*self)),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    fn div(&self, other: Box<dyn DynMath>) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self / other.as_number())),
            Category::Array => (*other).div_inv(Box::new(*self)),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    fn pow(&self, other: Box<dyn DynMath>) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            
            Category::Number => Ok(Box::new(self.powf(other.as_number()))),
            Category::Array => (*other).div_inv(Box::new(*self)),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    
}