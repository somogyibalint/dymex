/// Turn an expression string into a TokenStream
///
///  

use std::fmt;
use crate::Float;

const MAX_FUNC_ARGS: usize = 64;

const INVALIDCHAR : &str = "#?˝`\'&|$@%{}";
const SPECIAL_CHARS  : &str = "()[].,:+-*/^=<>!";
const FORBIDDEN_IDS: [&str; 18] = ["min", "max", "avg", "mean", "std", "sin", "cos",
"tan", "cotan", "exp", "log", "log2", "log10", "sqrt", "pi", "e", "sqrt2", "sqrt3"];


/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerError {
    SyntaxError(usize), /// uncategorized tokenizer error
    InvalidCharacter(char, usize),
    InvalidNumberFormat(usize),
    UndefinedVariable(usize, String),
    InvalidVariableIdentifier(String),
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum  ArithmeticOperator {
    Plus,
    Minus,
    Mul,
    Div,
    Rem,
    Pow,
    Negate,    
}
impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArithmeticOperator::Plus => write!(f, "+"),
            ArithmeticOperator::Minus => write!(f, "-"),
            ArithmeticOperator::Mul => write!(f, "*"),
            ArithmeticOperator::Div => write!(f, "/"),
            ArithmeticOperator::Pow => write!(f, "**"),
            ArithmeticOperator::Rem => write!(f, "%"),
            ArithmeticOperator::Negate => write!(f, "~"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelationalOperator {
    Greater,
    Lesser,
    Eq,
    Neq,
    Leq,
    Geq,
}
impl fmt::Display for RelationalOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RelationalOperator::Greater => write!(f, ">"),
            RelationalOperator::Lesser => write!(f, "<"),
            RelationalOperator::Eq => write!(f, "=="),
            RelationalOperator::Neq => write!(f, "≠"),
            RelationalOperator::Leq => write!(f, "≤"),
            RelationalOperator::Geq => write!(f, "≥"),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum  LogicalOperator {
    And,
    Or
}
impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogicalOperator::And => write!(f, "and"),
            LogicalOperator::Or => write!(f, "or"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentOperator {
    Assign,
    PlusEq,
    MinusEq,
    TimesEq,
    DivideEq,
}
impl fmt::Display for AssignmentOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AssignmentOperator::Assign => write!(f, "="),
            AssignmentOperator::PlusEq => write!(f, "+="),
            AssignmentOperator::MinusEq => write!(f, "-="),
            AssignmentOperator::TimesEq => write!(f, "*="),
            AssignmentOperator::DivideEq => write!(f, "/="),
        }
    }
}
/// Built in functions
#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    Min,
    Max,
    Avg,
    Std,
    Sum,
    Range,
    Sin,
    Cos,
    Tan,
    Cot,
    Exp,
    Log,
    Log2,
    Log10,
    Sqrt,
}
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

/// Built in mathematical constants
#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    Pi,
    Euler,
    Sqrt2,
    Sqrt3,
    PiTimes2,
    SqrtPi,
    Pi2
}
impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Constant::Pi => write!(f, "π"),
            Constant::Euler => write!(f, "e"),
            Constant::Pi2 => write!(f, "π²"),
            Constant::PiTimes2 => write!(f, "2π"),
            Constant::Sqrt2 => write!(f, "√2"),
            Constant::Sqrt3 => write!(f, "√3"),
            Constant::SqrtPi => write!(f, "√π"),
        }
    }
}

/// A token with additional context. The position in the original expression
/// and the length of the string representation is stored in `at` and `len`.
/// As `at` is unique for each token, it also serves as an ID.  
#[derive(Debug, PartialEq, Clone)]
pub struct TokenContext {
    pub token: Token,
    pub at: usize,
    pub len: usize
}
impl TokenContext {
    pub fn new(token: Token, at: usize, len: usize) -> Self {
        Self { token, at, len }
    }
    /// Returns a TokenContext without context (e.g for tests).
    pub fn dummy(token: Token) -> Self {
        Self { token, at: 0, len: 0 }
    }
}

