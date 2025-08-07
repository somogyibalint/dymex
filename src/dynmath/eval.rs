
use std::{collections::HashMap, hash::Hash};
use std::fmt::{format, Display};
use std::rc::Rc;
use itertools::join; //TODO: remove later
use crate::*;



pub struct Evaluator {
    values: HashMap<u16, Rc<dyn DynMath>>,
    expressions: HashMap<u16, Evaluand>,
    aliases: HashMap<String, u16>
}

impl Evaluator {
    pub fn new(expression: &str, variables: &[&str]) -> Result<Self, ParsingError> {
        let mut ts = TokenStream::new();
        ts.update(expression, variables);
        let mut ast = AST::new(ts);
        if let Err(err) = ast.parse_tokens() {
            return Err(err);
        }
        Ok(Self::from_ast(ast))
    }

    fn from_ast(ast: AST) -> Self {
        let (val, aliases, expr) = flatten_tree(ast);     
        Self {
            values: val,
            expressions: expr,
            aliases: aliases
        }
    }

    pub fn evaluate(&mut self, inputs: &HashMap<String, Rc<dyn DynMath>>) -> Result<Rc<dyn DynMath>, EvaluationError> {
        for (varname, value) in inputs.iter() {
            let id = self.aliases.get(varname).unwrap();
            self.values.insert(*id, value.clone());
        };

        let mut eval_order = self.expressions.keys()
            .copied()
            .collect::<Vec<u16>>();
        eval_order.sort_by(|a, b| b.cmp(a));

        let final_result_id = eval_order.last().unwrap();
        for expr_id in &eval_order {
            let result = self.expressions[&expr_id].eval(&self.values);
            match result {
                Err(e) => return Err(e),
                Ok(res) => {
                    self.values.insert(*expr_id, Rc::from(res)); 
                }
            }
        }
        Ok(self.values[final_result_id].clone())
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
                    //RFO: here we insert a new value for every occurance of the same number/constant/var 
                    match a.token.to_owned() {
                        Token::Const(c) => {
                            println!("Insert const {} {}", id, c.value()); // DEBUG 
                            values.insert(id,Rc::new(c.value()));
                        }
                        Token::Number(x) => {
                            println!("Insert number {} {}", id, x); // DEBUG 
                            values.insert(id,Rc::new(x));
                        }
                        Token::Var(v) => {
                            println!("Insert var {} {}", id, v); // DEBUG 
                            aliases.insert( v, id);
                        }
                        _ => panic!("Unexpected token in transform_tree(). This is likely a bug!")
                    };
                },
                Branch::Expression(exp, args) => {
                    // this is not good: if one arg is a const/number/variable id already exists!
                    // let arg_ids: Vec<u16> = args.iter().map(|_| id_gen.get_id()).collect();
                    // instead this complicated mess:
                    let mut arg_ids: Vec<u16> = Vec::new();
                    for arg in args {
                        if let Branch::Atom(at) = arg 
                        && let Token::Var(v) = &at.token {
                            match aliases.get(v) {
                                None => arg_ids.push(id_gen.get_id()),
                                Some(id)  => arg_ids.push(*id)
                            }
                        } else {
                            arg_ids.push(id_gen.get_id());
                        }
                    }

                    let eval = Evaluand {
                            op: exp.to_owned(), 
                            args: arg_ids.to_owned() 
                        };
                    println!("Insert expr {} {}", id, exp.token); // DEBUG 
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
        
        let get_val = |id| &*values[id];
        
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
                    println!("{}", values.iter().map(|(k,_)| format!("{}", k)).collect::<Vec<String>>().join(", "));
                    println!("get id {}", &self.args[0]); // debug
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