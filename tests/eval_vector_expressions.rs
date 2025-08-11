use dymex::*;
use std::{any::Any, arch::x86_64};


mod helpers;
use helpers::approx_eq;


#[test]
fn test_ax_plus_b() {
    let expression = "a*x + b";

    let mut variables= InputVars::new();
    variables.insert_owned("a".to_owned(), 2.0);
    variables.insert_owned("b".to_owned(), 1.0);
    variables.insert_owned("x".to_owned(), vec![1.0, 2.0, 3.0]);

    let target = [3.0, 5.0, 7.0];

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = &evalutor.evaluate( &variables).unwrap();
    let res = result.as_any().downcast_ref::<Vec<Float>>().unwrap();
    
    for (x1, x2) in res.iter().zip(target.iter()) {
        approx_eq(*x1, *x2);
    }
}

#[test]
fn test_stats() {
    let expression = "std(v) / avg(v)";

    let mut variables= InputVars::new();
    variables.insert_owned("v".to_owned(), 
    vec![0.16126227, 0.55013359, 0.89688053, 0.58357566, 0.35384424,
       0.98168083, 0.67449156, 0.62165282, 0.21484945, 0.59141298]);

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = &evalutor.evaluate( &variables).unwrap();

    assert!(approx_eq(result.as_number(), 0.4459756077414119))
}