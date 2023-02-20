use std::{
    fmt::{self, Display},
    str,
};

/// All error kinds
#[derive(Debug)]
pub enum Error {
    /// Raised if an error occurs while converting a [`Cbor value`](https://docs.rs/cbor-data/latest/cbor_data/struct.Cbor.html) to a [`CborPath`](crate::CborPath)
    /// # See
    /// [`CborPath::from_value`](crate::CborPath::from_value)
    Conversion(String),
    /// Raised if an error occurs while parsing an input value to evaluate
    /// # See
    /// [`CborPath::from_bytes`](crate::CborPath::from_bytes)
    Parsing(String),
    /// Raised if an error occurs while calling [`CborPath::write`](crate::CborPath::write) 
    /// or [`CborPath::write_from_bytes`](crate::CborPath::write_from_bytes)
    Write(String,)
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

impl From<cbor_data::ParseError> for Error {
    fn from(e: cbor_data::ParseError) -> Self {
        Error::Parsing(e.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
