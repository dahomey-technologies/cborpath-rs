use cbor_data::{CborOwned, Cbor};
use cbor_diag::{parse_diag, parse_bytes};

pub fn diag_to_cbor(cbor_diag_str: &str) -> CborOwned {
    let buf = diag_to_bytes(cbor_diag_str);
    CborOwned::canonical(buf).unwrap()
}

pub fn diag_to_bytes(cbor_diag_str: &str) -> Vec<u8> {
    parse_diag(cbor_diag_str).unwrap().to_bytes()
}

pub fn cbor_to_diag(cbor: &Cbor) -> String {
    bytes_to_diag(cbor.as_ref())
}

pub fn bytes_to_diag(cbor: &[u8]) -> String {
    parse_bytes(cbor).unwrap().to_diag()
}

pub fn log_try_init() {
    let _ = env_logger::builder()
        .format_target(false)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Trace)
        .target(env_logger::Target::Stdout)
        .is_test(true)
        .try_init();
}
