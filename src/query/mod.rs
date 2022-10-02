pub mod parse;

type Result<T> = std::result::Result<T, QueryError>;

#[derive(Debug)]
pub enum QueryErrorType {
    SyntaxError,
}

#[derive(Debug)]
pub struct QueryError(QueryErrorType, String); // type and message
