use dymex::*;
use std::{collections::HashMap, hash::Hash, rc::Rc};


#[test]
fn simple_expression_binop() {
    let expression = "(1.0 + (a - b)*c) / 2";
    let mut variables: HashMap<String, Rc<dyn DynMath>> = HashMap::new();
    variables.insert("a".to_owned(), Rc::new(2.0));
    variables.insert("b".to_owned(), Rc::new(1.0));
    variables.insert("c".to_owned(), Rc::new(3.0));

    let varnames: Vec<&str> = variables.iter().map(|(k, _)| k.as_str()).collect();

    let mut evalutor = Evaluator::new(&expression, &varnames).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert_eq!(2.0, result.as_number());   
}

#[test]
fn simple_expression_trig() {
    let expression = "cos(pi * (sin(x)^2 + cos(x)^2) / 2)";
    let mut variables: HashMap<String, Rc<dyn DynMath>> = HashMap::new();
    variables.insert("x".to_owned(), Rc::new(0.12345));

    let varnames: Vec<&str> = variables.iter().map(|(k, _)| k.as_str()).collect();

    let mut evalutor = Evaluator::new(&expression, &varnames).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert!(result.as_number().abs() < 1E-10); // should be 0
}