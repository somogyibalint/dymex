use dymex::{Float, Token, TokenContext};


// pub fn input_formatter(s: &str) -> String {
//     match s.parse::<f64>() {
//         Ok(x) => float_formatter(Some(&x)),
//         Err(_) => s.to_string(),
//     }
// }

// pub fn float_formatter(f: Option<&f64>) -> String {
//     match f {
//         Some(x) => format!("{:.6}", x),
//         None => "".to_string()
//     }
// }

pub fn format_num_result(x: Option<Float>) -> String {
    match x {
        Some(x) => format!("{:.4}", x),
        None => " ⚠ ".to_string()
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






