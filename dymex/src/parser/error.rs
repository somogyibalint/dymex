use std::usize;

use itertools::PadUsing;

use crate::{UserMessage};


/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsingError {
    UnexpectedToken(usize),
    UnexpectedLP(usize),
    MissingRP(i32),
    MissingArgument(usize),
    TooManyArguments(usize),
    InvalidOperation(usize, String),
    NotImplemented(String),
    UndefinedVariable(String, usize),
    InvalidAssignment(String, usize)
}
impl ParsingError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            Self::UnexpectedToken(i) => UserMessage::new(
                    format!("Unexpected token:"),
                    Some(*i),
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
            Self::InvalidAssignment(details, i) => UserMessage::new(
                    format!("Invalid assignment: `{}`", details),
                    Some(*i),
                    None,
                    None),
        }
    }
}