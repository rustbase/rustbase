use self::parser::Rule;
use pest::error;
use pest::iterators::Pair;

use crate::server::wirewave::server::{Error, Status};

pub mod parser;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct GrammarError; // type and message

impl GrammarError {
    pub fn with_message(msg: &str) -> Error {
        Error {
            message: msg.to_string(),
            query_message: None,
            status: Status::SyntaxError,
        }
    }

    pub fn with_pair(msg: &str, pair: Pair<Rule>) -> Error {
        let error_variant: error::ErrorVariant<Rule> = error::ErrorVariant::CustomError {
            message: msg.to_string(),
        };

        let grammar_error = error::Error::new_from_span(error_variant, pair.as_span());

        Error {
            message: grammar_error.to_string(),
            query_message: None,
            status: Status::SyntaxError,
        }
    }
}
