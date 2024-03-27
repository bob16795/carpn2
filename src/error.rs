use thiserror::Error;

#[derive(Error, Debug)]
pub enum CarpnError {
    #[error("parser error: {0}")]
    Parse(#[from] ParseError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("the prototype has no name")]
    PrototypeMissingName,
    #[error("unexpected eof")]
    ParserEOF,
    #[error("invalid expression")]
    InvalidExpression,
    #[error("proc is missing body")]
    MissingBody,
    #[error("missing closing curly")]
    MissingCloseCurly,
    #[error("missing struct name")]
    MissingStructName,
    #[error("unreachable")]
    Unreachable,
}
