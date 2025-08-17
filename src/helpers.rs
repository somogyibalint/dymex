use crate::Float;
use crate::float;
use crate::Token;


const ZERO_THR: Float = 1E2 * float::EPSILON;
const REL_ERR: Float  = 1E2 * float::EPSILON;

/// Compare two numbers and return true if they are close enoguh (test util)
// This is meant to be a fairly lenient comparison as our goal is not to test 
// floating-point artihmetic, but to see if the evaluated result is what we excepted 
pub fn approx_eq(x1: Float, x2: Float) -> bool {
    if x1.abs() < ZERO_THR 
    && x2.abs() < ZERO_THR {
        return true
    }
    if 2.0 * (x1-x2).abs() / (x1 + x2).abs() > REL_ERR {
        return false
    }
    true
}

pub fn same_num_tokens(t1: Token, t2: Token) -> bool {
    match (t1, t2) {
        (Token::Number(x1), Token::Number(x2)) => approx_eq(x1, x2),
        _ => false
    }
}