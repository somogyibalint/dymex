use crate::{float, Float};
use std::fmt;

const PISQUARED: Float = float::consts::PI*float::consts::PI;
const SQRT3: Float = 1.73205080757; // .sqrt() is not const, const::SQRT_3 is unstable feature
const SQRTPI: Float = 1.77245385091; // .sqrt() is not const 


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
// TODO: maybe separate variadic and single variable functions ???
#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    Min,
    Max,
    Avg,
    Std,
    Sum,
    Range,
    Abs,
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
impl Constant {
    pub fn value(&self) -> Float {
        match self {
            Constant::Pi => float::consts::PI,
            Constant::Euler => float::consts::E,
            Constant::Pi2 => PISQUARED,
            Constant::PiTimes2 => float::consts::TAU,
            Constant::Sqrt2 => float::consts::SQRT_2,
            Constant::Sqrt3 => SQRT3,
            Constant::SqrtPi => SQRTPI,
        }
    }
}