use crate::{
    builder::{
        abs_path, count, eq, gt, gte, length, lt, lte, neq, rel_path, segment, sing_abs_path,
        sing_rel_path, val,
    },
    CborPath, Error,
};
use ciborium::value::Value;

use super::util::diag_to_value;

fn from_cbor(cbor_diag_str: &str) -> Result<CborPath, Error> {
    let value = diag_to_value(cbor_diag_str);
    CborPath::from_value(&value)
}

#[test]
fn deserialize_cbor_path() -> Result<(), Error> {
    let cbor_path: CborPath = from_cbor(r#""$""#)?;
    assert_eq!(CborPath::builder().build(), cbor_path);

    let cbor_path: CborPath = from_cbor(r#"["$", "a"]"#)?;
    assert_eq!(CborPath::builder().key("a").build(), cbor_path);

    let cbor_path: CborPath =
        from_cbor(r##"["$", "foo", 12, 12.12, true, 'binary', {"#": 1}, {":": [0, -1, 1]}]"##)?;
    assert_eq!(
        CborPath::builder()
            .key("foo")
            .key(12)
            .key(12.12)
            .key(true)
            .key(Value::Bytes("binary".as_bytes().to_vec()))
            .index(1)
            .slice(0, -1, 1)
            .build(),
        cbor_path
    );

    let cbor_path: CborPath = from_cbor(r##"["$", {"?": ["$", "a"]}, {"?": ["@", "a"]}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(abs_path().key("a"))
            .filter(rel_path().key("a"))
            .build(),
        cbor_path
    );

    let cbor_path: CborPath = from_cbor(
        r##"[
        "$", 
        {"?": {"<": [12, 13]}}, 
        {"?": {"<=": [12, 13]}}, 
        {"?": {"!=": [12, ["$", {"#": 1}]]}}, 
        {"?": {"==": [["$", "a"], ["@", "b"]]}}, 
        {"?": {">=": [12, 13]}}, 
        {"?": {">": [12, 13]}}]"##,
    )?;
    assert_eq!(
        CborPath::builder()
            .filter(lt(val(12), val(13)))
            .filter(lte(val(12), val(13)))
            .filter(neq(val(12), sing_abs_path().index(1)))
            .filter(eq(sing_abs_path().key("a"), sing_rel_path().key("b")))
            .filter(gte(val(12), val(13)))
            .filter(gt(val(12), val(13)))
            .build(),
        cbor_path
    );

    let cbor_path: CborPath =
        from_cbor(r##"["$", {"?": {">=": [{"length": ["@", "authors"]}, 5]}}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(gte(length(sing_rel_path().key("authors")), val(5)))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath =
        from_cbor(r##"["$", {"?": {">=": [{"count": ["@", {"*": 1}, "authors"]}, 5]}}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(gte(count(rel_path().wildcard().key("authors")), val(5)))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath = from_cbor(r#"["$", ["a", "b"]]"#)?;
    assert_eq!(
        CborPath::builder()
            .child(segment().key("a").key("b"))
            .build(),
        cbor_path,
    );

    Ok(())
}
