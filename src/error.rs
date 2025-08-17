use std::fmt::{Display, Formatter, Result, Write};
use crate::{ParsingError, TokenizerError};


#[derive(Debug, Clone, PartialEq)]
pub enum DymexError {
    LexicalError(TokenizerError),
    ParsingError(ParsingError),

}
impl DymexError {
    fn user_message(&self) -> UserMessage {
        match self {
            Self::LexicalError(e) => e.user_message(),
            Self::ParsingError(e) => e.user_message(),
        }
    }
}

pub struct UserMessage {
    msg: String,
    cursor: Option<usize>,
    hint: Option<&'static str>,
    examples: Option<&'static str>
}
impl UserMessage {
    pub fn new(message: impl Into<String>, 
        cursor: Option<usize>,
        hint: Option<&'static str>,
        examples: Option<&'static str>,
    ) -> Self {
        Self {
            msg: message.into(),
            cursor: cursor,
            hint: hint,
            examples: examples
        }
    }

    fn full_message(&self, expression: &str) -> String {
        let mut msg = format!("{}\n", self.msg);
        msg.push_str(expression);
        if let Some(cursor) = self.cursor {
            for _ in 0..cursor { msg.push(' '); };
            msg.push_str("^\n");
        }
        if let Some(hint) = self.hint {
            writeln!(msg, "{}", hint).unwrap();
        }
        msg
    }
}

impl Display for UserMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}\n", self.msg)
    }
}
