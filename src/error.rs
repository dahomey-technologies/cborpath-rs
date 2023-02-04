use std::{
    fmt::{self, Display},
    str,
};

#[derive(Debug)]
pub enum Error {
    Conversion(String),
    Serialization(String)
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::Conversion(format!("invalid UTF8 string: {e}"))
    }
}

impl From<regex::Error> for Error {
    fn from(e: regex::Error) -> Self {
        Error::Conversion(format!("invalid regex: {e}"))
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(e: std::num::TryFromIntError) -> Self {
        Error::Conversion(format!("invalid integer conversion: {e}"))
    }
}

impl From<ciborium::value::Error> for Error {
    fn from(e: ciborium::value::Error) -> Self {
        Error::Conversion(e.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
