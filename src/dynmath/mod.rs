use std::any::Any;
use std::rc::Rc;
use std::slice::Iter;
use thiserror::Error;
use crate::{float, Float};

pub const MAXDIM: usize = 3;

mod number;
mod vector;
mod eval;
pub use eval::*;

pub enum DynVar<T: DynMath> {
    Number(Float),
    Composite(T)
}

pub enum Category {
    /// Floating point number
    Number,
    /// Indexable, iterable array type (such as Vec)
    Array,
    /// Unique type. Binary operations with different types are not supported
    Unqiue,
}
impl Category {
    fn to_str(&self) -> &'static str {
        match *self {
            Self::Number => "Number",
            Self::Array  => "Array",
            Self::Unqiue => "Unique"
        }
    }
}

type Unary = fn(Float) -> Float;

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("operation `{operation:?}` not supported between `{lhs:?}` and `{rhs:?}`")]
    InvalidBinaryOperation {
        operation: String,
        lhs: String,
        rhs: String,
    },
    #[error("operation `{operation:?}` not applicable to `{operand:?}`")]
    InvalidUnaryOperation {
        operation: String,
        operand: String,
    },

    #[error("`{type_name:?}` has no field named `{field:?}`")]
    InvalidField {
        type_name: &'static str,
        field: String,
    },
    #[error("invalid arguments for `{function:?}`: {details:?}")]
    InvalidArguments {
        function: String,
        details: String
    },
    #[error("unknown error")]
    Unknown,
}

/// Unimplemented binary operation `op` between `lhs` and `rhs`
fn unimpl_binary(lhs: &str, rhs: &str, op: &str) -> Result<Box<dyn DynMath>, EvaluationError>
{
    Err(EvaluationError::InvalidBinaryOperation { 
        operation: op.into(), 
        lhs: lhs.into(), 
        rhs: rhs.into() 
    })
}

/// Unimplemented unary function that returns a number (min, max, sum ...)
fn unimpl_unary<L>(data: &L, op: &str) -> Result<Float, EvaluationError>
where
    L: DynMath + ?Sized, 
{
    Err(EvaluationError::InvalidUnaryOperation { 
        operation: op.into(), 
        operand: data.type_name().into(), 
    })
}


pub trait DynMath : Any {

    fn category(&self) -> Category;

    fn shape(&self) -> [usize; MAXDIM];

    fn shape_matches(&self, other: &dyn DynMath) -> bool {
        match (self.category(), other.category()) {
            (Category::Number, _) => true,
            (_, Category::Number) => true,
            (Category::Array, Category::Array) => {
                self.shape() == other.shape() 
            },
            _ => false // maybe panic here?
        }
    }

