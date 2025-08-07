use crate::{float, Float, MAXDIM};
use super::{DynMath, EvaluationError, Category, unimpl_binary};
use std::slice::Iter;

impl DynMath for Vec<Float> {

    fn category(&self) -> Category { Category::Array }

    fn shape(&self) -> [usize; MAXDIM] { 
        let mut shape = [0; MAXDIM]; 
        shape[0] = self.len();
        shape
    }

    fn iterate(&self) -> Iter<'_, Float> {
        self.iter()
    }

    fn add(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(
                self.iter().map(|x| x + other.as_number()).collect::<Vec<Float>>()
            )),
            Category::Array => Ok(Box::new(
                self.iter().zip(other.iterate()).map(|(a, b)| a + b).collect::<Vec<Float>>()
            )),
            _ => unimpl_binary(self.type_name(), other.type_name(), "+")
        }
    }

    fn sub(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(
                self.iter().map(|a| a - other.as_number()).collect::<Vec<Float>>()
            )),
            Category::Array => Ok(Box::new(
                self.iter().zip(other.iterate()).map(|(a, b)| a - b).collect::<Vec<Float>>()
            )),
            _ => unimpl_binary(self.type_name(), other.type_name(), "-")
        }
    }

    fn mul(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(
                self.iter().map(|x| x * other.as_number()).collect::<Vec<Float>>()
            )),
            Category::Array => Ok(Box::new(
                self.iter().zip(other.iterate()).map(|(a, b)| a * b).collect::<Vec<Float>>()
            )),
            _ => unimpl_binary(self.type_name(), other.type_name(), "*")
        }
    }

    fn div(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(
                self.iter().map(|x| x / other.as_number()).collect::<Vec<Float>>()
            )),
            Category::Array => Ok(Box::new(
                self.iter().zip(other.iterate()).map(|(a, b)| a / b).collect::<Vec<Float>>()
            )),
            _ => unimpl_binary(self.type_name(), other.type_name(), "/")
        }
    }

    fn pow(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        match other.category() {
            Category::Number => Ok(Box::new(
                self.iter().map(|x| x.powf(other.as_number())).collect::<Vec<Float>>()
            )),
            Category::Array => Ok(Box::new(
                self.iter().zip(other.iterate()).map(|(a, b)| a.powf(*b)).collect::<Vec<Float>>()
            )),
            _ => unimpl_binary(self.type_name(), other.type_name(), "^")
        }
    }

    
    fn min(&self) -> Result<Float, EvaluationError> {
        Ok(self.iter().fold(float::INFINITY, |a, &b| a.min(b)))
    }
    fn max(&self) -> Result<Float, EvaluationError> {
        Ok(self.iter().fold(float::NEG_INFINITY, |a, &b| a.max(b)))
    }
    fn range(&self) -> Result<Float, EvaluationError> {
        Ok(self.max()? - self.min()?)
    }
    fn sum(&self) -> Result<Float, EvaluationError> {
        Ok(self.iter().sum::<Float>())
    }
    fn avg(&self) -> Result<Float, EvaluationError> {
        Ok(self.iter().sum::<Float>() / (self.len() as Float))
    }
    fn std(&self) -> Result<Float, EvaluationError> {
        let avg = self.avg()?;
        let sq_err = self.iter().map(|e| (e-avg)*(e-avg)).sum::<Float>();
        Ok((sq_err / (self.len() as Float)).sqrt())
    }
    fn l2_norm(&self) -> Result<Float, EvaluationError> {
        let sq_sum = self.iter().map(|e| e*e).sum::<Float>();
        Ok(sq_sum.sqrt())
    }
    fn l1_norm(&self) -> Result<Float, EvaluationError> {
        Ok(self.iter().map(|e| e.abs()).sum::<Float>())
    }
    
}



fn elementwise<T>(array: &[f64], func: T) -> Result<Vec<Float>, EvaluationError> 
where 
    T: Fn(&Float) -> Float
{
    Ok(array.iter().map(|x| func(x)).collect())
} 