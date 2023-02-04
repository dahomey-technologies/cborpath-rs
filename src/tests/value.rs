use crate::{
    BooleanExpr, CborPath, Comparable, ComparisonOperator, Error, Function, Path, Segment,
    Selector, SingularPath, SingularSegment,
};
use cbor_diag::parse_diag;
use ciborium::{value::Value, de::from_reader};

fn from_cbor(cbor_diag_str: &str) -> Result<CborPath, Error> {
    let buf = parse_diag(cbor_diag_str).unwrap().to_bytes();
    let value: Value = from_reader(buf.as_slice()).unwrap();
    CborPath::from_value(&value)
}

#[test]
fn convert_to_cbor_path() -> Result<(), Error> {
    let cbor_path: CborPath = from_cbor(r#""$""#)?;
    assert_eq!(cbor_path, CborPath::new(vec![]));

    let cbor_path: CborPath = from_cbor(r#"["$", "a"]"#)?;
    assert_eq!(
        cbor_path,
        CborPath::new(vec![Segment::Child(vec![Selector::key("a".into())])])
    );

    let cbor_path: CborPath =
        from_cbor(r##"["$", "foo", 12, 12.12, true, 'binary', {"#": 1}, {":": [0, -1, 1]}]"##)?;
    assert_eq!(
        cbor_path,
        CborPath::new(vec![
            Segment::Child(vec![Selector::key("foo".into()),]),
            Segment::Child(vec![Selector::key(12.into()),]),
            Segment::Child(vec![Selector::key(12.12.into()),]),
            Segment::Child(vec![Selector::key(true.into()),]),
            Segment::Child(vec![Selector::key(Value::Bytes(
                "binary".as_bytes().to_vec()
            )),]),
            Segment::Child(vec![Selector::index(1)]),
            Segment::Child(vec![Selector::slice(0, -1, 1)])
        ])
    );

    let cbor_path: CborPath = from_cbor(r##"["$", {"?": ["$", "a"]}, {"?": ["@", "a"]}]"##)?;
    assert_eq!(
        cbor_path,
        CborPath::new(vec![
            Segment::Child(vec![Selector::filter(BooleanExpr::path(Path::abs(vec![
                Segment::Child(vec![Selector::key("a".into()),])
            ]))),]),
            Segment::Child(vec![Selector::filter(BooleanExpr::path(Path::rel(vec![
                Segment::Child(vec![Selector::key("a".into()),])
            ]))),])
        ])
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
        cbor_path,
        CborPath::new(vec![
            Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
                Comparable::Value(12.into()),
                ComparisonOperator::Lt,
                Comparable::Value(13.into())
            )),]),
            Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
                Comparable::Value(12.into()),
                ComparisonOperator::Lte,
                Comparable::Value(13.into())
            )),]),
            Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
                Comparable::Value(12.into()),
                ComparisonOperator::Neq,
                Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::index(1)])),
            )),]),
            Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key("a".into())])),
                ComparisonOperator::Eq,
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key("b".into())])),
            )),]),
            Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
                Comparable::Value(12.into()),
                ComparisonOperator::Gte,
                Comparable::Value(13.into())
            )),]),
            Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
                Comparable::Value(12.into()),
                ComparisonOperator::Gt,
                Comparable::Value(13.into())
            )),]),
        ])
    );

    let cbor_path: CborPath =
        from_cbor(r##"["$", {"?": {">=": [{"length": ["@", "authors"]}, 5]}}]"##)?;
    assert_eq!(
        cbor_path,
        CborPath::new(vec![Segment::Child(vec![Selector::filter(
            BooleanExpr::comparison(
                Comparable::Function(Function::length(Comparable::SingularPath(
                    SingularPath::rel(vec![SingularSegment::key("authors".into())])
                ))),
                ComparisonOperator::Gte,
                Comparable::Value(5.into())
            )
        ),]),])
    );

    let cbor_path: CborPath =
        from_cbor(r##"["$", {"?": {">=": [{"count": ["@", {"*": 1}, "authors"]}, 5]}}]"##)?;
    assert_eq!(
        cbor_path,
        CborPath::new(vec![Segment::Child(vec![Selector::filter(
            BooleanExpr::comparison(
                Comparable::Function(Function::count(Path::rel(vec![
                    Segment::Child(vec![Selector::wildcard()]),
                    Segment::Child(vec![Selector::key("authors".into())])
                ]))),
                ComparisonOperator::Gte,
                Comparable::Value(5.into())
            )
        ),]),])
    );

    let cbor_path: CborPath = from_cbor(r#"["$", ["a", "b"]]"#)?;
    assert_eq!(
        cbor_path,
        CborPath::new(vec![Segment::Child(vec![
            Selector::key("a".into()),
            Selector::key("b".into()),
        ])])
    );

    Ok(())
}
