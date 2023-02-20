use crate::{
    builder::IntoCborOwned,
    tests::util::{cbor_to_diag, diag_to_cbor},
    CborPath,
};
use cbor_data::{Cbor, CborBuilder, ItemKind, Writer};
use std::borrow::Cow;

/// Based on https://redis.io/commands/json.arrappend/
fn array_append<'a>(cbor_path: &CborPath, cbor: &'a Cbor, value: &'a Cbor) -> (Cow<'a, Cbor>, Vec::<Option<usize>>) {
    let mut array_sizes = Vec::<Option<usize>>::new();

    let new_value = cbor_path.write(cbor, |old_value| {
        if let ItemKind::Array(array) = old_value.kind() {
            Some(Cow::Owned(CborBuilder::new().write_array(None, |builder| {
                let mut size = 0;
                for item in array {
                    size += 1;
                    builder.write_item(item);
                }
                builder.write_item(value);
                size += 1;
                array_sizes.push(Some(size));
            })))
        } else {
            array_sizes.push(None);
            Some(Cow::Borrowed(old_value))
        }
    });

    (new_value, array_sizes)
}

#[test]
fn simple_array() {
    let cbor = diag_to_cbor(r#"["a","b","c"]"#);
    let new_value = IntoCborOwned::into("d");

    // ["$"]
    let cbor_path = CborPath::builder().build();
    let (new_value, array_sizes) = array_append(&cbor_path, &cbor, &new_value);

    assert_eq!(r#"["a","b","c","d"]"#, cbor_to_diag(&new_value));
    assert_eq!(vec![Some(4)], array_sizes);
}

#[test]
fn deep_array() {
    let cbor = diag_to_cbor(r#"{"foo":["a","b","c"]}"#);
    let new_value = IntoCborOwned::into("d");

    // ["$", "foo"]
    let cbor_path = CborPath::builder().key("foo").build();
    let (new_value, array_sizes) = array_append(&cbor_path, &cbor, &new_value);

    assert_eq!(r#"{"foo":["a","b","c","d"]}"#, cbor_to_diag(&new_value));
    assert_eq!(vec![Some(4)], array_sizes);
}

#[test]
fn multiple_arrays() {
    let cbor = diag_to_cbor(r#"{"foo":["a","b","c"],"bar":[1,2,3,4]}"#);
    let new_value = IntoCborOwned::into("d");

    // ["$", "*"]
    let cbor_path = CborPath::builder().wildcard().build();
    let (new_value, array_sizes) = array_append(&cbor_path, &cbor, &new_value);

    assert_eq!(
        r#"{"foo":["a","b","c","d"],"bar":[1,2,3,4,"d"]}"#,
        cbor_to_diag(&new_value)
    );
    assert_eq!(vec![Some(4), Some(5)], array_sizes);
}

#[test]
fn not_an_array() {
    let cbor = diag_to_cbor(r#"{"foo":12,"bar":[1,2,3]}"#);
    let new_value = IntoCborOwned::into("d");

    // ["$", "*"]
    let cbor_path = CborPath::builder().wildcard().build();
    let (new_value, array_sizes) = array_append(&cbor_path, &cbor, &new_value);

    assert_eq!(r#"{"foo":12,"bar":[1,2,3,"d"]}"#, cbor_to_diag(&new_value));
    assert_eq!(vec![None, Some(4)], array_sizes);
}
