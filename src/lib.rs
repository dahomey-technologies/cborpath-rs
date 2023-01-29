
#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
cborpath is a CborPath engine written in Rust.
*/

mod error;
mod cbor_path;
mod deserialization;

pub use error::*;
pub use cbor_path::*;

#[cfg(test)]
mod tests;
