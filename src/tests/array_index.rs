use crate::{builder::IntoCborOwned, tests::util::diag_to_cbor, CborPath};
use cbor_data::{Cbor, ItemKind};

/// Based on https://redis.io/commands/json.arrindex/
fn array_index(
    cbor_path: &CborPath,
    cbor: &Cbor,
    value: &Cbor,
    start: usize,
    stop: usize,
) -> Vec<Option<isize>> {
    let results = cbor_path.read(cbor);
    let mut matches = Vec::<Option<isize>>::new();

    for result in results {
        if let ItemKind::Array(array) = result.kind() {
            let index = array
                .skip(start)
                .take(stop - start)
                .position(|item| item == value)
                .map(|idx| (idx + start) as isize)
                .unwrap_or_else(|| -1);
            matches.push(Some(index))
        } else {
            matches.push(None);
        }
    }

    matches
}

#[test]
fn simple_array() {
    let cbor = diag_to_cbor(r#"["a","b","c"]"#);
    let value = IntoCborOwned::into("c");

    // ["$"]
    let cbor_path = CborPath::builder().build();
    let results = array_index(&cbor_path, &cbor, &value, 0, 3);

    assert_eq!(vec![Some(2)], results);
}

#[test]
fn deep_array() {
    let cbor = diag_to_cbor(r#"{"foo":["a","b","c"]}"#);
    let value = IntoCborOwned::into("c");

    // ["$", "foo"]
    let cbor_path = CborPath::builder().key("foo").build();
    let results = array_index(&cbor_path, &cbor, &value, 0, 3);

    assert_eq!(vec![Some(2)], results);
}

#[test]
fn multiple_arrays() {
    let cbor = diag_to_cbor(r#"{"foo":["a","b","c"],"bar":["c","b","a"]}"#);
    let value = IntoCborOwned::into("c");

    // ["$", "*"]
    let cbor_path = CborPath::builder().wildcard().build();
    let results = array_index(&cbor_path, &cbor, &value, 0, 4);

    assert_eq!(vec![Some(2), Some(0)], results);
}

#[test]
fn not_an_array_not_found() {
    let cbor = diag_to_cbor(r#"{"foo":12,"bar":["a","b","c"]}"#);
    let value = IntoCborOwned::into("d");

    // ["$", "*"]
    let cbor_path = CborPath::builder().wildcard().build();
    let results = array_index(&cbor_path, &cbor, &value, 0, 4);

    assert_eq!(vec![None, Some(-1)], results);
}
