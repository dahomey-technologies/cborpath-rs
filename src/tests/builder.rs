use crate::{
    builder::{eq, gte, rel_path, segment, sing_rel_path, val, value},
    BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, FilterPath,
    FilterSelector, Function, KeySelector, RelativePath, Segment, Selector, SingularPath,
    SingularSegment,
};
use cbor_data::{CborBuilder, Writer};

#[test]
fn build() {
    // [ "$", "a", {"?": {">=": [["@", "b"], 5]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(gte(sing_rel_path().key("b"), val(5)))
        .build();

    assert_eq!(
        CborPath::new(vec![
            Segment::Child(vec![Selector::Key(KeySelector::new(
                CborBuilder::new().write_str("a", None)
            ))]),
            Segment::Child(vec![Selector::Filter(FilterSelector::new(
                BooleanExpr::Comparison(ComparisonExpr::new(
                    Comparable::SingularPath(SingularPath::Rel(vec![SingularSegment::Key(
                        KeySelector::new(CborBuilder::new().write_str("b", None))
                    )])),
                    ComparisonOperator::Gte,
                    Comparable::Value(CborBuilder::new().write_pos(5, None))
                ))
            ))])
        ]),
        cbor_path
    );
}

#[test]
fn value_function() {
    // ["$", {"?": {"==": [{"value": ["@", {"..": "color"}]}, "red"]}}]
    let cbor_path = CborPath::builder()
        .filter(eq(
            value(rel_path().descendant(segment().key("color"))),
            val("red"),
        ))
        .build();

    assert_eq!(
        CborPath::new(vec![Segment::Child(vec![Selector::Filter(
            FilterSelector::new(BooleanExpr::Comparison(ComparisonExpr::new(
                Comparable::Function(Function::Value(FilterPath::Rel(RelativePath::new(vec![
                    Segment::Descendant(vec![Selector::Key(KeySelector::new(
                        CborBuilder::new().write_str("color", None)
                    ))])
                ])))),
                ComparisonOperator::Eq,
                Comparable::Value(CborBuilder::new().write_str("red", None))
            )))
        )])]),
        cbor_path
    );
}
