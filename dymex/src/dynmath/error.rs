use thiserror::Error;
use crate::Float;
use super::*;

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("operation `{operation:?}` not applicable to `{operand:?}`")]
    InvalidUnaryOperation {
        operation: String,
        operand: String,
    },
    #[error("operation `{operation:?}` not supported between `{lhs:?}` and `{rhs:?}`")]
    InvalidBinaryOperation {
        operation: String,
        lhs: String,
        rhs: String,
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
    #[error("Missing input variable: '{varname:?}' ")]
    MissingInputVariable {
        varname: String
    },
    #[error("invalid operation: {info:?}")]
    InvalidOperation {
        info: String,
    },
    #[error("Missing expression for final result")]
    MissingFinalExpression,

    #[error("unknown error")]
    Unknown,
}

/// Unimplemented unary function that returns a number (min, max, sum ...)
pub(super) fn unimpl_unary<L>(data: &L, op: &str) -> Result<Float, EvaluationError>
where
    L: DynMath + ?Sized,
{
    Err(EvaluationError::InvalidUnaryOperation {
        operation: op.into(),
        operand: data.type_name().into(),
    })
}

/// Unimplemented binary operation `op` between `lhs` and `rhs`
pub(super) fn unimpl_binary(lhs: &str, rhs: &str, op: &str) -> Result<Box<dyn DynMath>, EvaluationError>
{
    Err(EvaluationError::InvalidBinaryOperation {
        operation: op.into(),
        lhs: lhs.into(),
        rhs: rhs.into()
    })
}

