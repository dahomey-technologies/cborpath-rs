use super::util::{diag_to_bytes, diag_to_cbor};
use crate::{
    builder::{
        abs_path, count, eq, gt, gte, length, lt, lte, neq, rel_path, segment, sing_abs_path,
        sing_rel_path, val,
    },
    CborPath, Error,
};

fn from_value(cbor_diag_str: &str) -> Result<CborPath, Error> {
    let value = diag_to_cbor(cbor_diag_str);
    CborPath::from_value(&value)
}

fn from_bytes(cbor_diag_str: &str) -> Result<CborPath, Error> {
    let value = diag_to_bytes(cbor_diag_str);
    CborPath::from_bytes(&value)
}

#[test]
fn cbor_path_from_value() -> Result<(), Error> {
    let cbor_path: CborPath = from_value(r#""$""#)?;
    assert_eq!(CborPath::root(), cbor_path);

    let cbor_path: CborPath = from_value(r#"["$", "a"]"#)?;
    assert_eq!(CborPath::builder().key("a").build(), cbor_path);

    let cbor_path: CborPath =
        from_value(r##"["$", "foo", 12, 12.12, true, 'binary', {"#": 1}, {":": [0, -1, 1]}]"##)?;
    assert_eq!(
        CborPath::builder()
            .key("foo")
            .key(12)
            .key(12.12)
            .key(true)
            .key("binary".as_bytes())
            .index(1)
            .slice(0, -1, 1)
            .build(),
        cbor_path
    );

    let cbor_path: CborPath = from_value(r##"["$", {"?": ["$", "a"]}, {"?": ["@", "a"]}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(abs_path().key("a"))
            .filter(rel_path().key("a"))
            .build(),
        cbor_path
    );

    let cbor_path: CborPath = from_value(
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
        from_value(r##"["$", {"?": {">=": [{"length": ["@", "authors"]}, 5]}}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(gte(length(sing_rel_path().key("authors")), val(5)))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath =
        from_value(r##"["$", {"?": {">=": [{"count": ["@", {"*": 1}, "authors"]}, 5]}}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(gte(count(rel_path().wildcard().key("authors")), val(5)))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath = from_value(r#"["$", ["a", "b"]]"#)?;
    assert_eq!(
        CborPath::builder()
            .child(segment().key("a").key("b"))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath = from_value(r#"["$",{"..":"a"}]"#)?;
    assert_eq!(
        CborPath::builder()
            .descendant(segment().key("a"))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath = from_value(r#"["$",{"..":["a","b"]}]"#)?;
    assert_eq!(
        CborPath::builder()
            .descendant(segment().key("a").key("b"))
            .build(),
        cbor_path,
    );

    Ok(())
}

#[test]
fn cbor_path_from_bytes() -> Result<(), Error> {
    let cbor_path: CborPath = from_bytes(r#""$""#)?;
    assert_eq!(CborPath::root(), cbor_path);

    let cbor_path: CborPath = from_bytes(r#"["$", "a"]"#)?;
    assert_eq!(CborPath::builder().key("a").build(), cbor_path);

    let cbor_path: CborPath =
        from_bytes(r##"["$", "foo", 12, 12.12, true, 'binary', {"#": 1}, {":": [0, -1, 1]}]"##)?;
    assert_eq!(
        CborPath::builder()
            .key("foo")
            .key(12)
            .key(12.12)
            .key(true)
            .key("binary".as_bytes())
            .index(1)
            .slice(0, -1, 1)
            .build(),
        cbor_path
    );

    let cbor_path: CborPath = from_bytes(r##"["$", {"?": ["$", "a"]}, {"?": ["@", "a"]}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(abs_path().key("a"))
            .filter(rel_path().key("a"))
            .build(),
        cbor_path
    );

    let cbor_path: CborPath = from_bytes(
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
        from_bytes(r##"["$", {"?": {">=": [{"length": ["@", "authors"]}, 5]}}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(gte(length(sing_rel_path().key("authors")), val(5)))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath =
        from_bytes(r##"["$", {"?": {">=": [{"count": ["@", {"*": 1}, "authors"]}, 5]}}]"##)?;
    assert_eq!(
        CborPath::builder()
            .filter(gte(count(rel_path().wildcard().key("authors")), val(5)))
            .build(),
        cbor_path,
    );

    let cbor_path: CborPath = from_bytes(r#"["$", ["a", "b"]]"#)?;
    assert_eq!(
        CborPath::builder()
            .child(segment().key("a").key("b"))
            .build(),
        cbor_path,
    );

    Ok(())
}
