// TokenStream -> AST utilizing a Pratt parser

// use std::collections::HashMap;
use std::{collections::HashMap, fmt::Write};
use colored::{Colorize, Color};
use crate::{ArithmeticOperator, AssignmentOperator, Token, TokenContext, TokenStream};


/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsingError {
    UnexpectedToken(usize),
    UnexpectedLP(usize), 
    MissingRP(i32), 
    MissingArgument(usize),
    TooManyArguments(usize),
    InvalidOperation(usize, String),
    NotImplemented(String)
}

/// AST
pub struct AST {
    ts: TokenStream,
    pub tree: Option<Branch>
}

impl AST {
    pub fn new(ts: TokenStream) -> Self {
        Self {
            ts,
            tree: None
        }
    }

    pub fn update_tokens(&mut self, new_ts: TokenStream) {
        if self.ts.identical(&new_ts) { return; }
        self.ts = new_ts;
        self.tree = None;
    }

    pub fn parse_tokens(&mut self) -> Result<(), ParsingError> {
        if let Err(e) = self.check_parens() {
            return Err(e);
        }
        if let Err(e) = self.check_tokens() {
            return Err(e);
        }
        match parse_tokenstream(&mut self.ts) {
            Err(e) => return Err(e),
            Ok(branch) => self.tree = Some(branch)
        }
        Ok(())
    }

    fn check_parens(&self) -> Result<(), ParsingError> {
        let mut n: i32 = 0;
        for tc in &self.ts.tokens {
            let token = &tc.token;
            match token {
                Token::LP => n += 1,
                Token::RP => n -= 1,
                _ => {}
            }
            if n == -1 {return Err(ParsingError::UnexpectedLP(tc.at))}
        }
        match n {
            0 => Ok(()),
            _ => Err(ParsingError::MissingRP(n))
        }
    } 

