use crate::{Token, ArithmeticOperator, Function, };
use super::Branch;
use crate::Latex;
use crate::tokenizer::*;

const GREEK: [&str; 23] = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta",
"theta", "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi", "rho",
"sigma", "tau", "upsilon", "phi", "chi", "psi", "omega"];
const GREEK_CAPITAL: [&str; 23] = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon",
"Zeta", "Theta", "Iota", "Kappa", "Lambda", "Mu", "Nu", "Xi", "Omicron", "Pi",
"Rho", "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega"];

impl Latex for Token {
    fn latex(&self) -> String {
        match self {
            Self::ArOp(op) => match op {
                ArithmeticOperator::Plus => "+",
                ArithmeticOperator::Minus => "-",
                ArithmeticOperator::Mul => "⋅",
                ArithmeticOperator::Div => "#", // special case
                ArithmeticOperator::Pow => "#", // special case
                ArithmeticOperator::Negate => "~",
                ArithmeticOperator::Rem => "%",
            }.to_string(),
            Self::RelOp(op) => match op {
                RelationalOperator::Eq => r"\stackrel{?}{=}",
                RelationalOperator::Neq => r"\neq",
                RelationalOperator::Greater => ">",
                RelationalOperator::Lesser => "<",
                RelationalOperator::Geq => r"\geq",
                RelationalOperator::Leq => r"\leq",
            }.to_string(),
            Self::LogicOp(op) => match op {
                LogicalOperator::And => r"\land",
                LogicalOperator::Or => r"\lor",
            }.to_string(),
            Self::AssignOp(op) => match op {
                AssignmentOperator::Assign => "=",
                AssignmentOperator::PlusEq => "+=",
                AssignmentOperator::MinusEq => "-=",
                AssignmentOperator::TimesEq => "*=",
                AssignmentOperator::DivideEq => "/=",
            }.to_string(),
            Self::LP => r"\left (".to_string(),
            Self::RP => r"\right )".to_string(),
            Self::LB => r"\left [".to_string(),
            Self::RB => r"\right [".to_string(),
            Self::Comma => ",".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::Dot => ".".to_string(),
            Self::Colon => ":".to_string(),
            Self::Const(c) => match c {
                Constant::Pi => r"\pi",
                Constant::Pi2 => r"\pi^{2}",
                Constant::PiTimes2 => r"2\pi",
                Constant::Sqrt2 => r"\sqrt{2}",
                Constant::Sqrt3 => r"\sqrt{2}",
                Constant::SqrtPi => r"\sqrt{\pi}",
                Constant::Euler => r"e",
            }.to_string(),
            Self::Func(f, _) => match f {
                Function::Max => r"\max",
                Function::Min => r"\min",
                Function::Abs => "#", // special case
                Function::Avg => r"\text{avg }",
                Function::Std => "std dev",
                Function::Sqrt => "#", // special case
                Function::Sum => r"\sum",
                Function::Range => r"(\max - \min)",
                Function::Sin => r"\sin",
                Function::Cos => r"\cos",
                Function::Cot => r"\cot",
                Function::Tan => r"\tan",
                Function::Log => r"\ln",
                Function::Log2 => r"\log_{2}",
                Function::Log10 => r"\log_{10}",
                Function::Exp => "#", // special case
            }.to_string(),
            Self::Var(s) => format_var_name(s),
            Self::Eof => "".to_string(),
            Self::Number(x) => format!("{}", x),
        }
    }
}

fn format_var_name(v: &str) -> String {
    format_greek_character(&format_ending_digits(v))
}

fn format_ending_digits(v: &str) -> String {
    if let Some(ending_digits) = v.chars().rev().position(|c| !c.is_ascii_digit())
    && ending_digits > 0 {

        let name = v.chars().collect::<Vec<char>>();
        let mut formatted = String::new();
        let normal : String = name[0..name.len()-ending_digits].iter().collect();
        let subscript: String = name[name.len()-ending_digits..].iter().collect();

        println!("{} {} {}", ending_digits, normal, subscript);


        formatted.push_str(&format!("{}_{{{}}}", normal, subscript));
        formatted
    } else {
        v.to_string()
    }
}

fn format_greek_character(v: &str) -> String {
    let chars: Vec<char> = v.chars().collect();
    if let Some(t) = chars.iter().position(|c| c.is_ascii_digit() || *c == '_' ) {
        let start = String::from_iter(chars[..t].iter());
        let end: String = String::from_iter(chars[t..].iter());

        if is_greek(&start) || is_greek_capital(&start){
            return format!("\\{}{}", start, end);
        }
    }
    if is_greek(&v) || is_greek_capital(&v){
        return format!("\\{}", v);
    }
    return v.to_string();
}

fn is_greek(v: &str) -> bool {
    return GREEK.iter().any(|g| *g == v)
}
fn is_greek_capital(v: &str) -> bool {
    return GREEK_CAPITAL.iter().any(|g| *g == v)
}



impl Latex for Branch {
    fn latex(&self) -> String {
        fn arg_list(args: &[Branch]) -> String {
            args
            .iter()
            .map(|a| a.latex())
            .collect::<Vec<String>>()
            .join(", ")
        }
        match self {
            Self::Atom(tc) => return tc.token.latex(),
            Self::Expression(tc, c) => {
                match &tc.token {
                    Token::ArOp(op) => match (op, c.len()) {
                        (_, 1) => return [tc.token.latex(), c[0].latex()].concat(),
                        (ArithmeticOperator::Div, _) => return format!("\\frac{{{}}}{{{}}}", c[0].latex(), c[1].latex()),
                        (ArithmeticOperator::Pow, _) => return format!("({})^{{{}}}", c[0].latex(), c[1].latex()),
                        _ => return [c[0].latex(), tc.token.latex(), c[1].latex()].concat()
                    }
                    Token::RelOp(_)
                    |Token::AssignOp(_)
                    |Token::LogicOp(_) => return [c[0].latex(), tc.token.latex(), c[1].latex()].concat(),
                    Token::Func(f, _) => match f {
                        Function::Abs => return format!("\\left | {} \\right |", c[0].latex()),
                        Function::Sqrt => return format!("\\sqrt {{{}}}", c[0].latex()),
                        Function::Exp =>  return format!("e^{{{}}}", c[0].latex()),
                        _ => return format!("{}\\left ( {} \\right )", tc.token.latex(), arg_list(c))
                    }
                    _ => tc.token.latex() // number, constant, variable, ()[],.;:
                }
            }
        }
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_var() {
        let var = "T";
        let res = format_ending_digits(var);
        assert_eq!(res, var.to_string());

    }

}