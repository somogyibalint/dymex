use std::usize;

use crate::{UserMessage, TokenizerError, TokenContext};

const VARNAME_EXAMPLES: &str = "Valid: a, ϕ0, _mass, phase_x_2_, e0_π2, ...\n Invalid: π, e, 1a, 1_b, ... ";
pub(super) const VARNAME_ERR1: &str = "This is a reserved keyword, please choose a different name!";
pub(super) const VARNAME_ERR2: &str = "Variable names may contain `_`, numbers or alphabetic characters.";
pub(super) const VARNAME_ERR3: &str = "Variable names cannot start with a number.";

/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsingError {
    UnexpectedToken(TokenContext, usize),
    UnexpectedLP(usize),
    MissingRP(i32),
    MissingArgument(usize),
    TooManyArguments(usize),
    InvalidOperation(usize, String),
    NotImplemented(String),
    UndefinedVariable(String, usize),
    InvalidVariableName(String,  &'static str),
    InvalidAssignment(String, usize),
    LexingError(TokenizerError)
}
impl ParsingError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            Self::UnexpectedToken(tc, tmp) => UserMessage::new(
                    format!("Unexpected token: {}, {}", tc.token, tmp),
                    Some(*&tc.at),
                    None,
                    None),
            Self::UnexpectedLP(i) => UserMessage::new(
                    format!("Unexpected ("),
                    Some(*i),
                    None,
                    None),
            Self::MissingRP(i) => UserMessage::new(
                    format!("Missing )"),
                    Some(*i as usize),
                    None,
                    None),
            Self::MissingArgument(i) => UserMessage::new(
                    format!("Missing argument:"),
                    Some(*i as usize),
                    None,
                    None),
            Self::TooManyArguments(i) => UserMessage::new(
                    format!("Too many arguments:"),
                    Some(*i as usize),
                    None,
                    None),
            Self::InvalidOperation(i, op) => UserMessage::new(
                    format!("Invalid operation: {}", op),
                    Some(*i as usize),
                    None,
                    None),
            Self::NotImplemented(feature) => UserMessage::new(
                    format!("{} is not yet implemented.", feature),
                    Some(0),  //TODO
                    None,
                    None),
            Self::UndefinedVariable(varname, i) => UserMessage::new(
                    format!("Undefined variable: `{}`", varname),
                    Some(*i),
                    None,
                    None),
            Self::InvalidVariableName(varname, hint) => {
                UserMessage::new(format!("Invalid variable name: {}", varname),
                None,
                Some(hint),
                Some(VARNAME_EXAMPLES))
            }
            Self::InvalidAssignment(details, i) => UserMessage::new(
                    format!("Invalid assignment: `{}`", details),
                    Some(*i),
                    None,
                    None),
            Self::LexingError(err) => err.user_message()
        }
    }
}