    fn check_tokens(&self) -> Result<(), ParsingError> {
        for tc in &self.ts.tokens {
            let token = &tc.token;
            match token {
                Token::AssignOp(x) if *x != AssignmentOperator::Assign => {
                    return Err(ParsingError::InvalidOperation(tc.at, "Only simple assignment is allowed.".into()));
                }
                Token::LogicOp(_) => {
                    return Err(ParsingError::InvalidOperation(tc.at, "Logical operators are not allowed.".into()));
                }
                Token::RelOp(_) => {
                    return Err(ParsingError::InvalidOperation(tc.at, "Comparison operators are not allowed.".into()));
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn rpn_repr(&self) -> String {
        match &self.tree {
            Some(t) => t.as_rpn_str(),
            None => "".into()
        }
    }

    pub fn flatten_ast(&self) -> FlatAst {
        let mut ast = FlatAst::new();
        match &self.tree {
            None => {
                panic!("AST is empty!");
            },
            Some(branch) => {
                traverse_ast(branch, &mut ast, 0);
            }
        }
        ast
    }

}


/// Lisp S-expression representing the AST
/// atom: number, constant or input variable
/// Expression: an operation (head) and the operands (sub-branches)
#[derive(Debug, PartialEq, Clone)]
pub enum Branch {
    Atom(TokenContext),
    Expression(TokenContext, Vec<Branch>)
}

impl Branch {
    /// Print the (sub)tree in reverse polish notation.
    pub fn as_rpn_str(&self) -> String {
        let mut s = String::new();
        self.recurse_tree(&mut s)
    }

    /// Print expression with syntax highlighting
    // TODO: Currently only parens are colored
    pub fn print_rpn_colored(&self) -> () {
        fn get_paren_color(i: i32) -> Color {
            match i % 7 {
                0 => {Color::Red},
                1 => {Color::Green},
                2 => {Color::Blue},
                3 => {Color::Magenta},
                4 => {Color::Yellow},
                5 => {Color::Cyan},
                _ => {Color::Black}
            }
        }
        let s = self.as_rpn_str();
        let mut i = 0;
        for c in s.chars() {
            match c {
                '(' => {
                    print!("{}", "(".color(get_paren_color(i)));
                    i += 1;
                },
                ')' => {
                    i -= 1;
                    print!("{}", ")".color(get_paren_color(i)));
                },
                _ => {print!("{}", c)}
            }
        }
        print!("\r\n");
    }

    fn recurse_tree(&self, s: &mut String) -> String {
        match self {
            Self::Atom(tc) => write!(s, "{}", tc.token).unwrap(),
            Self::Expression(tc, children) => {
                write!(s, "({}: ", tc.token).unwrap();
                for branch in children {
                    branch.recurse_tree(s);
                    write!(s, ", ").unwrap();
                }
                s.pop(); s.pop();
                write!(s, ")").unwrap()
            }
        }
        s.clone()
    }

}


/// This function build the AST from the provided TokenStream
fn parse_tokenstream(ts: &mut TokenStream) -> Result<Branch, ParsingError> {
    pratt_parser(ts, 0)
} 

/// Pratt-parser inspired by: matklad's "Simple but Powerful Pratt Parsing"
/// See: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn pratt_parser(ts: &mut TokenStream, min_precedence: usize) -> Result<Branch, ParsingError> {
    let next = ts.next();

    let mut lhs = match next.token {
        // atom -> move to loop
        Token::Var(_) | Token::Const(_) | Token::Number(_) => {
            Branch::Atom(next.clone())
        }
        // (    -> recursion
        Token::LP => {
            let res = pratt_parser(ts, 0);
            match res {
                Ok(lhs) if ts.next().token == Token::RP =>  lhs,
                Ok(_) => return Err(ParsingError::MissingRP(1)), // ! FIXME: 
                Err(e) => return Err(e),
            }
        }
        // found a function
        Token::Func(_, _) => {
            let mut args = Vec::<Branch>::new();
            if ts.next().token != Token::LP {
                return Err(ParsingError::UnexpectedToken(1))
            }
            loop {
                let res = pratt_parser(ts, 0);
                match res {
                    Ok(arg) => {args.push(arg)},
                    Err(e) => {return Err(e);}
                }
                let next = ts.next(); 
                match next.token {
                    Token::RP => { break; },
                    Token::Comma => {},
                    _ => return Err(ParsingError::UnexpectedToken(next.at))
                };
            }
            if args.len() == 0 { 
                return Err(ParsingError::MissingArgument(next.at));
            }
            Branch::Expression(next.clone(), args)
        }

        // operator -> recursion
        _=> {
            if let Some((_, r_bp)) = prefix_precedence(&next.token) {
                let rhs = pratt_parser(ts, r_bp).unwrap();
            Branch::Expression(next, vec![rhs])
            } else {
                return Err(ParsingError::UnexpectedToken(next.at)); // prefix operator that is not + - 
            }
        }
    };

    loop {
        let peeked = ts.peek();
        let op = match peeked.token.clone() {
            Token::Eof => break,
            Token::Number(_) | Token::Const(_) | Token::Var(_) => 
                return Err(ParsingError::UnexpectedToken(peeked.at)),
            t => t,
        };

        // postfix
        if let Some((l_bp, _)) = postfix_precedence(&op) { 
            if l_bp < min_precedence {
                break;
            }
            ts.next();
            lhs = if op == Token::LB {
                if let Ok(rhs) = pratt_parser(ts, 0) {
                    assert_eq!(ts.next().token, Token::RB); // TODO: fix error handling
                    Branch::Expression(peeked, vec![lhs, rhs])
                } else {
                    return Err(ParsingError::UnexpectedToken(peeked.at));
                }
            } else {
                Branch::Expression(peeked, vec![lhs])
            };
            continue;
        }

        // infix
        if let Some((l_bp, r_bp)) = infix_precedence(&op) {
            if l_bp < min_precedence {
                break;
            }
            ts.next();

            lhs = if let Ok(rhs) = pratt_parser(ts, r_bp) {
                Branch::Expression(peeked, vec![lhs, rhs])
            } else {
                return Err(ParsingError::UnexpectedToken(peeked.at));
            };
            continue;    
        }

        break;
    }
    Ok(lhs)
} 

// Field expressions: left to right
// Function calls, array indexing	
// ** 
// * / %	left to right
// + -	left to right binary and unary
// == != < > <= >=	Require parentheses
// &&	left to right
// ||	left to right
// .. ..=	Require parentheses
// = += -= *= /= %=

fn prefix_precedence(t: & Token) -> Option<(usize, usize)> {
    use ArithmeticOperator as AO;
    match t {
        Token::ArOp(o) => match o {
            AO::Plus | AO::Minus => Some((0, 9)),
            _ => None,
        },
        _ => None
    }
}

fn postfix_precedence(t: & Token) -> Option<(usize, usize)> {
    match t {
        Token::LB => Some((11, 0)),
        _ => None
    }
}

fn infix_precedence(t: & Token) -> Option<(usize, usize)> {
    use ArithmeticOperator as AO;
    match t {
        Token::ArOp(o) => match o {
            AO::Plus | AO::Minus => Some((10, 11)),
            AO::Mul | AO::Div => Some((12, 13)),
            AO::Pow => Some((14, 15)),
            _ => None,
        },
        Token::RelOp(_) => Some((2, 1)),
        Token::LogicOp(_) => Some((4, 3)),
        Token::AssignOp(_) => Some((2, 1)),
        Token::Dot => Some((14, 13)),
        Token::Colon => Some((6, 5)),
        _ => None
    }
}

fn is_atom(t: & Token) -> bool {
    match t {
        Token::Var(_) | Token::Number(_) | Token::Const(_) => true,
        _=> false
    }
}


/// Non-recursive representation of the AST
/// 
pub struct FlatAst {
    nodes: HashMap<u8, TokenContext>,
    edges: HashMap<u8, u8>,
    node_id: u8,
}
impl FlatAst {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_id: 0,
        }
    }
    fn add_node(&mut self, tc: TokenContext, parent: u8) -> u8 {
        let id = self.node_id; 
        self.nodes.insert(id, tc);
        self.node_id += 1;        
        self.add_edge(parent, id);
        id
    }
    fn add_edge(&mut self, op: u8, arg: u8) {
        self.edges.insert(arg, op);
    }
    fn print_ast(&self) {
        println!("Nodes:");
        for (node_id, node) in &self.nodes {
            println!("  {node_id:3}: {}", node.token);
        }
        println!("Edges:");
        for (fr, to) in &self.edges {
            println!("  {fr:3} -> {to:3}");
        }
    }
}

fn traverse_ast(branch: &Branch, ast: &mut FlatAst, parent: u8) {
    match branch {
        Branch::Atom(tc) => {
            let _ = ast.add_node(tc.clone(), parent);
        },
        Branch::Expression(tc, args) => {
            let id = ast.add_node(tc.clone(), parent);
            for arg in args {
                traverse_ast(arg, ast, id);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::*;
    use super::Branch;

    fn test_parsing(expr: &str, var: &[&str], rpn: &str) {
        let mut ts = TokenStream::new();
        ts.update(expr, var);
        let mut ast = AST::new(ts);
        let success = ast.parse_tokens();
        assert_matches!(success, Ok(()));
        assert_eq!(ast.rpn_repr(), rpn);
    }

    #[test]
    fn test_rpn() {

        // RPN repr for: (1 + max(2,3,4) * 5 - π²) / (r.x + 1.23)
        // not a real test.
        use TokenContext as TC;
        let a1 = Branch::Atom(TC::dummy(Token::Number(1.0)));
        let a2 = Branch::Atom(TC::dummy(Token::Number(2.0)));
        let a3 = Branch::Atom(TC::dummy(Token::Number(3.0)));
        let a4 = Branch::Atom(TC::dummy(Token::Number(4.0)));
        let a5 = Branch::Atom(TC::dummy(Token::Number(5.0)));
        let a6 = Branch::Atom(TC::dummy(Token::Const(Constant::Pi)));
        let a7 = Branch::Atom(TC::dummy(Token::Number(2.0)));

        let a8 = Branch::Atom(TC::dummy(Token::Var("r".into())));
        let a9 = Branch::Atom(TC::dummy(Token::Var("x".into())));
        let a10 = Branch::Atom(TC::dummy(Token::Number(1.23)));

        let ex1 = Branch::Expression(TC::dummy(Token::Func(Function::Max, 10)), vec![a2, a3, a4]);
        let ex2 = Branch::Expression(TC::dummy(Token::ArOp(ArithmeticOperator::Pow)), vec![a6, a7]);
        let ex3 = Branch::Expression(TC::dummy(Token::Dot), vec![a8, a9]);

        let ex4 = Branch::Expression(TC::dummy(Token::ArOp(ArithmeticOperator::Mul)), vec![ex1, a5]);
        let ex5 = Branch::Expression(TC::dummy(Token::ArOp(ArithmeticOperator::Plus)), vec![ex3, a10]);

        let ex6 = Branch::Expression(TC::dummy(Token::ArOp(ArithmeticOperator::Plus)), vec![a1, ex4]);
        let ex7 = Branch::Expression(TC::dummy(Token::ArOp(ArithmeticOperator::Plus)), vec![ex6, ex2]);

        let full_expr = Branch::Expression(TC::dummy(Token::ArOp(ArithmeticOperator::Div)), vec![ex7, ex5]);
        full_expr.print_rpn_colored();
        let s = full_expr.as_rpn_str();

        let num_lp = s.chars().filter(|c| *c == '(').count();
        assert_eq!(num_lp, 8);
    }

    #[test]
    fn test_simple_expressions() {

        test_parsing("1 + 2 * 3", &vec![], "(+: 1, (*: 2, 3))");
        test_parsing("(1 + x) * 3", &vec!["x"], "(*: (+: 1, x), 3)");
        test_parsing("((pi + x)**2 - 3) / 3", &vec!["x"], "(/: (-: (**: (+: π, x), 2), 3), 3)");
    }

    #[test]
    fn test_simple_functions() {
        test_parsing("max(0, sqrt(min(1,2,3,4)))", &vec![], "(Max: 0, (Sqrt: (Min: 1, 2, 3, 4)))");   
    }

    #[test]
    fn test_indexing() {
        test_parsing("v[1:-1]", &vec!["v"], "([: v, (:: 1, -1))");
    }

    #[test]
    fn test_get_field() {
        test_parsing("r.x - x0", &vec!["r", "x0"], "(-: (.: r, x), x0)");   
    }

    #[test]
    fn test_flattened_ast() {
        let expr = "x + max(0, sqrt(min(1,2,3,4)))";
        let var =  &vec!["x"];
        let mut ts = TokenStream::new();
        ts.update(expr, var);
        let mut ast = AST::new(ts);
        let _ = ast.parse_tokens();

        let flat_ast = ast.flatten_ast();
        println!("Expression: {}", expr);
        flat_ast.print_ast();
    }

}