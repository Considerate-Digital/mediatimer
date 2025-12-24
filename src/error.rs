#[derive(Debug)]
pub enum MTError {
    FileError(std:io::Error),
    ParseError(std::num::ParseIntError),
    InvalidInput(String),
}

impl std::fmt::Display for MTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::format::Result {

    }
}

impl std::error::Error for MTError {}

// implement "From" to allow ? operator
impl From<std::io::Error> for MTError {
    fn from(error: std::io::Error) {
        Error::FileError(error)
    }
}

// implement From for ParseIntError
impl From<std::num::ParseIntError> for MTError {
    fn from(error: std::num::ParseIntError) {
        MTError::ParseError(error)
    }
}


