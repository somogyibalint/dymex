
use thiserror::Error;

use std::f64 as float;
type Float = f64;
// https://stackoverflow.com/questions/61835421/macro-to-use-import-depending-on-type-alias

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
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
        type_name: String,
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


fn unimplemented_binary_err<L, R, U>(l: &L, r: &R, op: &str) -> Result<U, EvaluationError>
where
    L: DynMath + ?Sized, 
    R: DynMath, 
    U: DynMath
{
    Err(EvaluationError::InvalidBinaryOperation { 
        operation: op.into(), 
        lhs: l.type_name().into(), 
        rhs: r.type_name().into() 
    })
}

fn unimplemented_unary_number_err<L>(data: &L, op: &str) -> Result<Float, EvaluationError>
where
    L: DynMath + ?Sized, 
{
    Err(EvaluationError::InvalidUnaryOperation { 
        operation: op.into(), 
        operand: data.type_name().into(), 
    })
}

fn unimplemented_unary_same_err<L>(data: &L, op: &str) -> Result<L, EvaluationError>
where
    L: DynMath + Sized, 
{
    Err(EvaluationError::InvalidUnaryOperation { 
        operation: op.into(), 
        operand: data.type_name().into(), 
    })
}

pub trait DynMath {

    fn type_name(&self) -> String;

    fn is_scalar(&self) -> bool;

    fn shape(&self) -> &[usize];

    fn shape_matches(&self, other: impl DynMath) -> bool {
        if self.is_scalar() 
            || other.is_scalar() 
            || self.shape() == other.shape() {
                return true
            }
        false
    }

    fn number(&self) -> Result<Float, EvaluationError>;

    fn get_field<R>(&self, field_name: &str) -> Result<R, EvaluationError> 
    where
        R: DynMath
    {
        Err(EvaluationError::InvalidField { type_name: self.type_name(), field: field_name.into() })
    }


    // binary operations: Self, Other -> Self || Other

    fn add<O, R>(&self, other: O) -> Result<R, EvaluationError> 
    where
        O: DynMath,
        R: DynMath
    {
        unimplemented_binary_err(self, &other, "+")
    }

    fn sub<O, R>(&self, other: O) -> Result<R, EvaluationError> 
    where
        O: DynMath,
        R: DynMath
    {
        unimplemented_binary_err(self, &other, "-")
    }

    fn mul<O, R>(&self, other: O) -> Result<R, EvaluationError> 
    where
        O: DynMath,
        R: DynMath
    {
        unimplemented_binary_err(self, &other, "*")
    }

    fn div<O, R>(&self, other: O) -> Result<R, EvaluationError> 
    where
        O: DynMath,
        R: DynMath
    {
        unimplemented_binary_err(self, &other, "/")
    }

    #[allow(unused)]
    fn pow<R>(&self, other: Float) -> Result<R, EvaluationError> 
    where
        R: DynMath
    {
        Err(EvaluationError::InvalidBinaryOperation { 
            operation: "**".into(), 
            lhs: self.type_name().into(), 
            rhs: "Number".into() 
        })
    }

    // unary operations: Self -> Float
    fn min(&self) -> Result<Float, EvaluationError> {
        unimplemented_unary_number_err(self, "min()")
    }
    fn max(&self) -> Result<Float, EvaluationError> {
        unimplemented_unary_number_err(self, "max()")
    }
    fn avg(&self) -> Result<Float, EvaluationError> {
        unimplemented_unary_number_err(self, "avg()")
    }
    fn std(&self) -> Result<Float, EvaluationError> {
        unimplemented_unary_number_err(self, "std()")
    }
    fn sum(&self) -> Result<Float, EvaluationError> {
        unimplemented_unary_number_err(self, "sum()")
    }

    // unary operations: Self -> Self

    fn sin(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "sin()")
    }

    fn cos(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "cos()")
    }

    fn tan(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "tan()")
    }

    fn cotan(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "cotan()")
    }

    fn exp(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "exp()")
    }

    fn log(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "log()")
    }

    fn log2(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "log2()")
    }

    fn log10(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "log10()")
    }

    fn sqrt(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "sqrt()")
    }

    fn pow2(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "^2")
    }

    fn pow3(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "^3")
    }    

    fn pow4(&self) -> Result<Self, EvaluationError> 
    where 
        Self: Sized
    {
        unimplemented_unary_same_err(self, "^4")
    }

}

fn invalid_args_err(func: &str, details: &str) -> Result<Float, EvaluationError> {
    Err(EvaluationError::InvalidArguments { 
        function: func.into(), 
        details: details.into() 
    }) 
}

fn all_scalars(args: &[impl DynMath]) -> bool 
{
    args.iter().all(|e| e.is_scalar())
}

fn unbox_numbers(args: &[impl DynMath], func: &str) -> Result<Vec::<Float>, EvaluationError> {
    let mut res = Vec::new();
    for v in args {
        if let Ok(n) = v.number() {
            res.push(n);
        } else {
            return Err(EvaluationError::InvalidArguments { 
                function: func.into(), 
                details: "every argument should be a number".into()
            })
        }
    }
    return Ok(res);
}

const ZERO_ARGS_ERR: &str = "needs at least one argument";
const MULTI_ARGS_ERR: &str = "accepts a single array or multiple scalar values";

fn dynmath_min<T>(args: &[T]) -> Result<Float, EvaluationError> 
where 
    T: DynMath
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

fn dynmath_max<T>(args: &[T]) -> Result<Float, EvaluationError> 
where 
    T: DynMath
{
    match args.len() {
        0 => invalid_args_err("max", ZERO_ARGS_ERR), 
        1 => return args[0].max(),
        _ if !all_scalars(args) => invalid_args_err("max", MULTI_ARGS_ERR),
        _ => match unbox_numbers(args, "max") {
            Err(e) => Err(e),
            Ok(v) => {
                Ok(v.iter().fold(float::INFINITY, |a, &b| a.max(b)))
            }
        }
    }
}

fn dynmath_avg<T>(args: &[T]) -> Result<Float, EvaluationError> 
where 
    T: DynMath
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

fn dynmath_std<T>(args: &[T]) -> Result<Float, EvaluationError> 
where 
    T: DynMath
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