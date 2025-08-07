
use std::{collections::HashMap, hash::Hash};
use std::fmt::{format, Display};
use std::rc::Rc;
use crate::*;


pub struct Evaluator {
    values: HashMap<u16, Rc<dyn DynMath>>,
    expressions: HashMap<u16, Evaluand>,
    aliases: HashMap<String, u16>
}

impl Evaluator {
    fn new(ast: AST) -> Self {

        let (val, aliases, expr) = flatten_tree(ast);     
        Self {
            values: val,
            expressions: expr,
            aliases: aliases
        }
    }

    fn evaluate(&mut self, inputs: &HashMap<String, Rc<dyn DynMath>>) -> Result<Box<dyn DynMath>, EvaluationError> {
        for (varname, value) in inputs.iter() {
            let id = self.aliases.get(varname).unwrap();
            self.values.insert(*id, value.clone());
        };

        let mut eval_order = self.expressions.keys()
            .copied()
            .collect::<Vec<u16>>();
        eval_order.sort_by(|a, b| b.cmp(a));

        for expr_id in eval_order {
            let result = self.expressions[&expr_id].eval(&self.values);
            match result {
                Err(e) => return Err(e),
                Ok(res) => {
                    self.values.insert(expr_id, Rc::from(res)); 

                }
            }
            
        }
        todo!();
    }
}


struct IdGenerator {
    _id: u16
}
impl IdGenerator {
    fn new() -> Self { Self {_id : 0 } }
    fn get_id(&mut self) -> u16 {
        let value = self._id;
        self._id += 1;
        value
    }
}


fn flatten_tree(ast: AST)
    -> (HashMap<u16, Rc<dyn DynMath>>, 
        HashMap<String, u16>, 
        HashMap<u16, Evaluand>) {

    let mut id_gen = IdGenerator::new();
    
    // variables, constants and _evaluated_ results
    let mut values: HashMap<u16, Rc<dyn DynMath>> = HashMap::new();
    // mapping between variable name and id
    let mut aliases: HashMap<String, u16> = HashMap::new();
    // evaluands: only expressions!
    let mut evaluands: HashMap<u16, Evaluand> = HashMap::new();

    fn recurse_tree(tree: &Branch, 
        values: &mut HashMap<u16, Rc<dyn DynMath>>, 
        expressions: &mut HashMap<u16, Evaluand>,
        aliases: &mut HashMap<String, u16>,
        id_gen: &mut IdGenerator,
        id: u16) {
            match tree {
                Branch::Atom(a) => {
                    
                    match a.token.to_owned() {
                        Token::Const(c) => {
                            values.insert(id,Rc::new(c.value()));
                        }
                        Token::Number(x) => {
                            values.insert(id,Rc::new(x));
                        }
                        Token::Var(v) => {
                            aliases.insert( v, id);
                        }
                        _ => panic!("Unexpected token in transform_tree(). This is likely a bug!")
                    };
                },
                Branch::Expression(exp, args) => {
                    let arg_ids: Vec<u16> = args.iter().map(|_| id_gen.get_id()).collect();
                    let eval = Evaluand {
                            op: exp.to_owned(), 
                            args: arg_ids.to_owned() 
                        };
                    expressions.insert( id, eval);


                    for (id, arg) in arg_ids.iter().zip(args.iter()) {
                        recurse_tree(
                            arg, 
                            values, 
                            expressions, 
                            aliases,
                            id_gen, 
                            *id
                        );
                    }
                }
            }
        } 
    
    if let Some(tree) = ast.tree {
        let id = id_gen.get_id();
        recurse_tree(
            &tree, 
            &mut values, 
            &mut evaluands, 
            &mut aliases, 
            &mut id_gen, 
            id
        );
    } else {
        // empty expression
    }
    
    (values, aliases, evaluands)
}


// by limiting args, this could be kept on the stack
struct Evaluand {
    op: TokenContext,
    args: Vec<u16> 
}

