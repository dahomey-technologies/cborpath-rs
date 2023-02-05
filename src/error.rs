use std::{
    fmt::{self, Display},
    str,
};

/// All error kinds
#[derive(Debug)]
pub enum Error {
    /// Raised if an error occurs while converting a [`Value`](https://docs.rs/ciborium/latest/ciborium/value/enum.Value.html) to a [`CborPath`](crate::CborPath)
    /// # See
    /// [`CborPath::from_value`](crate::CborPath::from_value)
    Conversion(String),
    /// Raised if an error occurs while deserializing a [`CborPath`](crate::CborPath)
    /// # See
    /// [`CborPath::from_reader`](crate::CborPath::from_reader)
    Deserialization(String),
    /// Raised by external crates like [`ciborium`](https://docs.rs/ciborium)
    External(String)
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
