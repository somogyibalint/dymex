use std::collections::HashMap;

use dioxus::html::OptionExtension;
use dymex::{DymexError, Float, Token, TokenContext};
use dymex::{Evaluator, InputVars, EvaluationError};

pub fn float_formatter(f: Option<&f64>) -> String {
    match f {
        Some(x) => format!("{:.6}", x),
        None => "".to_string()
    }
}

pub fn token_style(t: &TokenContext) -> String {
    match t.token {
        Token::ArOp(_) 
        | Token::AssignOp(_) 
        | Token::LogicOp(_) 
        | Token::RelOp(_) => "token opToken".to_string(),
        Token::Colon
        | Token::Semicolon 
        | Token::Comma 
        | Token::Dot 
        | Token::LB 
        | Token::RB 
        | Token::LP 
        | Token::RP => "token commonToken".to_string(),
        Token::Number(_) | Token::Const(_) => "token constToken".to_string(),
        Token::Var(_) => "token varToken".to_string(),
        Token::Func(_, _) => "token funcToken".to_string(),
        _ => "".to_string()
    }
}

// Evaluator {pub fn new(expression: &str, variables: &[&str])
pub fn evaluate(expr: &str, var: &[String], values: HashMap<String, f64>) -> Result<f64, EvaluationError> {
    let variables: Vec<&str> = var.iter().map(String::as_str).collect();
    let mut eval = Evaluator::new(expr, &variables).unwrap();
    let mut input = InputVars::new();
    for (k,v) in values.into_iter() {
        input.insert_owned(k, v);
    }
    match eval.evaluate(&input) {
        Ok(x) => { Ok(x.as_number()) },
        Err(err) => Err(err)
    }
}

pub fn format_num_result(x: Option<Float>) -> String {
    match x {
        Some(x) => format!("{:.4}", x),
        None => " ⚠ ".to_string()
    }
}