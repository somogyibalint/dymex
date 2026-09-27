/// Turn a stream of tokens into an anstract syntax tree

use std::{collections::{HashMap, VecDeque}, fmt::Write, iter::Peekable};
use colored::{Colorize, Color};
use crate::{ArithmeticOperator, AssignmentOperator, Token, TokenContext, TokenStream, TokenizedLines, is_ident_char, Method};
use error::{VARNAME_ERR1, VARNAME_ERR2, VARNAME_ERR3};
mod latex;
pub use latex::*;
mod error;
pub use error::ParsingError;
mod mermaid;
pub use mermaid::*;

const RESERVED_IDS: [&str; 19] = ["min", "max", "avg", "mean", "std", "sin", "cos", "abs",
"tan", "cotan", "exp", "log", "log2", "log10", "sqrt", "pi", "e", "sqrt2", "sqrt3"];

/// Abstract syntax tree
#[derive(Debug, Clone)]
pub struct AST {
    pub tree: Branch,
    pub assigned_to: Option<String>
}

impl AST {
    pub fn new(mut ts: TokenStream) -> Result<Self, ParsingError> {

        if let Err(e) = Self::check_parens(&ts) {
            return Err(e);
        }
        if let Err(e) = Self::check_tokens(&ts) {
            return Err(e);
        }
        ts = AST::run_pre_parsing_passes(ts);
        let tree = match pratt_parser(&mut ts, 0) {
            Err(e) => return Err(e),
            Ok(branch) => Some(branch)
        };

        let ast = AST {
            tree: tree.unwrap(),
            assigned_to: None
        };

        ast.check_assigment()
    }

    pub fn from_expression(expr: &str) -> Result<Self, ParsingError> {
        match TokenStream::new(expr) {
            Ok(ts) => Self::new(ts),
            Err(e) => Err(ParsingError::LexingError(e))
        }
    }

    pub fn rpn_repr(&self) -> String {
        self.tree.as_rpn_str()
    }

    pub fn flatten_ast(&self) -> FlatAst {
        let mut ast = FlatAst::new();
        traverse_ast(&self.tree, &mut ast, 0);
        ast
    }

