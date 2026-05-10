use dymex::*;



#[test]
fn simple_expression_trig() {
    let expression = "cos(pi * (sin(x)^2 + cos(x)^2) / 2)";
    let mut variables = InputVars::new();
    variables.insert_owned("x".to_owned(), 0.12345);

    let mut evalutor = MultiExpEvaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert!(result.as_number().abs() < 1E-10); // should be 0
}

#[test]
fn multi_expression_scalar() {
    let expression = "x = a*b+c\n y = 2*c-b/a\n(x*3 + y*2) - 2"; // x = 5,  y 5.5

    let mut variables= InputVars::new();
    variables.insert_owned("a".to_owned(), 2.0);
    variables.insert_owned("b".to_owned(), 1.0);
    variables.insert_owned("c".to_owned(), 3.0);

    let mut evalutor = MultiExpEvaluator::new(&expression, &variables.names()).unwrap();

    let result = evalutor.evaluate( &variables).unwrap();
    assert_eq!(24.0, result.as_number());
}
