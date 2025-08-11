use dymex::Float;
use dymex::float;

const ZERO_THR: Float = float::EPSILON * 1E2;
const REL_ERR: Float  = float::EPSILON * 1E2;


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