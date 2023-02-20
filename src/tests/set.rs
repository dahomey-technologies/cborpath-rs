use crate::{
    builder::{segment, IntoCborOwned},
    tests::util::{cbor_to_diag, diag_to_cbor},
    CborPath,
};
use cbor_data::CborOwned;

#[test]
fn primitive() {
    let cbor = diag_to_cbor(r#"12"#);
    let new_value: CborOwned = IntoCborOwned::into(13);

    let cbor_path = CborPath::builder().build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"13"#, cbor_to_diag(&result));
}

#[test]
fn simple_array() {
    let cbor = diag_to_cbor(r#"["a", "b", "c"]"#);
    let new_value: CborOwned = IntoCborOwned::into("d");

    let cbor_path = CborPath::builder().index(1).build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"["a","d","c"]"#, cbor_to_diag(&result));
}

#[test]
fn simple_map() {
    let cbor = diag_to_cbor(r#"{"a":1,"b":2}"#);
    let new_value: CborOwned = IntoCborOwned::into(3);

    let cbor_path = CborPath::builder().key("b").build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"{"a":1,"b":3}"#, cbor_to_diag(&result));
}

#[test]
fn map() {
    let cbor = diag_to_cbor(r#"{"foo":{"a":{"b":1},"c":2}}"#);
    let new_value: CborOwned = IntoCborOwned::into(12);

    let cbor_path = CborPath::builder().key("foo").key("a").build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"{"foo":{"a":12,"c":2}}"#, cbor_to_diag(&result));
}

#[test]
fn store() {
    let cbor = diag_to_cbor(
        r#"
    { 
      "store": {
        "book": [
          { "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          { "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          },
          { "category": "fiction",
            "author": "Herman Melville",
            "title": "Moby Dick",
            "isbn": "0-553-21311-3",
            "price": 8.99
          },
          { "category": "fiction",
            "author": "J. R. R. Tolkien",
            "title": "The Lord of the Rings",
            "isbn": "0-395-19395-8",
            "price": 22.99
          }
        ],
        "bicycle": {
          "color": "red",
          "price": 399
        }
      }
    }"#,
    );

    let new_value: CborOwned = IntoCborOwned::into("new_book");

    // all books
    let cbor_path = CborPath::builder()
        .descendant(segment().key("book"))
        .wildcard()
        .build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(
        diag_to_cbor(
            r#"{"store":{"book":["new_book","new_book","new_book","new_book"],"bicycle":{"color":"red","price":399}}}"#
        ),
        result.into_owned()
    );
}

#[test]
fn no_match() {
    let cbor = diag_to_cbor(r#"["a", "b", "c"]"#);
    let new_value: CborOwned = IntoCborOwned::into("d");

    let cbor_path = CborPath::builder().index(4).build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"["a","b","c"]"#, cbor_to_diag(&result));
}