    fn check_parens(ts: &TokenStream) -> Result<(), ParsingError> {
        let mut n: i32 = 0;
        for tc in ts.tokens() {
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

    fn check_tokens(ts: &TokenStream) -> Result<(), ParsingError> {
        for tc in ts.tokens() {
            let token = &tc.token;
            match token {
                // TODO: config maybe
                // Token::AssignOp(x) if *x != AssignmentOperator::Assign => {
                //     return Err(ParsingError::InvalidOperation(tc.at, "Only simple assignment is implemented.".into()));
                // }
                Token::LogicOp(_) => {
                    return Err(ParsingError::InvalidOperation(tc.at, "Logical operators are not implemented.".into()));
                }
                Token::RelOp(_) => {
                    return Err(ParsingError::InvalidOperation(tc.at, "Comparison operators are not implemented.".into()));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_assigment(mut self) -> Result<Self, ParsingError> {
        // Check for top level assignement, and transform self accordingly
        //     Self {tree: `varname = expression`, assigned_to = None}
        //                         ⇓  ⇓  ⇓
        //     Self {tree: `expression`, assigned_to = Some(varname)}
        match &self.tree {
            Branch::Atom(_) => {},
            Branch::Expression(tc, children ) => {
                match &tc.token {
                    Token::AssignOp(op) if *op == AssignmentOperator::Assign => {
                        match (children.get(0), children.get(1), children.get(2)) {
                            (Some(lhs), Some(rhs), None) => {
                                let assigned_to = if let Branch::Atom(lhs) = lhs
                                    && let Token::Var(varname) = &lhs.token {
                                        varname
                                } else {
                                    return Err(ParsingError::InvalidAssignment("Only assignement to variables is supported".to_string(), tc.at));
                                };
                                self.assigned_to = Some(assigned_to.clone());
                                self.tree = rhs.clone(); // unnecessary clone, non-trivial to circumvent
                            },
                            _ => panic!("Assignement should have exactly two arguments, found instead {}", children.len())
                        }
                    }
                    _ => {}
                }
            }
        }
        // Check for deeper assignements
        for branch in self.tree.iter_dfs() {
            if let Branch::Expression(tc, _ ) = branch
            && let Token::AssignOp(assignment) = &tc.token
            && let AssignmentOperator::Assign = assignment {
                return Err(ParsingError::InvalidAssignment("Only top level assignement is supported".to_string(), tc.at));
            }
        }
        Ok(self)
    }

    fn run_pre_parsing_passes(ts: TokenStream) -> TokenStream {
        first_indexing_pass(ts)
    }
    ///
    pub fn check_input_vars<S: AsRef<str>>(&self, inputs: &[S]) -> Result<(), ParsingError> {
        let variables: Vec<&str> = inputs.iter().map(|s| s.as_ref()).collect();
        if let Err(err) = check_for_invalid_inputs(&variables) {
            return Err(err);
        }
        for t in self.tree.iter_dfs() {
            match &t.tc().token {
                Token::Var(varname) => {
                    if !variables.contains(&varname.as_str()) {
                        return Err(ParsingError::UndefinedVariable(varname.clone(), t.tc().at));
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }
}


/// Abstract syntax tree series
#[derive(Clone)]
pub struct ASTSeries {
    forest: Vec<AST>
}

impl ASTSeries {
    // TODO unify contructor api with AST
    pub fn new(multi_line_expr: &str) -> Result<Self, ParsingError> {
        let tss = TokenizedLines::new(multi_line_expr);
        match tss {
            Err(err) => Err(ParsingError::LexingError(err)),
            Ok(tss) => {
                let mut parsed_expr = Vec::new();
                for ts in tss.lines() {
                    match AST::new(ts.clone()) {
                        Err(err) => return Err(err),
                        Ok(ast) => parsed_expr.push(ast),
                    };
                }
                Ok(Self{forest: parsed_expr})
            }
        }
    }

    pub fn check_input_vars<S: AsRef<str>>(&self, inputs: &[S]) -> Result<(), ParsingError> {
        let mut variables: Vec<&str> = inputs.iter().map(|s| s.as_ref()).collect();
        for ast in &self.forest {
            if let Err(err) =  ast.check_input_vars(&variables) {
                return  Err(err);
            }
            if let Some(tmp_var) = &ast.assigned_to {
                variables.push(&tmp_var);
            }
        }
        Ok(())
    }
    pub fn forest(&self) -> &[AST] {
        &self.forest
    }
}


/// Resursive data structure for the AST
/// Lisp S-expression representing the AST atom: number, constant or input variable
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
        self.recurse_tree_rpn(&mut s)
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

    pub fn tc(&self) -> &TokenContext {
        match self {
            Branch::Atom(tc) => &tc,
            Branch::Expression(tc, _) => &tc
        }
    }

    fn recurse_tree_rpn(&self, s: &mut String) -> String {
        match self {
            Self::Atom(tc) => write!(s, "{}", tc.token).unwrap(),
            Self::Expression(tc, children) => {
                write!(s, "({}: ", tc.token).unwrap();
                for branch in children {
                    branch.recurse_tree_rpn(s);
                    write!(s, ", ").unwrap();
                }
                s.pop(); s.pop(); // remove trailing ", "
                write!(s, ")").unwrap()
            }
        }
        s.clone()
    }

    fn iter_dfs(&self) -> DFSBranchIter<'_>
    where
    Self: Sized,
    {
        DFSBranchIter::new(self)
    }

    fn iter_bfs(&self) -> BFSBranchIter<'_>
    where
    Self: Sized,
    {
        BFSBranchIter::new(self)
    }
}



pub struct DFSBranchIter<'a> {
    queue: VecDeque<&'a Branch>,
}
pub struct BFSBranchIter<'a> {
    queue: VecDeque<&'a Branch>,
}

impl<'a> DFSBranchIter<'a> {
    pub fn new(root: &'a Branch) -> Self {
        Self { queue: VecDeque::from(vec![root]) }
    }
}
impl<'a> BFSBranchIter<'a> {
    pub fn new(root: &'a Branch) -> Self {
        Self { queue: VecDeque::from(vec![root]) }
    }
}

impl<'a,> Iterator for BFSBranchIter<'a> {
    type Item = &'a Branch;

    fn next(&mut self) -> Option<Self::Item> {
        let branch = self.queue.pop_front()?;
        match branch {
            Branch::Atom(_) => {},
            Branch::Expression(_, children) => {
                self.queue.extend(children);
            }
        }
        Some(branch)
    }
}

impl<'a,> Iterator for DFSBranchIter<'a> {
    type Item = &'a Branch;
    fn next(&mut self) -> Option<Self::Item> {
        let branch = self.queue.pop_front()?;
        match branch {
            Branch::Atom(_) => {},
            Branch::Expression(_, children)  => {
                for child in children.iter().rev() {
                    self.queue.push_front(child);
                }
            },
        }
        Some(branch)
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
        Token::Var(_) | Token::Const(_) | Token::Number(_) | Token::Attr(_) => {
            Branch::Atom(next.clone())
        }
        // (    -> recursion
        Token::LP => {
            let res = pratt_parser(ts, 0);
            match res {
                Ok(lhs) if ts.next().token == Token::RP => lhs,
                Ok(_) => return Err(ParsingError::MissingRP(1)), // ! FIXME: location info?
                Err(e) => return Err(e),
            }
        }
        // found a function
        Token::Func(_, _) | Token::Method(_)=> {
            let mut args = Vec::<Branch>::new();
            let should_be_lp = ts.next();
            if should_be_lp.token != Token::LP {
                return Err(ParsingError::UnexpectedToken(should_be_lp, 10)) // ! missing location info, should be missing LP instead
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
                    _ => return Err(ParsingError::UnexpectedToken(next, 11))
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
                let try_rhs = pratt_parser(ts, r_bp);
                match try_rhs {
                    Ok(rhs) => return Ok(Branch::Expression(next, vec![rhs])),
                    Err(err) => return Err(err)
                }
            } else {
                return Err(ParsingError::UnexpectedToken(next, 12)); // prefix operator that is not + -
            }
        }
    };

    loop {
        let peeked = ts.peek();
        println!("peeked: {:?}", peeked);
        let op = match peeked.token.clone() {
            Token::Eof => break,
            Token::Number(_) | Token::Const(_) | Token::Var(_) =>
                return Err(ParsingError::UnexpectedToken(peeked, 13)),
            t => t,
        };

        // postfix
        if let Some((l_bp, _)) = postfix_precedence(&op) {
            if l_bp < min_precedence {
                break;
            }
            ts.next();
            lhs = Branch::Expression(peeked, vec![lhs]);

            // old indexing parser:
            // lhs = if op == Token::LB {
            //     if let Ok(rhs) = pratt_parser(ts, 0) {
            //         assert_eq!(ts.next().token, Token::RB); // TODO: fix error handling
            //         Branch::Expression(peeked, vec![lhs, rhs])
            //     } else {
            //         return Err(ParsingError::UnexpectedToken(peeked.at));
            //     }
            // } else {
            //     Branch::Expression(peeked, vec![lhs])
            // };
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
                println!(">> : {:?} {:?}", next, peeked);
                return Err(ParsingError::UnexpectedToken(peeked, 14));
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

/// Check if there is an input variable with a name that collides with reserved words.
fn check_for_invalid_inputs(variables: &[&str]) -> Result<(), ParsingError> {
    for var_name in variables {
        if RESERVED_IDS.contains(var_name) {
            return Err(ParsingError::InvalidVariableName((*var_name).into(), VARNAME_ERR1));
        }
        if !var_name.chars().all(|x| is_ident_char(x) ) {
            return Err(ParsingError::InvalidVariableName((*var_name).into(), VARNAME_ERR2));
        }
        let first = var_name.chars().next().unwrap();
        if first.is_ascii_digit() {
            return Err(ParsingError::InvalidVariableName((*var_name).into(), VARNAME_ERR3));
        }
    }
    Ok(())
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



// transform indexing brackets to method syntax for easier parsing:
//   array[ ... ] -> array.index( ... )
fn first_indexing_pass(mut ts: TokenStream) -> TokenStream {
    let mut tmp = Vec::new();
    for tc in ts.tokens() {
        match tc.token {
            Token::LB => {
                tmp.push(TokenContext { token: Token::Dot, at: tc.at, len: 1});
                tmp.push(TokenContext { token: Token::Method(Method::TakeSlice), at: tc.at, len: tc.len });
                tmp.push(TokenContext { token: Token::LP, at: tc.at, len: tc.len });
            },
            Token::RB => tmp.push(TokenContext { token: Token::RP, at: tc.at, len: tc.len }),
            _ => tmp.push(tc.clone()),
        }
    }
    ts.set_tokens(tmp);
    ts
}

// transform methods from operator structure into simple function structure:
//   array . index(...)  =>  index(array, ...)

#[cfg(test)]
mod tests {
    use std::{assert_matches};
    use super::error::{VARNAME_ERR1, VARNAME_ERR2, VARNAME_ERR3};
    use super::check_for_invalid_inputs;
    use crate::parser::first_indexing_pass;
    use crate::*;
    use super::Branch;

    fn test_parsing(expr: &str, input_variables: &[&str], rpn: &str) {
        let ast = AST::from_expression(expr);
        match ast {
            Err(err) => panic!("Parsing failed: {:?}", err),
            Ok(ast) => {
                println!("{}", ast.rpn_repr()); //
                assert_eq!(ast.rpn_repr(), rpn);
                match ast.check_input_vars(input_variables) {
                    Ok(()) => {},
                    Err(err) => panic!("Parsing failed: {:?}", err),
                }
            }
        }
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
    fn test_get_field() {
        test_parsing("r.x - x0", &vec!["r", "x0"], "(-: (.: r, x), x0)");
    }

    #[test]
    #[should_panic]
    fn test_get_field2() {
        test_parsing("x.r - x0", &vec!["r", "x0"], "(-: (.: r, x), x0)");
    }

    #[test]
    fn test_assignment_ok() {
        let ast = AST::from_expression("x = 1 + 2*y").unwrap();
        assert_matches!(ast.check_input_vars(&["y"]), Ok(()));
        assert_matches!(ast.check_input_vars(&["x"]), Err(_));
        assert_eq!(ast.assigned_to, Some("x".to_string()));
        if let Branch::Expression(tc, children) = ast.tree {
            assert_matches!(tc.token, Token::ArOp(ArithmeticOperator::Plus));
            assert_matches!(children.len(), 2);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_assignment_fail() {
        let ast = AST::from_expression("3 * x = y");
        assert_matches!(ast, Err(ParsingError::InvalidAssignment(_, _)));

        let ast = AST::from_expression("x = y = 3");
        assert_matches!(ast, Err(ParsingError::InvalidAssignment(_, _)));

        let ast = AST::from_expression("x + 2*(1 + max(3, y = 2))");
        assert_matches!(ast, Err(ParsingError::InvalidAssignment(_, _)));
    }

    #[test]
    fn test_arithmetic_assignment() {
        let ast = AST::from_expression("x += 1");
        if let Ok(ast) = &ast {
            assert_matches!(ast.check_input_vars(&["x"]), Ok(()));
            assert_matches!(ast.check_input_vars(&["t"]), Err(_));
            assert_eq!(ast.assigned_to, None);
            if let Branch::Expression(tc, _) = &ast.tree {
                assert_matches!(tc.token, Token::AssignOp(AssignmentOperator::PlusEq));
                return;
            }
        }
        panic!()
        // assignment:
        // let id1 = &charslice("new_var == 1 + center");
        // let res1 = parse_identifier(id1, start, &mut input_vars, None);
        // let id2 = &charslice("new_var = 1 + center");
        // let res2 = parse_identifier(id2, start, &mut input_vars, None);
        // let id3 = &charslice("new_var + center");
        // let res3 = parse_identifier(id3, start, &mut input_vars, None);
        // assert_eq!(res1, Err(TokenizerError::UndefinedVariable(start, "new_var".to_string())));
        // assert_eq!(res2, Ok((Token::Var("new_var".into()), 7)));
        // assert_eq!(res3, Ok((Token::Var("new_var".into()), 7)));

    }

    #[test]
    fn test_flattened_ast() {
        let expr = "x + max(0, sqrt(min(1,2,3,4)))";
        let var =  &vec!["x"];
        let ts = TokenStream::new(expr).unwrap();
        let ast = AST::new(ts).unwrap();
        ast.check_input_vars(var).unwrap();
        let flat_ast = ast.flatten_ast();
        println!("Expression: {}", expr);
        flat_ast.print_ast();
    }

    #[test]
    fn test_iter_ast() {
        let expr = "(1 - y)*max(0.0, 4.0, z**2) + x/3";
        let ts = TokenStream::new(expr).unwrap();
        let tree = AST::new(ts).unwrap().tree;

        // breadth first
        let expected_bfs = vec![
            Token::ArOp(ArithmeticOperator::Plus),
            Token::ArOp(ArithmeticOperator::Mul),
            Token::ArOp(ArithmeticOperator::Div),
            Token::ArOp(ArithmeticOperator::Minus),
            Token::Func(Function::Max, 64),
            Token::Var("x".to_string()),
            Token::Number(3f64),
            Token::Number(1f64),
            Token::Var("y".to_string()),
            Token::Number(0f64),
            Token::Number(4f64),
            Token::ArOp(ArithmeticOperator::Pow),
            Token::Var("z".to_string()),
            Token::Number(2f64),
        ];
        let result_bfs : Vec<Token>= tree.iter_bfs().map(|b| b.tc().token.clone()).collect();
        assert_eq!(expected_bfs, result_bfs);

        // depth first
        let expected_dfs = vec![
            Token::ArOp(ArithmeticOperator::Plus),
            Token::ArOp(ArithmeticOperator::Mul),
            Token::ArOp(ArithmeticOperator::Minus),
            Token::Number(1f64),
            Token::Var("y".to_string()),
            Token::Func(Function::Max, 64), // TODO
            Token::Number(0f64),
            Token::Number(4f64),
            Token::ArOp(ArithmeticOperator::Pow),
            Token::Var("z".to_string()),
            Token::Number(2f64),
            Token::ArOp(ArithmeticOperator::Div),
            Token::Var("x".to_string()),
            Token::Number(3f64),
        ];
        let result_dfs : Vec<Token>= tree.iter_dfs().map(|b| b.tc().token.clone()).collect();
        assert_eq!(expected_dfs, result_dfs);

    }

    #[test]
    fn test_udefined_variables() {
        let expr = "x + 2*y";
        let ts = TokenStream::new(expr).unwrap();
        let ast = AST::new(ts).unwrap();
        let result_ok = ast.check_input_vars(&["x", "y"]);
        let result_fail = ast.check_input_vars(&["x"]);

        assert_matches!(result_ok,  Ok(()));
        assert_matches!(result_fail, Err(ParsingError::UndefinedVariable(_, 6))); // TODO
    }

    #[test]
    fn test_multi_line_var() {
        let expr = "t = 2*x + y\nu = (t - 1.0) / x\n max(t,u,x*y)";
        let good_inputs = ["x", "y"];
        let missing_inputs = ["x", "w"];
        let ast = ASTSeries::new(expr).unwrap();
        assert_matches!(ast.check_input_vars(&good_inputs), Ok(_));
        assert_matches!(ast.check_input_vars(&missing_inputs), Err(_));
        let assigned_to : Vec<String> = ast.forest().iter().map(|ast| ast.assigned_to.clone()).flatten().collect();
        assert_eq!(&assigned_to, &["t".to_string(), "u".to_string()])
    }

    #[test]
    fn test_invalid_names() {
        assert_eq!(check_for_invalid_inputs(&["pi"]), Err(ParsingError::InvalidVariableName("pi".into(), VARNAME_ERR1)));
        assert_eq!(check_for_invalid_inputs(&["ip"]), Ok(()));
        assert_eq!(check_for_invalid_inputs(&["2pi"]), Err(ParsingError::InvalidVariableName("2pi".into(), VARNAME_ERR3)));
        assert_eq!(check_for_invalid_inputs(&["_2pi"]), Ok(()));
        assert_eq!(check_for_invalid_inputs(&["pi*2"]), Err(ParsingError::InvalidVariableName("pi*2".into(), VARNAME_ERR2)));
    }

    #[test]
    fn test_indexing_pass() {
        let ts = TokenStream::new("array[:, 4,  ::-2]").unwrap();
        let ts = first_indexing_pass(ts);
        let expected = [
            Token::Var("array".to_string()),
            Token::Dot,
            Token::Method(Method::TakeSlice),
            Token::LP,
            Token::Colon,
            Token::Comma,
            Token::Number(4f64),
            Token::Comma,
            Token::Colon, Token::Colon,
            Token::Number(-2f64),
            Token::RP
        ];
        assert!(same_tokens(&unwrap_contexts(ts.tokens()), &expected));
    }

    #[test]
    fn test_indexing() {

        test_parsing("v[1:-1]", &vec!["v"], "(.: v, (TakeSlice: (:: 1, -1)))");

        let ts = TokenStream::new("v[:, ::-1, 0]").unwrap();
        let ts = first_indexing_pass(ts);
        for tc in ts.tokens() {println!("{}", tc.token)}

        test_parsing("v[:, ::-1, 0]", &vec!["v"], "(.: v, (TakeSlice: (:: 1, -1)))");
        // let expr_list = ["v[1]", "v[0,0]", "v[0:10]", "v[0:10:-1]",  "v[:-1]",  "v[max(0, i):-1]"];
        // for expr in expr_list {
        //     let rpn = AST::from_expression(expr).unwrap().rpn_repr();
        //     println!("  >> {expr} => {rpn}")
        // }
    }


}