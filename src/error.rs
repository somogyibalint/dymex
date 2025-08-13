use crate::{ParsingError, TokenizerError};


#[derive(Debug, Clone, PartialEq)]
pub enum DymexError {
    LexicalError(TokenizerError),
    ParsingError(ParsingError),

}
impl DymexError {
    fn user_message(&self, expression: &str) -> String {
        match self {
            Self::LexicalError(e) => e.user_message(expression),
            Self::ParsingError(e) => e.user_message(expression),
        }
    }
}