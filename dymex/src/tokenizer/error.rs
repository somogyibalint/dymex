use crate::UserMessage;


pub(super) const VARNAME_ERR1: &str = "This is a reserved keyword, please choose a different name!";
pub(super) const VARNAME_ERR2: &str = "Variable names may contain `_`, numbers or alphabetic characters.";
pub(super) const VARNAME_ERR3: &str = "Variable names cannot start with a number.";
const VARNAME_EXAMPLES: &str = "Valid: a, ϕ0, _mass, phase_x_2_, e0_π2, ...\n Invalid: π, e, 1a, 1_b, ... ";


/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerError {
    /// Uncategorized tokenizer error
    SyntaxError(usize), 
    InvalidCharacter(char, usize),
    InvalidNumberFormat(usize),
    UndefinedVariable(usize, String),
    InvalidVariableName(String, &'static str),
}
impl TokenizerError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            Self::SyntaxError(i) => {
                UserMessage::new(
                    format!("Syntax error:"), 
                    Some(*i), 
                    None, 
                    None) 
            }
            Self::InvalidCharacter(c, i) => {
                UserMessage::new(format!("Invalid character: {}", c), 
                Some(*i), 
                Some("The following charaters are invalid: #?˝`\'&|$@%{}"), 
                None
            )
            }
            Self::InvalidNumberFormat(i) => {
                UserMessage::new(format!("Invalid number formatting:"), 
                Some(*i), 
                Some("Valid formats are: 1, 3.14, 1e-10, 1.23E10, 1E+9, 1_000_000"),
                None)
            }
            Self::UndefinedVariable(i, name) => {
                UserMessage::new(format!("Undefined variable: {}", name), 
                Some(*i), 
                None, 
                None)
            }
            Self::InvalidVariableName(varname, hint) => {
                UserMessage::new(format!("Invalid variable name: {}", varname), 
                None, 
                Some(*hint), 
                Some(VARNAME_EXAMPLES))
            }
        }
    }
}

#[cfg(test)]
mod test_prevalidation {

}


#[cfg(test)]
mod test_numbers {
    use crate::same_num_tokens;
    use crate::tokenizer::*;


    #[test]
    fn test_allowed_num_edgecases() {
        // maybe throw errerif their is alphabetic or '_' right after number
        let (t, _) = parse_number(&charslice("2.0O3")).unwrap();
        assert_eq!(t, Token::Number(2.0));
        
        // this could lead to confusion, but valid syntax
        let (t, size) = parse_number(&charslice("12,34")).unwrap();
        assert_eq!(t, Token::Number(12.0));
        assert_eq!(size, 2);

        let (t, size) = parse_number(&charslice("1.2E+2")).unwrap();
        assert!(same_num_tokens(t, Token::Number(120.0)));
        assert_eq!(size, 6);

        let (t, size) = parse_number(&charslice("1_000_000")).unwrap();
        assert!(same_num_tokens(t, Token::Number(1E6)));
        assert_eq!(size, 9);
    }

    #[test]
    fn test_number_errors1() {
        let expr = "y*(x + 12.34.5)"; 
        let res = tokenize(expr, &["x", "y"]);
        if let Err(e) = res {
            assert!(matches!(e, crate::TokenizerError::InvalidNumberFormat(7)));
            print!("{}", e.user_message());
        } else {panic!()}
    }

    #[test]
    fn test_number_errors2() {
        let expr = "   12.34e-x";
        let res = tokenize(expr, &["x"]);
        if let Err(e) = res {
            assert!(matches!(e, crate::TokenizerError::InvalidNumberFormat(3)));
            print!("{}", e.user_message());
        } else {panic!()}
    }    
}