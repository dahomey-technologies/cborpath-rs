use cbor_data::CborOwned;
use cbor_diag::parse_diag;

pub fn diag_to_cbor(cbor_diag_str: &str) -> CborOwned {
    let buf = parse_diag(cbor_diag_str).unwrap().to_bytes();
    CborOwned::canonical(&buf).unwrap()
}

pub fn diag_to_bytes(cbor_diag_str: &str) -> Vec<u8> {
    parse_diag(cbor_diag_str).unwrap().to_bytes()
}
