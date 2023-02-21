use crate::{
    tests::util::{cbor_to_diag, diag_to_cbor, log_try_init},
    CborPath,
};

#[test]
fn simple_array() {
    let cbor = diag_to_cbor(r#"["a","b","c"]"#);

    let cbor_path = CborPath::builder().index(1).build();
    let result = cbor_path.delete(&cbor).unwrap();

    assert_eq!(r#"["a","c"]"#, cbor_to_diag(&result));
}

#[test]
fn deep_array() {
    let cbor = diag_to_cbor(r#"{"foo":["a","b","c"]}"#);

    let cbor_path = CborPath::builder().key("foo").index(1).build();
    let result = cbor_path.delete(&cbor).unwrap();

    assert_eq!(r#"{"foo":["a","c"]}"#, cbor_to_diag(&result));
}

#[test]
fn simple_map() {
    let cbor = diag_to_cbor(r#"{"a":1,"b":2}"#);

    let cbor_path = CborPath::builder().key("b").build();
    let result = cbor_path.delete(&cbor).unwrap();

    assert_eq!(r#"{"a":1}"#, cbor_to_diag(&result));
}

#[test]
fn deep_map() {
    let cbor = diag_to_cbor(r#"{"foo":{"a":1,"b":2}}"#);

    let cbor_path = CborPath::builder().key("foo").key("b").build();
    let result = cbor_path.delete(&cbor).unwrap();

    assert_eq!(r#"{"foo":{"a":1}}"#, cbor_to_diag(&result));
}

#[test]
fn map_as_value() {
    log_try_init();

    let cbor = diag_to_cbor(r#"{"foo":{"a":{"b":1},"c":2}}"#);

    let cbor_path = CborPath::builder().key("foo").key("a").build();
    let result = cbor_path.delete(&cbor).unwrap();

    assert_eq!(r#"{"foo":{"c":2}}"#, cbor_to_diag(&result));
}

#[test]
fn array_as_value() {
    log_try_init();
    
    let cbor = diag_to_cbor(r#"{"foo":{"a":[1,2,3],"c":2}}"#);

    let cbor_path = CborPath::builder().key("foo").key("a").build();
    let result = cbor_path.delete(&cbor).unwrap();

    assert_eq!(r#"{"foo":{"c":2}}"#, cbor_to_diag(&result));
}
