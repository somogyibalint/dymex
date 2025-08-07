use crate::{float, Float, MAXDIM};
use super::{DynMath, EvaluationError, Category, unimpl_binary};


impl DynMath for Float {

    fn category(&self) -> Category { Category::Number }

    fn shape(&self) -> [usize; MAXDIM] { [0; MAXDIM] }

    fn as_number(&self) -> Float {
        *self
    }


    fn add(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self + other.as_number())),
            Category::Array => (*other).add(self),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    fn sub(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self - other.as_number())),
            Category::Array => (*other).sub_inv(self),
            _ => unimpl_binary(self.type_name(), other.type_name(), "-")             
        }
    }

    fn mul(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self * other.as_number())),
            Category::Array => (*other).mul(self),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    fn div(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(*self / other.as_number())),
            Category::Array => (*other).div_inv(self),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    fn pow(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            
            Category::Number => Ok(Box::new(self.powf(other.as_number()))),
            Category::Array => (*other).pow_inv(self), 
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")             
        }
    }

    
}