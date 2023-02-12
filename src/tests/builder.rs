use cbor_data::{CborBuilder, Writer};

use crate::{
    builder::{gte, sing_rel_path, val},
    BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, FilterSelector,
    KeySelector, Segment, Selector, SingularPath, SingularSegment,
};

#[test]
fn build() {
    // [ "$", "a", {"?": {">=": [["@", "b"], 5]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(gte(
            sing_rel_path().key("b"),
            val(5),
        ))
        .build();

    assert_eq!(
        CborPath::new(vec![
            Segment::Child(vec![Selector::Key(KeySelector::new(CborBuilder::new().write_str("a", None)))]),
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