    /// Type name as str for runtime error information
    fn type_name(&self) -> &'static str {
        &self.category().to_str()
    }

    /// Interpret as floatB
    // Here it's ok to panic, this should be only called after matching on self.category()
    fn as_number(&self) -> Float {
        panic!("Panic: `{}` is not a number.", self.type_name())
    }

    /// Return an iterator if possible
    // Here it's ok to panic, this should be only called after matching on self.category()
    fn iterate(&self) -> Iter<'_, Float> {
        panic!("Panic: trying to iterate non-iterable `{}`", self.type_name())
    }

    /// Apply provided function for self elementwise
    // Here it's ok to panic, this should be only called after matching on self.category()
    #[allow(unused_variables)]
    fn elementwise(&self, f: Unary) -> Result<Box<dyn DynMath>, EvaluationError> {
        panic!("Panic: applying unary function to `{}` is not supported.", self.type_name())
    }

    /// Helper function for unary functions, selects the appropriate method for the evaluation of `f`
    /// based on whether self is scalar, array or some other type
    fn unary_dispatcher(&self, f: Unary, op: &str) -> Result<Box<dyn DynMath>, EvaluationError> {
        match self.category() {
            Category::Number => Ok(Box::new(f(self.as_number()))),
            Category::Array => self.elementwise(f),
            _ => Err(EvaluationError::InvalidUnaryOperation {
                operation: op.into(),
                operand: self.type_name().into(),
            }),
        }
    }

    /// Returns `field_name` field of self 
    fn get_field(&self, field_name: &str) -> Result<Box<dyn DynMath>, EvaluationError> 
    {
        Err(EvaluationError::InvalidField { type_name: self.type_name(), field: field_name.into() })
    }

    //TODO: call method?

    // Binary operations: Self, Other -> Self || Other

    fn add(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        unimpl_binary(self.type_name(), other.type_name(), "+")
    }
    
    fn sub(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        unimpl_binary(self.type_name(), other.type_name(), "-")
    }

    fn mul(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        unimpl_binary(self.type_name(), other.type_name(), "*")
    }

    fn div(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        unimpl_binary(self.type_name(), &other.type_name(), "/")
    }

    fn pow(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        unimpl_binary(self.type_name(), &other.type_name(), "**")
    }

    /// Helpers for non-commutative binary operators
    /// Number + Array type would be tricky to implement, as it is not obvious how to 
    /// reconstruct an Array from its iterator. Instead, we call Array + Number.
    /// For the non-commutative binary operators of `-`, `/` and `^`, inverse methods
    /// need to be defined. 
    //TODO Instead of inverse, call it backwards, or righttoleft?  
    fn sub_inv(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {        
        other.sub(&self.as_number())
    }
    fn div_inv(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        other.div(&self.as_number())
    }
    fn pow_inv(&self, other: &dyn DynMath) -> Result<Box<dyn DynMath>, EvaluationError>
    {
        other.pow(&self.as_number())
    }

    
    // unary operations: Self -> Float
    // TODO: median
    fn min(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "min()")
    }
    fn max(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "max()")
    }
    fn avg(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "avg()")
    }
    fn std(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "std()")
    }
    fn sum(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "sum()")
    }
    fn range(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "range()")
    }
    fn l2_norm(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "l2_norm()")
    }
    fn l1_norm(&self) -> Result<Float, EvaluationError> {
        unimpl_unary(self, "l1_norm()")
    }

    // Unary operations: Self -> Self
    // todo: round, ceil, floor?
    fn dyn_sin(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.sin() , "sin()")
    }

    fn dyn_cos(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.cos() , "cos()")
    }

    fn dyn_tan(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.tan() , "tan()")
    }

    fn dyn_cot(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|1.0/x.tan() , "cot()")
    }

    fn dyn_exp(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.exp() , "exp()")
    }

    fn dyn_log(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.ln() , "log()")
    }

    fn dyn_log2(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.log2() , "log2()")
    }

    fn dyn_log10(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.log10() , "log10()")
    }

    fn dyn_sqrt(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.sqrt() , "sqrt()")
    }
    //TODO: add to parser
    fn dyn_cbrt(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x.cbrt() , "cbrt()")
    }
    //TODO: add to parser
    fn dyn_pow2(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x*x , "^2")
    }
    //TODO: add to parser
    fn dyn_pow3(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x*x*x , "^3")
    }
    //TODO: add to parser
    fn dyn_pow4(&self) -> Result<Box<dyn DynMath>, EvaluationError> {
        self.unary_dispatcher(|x|x*x*x*x , "^4")
    }

}

fn invalid_args_err(func: &str, details: &str) -> Result<Float, EvaluationError> {
    Err(EvaluationError::InvalidArguments { 
        function: func.into(), 
        details: details.into() 
    }) 
}

fn all_scalars(args: &[Rc<dyn DynMath>]) -> bool 
{
    args.iter().all(|e| matches!(e.category(), Category::Number))
}

// fn unbox_numbers(args: &[impl DynMath], func: &str) -> Result<Vec::<Float>, EvaluationError> {
//     let mut res = Vec::new();
//     for v in args {
//         if let Ok(n) = v.number() {
//             res.push(n);
//         } else {
//             return Err(EvaluationError::InvalidArguments { 
//                 function: func.into(), 
//                 details: "every argument should be a number".into()
//             })
//         }
//     }
//     return Ok(res);
// }

#[allow(unused_variables)]
fn unbox_numbers(args: &[Rc<dyn DynMath>], func: &str) -> Result<Vec::<Float>, EvaluationError> {
    let mut res = Vec::new();
    for v in args {
        res.push(v.as_number())
    }
    return Ok(res);
}