/// Supported tokens and token categories
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ArOp(ArithmeticOperator),
    RelOp(RelationalOperator),
    LogicOp(LogicalOperator),
    AssignOp(AssignmentOperator),
    LP, // left parens
    RP,
    LB, // left brackets
    RB,
    Comma,
    Semicolon,
    Dot,
    Colon,
    Number(Float),
    Const(Constant),
    Var(String),
    Func(Function, usize),
    Eof
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::ArOp(o) => write!(f, "{}", o),
            Token::RelOp(o) => write!(f, "{}", o),
            Token::LogicOp(o) => write!(f, "{}", o),
            Token::AssignOp(o) => write!(f, "{}", o),
            Token::LP => write!(f, "("),
            Token::RP => write!(f, ")"),
            Token::LB => write!(f, "["),
            Token::RB => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Number(x) => write!(f, "{}", x),
            Token::Const(c) => write!(f, "{}", c),
            Token::Var(s) => write!(f, "{}", s),
            Token::Func(func, _) => write!(f, "{}", func),
            Token::Eof => write!(f, "Eof"),
        }
    }
}

/// Contains the original expression, the list of variable keys and the list
/// of tokens in normal (for debugging) and reversed (for parsing) order. 
#[derive(Debug, PartialEq, Clone)]
pub struct TokenStream {
    pub tokens: Vec<TokenContext>,
    tokens_reversed: Vec<TokenContext>,
    expr: String,
    var: Vec<String>,
    err: Option<TokenizerError>,
}

impl TokenStream {
    pub fn new() -> Self {
        Self {
            tokens: vec!(),
            tokens_reversed: vec!(),
            expr: "".into(),
            var: vec!(),
            err: None
        }
    }

    pub fn identical(&self, other: &TokenStream) -> bool {
        if self.tokens.len() != other.tokens.len() {return false;}
        for (l, r) in self.tokens.iter().zip(other.tokens.iter()) {
            if l.token != r.token { return false; }
        }
        true
    }

    pub fn eof(&self) -> TokenContext {
        TokenContext { token: Token::Eof, at: self.expr.len(), len: 0 }
    }

    pub fn next(&mut self) -> TokenContext {
        self.tokens_reversed.pop().unwrap_or(self.eof())
    } 

    pub fn peek(&mut self) -> TokenContext {
        self.tokens_reversed.last().cloned().unwrap_or(self.eof())
    }

    /// Update the expression and variable keys, tokenize the expression if changed
    pub fn update(&mut self, expression: &str, variables: &[&str] ) {
        if self.expr == expression && self.var == variables {return;}

        self.expr = expression.into();
        self.var = variables.iter().copied().map(|s| s.into()).collect();
        self.tokenize();
    }

    fn tokenize(&mut self) -> bool {
        let vars : Vec<&str> = self.var.iter().map(|s| s as &str).collect();
        let res = tokenize(&self.expr, &vars);
        match res {
            Ok(v) => {
                self.tokens = v.clone();
                self.tokens_reversed = v;
                self.tokens_reversed.reverse();
                self.err = None;
                return true;
            }
            Err(e) => {
                self.tokens.clear();
                self.tokens_reversed.clear();
                self.err = Some(e);
                return false;
            }
        }
    }
}

