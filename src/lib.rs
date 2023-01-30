#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
cborpath is a CborPath engine written in Rust.
*/

mod cbor_path;
mod deserialization;
mod error;
mod parsing;

pub use cbor_path::*;
pub use error::*;

#[cfg(test)]
mod tests;