// fn unary_dispatcher(obj: &dyn DynMath, f: Unary, op: &str) -> Result<Box<dyn DynMath>, EvaluationError> {
//     match obj.category() {
//         Category::Number => Ok(Box::new(f(obj.as_number()))),
//         Category::Array => obj.elementwise(f),
//         _ => Err(EvaluationError::InvalidUnaryOperation {
//             operation: op.into(),
//             operand: obj.type_name().into(),
//         }),     
//     }
// }


const ZERO_ARGS_ERR: &str = "needs at least one argument";
const MULTI_ARGS_ERR: &str = "accepts a single array or multiple scalar values";

pub fn dynmath_min(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("min", ZERO_ARGS_ERR), 
        1 => return args[0].min(),
        _ if !all_scalars(args) => invalid_args_err("min", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "min") {
            Err(e) => Err(e),
            Ok(v) => {
                Ok(v.iter().fold(float::INFINITY, |a, &b| a.min(b)))
            }
        }
    }
}


pub fn dynmath_max(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("max", ZERO_ARGS_ERR), 
        1 => return args[0].max(),
        _ if !all_scalars(args) => invalid_args_err("max", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "max") {
            Err(e) => Err(e),
            Ok(v) => {
                Ok(v.iter().fold(float::NEG_INFINITY, |a, &b| a.max(b)))
            }
        }
    }
}

pub fn dynmath_range(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("range", ZERO_ARGS_ERR), 
        1 => match (args[0].max(), args[0].min()) {
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
            (Ok(_max), Ok(_min)) => Ok(_max - _min)
        }
        _ if !all_scalars(args) => invalid_args_err("range", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "range") {
            Err(e) => Err(e),
            Ok(v) => {
                let _max = v.iter().fold(float::INFINITY, |a, &b| a.max(b));
                let _min = v.iter().fold(float::INFINITY, |a, &b| a.min(b));
                Ok(_max - _min)
            }
        }
    }
}


pub fn dynmath_sum(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("sum", ZERO_ARGS_ERR), 
        1 => Ok(args[0].iterate().sum()),
        _ if !all_scalars(args) => invalid_args_err("sum", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "sum") {
            Err(e) => Err(e),
            Ok(v) => {
                Ok(v.iter().sum::<Float>())
            }
        }
    }
}

pub fn dynmath_avg(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("avg", ZERO_ARGS_ERR), 
        1 => return args[0].avg(),
        _ if !all_scalars(args) => invalid_args_err("avg", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "avg") {
            Err(e) => Err(e),
            Ok(v) => {
                Ok(v.iter().sum::<Float>() / (args.len() as Float))
            }
        }
    }
}

pub fn dynmath_std(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("std", ZERO_ARGS_ERR), 
        1 => return args[0].std(),
        _ if !all_scalars(args) => invalid_args_err("std", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "std") {
            Err(e) => Err(e),
            Ok(v) => {
                let avg = v.iter().sum::<Float>() / (args.len() as Float);
                let sq_err = v.iter().map(|e| (e-avg)*(e-avg)).sum::<Float>();
                Ok((sq_err / (args.len() as Float)).sqrt())
            }
        }      
    }
}

pub fn dynmath_l2(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("l2_norm", ZERO_ARGS_ERR), 
        1 => return args[0].l2_norm(),
        _ if !all_scalars(args) => invalid_args_err("l2_norm", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "l2_norm") {
            Err(e) => Err(e),
            Ok(v) => {
                let sq_sum = v.iter().map(|e| e*e).sum::<Float>();
                Ok(sq_sum.sqrt())
            }
        }
    }
}

pub fn dynmath_l1(args: &[Rc<dyn DynMath>]) -> Result<Float, EvaluationError> 
{
    match args.len() {
        0 => invalid_args_err("l1_norm", ZERO_ARGS_ERR), 
        1 => return args[0].l1_norm(),
        _ if !all_scalars(args) => invalid_args_err("l1_norm", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "l1_norm") {
            Err(e) => Err(e),
            Ok(v) => {
                let abs_sum = v.iter().map(|e| e.abs()).sum::<Float>();
                Ok(abs_sum)
            }
        }
    }
}