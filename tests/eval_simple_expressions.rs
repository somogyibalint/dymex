use dymex::*;

mod helpers;
use helpers::approx_eq;

#[test]
fn simple_expression_binop() {
    let expression = "(1.0 + (a - b)*c) / 2";

    let mut variables= InputVars::new();
    variables.insert_owned("a".to_owned(), 2.0);
    variables.insert_owned("b".to_owned(), 1.0);
    variables.insert_owned("c".to_owned(), 3.0);

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert_eq!(2.0, result.as_number());   
}

#[test]
fn simple_expression_trig() {
    let expression = "cos(pi * (sin(x)^2 + cos(x)^2) / 2)";
    let mut variables = InputVars::new();
    variables.insert_owned("x".to_owned(), 0.12345);

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert!(result.as_number().abs() < 1E-10); // should be 0
}

#[test]
fn variadic_expression_trig() {
    let expression = "min(a, max(5.0, max(abs(v))**2))";
    let mut variables = InputVars::new();
    variables.insert_owned("a".to_owned(), 10.0);
    variables.insert_owned("v".to_owned(), vec![-3.0, -1.0, 0.0, 2.0]); 

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert_eq!(result.as_number(), 9.0);
}

#[test]
fn nested_expression1() {
    // does not work, because pow(base, exp) not recognized
    // let expression = "pow(2.0, pow(pow(pow(x, y), z), 0.0))"; 
    //TODO: the error message was misleading: UnexpectedToken(37)
    //TODO: tokenize(expr, input_var) returns with different error for some reason!
    let expression = "2**(((x**y)**z)**0.0)"; 
    let mut variables = InputVars::new();
    variables.insert_owned("x".to_owned(), 3.0);
    variables.insert_owned("y".to_owned(), 2.0); 
    variables.insert_owned("z".to_owned(), 2.0);

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert_eq!(result.as_number(), 2.0);
}


//TODO UNICODE support
// #[test]
// fn nested_expression2() {
    
//     let expression = "cos(π/2 + sin(cos(sin(π/2))*π))"; 
//     let mut variables = InputVars::new();

//     let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

//     let result = evalutor.evaluate( &variables).unwrap();
//     assert_eq!(result.as_number(), 0.0);
// }



#[test]
fn nested_expression2() {
    
    let expression = "cos(pi/2 + sin(1 + cos(sin(pi/2)*pi)))"; 
    let variables = InputVars::new();

    let mut evalutor = Evaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert!(approx_eq(result.as_number(), 0.0));
}