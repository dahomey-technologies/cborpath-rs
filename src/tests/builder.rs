use crate::{
    builder::{gte, sing_rel_path, val},
    BooleanExpr, CborPath, Comparable, ComparisonOperator, Segment, Selector, SingularPath,
    SingularSegment, KeySelector, FilterSelector, ComparisonExpr,
};

#[test]
fn build() {
    // [ "$", "a", {"?": {">=": [["@", "b"], 5]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(gte(sing_rel_path().key("b"), val(5)))
        .build();

    assert_eq!(
        CborPath::new(vec![
            Segment::Child(vec![Selector::Key(KeySelector::new("a".into()))]),
            Segment::Child(vec![Selector::Filter(FilterSelector::new(BooleanExpr::Comparison(ComparisonExpr::new(
                Comparable::SingularPath(SingularPath::Rel(vec![SingularSegment::Key(KeySelector::new("b".into()))])),
                ComparisonOperator::Gte,
                Comparable::Value(5.into())
            ))))])
        ]),
        cbor_path
    );
}