impl Evaluand {    
    fn eval(&self, values: &HashMap<u16, Rc<dyn DynMath>>) -> Result<Box<dyn DynMath>, EvaluationError> {
        use ArithmeticOperator as AO;
        
        let get_val = |id| &**values.get(id).unwrap();
        
        match &self.op.token {
            Token::ArOp(op) => {
                // debug_assert is enough here if the parser works
                debug_assert_eq!(self.args.len(), 2);
                let lhs = get_val(&self.args[0]);
                let rhs = get_val(&self.args[1]);
                match op {
                    AO::Plus => return lhs.add(rhs),
                    AO::Minus => return lhs.sub(rhs),
                    AO::Mul => return lhs.mul(rhs),
                    AO::Div => return lhs.div(rhs),
                    AO::Pow => return lhs.pow(rhs),
                    _ => panic!("TODO: dkfskjkew")
                }
            }
            Token::Func(fun, max_args) => {
                debug_assert!(self.args.len() <= *max_args);
                if *max_args == 1 {
                    let arg = get_val(&self.args[0]);
                    match fun {
                        Function::Sin => return arg.dyn_sin(),
                        Function::Cos => return arg.dyn_cos(),
                        Function::Tan => return arg.dyn_tan(),
                        Function::Cot => return arg.dyn_cot(),
                        Function::Exp => return arg.dyn_exp(),
                        Function::Log => return arg.dyn_log(),
                        Function::Log2 => return arg.dyn_log2(),
                        Function::Log10 => return arg.dyn_log10(),
                        Function::Sqrt => return arg.dyn_sqrt(),
                        _ => panic!("ERROR: {} should have only a single parameter!", fun)
                    }
                } else {
                    let args: Vec<Rc<dyn DynMath>> = self.args.iter().map(
                            |id| values.get(id).unwrap().clone()
                        ).collect();

            
                    let result = match fun {
                        Function::Min => dynmath_min(&args),
                        Function::Max => dynmath_max(&args),
                        Function::Avg => dynmath_avg(&args),
                        Function::Std => dynmath_std(&args),
                        Function::Sum => dynmath_sum(&args),
                        Function::Range => dynmath_range(&args),
                        _ => panic!("ERROR: {} is variadic!", fun)
                    };
                    match result {
                        Err(err) => Err(err),
                        Ok(f) => Ok(Box::new(f))
                    }
                }
                    
            }
            _ => panic!("wtf")
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn test_addition() {
        let token = Token::ArOp(ArithmeticOperator::Plus);
        let tc = TokenContext {token: token, at: 0, len: 0 };
        let eval = Evaluand {
            op: tc,
            args: vec![0,1] 
        };

        let mut variables: HashMap<u16, Rc<dyn DynMath>> = HashMap::new();
        variables.insert(0, Rc::new(1.0));
        variables.insert(1, Rc::new(2.0));

        let res = eval.eval(&variables).unwrap();
        assert_eq!(res.as_number(), 3.0);
    }

    #[test]
    fn test_variadic_fnc1() {
        let token = Token::Func(Function::Max, 10);
        let tc = TokenContext {token: token, at: 0, len: 0};
        let eval = Evaluand {
            op: tc,
            args: vec![0, 1, 3, 5, 6,]
        };
        let mut variables: HashMap<u16, Rc<dyn DynMath>> = HashMap::new();
        let test_val = [3.4, 0.0, 99.0, 16.0, 3.0, 2.0, -99.0, 1.0, -1.0];
        for (i, x) in test_val.iter().enumerate() {
            variables.insert(i as u16, Rc::new(*x));
        }
        let res = eval.eval(&variables).unwrap();
        assert_eq!(res.as_number(), 16.0);
    }

    
    #[test]
    fn test_variadic_fnc2() {
        let token = Token::Func(Function::Max, 10);
        let tc = TokenContext {token: token, at: 0, len: 0};
        let eval = Evaluand {
            op: tc,
            args: vec![0]
        };
        let mut variables: HashMap<u16, Rc<dyn DynMath>> = HashMap::new();
        let vector = vec![-16.0, -4.0, 0.0, 4.0, 8.0];
        
        variables.insert(0, Rc::new(vector));
        
        let res = eval.eval(&variables).unwrap();
        assert_eq!(res.as_number(), 8.0);
    }
}