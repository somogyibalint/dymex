use dymex::*;


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