/// Turns the string representation into token with additional context
fn tokenize(input: &str, variables: &[&str]) -> Result<Vec<TokenContext>, TokenizerError> {

    if let Err(e) = check_input_variables(variables) { return Err(e); }
    if let Err(e) = check_illegal_characters(input) { return Err(e); } 
    
    let mut res: Vec<TokenContext> = vec![];
    let mut cursor = 0;

    loop {
        if let Some(next) = input.chars().nth(cursor) {
            let nextnext = input.chars().nth(cursor+1).unwrap_or(' ');
            if next.is_whitespace() {
                cursor +=1;
            } else if next.is_alphabetic() || next == '_' {
                match parse_identifier(&input[cursor..], cursor, variables, res.last()) { 
                    Ok((t, wordsize)) => {
                        res.push(TokenContext { token: t, at: cursor, len: wordsize });
                        cursor += wordsize;
                        continue;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else if next.is_ascii_digit() 
                || (next == '-' && nextnext.is_ascii_digit() ) {
                if let Some((t, wordsize)) = parse_number(&input[cursor..]) {
                    res.push(TokenContext { token: t, at: cursor, len: wordsize });
                    cursor += wordsize;
                    continue;
                } else {
                    return Err(TokenizerError::InvalidNumberFormat(cursor));
                } 
            } else if SPECIAL_CHARS.contains(next) {
                if let Some((t, advance)) = parse_special_characters(next, nextnext) {
                    res.push(TokenContext { token: t, at: cursor, len: advance });
                    cursor += advance;
                    continue;
                } else {
                    return Err(TokenizerError::SyntaxError(cursor))
                }
            } else {  
                return Err(TokenizerError::InvalidCharacter(next, cursor));
            }
        } else {
            break;
        }
    }
    Ok(res)
}


/// Check if there are any illegal characters in the expression
fn check_illegal_characters(s: &str) -> Result<(), TokenizerError>{
    for invchar in INVALIDCHAR.chars() {
        if let Some(at) = s.chars().position(|c| c == invchar )
        {
            return Err(TokenizerError::InvalidCharacter(invchar, at));
        }
    }
    Ok(())
}

/// Check if there is an input variable with name that collides with reserved words.
fn check_input_variables(variables: &[&str]) -> Result<(), TokenizerError> {
    for var in variables {
        if FORBIDDEN_IDS.contains(var) {
            return Err(TokenizerError::InvalidVariableIdentifier((*var).into()));
        }
    }
    Ok(())
}

/// Parses c1, c2 if c1 is special character.
fn parse_special_characters(c1: char, c2: char) -> Option<(Token, usize)> {
    if let Some(t) = parse_double_char_token(c1 , c2) {
        return Some((t, 2));
    }
    if let Some(t) = parse_single_char_token(c1) {
        return Some((t, 1));
    }
    None
} 

/// Parse single 'special' character token
fn parse_single_char_token(c: char) -> Option<Token> {
    match c {
        '(' => Some(Token::LP),
        ')' => Some(Token::RP),
        '[' => Some(Token::LB),
        ']' => Some(Token::RB),
        '.' => Some(Token::Dot),
        ',' => Some(Token::Comma),
        ':' => Some(Token::Colon),
        '+' => Some(Token::ArOp(ArithmeticOperator::Plus)),
        '-' => Some(Token::ArOp(ArithmeticOperator::Minus)),
        '/' => Some(Token::ArOp(ArithmeticOperator::Div)),
        '^' => Some(Token::ArOp(ArithmeticOperator::Plus)),
        '*' | '×' | '⋅' => Some(Token::ArOp(ArithmeticOperator::Mul)),
        '=' => Some(Token::AssignOp(AssignmentOperator::Assign)),
        '>' => Some(Token::RelOp(RelationalOperator::Greater)),
        '<' => Some(Token::RelOp(RelationalOperator::Lesser)),
        _ => None
    }
}

/// Parse double 'special' character token
fn parse_double_char_token(c1: char, c2: char) -> Option<Token> {
    match (c1, c2) {
        ('*', '*') => Some(Token::ArOp(ArithmeticOperator::Pow)),
        ('+', '=') => Some(Token::AssignOp(AssignmentOperator::PlusEq)),
        ('-', '=') => Some(Token::AssignOp(AssignmentOperator::MinusEq)),
        ('*', '=') => Some(Token::AssignOp(AssignmentOperator::TimesEq)),
        ('/', '=') => Some(Token::AssignOp(AssignmentOperator::DivideEq)),
        ('=', '=') => Some(Token::RelOp(RelationalOperator::Eq)),
        ('>', '=') => Some(Token::RelOp(RelationalOperator::Geq)),
        ('<', '=') => Some(Token::RelOp(RelationalOperator::Leq)),
        ('!', '=') => Some(Token::RelOp(RelationalOperator::Neq)),
        _ => None
    }
}

/// Parse number: 2, 2.103, 0.3E6, 3.0e-5
fn parse_number(s: &str) -> Option<(Token, usize)> {
    let mut numbrestring = s;
    let mut dots: usize = 0;
    let mut exp: usize = 0;
    let mut after_exp: bool = false;
    for (i, c) in s.chars().enumerate() {
        match (c, dots, exp) {
            ('.', 0, _) => {
                dots += 1;
            }
            ('e', _, 0) | ('E', _, 0) => {
                after_exp = true;
                exp += 1;
            }
            ('.', 1, _) => {
                return None;
            }            
            ('e', _, 1) | ('E', _, 1) => {
                return None;
            }
            ('-', _, _) if i == 0 => {

            }
            // special case for E-1
            ('-', _, _) if after_exp => { 
                after_exp = false;
            }
            _ => { if !c.is_ascii_digit() {
                    numbrestring = &s[..i];
                    break;
                }
            }
        }
    }
    if let Ok(x) = str::parse::<Float>(numbrestring) {
        Some((Token::Number(x), numbrestring.len()))
    } else {
        None
    }
}

/// Returns true c is valid character for an identifier
fn is_ident_char(c: char) -> bool {
    if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
        return  true;
    }
    false
}

/// Parses an identifier returning a function, constant, or user-defined
/// variable, or UndefinedVariable error. The `start` parameter is only 
/// needed for error reporting. The previous token needs to be provided 
/// to allow `a.b` even if `b` is not a user defined variable. 
fn parse_identifier(
    s: &str, 
    start:usize, 
    vars: &[&str], 
    prev: Option<&TokenContext>
    ) -> Result<(Token, usize), TokenizerError> {
    let word = match s.chars()
        .enumerate()
        .filter(|(_, c)| !is_ident_char(*c)).next() {
            Some((i_end, _) ) => &s[..i_end],
            None => &s,
        };
    if let Some(func) = parse_function(word) {
        return Ok((func, word.len()));
    }
    if let Some(constant) = parse_const(word) {
        return Ok((constant, word.len()));
    }
    return parse_variable(word, start, vars, prev);
}

fn parse_variable(
    word: &str,
    start: usize,
    vars: &[&str], 
    prev: Option<&TokenContext>
    ) -> Result<(Token, usize), TokenizerError> {
    // special case for fields: for `a.b`, only `a` needs to be a variable keys,
    // the existence of `b` can only be checked during evaluation.
    let res = (Token::Var(word.into()), word.len()); 
    match (prev, vars.contains(&word)) {
        (Some(tc), _) if tc.token == Token::Dot => {Ok(res)},
        (_, true) => {Ok(res)}
        (_, _) => { return Err(TokenizerError::UndefinedVariable(start, word.into())) }
    }
}


fn parse_function(word: &str) -> Option<Token>
{
    match word.to_lowercase().as_str() {
        "min" => Some(Token::Func(Function::Min, MAX_FUNC_ARGS)),
        "max" => Some(Token::Func(Function::Max, MAX_FUNC_ARGS)),
        "avg" => Some(Token::Func(Function::Avg, MAX_FUNC_ARGS)),
        "mean" => Some(Token::Func(Function::Avg, MAX_FUNC_ARGS)),
        "std" => Some(Token::Func(Function::Std, MAX_FUNC_ARGS)),
        "sum" => Some(Token::Func(Function::Sum, MAX_FUNC_ARGS)),
        "sin" => Some(Token::Func(Function::Sin, 1)),
        "cos" => Some(Token::Func(Function::Cos, 1)),
        "tan" => Some(Token::Func(Function::Tan, 1)),
        "cotan" => Some(Token::Func(Function::Cot, 1)),
        "exp" => Some(Token::Func(Function::Exp, 1)),
        "log" => Some(Token::Func(Function::Log2, 1)),
        "log2" => Some(Token::Func(Function::Log2, 1)),
        "log10" => Some(Token::Func(Function::Log10, 1)),
        "sqrt" => Some(Token::Func(Function::Sqrt, 1)),
        _ => None
    }
}

fn parse_const(word: &str) -> Option<Token>
{
    match word.to_lowercase().as_str() {
        "pi" => Some(Token::Const(Constant::Pi)),
        "e"  => Some(Token::Const(Constant::Euler)),
        "sqrt2" => Some(Token::Const(Constant::Sqrt2)),
        "sqrt3" => Some(Token::Const(Constant::Sqrt2)),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use crate::TokenizerError;
    use super::*;

    fn same_tokens(tokens1 : &[Token], tokens2 : &[Token]) -> bool {
        if tokens1.len() != tokens2.len() {return false;}
        tokens1.iter().zip(tokens2.iter()).map(|(l,r)| l!=r).count() > 0
    }

    fn unwrap_contexts(tcs: &[TokenContext]) -> Vec<Token> {
        tcs.iter().cloned().map(|tc| tc.token ).collect()
    }


    #[test]
    fn test_invalid_names() {
        assert_eq!(check_input_variables(&["pi"]), Err(TokenizerError::InvalidVariableIdentifier("pi".into())));
        assert_eq!(check_input_variables(&["ip"]), Ok(()));
    }
    
    #[test]
    fn test_characters() {
        assert_eq!(check_illegal_characters("([{"), Err(TokenizerError::InvalidCharacter('{', 2)));
        assert_eq!(check_illegal_characters("(a0bcdϕ.[:]π!=<>))"), Ok(()));
    }

    #[test]
    fn test_number() {
        assert_eq!(parse_number("0.123+pi"), Some((Token::Number(0.123), 5)));
        assert_eq!(parse_number("0.2E1 + XXX"), Some((Token::Number(2.0), 5)));
        assert_eq!(parse_number("20.0e-1"), Some((Token::Number(2.0), 7)));
        assert_eq!(parse_number("8 "), Some((Token::Number(8.0), 1)));
        assert_eq!(parse_number("2.0*pi "), Some((Token::Number(2.0), 3)));
        assert_eq!(parse_number("-1.0"), Some((Token::Number(-1.0), 4)));
        assert_eq!(parse_number("-1.0e-1"), Some((Token::Number(-0.1), 7)));
    }

    #[test]
    fn test_identifier() {
        let input_vars = vec!("a", "center", "eV2nm");
        let start = 13;

        let res = parse_identifier("max(15)", start, &input_vars, None);
        assert_matches!(res, Ok((Token::Func(Function::Max, _), 3)) );

        let res = parse_identifier("_center", start, &input_vars, None);
        assert_eq!(res, Err(TokenizerError::UndefinedVariable(start, "_center".into())) );

        let res = parse_identifier("center*5", start, &input_vars, None);
        assert_eq!(res, Ok((Token::Var("center".into()), 6)));

        let res = parse_identifier("pi^2", start, &input_vars, None);
        assert_eq!(res, Ok((Token::Const(Constant::Pi), 2)));

        let res = parse_identifier("eV2nm * λ", start, &input_vars, None);
        assert_eq!(res, Ok((Token::Var("eV2nm".into()), 5)));
    }

    #[test]
    fn test_tokenizer_expr1() {
        let input_var = &["x", "y"];
        let expr1 = "(2.0*pi * exp(-x*x)) / max(1.0 + sqrt(y), 0)";
        let expr2 = "(2.0*pi*exp(-x*x))/max(1.0+sqrt(y),0)";
        let expr3 = "(2.0 * pi *exp( -x * x)) / max(1.0 + sqrt(y), 0)   ";

        let target = &[Token::LP, Token::Number(2.0), Token::ArOp(ArithmeticOperator::Mul), 
        Token::Const(Constant::Pi), Token::ArOp(ArithmeticOperator::Mul), Token::Func(Function::Exp, 1), 
        Token::LP, Token::ArOp(ArithmeticOperator::Minus), Token::Var("x".into()), Token::ArOp(ArithmeticOperator::Mul),
        Token::Var("x".into()), Token::RP, Token::RP, Token::ArOp(ArithmeticOperator::Div), Token::Func(Function::Max, 64), 
        Token::LP, Token::Number(1.0), Token::ArOp(ArithmeticOperator::Plus), Token::Func(Function::Sqrt, 1), 
        Token::LP, Token::Var("y".into()), Token::RP, Token::Comma, Token::Number(0.0), Token::RP];

        let res1 = tokenize(expr1, input_var).unwrap();
        let res2 = tokenize(expr2, input_var).unwrap();
        let res3 = tokenize(expr3, input_var).unwrap();
        assert!(same_tokens(&unwrap_contexts(&res1), target));
        assert!(same_tokens(&unwrap_contexts(&res2), target));
        assert!(same_tokens(&unwrap_contexts(&res3), target));
    }

    #[test]
    fn test_tokenizer_expr2() {
        let input_var = &["spectrum"];
        let expr = "spectrum.x[-1] - spectrum.x[0]";
        let res = tokenize(expr, input_var).unwrap();
        let target = &[Token::Var("spectrum".into()), Token::Dot, 
            Token::Var("x".into()), Token::LB, Token::Number(-1.0), Token::RB, 
            Token::ArOp(ArithmeticOperator::Minus), Token::Var("spectrum".into()), 
            Token::Dot, Token::Var("x".into()), Token::LB, Token::Number(0.0), Token::RB
        ];

        assert!(same_tokens(&unwrap_contexts(&res), target));
    }

    #[test]
    fn test_tokenizer_expr3() {
        let input_var = &["a"];
        let expr1 = "a.b"; // this is ok
        let expr2 = "b.a"; // error
        let target1 = &[Token::Var("a".into()), Token::Dot, Token::Var("b".into())];

        let res1 = tokenize(expr1, input_var).unwrap();
        let res2 = tokenize(expr2, input_var);
        assert!(same_tokens(&unwrap_contexts(&res1), target1));
        assert_matches!(res2, Err(TokenizerError::UndefinedVariable(_, _)));
    }

    #[test]
    fn test_tokenizer_expr4() {
        let input_var = &["v"];
        let expr = "v[0:-1]"; // error
        let target = &[Token::Var("v".into()), Token::LB, 
            Token::Number(0.0), Token::Colon, Token::Number(-1.0), Token::RB
            ];

        let res = tokenize(expr, input_var).unwrap();
        assert!(same_tokens(&unwrap_contexts(&res), target));
    }

    #[test]
    fn test_tokenstream1() {
        let mut ts = TokenStream::new();
        ts.update("(1 + x) * 3", &vec!["x"]);
        assert!(ts.tokenize());
        for t in ts.tokens {
            println!("{}", t.token);
        }
    }


}


