use std::rc::Rc;
use crate::*;


#[derive(Clone)]
pub struct MultiExpEvaluator {
    expressions: Vec<Evaluator>,
}


impl MultiExpEvaluator {

    pub fn new<S: AsRef<str>>(expression: &str, variables: &[S]) -> Result<Self, DymexError> {
        let asts = ASTSeries::new(expression)?;
        asts.check_input_vars(variables)?;

        Ok( Self {
            expressions: asts.forest()
            .iter()
            .map(|ast| Evaluator::from_ast(&ast))
            .collect()
        })

    }

    pub fn evaluate(&mut self, inputs: &InputVars) -> Result<Box<dyn DynMath>, EvaluationError> {
        let mut inputs = inputs.clone();
        let mut final_result = None;
        for exp in self.expressions.iter_mut() {
            match (exp.assigned_to(), exp.evaluate(&inputs)) {
                (Some(var), Ok(result)) => {
                    inputs.insert_ref(var, Rc::from(result));
                },
                (None, Ok(result)) => {
                    final_result = Some(result)
                }
                (_, Err(e)) => return Err(e)
            }
        }
        return match final_result {
            Some(res) => Ok(res.clone_boxed()),
            None => Err(EvaluationError::MissingFinalExpression)
        }
    }
}

