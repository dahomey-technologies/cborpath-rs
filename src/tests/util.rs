use cbor_diag::{parse_diag, parse_bytes};
use ciborium::{value::Value, de::from_reader};

pub fn diag_to_value(cbor_diag_str: &str) -> Value {
    let buf = parse_diag(cbor_diag_str).unwrap().to_bytes();
    from_reader(buf.as_slice()).unwrap()
}

pub fn diag_to_bytes(cbor_diag_str: &str) -> Vec<u8> {
    parse_diag(cbor_diag_str).unwrap().to_bytes()
}

pub fn bytes_to_diag(bytes: &[u8]) -> String {
    parse_bytes(bytes).unwrap().to_diag()
}