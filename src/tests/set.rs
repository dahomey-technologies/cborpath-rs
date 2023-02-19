use crate::{
    builder::{segment, IntoCborOwned},
    tests::util::{cbor_to_diag, diag_to_cbor},
    CborPath, Error,
};
use cbor_data::CborOwned;

#[test]
fn simple_array() -> Result<(), Error> {
    let cbor = diag_to_cbor(r#"["a", "b", "c"]"#);
    let new_value: CborOwned = IntoCborOwned::into("d");

    let cbor_path = CborPath::builder().index(1).build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"["a","d","c"]"#, cbor_to_diag(&result));

    Ok(())
}

#[test]
fn simple_map() -> Result<(), Error> {
    let cbor = diag_to_cbor(r#"{"a":1,"b":2}"#);
    let new_value: CborOwned = IntoCborOwned::into(3);

    let cbor_path = CborPath::builder().key("b").build();
    let result = cbor_path.set(&cbor, &new_value);

    assert_eq!(r#"{"a":1,"b":3}"#, cbor_to_diag(&result));

    Ok(())
}

#[test]
fn store() -> Result<(), Error> {
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
        result
    );

    Ok(())
}
