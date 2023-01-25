use std::str;

#[derive(Debug)]
pub enum Error {
    Parser(String),
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::Parser(format!("invalid UTF8 string: {e}"))
    }
}

impl From<regex::Error> for Error {
    fn from(e: regex::Error) -> Self {
        Error::Parser(format!("invalid regex: {e}"))
    }
}