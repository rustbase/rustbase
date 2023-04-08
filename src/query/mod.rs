pub mod parser;

type Result<T> = std::result::Result<T, QueryError>;

#[derive(Debug)]
pub enum QueryErrorType {
    SyntaxError,
    UnexpectedToken,
}

#[derive(Debug)]
pub struct QueryError(pub QueryErrorType, pub String); // type and message
