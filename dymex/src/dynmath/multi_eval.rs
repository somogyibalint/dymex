use std::rc::Rc;
// use crate::parser::{A};
use crate::*;

const FINAL_RESULT_ALIAS: &str = "RESULT";

#[derive(Clone)]
pub struct MultiExpEvaluator {
    expressions: Vec<Evaluator>,
    temporaries: Vec<String>,
}


impl MultiExpEvaluator {
    pub fn new(expression: &str, variables: &[&str]) -> Result<Self, DymexError> {
        let mut var = variables.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let mut evaluators = Vec::new();
        let mut temp_variables = Vec::new();
        for line in expression.lines() {
            match split_assignement(line) {
                (None, Some(exp)) => {
                    match parse_expr(&exp, &var) {
                        Ok(ast) => {
                            evaluators.push(Evaluator::from_ast(ast));
                            temp_variables.push(FINAL_RESULT_ALIAS.to_string());
                        },
                        Err(e) => return Err(e)
                    }
                },
                (Some(var_id), Some(exp)) => {
                    match parse_expr(&exp, &var) {
                        Ok(ast) => {
                            evaluators.push(Evaluator::from_ast(ast));
                            var.push(var_id.clone());
                            temp_variables.push(var_id);
                        },
                        Err(e) => return Err(e)
                    }
                },
                _ => {
                    // return Err here?
                 }
            }
        }

        Ok(Self {
            expressions: evaluators,
            temporaries: temp_variables
        })
    }


    pub fn evaluate(&mut self, inputs: &InputVars) -> Result<Box<dyn DynMath>, EvaluationError> {
        let mut inputs = inputs.clone();
        for (var_id, exp) in self.temporaries.iter().zip(self.expressions.iter_mut()) {
            match exp.evaluate(&inputs) {
                Ok(result) => {
                    // intentionally not returning early here, even if `var_id == FINAL_RESULT_ALIAS
                    inputs.insert_ref(var_id.clone(), Rc::from(result));
                },
                Err(e) => return Err(e)
            }
        }
        return match inputs.get(FINAL_RESULT_ALIAS) {
            Some(res) => Ok(res.clone_boxed()),
            None => Err(EvaluationError::MissingFinalExpression)
        }
    }
}


fn split_assignement(exp: &str) -> (Option<String>, Option<String>) {
    if !exp.contains("=") {
        return (None, Some(exp.to_string()));
    }
    let mut tmp = exp.splitn( 2, "=");
    let key = match tmp.next() {
        Some(s) => Some(s.trim().to_string()),
        None => None,
    };
    let value = tmp.next().map(str::to_string);
    (key, value)
}

fn parse_expr(expression: &str, variables: &[String]) -> Result<AST, DymexError> {
    let v = &variables.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
    let ts = match TokenStream::new(expression, v) {
        Ok(ts) => ts,
        Err(err) => return Err(DymexError::LexicalError(err))
    };

    match AST::new(ts) {
        Ok(ast) => Ok(ast),
        Err(err) => Err(DymexError::ParsingError(err))
    }
}