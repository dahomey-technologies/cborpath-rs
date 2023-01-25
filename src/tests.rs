use crate::{
    BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, Error, Function, Path,
    Segment, Selector, SingularPath, SingularSegment,
};
use serde_cbor::Value;
use std::collections::BTreeMap;

#[test]
fn evaluate_root() {
    let value = Value::Map(BTreeMap::from([(
        Value::Text("k".to_owned()),
        Value::Text("v".to_owned()),
    )]));

    let cbor_path = CborPath::new(vec![]);
    let result = cbor_path.evaluate(&value);

    assert_eq!(
        vec![&Value::Map(BTreeMap::from([(
            Value::Text("k".to_owned()),
            Value::Text("v".to_owned()),
        )]))],
        result
    );
}

#[test]
fn evaluate_key() {
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("o".to_owned()),
            Value::Map(BTreeMap::from([(
                Value::Text("j j".to_owned()),
                Value::Map(BTreeMap::from([(
                    Value::Text("k k".to_owned()),
                    Value::Integer(3),
                )])),
            )])),
        ),
        (
            Value::Text("'".to_owned()),
            Value::Map(BTreeMap::from([(
                Value::Text("@".to_owned()),
                Value::Integer(2),
            )])),
        ),
    ]));

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("o".to_owned()))]),
        Segment::Child(vec![Selector::key(Value::Text("j j".to_owned()))]),
        Segment::Child(vec![Selector::key(Value::Text("k k".to_owned()))]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Integer(3)], result);

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("'".to_owned()))]),
        Segment::Child(vec![Selector::key(Value::Text("@".to_owned()))]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Integer(2)], result);
}

#[test]
fn evaluate_wildcard() {
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("o".to_owned()),
            Value::Map(BTreeMap::from([
                (Value::Text("j".to_owned()), Value::Integer(1)),
                (Value::Text("k".to_owned()), Value::Integer(2)),
            ])),
        ),
        (
            Value::Text("a".to_owned()),
            Value::Array(vec![Value::Integer(5), Value::Integer(3)]),
        ),
    ]));

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::Wildcard])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![
            &Value::Array(vec![Value::Integer(5), Value::Integer(3)]),
            &Value::Map(BTreeMap::from([
                (Value::Text("j".to_owned()), Value::Integer(1)),
                (Value::Text("k".to_owned()), Value::Integer(2)),
            ])),
        ],
        result
    );

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("o".to_owned()))]),
        Segment::Child(vec![Selector::Wildcard]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Integer(1), &Value::Integer(2)], result);

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::Wildcard]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Integer(5), &Value::Integer(3)], result);
}

#[test]
fn evaluate_index() {
    let value = Value::Array(vec![
        Value::Text("a".to_owned()),
        Value::Text("b".to_owned()),
    ]);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::index(1)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Text("b".to_owned())], result);

    let value = Value::Array(vec![
        Value::Text("a".to_owned()),
        Value::Text("b".to_owned()),
    ]);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::index(-2)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Text("a".to_owned())], result);
}

#[test]
fn evaluate_slice() {
    let value = Value::Array(vec![
        Value::Text("a".to_owned()),
        Value::Text("b".to_owned()),
        Value::Text("c".to_owned()),
        Value::Text("d".to_owned()),
        Value::Text("e".to_owned()),
        Value::Text("f".to_owned()),
        Value::Text("g".to_owned()),
    ]);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(1, 3, 1)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![&Value::Text("b".to_owned()), &Value::Text("c".to_owned())],
        result
    );

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(1, 5, 2)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![&Value::Text("b".to_owned()), &Value::Text("d".to_owned())],
        result
    );

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(5, 1, -2)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![&Value::Text("f".to_owned()), &Value::Text("d".to_owned())],
        result
    );

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(6, -8, -1)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![
            &Value::Text("g".to_owned()),
            &Value::Text("f".to_owned()),
            &Value::Text("e".to_owned()),
            &Value::Text("d".to_owned()),
            &Value::Text("c".to_owned()),
            &Value::Text("b".to_owned()),
            &Value::Text("a".to_owned()),
        ],
        result
    );
}

#[test]
fn comparison() {
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("obj".to_owned()),
            Value::Map(BTreeMap::from([(
                Value::Text("x".to_owned()),
                Value::Text("y".to_owned()),
            )])),
        ),
        (
            Value::Text("arr".to_owned()),
            Value::Array(vec![Value::Integer(2), Value::Integer(3)]),
        ),
    ]));

    // $.absent1 == $.absent2
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Eq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent2".to_owned(),
        ))])),
    );
    assert!(comparison.evaluate(&value, &value));

    // $.absent1 <= $.absent2
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Lte,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent2".to_owned(),
        ))])),
    );
    assert!(comparison.evaluate(&value, &value));

    // $.absent1 == "g"
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Eq,
        Comparable::Value(Value::Text("g".to_owned())),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.absent1 != $.absent2
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Neq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent2".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.absent1 != "g"
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Neq,
        Comparable::Value(Value::Text("g".to_owned())),
    );
    assert!(comparison.evaluate(&value, &value));

    // 1 <= 2
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(1)),
        ComparisonOperator::Lte,
        Comparable::Value(Value::Integer(2)),
    );
    assert!(comparison.evaluate(&value, &value));

    // 1 > 2
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(1)),
        ComparisonOperator::Gt,
        Comparable::Value(Value::Integer(2)),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 13 == "13"
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(13)),
        ComparisonOperator::Eq,
        Comparable::Value(Value::Text("13".to_owned())),
    );
    assert!(!comparison.evaluate(&value, &value));

    // "a" <= "b"
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Text("a".to_owned())),
        ComparisonOperator::Lte,
        Comparable::Value(Value::Text("b".to_owned())),
    );
    assert!(comparison.evaluate(&value, &value));

    // "a" > "b"
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Text("a".to_owned())),
        ComparisonOperator::Gt,
        Comparable::Value(Value::Text("b".to_owned())),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.obj == $.arr
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "obj".to_owned(),
        ))])),
        ComparisonOperator::Eq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.obj != $.arr
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "obj".to_owned(),
        ))])),
        ComparisonOperator::Neq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(comparison.evaluate(&value, &value));

    // $.obj == $.obj
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "obj".to_owned(),
        ))])),
        ComparisonOperator::Eq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "obj".to_owned(),
        ))])),
    );
    assert!(comparison.evaluate(&value, &value));

    // $.obj != $.obj
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "obj".to_owned(),
        ))])),
        ComparisonOperator::Neq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "obj".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.arr == $.arr
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
        ComparisonOperator::Eq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(comparison.evaluate(&value, &value));

    // $.arr != $.arr
    let comparison = ComparisonExpr(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
        ComparisonOperator::Neq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 <= $.arr
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(1)),
        ComparisonOperator::Lte,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 >= $.arr
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(1)),
        ComparisonOperator::Gte,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 > $.arr
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(1)),
        ComparisonOperator::Gt,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 < $.arr
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Integer(1)),
        ComparisonOperator::Lt,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // true <= true
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Bool(true)),
        ComparisonOperator::Lte,
        Comparable::Value(Value::Bool(true)),
    );
    assert!(comparison.evaluate(&value, &value));

    // true > true
    let comparison = ComparisonExpr(
        Comparable::Value(Value::Bool(true)),
        ComparisonOperator::Gt,
        Comparable::Value(Value::Bool(true)),
    );
    assert!(!comparison.evaluate(&value, &value));
}

#[test]
fn filter() -> Result<(), Error> {
    // {
    //     "a": [3, 5, 1, 2, 4, 6, {"b": "j"}, {"b": "k"},
    //     {"b": {}}, {"b": "kilo"}],
    //     "o": {"p": 1, "q": 2, "r": 3, "s": 5, "t": {"u": 6}},
    //     "e": "f"
    // }
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("a".to_owned()),
            Value::Array(vec![
                Value::Integer(3),
                Value::Integer(5),
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(4),
                Value::Integer(6),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Text("j".to_owned()),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Text("k".to_owned()),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Map(BTreeMap::new()),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Text("kilo".to_owned()),
                )])),
            ]),
        ),
        (
            Value::Text("o".to_owned()),
            Value::Map(BTreeMap::from([
                (Value::Text("p".to_owned()), Value::Integer(1)),
                (Value::Text("q".to_owned()), Value::Integer(2)),
                (Value::Text("r".to_owned()), Value::Integer(3)),
                (Value::Text("s".to_owned()), Value::Integer(5)),
                (
                    Value::Text("t".to_owned()),
                    Value::Map(BTreeMap::from([(
                        Value::Text("u".to_owned()),
                        Value::Integer(6),
                    )])),
                ),
            ])),
        ),
        (Value::Text("e".to_owned()), Value::Text("f".to_owned())),
    ]));

    // ["$", "a", {"?": {"==": [["@", "b"], "kilo"]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(Value::Text(
                "b".to_owned(),
            ))])),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Text("kilo".to_owned())),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "kilo"}]
    assert_eq!(
        vec![&Value::Map(BTreeMap::from([(
            Value::Text("b".to_owned()),
            Value::Text("kilo".to_owned()),
        )])),],
        result
    );

    // ["$", "a", {"?": {">": [["@"], 3]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![])),
            ComparisonOperator::Gt,
            Comparable::Value(Value::Integer(3)),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![5, 4, 6]
    assert_eq!(
        vec![&Value::Integer(5), &Value::Integer(4), &Value::Integer(6)],
        result
    );

    // ["$", "a", {"?": ["@", "b"]]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::path(Path::rel(vec![
            Segment::Child(vec![Selector::key(Value::Text("b".to_owned()))]),
        ])))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "j"}, {"b": "k"}, {"b": {}}, {"b": "kilo"}]
    assert_eq!(
        vec![
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("j".to_owned()),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("k".to_owned()),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Map(BTreeMap::new()),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("kilo".to_owned()),
            )])),
        ],
        result
    );

    // ["$", {"?": ["@", "*"]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::filter(
        BooleanExpr::path(Path::rel(vec![Segment::Child(vec![Selector::Wildcard])])),
    )])]);
    let result = cbor_path.evaluate(&value);
    // vec![[3, 5, 1, 2, 4, 6, {"b": "j"}, {"b": "k"}, {"b": {}}, {"b": "kilo"}], {"p": 1, "q": 2, "r": 3, "s": 5, "t": {"u": 6}}]
    assert_eq!(
        vec![
            &Value::Array(vec![
                Value::Integer(3),
                Value::Integer(5),
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(4),
                Value::Integer(6),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Text("j".to_owned()),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Text("k".to_owned()),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Map(BTreeMap::new()),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("b".to_owned()),
                    Value::Text("kilo".to_owned()),
                )])),
            ]),
            &Value::Map(BTreeMap::from([
                (Value::Text("p".to_owned()), Value::Integer(1)),
                (Value::Text("q".to_owned()), Value::Integer(2)),
                (Value::Text("r".to_owned()), Value::Integer(3)),
                (Value::Text("s".to_owned()), Value::Integer(5)),
                (
                    Value::Text("t".to_owned()),
                    Value::Map(BTreeMap::from([(
                        Value::Text("u".to_owned()),
                        Value::Integer(6),
                    )])),
                ),
            ]))
        ],
        result
    );

    // ["$", {"?": ["@", {"?": ["@", "b"]}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::filter(
        BooleanExpr::path(Path::rel(vec![Segment::Child(vec![Selector::filter(
            BooleanExpr::path(Path::rel(vec![Segment::Child(vec![Selector::key(
                Value::Text("b".to_owned()),
            )])])),
        )])])),
    )])]);
    let result = cbor_path.evaluate(&value);
    // vec![[3, 5, 1, 2, 4, 6, {"b": "j"}, {"b": "k"}, {"b": {}}, {"b": "kilo"}]]
    assert_eq!(
        vec![&Value::Array(vec![
            Value::Integer(3),
            Value::Integer(5),
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(6),
            Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("j".to_owned()),
            )])),
            Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("k".to_owned()),
            )])),
            Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Map(BTreeMap::new()),
            )])),
            Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("kilo".to_owned()),
            )])),
        ])],
        result
    );

    // ["$", "o", [{"?": {"<", [["@"], 3]}}, {"?": {"<", [["@"], 3]}}]]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("o".to_owned()))]),
        Segment::Child(vec![
            Selector::filter(BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(Value::Integer(3)),
            )),
            Selector::filter(BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(Value::Integer(3)),
            )),
        ]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![1, 2, 1, 2]
    assert_eq!(
        vec![
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(1),
            &Value::Integer(2)
        ],
        result
    );

    // ["$", "a", {"?": {'||': [{"<": [["@"], 2]}, {"==": [["@", "b"], "k"]}]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::or(
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(Value::Integer(2)),
            ),
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(
                    Value::Text("b".to_owned()),
                )])),
                ComparisonOperator::Eq,
                Comparable::Value(Value::Text("k".to_owned())),
            ),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![1, {"b": "k"}]
    assert_eq!(
        vec![
            &Value::Integer(1),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("k".to_owned()),
            )])),
        ],
        result
    );

    // ["$", "a", {"?": {"match": [["@", "b"], "[jk]"]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::function(
            Function::_match(
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(
                    Value::Text("b".to_owned()),
                )])),
                "[jk]",
            )?,
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "j"}, {"b": "k"}]
    assert_eq!(
        vec![
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("j".to_owned()),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("k".to_owned()),
            )])),
        ],
        result
    );

    // ["$", "a", {"?": {"search": [["@", "b"], "[jk]"]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::function(
            Function::search(
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(
                    Value::Text("b".to_owned()),
                )])),
                "[jk]",
            )?,
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "j"}, {"b": "k"}]
    assert_eq!(
        vec![
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("j".to_owned()),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("k".to_owned()),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("b".to_owned()),
                Value::Text("kilo".to_owned()),
            )])),
        ],
        result
    );

    // ["$", "o", {"?": {"&&": [{">": [["@"], 1]}, {"<": [["@", 4]]}]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("o".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::and(
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Gt,
                Comparable::Value(Value::Integer(1)),
            ),
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(Value::Integer(4)),
            ),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![2, 3]
    assert_eq!(vec![&Value::Integer(2), &Value::Integer(3),], result);

    // ["$", "o", {"?": {"&&": [{">": [["@"], 1]}, {"<": [["@", 4]]}]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("o".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::or(
            BooleanExpr::Path(Path::rel(vec![Segment::Child(vec![Selector::key(
                Value::Text("u".to_owned()),
            )])])),
            BooleanExpr::Path(Path::rel(vec![Segment::Child(vec![Selector::key(
                Value::Text("x".to_owned()),
            )])])),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"u", 6}]
    assert_eq!(
        vec![&Value::Map(BTreeMap::from([(
            Value::Text("u".to_owned()),
            Value::Integer(6),
        )])),],
        result
    );

    // ["$", "a", {"?": {"==": [["@", "b"], ["$", "x"]]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(Value::Text(
                "b".to_owned(),
            ))])),
            ComparisonOperator::Eq,
            Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
                "x".to_owned(),
            ))])),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"u", 6}]
    assert_eq!(
        vec![
            &Value::Integer(3),
            &Value::Integer(5),
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(4),
            &Value::Integer(6),
        ],
        result
    );

    Ok(())
}

#[test]
fn logical() {
    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Integer(1)),
        ),
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Neq,
            Comparable::Value(Value::Integer(1)),
        ),
    );
    assert!(!logical.evaluate(&Value::Integer(1), &Value::Integer(1)));

    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Neq,
            Comparable::Value(Value::Integer(1)),
        ),
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Integer(1)),
        ),
    );
    assert!(!logical.evaluate(&Value::Integer(1), &Value::Integer(1)));

    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Neq,
            Comparable::Value(Value::Integer(1)),
        ),
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Neq,
            Comparable::Value(Value::Integer(1)),
        ),
    );
    assert!(!logical.evaluate(&Value::Integer(1), &Value::Integer(1)));

    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Integer(1)),
        ),
        BooleanExpr::comparison(
            Comparable::Value(Value::Integer(1)),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Integer(1)),
        ),
    );
    assert!(logical.evaluate(&Value::Integer(1), &Value::Integer(1)));
}

#[test]
fn child_segment() {
    // ["a", "b", "c", "d", "e", "f", "g"]
    let value = Value::Array(vec![
        Value::Text("a".to_owned()),
        Value::Text("b".to_owned()),
        Value::Text("c".to_owned()),
        Value::Text("d".to_owned()),
        Value::Text("e".to_owned()),
        Value::Text("f".to_owned()),
        Value::Text("g".to_owned()),
    ]);

    // ["$", [{"#": 0}, {"#": 3}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![
        Selector::index(0),
        Selector::index(3),
    ])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "d"}]
    assert_eq!(
        vec![&Value::Text("a".to_owned()), &Value::Text("d".to_owned())],
        result
    );

    // ["$", [{":": [0, 2, 1]}, {"#": 5}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![
        Selector::slice(0, 2, 1),
        Selector::index(5),
    ])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "b", "f"}]
    assert_eq!(
        vec![
            &Value::Text("a".to_owned()),
            &Value::Text("b".to_owned()),
            &Value::Text("f".to_owned())
        ],
        result
    );

    // ["$", [{{"#": 0}, {"#": 0}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![
        Selector::index(0),
        Selector::index(0),
    ])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "b", "f"}]
    assert_eq!(
        vec![&Value::Text("a".to_owned()), &Value::Text("a".to_owned())],
        result
    );
}

#[test]
fn descendant_segment() {
    // {
    //   "o": {"j": 1, "k": 2},
    //   "a": [5, 3, [{"j": 4}, {"k": 6}]]
    // }
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("o".to_owned()),
            Value::Map(BTreeMap::from([
                (Value::Text("j".to_owned()), Value::Integer(1)),
                (Value::Text("k".to_owned()), Value::Integer(2)),
            ])),
        ),
        (
            Value::Text("a".to_owned()),
            Value::Array(vec![
                Value::Integer(5),
                Value::Integer(3),
                Value::Array(vec![
                    Value::Map(BTreeMap::from([(
                        Value::Text("j".to_owned()),
                        Value::Integer(4),
                    )])),
                    Value::Map(BTreeMap::from([(
                        Value::Text("k".to_owned()),
                        Value::Integer(6),
                    )])),
                ]),
            ]),
        ),
    ]));

    // ["$", {"..": "j"}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::key(Value::Text(
        "j".to_owned(),
    ))])]);
    let result = cbor_path.evaluate(&value);
    // vec![1, 4]
    assert_eq!(vec![&Value::Integer(1), &Value::Integer(4)], result);

    // ["$", {"..": {"#": 0}}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::index(0)])]);
    let result = cbor_path.evaluate(&value);
    // vec![5, {"j": 4}]
    assert_eq!(
        vec![
            &Value::Integer(5),
            &Value::Map(BTreeMap::from([(
                Value::Text("j".to_owned()),
                Value::Integer(4)
            ),]))
        ],
        result
    );

    // ["$", {"..": "*"}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::wildcard()])]);
    let result = cbor_path.evaluate(&value);
    // vec![[5, 3, [{"j": 4}, {"k": 6}]], {"j": 1, "k": 2}, 5, 3, [{"j": 4}, {"k": 6}], 1, 2, {"j": 4}, {"k": 6}, 4, 6]
    assert_eq!(
        vec![
            &Value::Array(vec![
                Value::Integer(5),
                Value::Integer(3),
                Value::Array(vec![
                    Value::Map(BTreeMap::from([(
                        Value::Text("j".to_owned()),
                        Value::Integer(4),
                    )])),
                    Value::Map(BTreeMap::from([(
                        Value::Text("k".to_owned()),
                        Value::Integer(6),
                    )])),
                ]),
            ]),
            &Value::Map(BTreeMap::from([
                (Value::Text("j".to_owned()), Value::Integer(1)),
                (Value::Text("k".to_owned()), Value::Integer(2)),
            ])),
            &Value::Integer(5),
            &Value::Integer(3),
            &Value::Array(vec![
                Value::Map(BTreeMap::from([(
                    Value::Text("j".to_owned()),
                    Value::Integer(4),
                )])),
                Value::Map(BTreeMap::from([(
                    Value::Text("k".to_owned()),
                    Value::Integer(6),
                )])),
            ]),
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Map(BTreeMap::from([(
                Value::Text("j".to_owned()),
                Value::Integer(4),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("k".to_owned()),
                Value::Integer(6),
            )])),
            &Value::Integer(4),
            &Value::Integer(6),
        ],
        result
    );

    // ["$", {"..": "o"}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::key(Value::Text(
        "o".to_owned(),
    ))])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"j": 1, "k": 2}]
    assert_eq!(
        vec![&Value::Map(BTreeMap::from([
            (Value::Text("j".to_owned()), Value::Integer(1)),
            (Value::Text("k".to_owned()), Value::Integer(2)),
        ])),],
        result
    );

    // ["$", "o", {"..": ["*", "*"]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("o".to_owned()))]),
        Segment::Descendant(vec![Selector::wildcard(), Selector::wildcard()]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![1, 2, 1, 2]
    assert_eq!(
        vec![
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(1),
            &Value::Integer(2),
        ],
        result
    );

    // ["$", "a", {"..": [0, 1]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Descendant(vec![Selector::index(0), Selector::index(1)]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![5, 3, {"j": 4}, {"k": 6}]
    assert_eq!(
        vec![
            &Value::Integer(5),
            &Value::Integer(3),
            &Value::Map(BTreeMap::from([(
                Value::Text("j".to_owned()),
                Value::Integer(4),
            )])),
            &Value::Map(BTreeMap::from([(
                Value::Text("k".to_owned()),
                Value::Integer(6),
            )])),
        ],
        result
    );
}

#[test]
fn null() {
    // {"a": null, "b": [null], "c": [{}], "null": 1}
    let value = Value::Map(BTreeMap::from([
        (Value::Text("a".to_owned()), Value::Null),
        (Value::Text("b".to_owned()), Value::Array(vec![Value::Null])),
        (
            Value::Text("c".to_owned()),
            Value::Array(vec![Value::Map(BTreeMap::new())]),
        ),
        (Value::Text("null".to_owned()), Value::Integer(1)),
    ]));

    // ["$", "a"]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::key(Value::Text(
        "a".to_owned(),
    ))])]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "a", {"#": 0}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::index(0)]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "a", "d"]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("a".to_owned()))]),
        Segment::Child(vec![Selector::key(Value::Text("d".to_owned()))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "b", {"#": 0}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("b".to_owned()))]),
        Segment::Child(vec![Selector::index(0)]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", "*"]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("b".to_owned()))]),
        Segment::Child(vec![Selector::wildcard()]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": "@"}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("b".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::Path(Path::rel(vec![])))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": {"==": ["@", null]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("b".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![])),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Null),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": {"==": [["@", "d"], null]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key(Value::Text("b".to_owned()))]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(Value::Text(
                "d".to_owned(),
            ))])),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Null),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", null]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::key(Value::Text(
        "null".to_owned(),
    ))])]);
    let result = cbor_path.evaluate(&value);
    // vec![1]
    assert_eq!(vec![&Value::Integer(1)], result);
}

#[test]
fn count() {
    // {
    //   "o": {"j": 1, "k": 2},
    //   "a": [5, 3, [{"j": 4}, {"k": 6}]]
    // }
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("o".to_owned()),
            Value::Map(BTreeMap::from([
                (Value::Text("j".to_owned()), Value::Integer(1)),
                (Value::Text("k".to_owned()), Value::Integer(2)),
            ])),
        ),
        (
            Value::Text("a".to_owned()),
            Value::Array(vec![
                Value::Integer(5),
                Value::Integer(3),
                Value::Array(vec![
                    Value::Map(BTreeMap::from([(
                        Value::Text("j".to_owned()),
                        Value::Integer(4),
                    )])),
                    Value::Map(BTreeMap::from([(
                        Value::Text("k".to_owned()),
                        Value::Integer(6),
                    )])),
                ]),
            ]),
        ),
    ]));

    // ["$", {"?": {"==" : [{"count": ["@", "*"]}, 2]}}]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::filter(
        BooleanExpr::comparison(
            Comparable::Function(Function::count(Path::rel(vec![Segment::Child(vec![
                Selector::wildcard(),
            ])]))),
            ComparisonOperator::Eq,
            Comparable::Value(Value::Integer(2)),
        ),
    )])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![&Value::Map(BTreeMap::from([
            (Value::Text("j".to_owned()), Value::Integer(1)),
            (Value::Text("k".to_owned()), Value::Integer(2)),
        ])),],
        result
    );
}

#[test]
fn filter_root_current() {
    // {
    //   "a": {"k": 1},
    //   "b": {"k": 3},
    //   "c": 2
    // }
    let value = Value::Map(BTreeMap::from([
        (
            Value::Text("a".to_owned()),
            Value::Map(BTreeMap::from([
                (Value::Text("k".to_owned()), Value::Integer(1)),
            ])),
        ),
        (
            Value::Text("b".to_owned()),
            Value::Map(BTreeMap::from([
                (Value::Text("k".to_owned()), Value::Integer(3)),
            ])),
        ),
        (
            Value::Text("c".to_owned()),
            Value::Integer(2)
        ),
    ]));

    // ["$", {"..": {"?": {"<": [["@", "k"], ["$", "c""]]}}}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::filter(
        BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(Value::Text("k".to_owned()))])),
            ComparisonOperator::Lt,
            Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text("c".to_owned()))])),
        ),
    )])]);
    let result = cbor_path.evaluate(&value);

    println!("result: {result:?}");
    assert_eq!(
        vec![&Value::Map(BTreeMap::from([
            (Value::Text("k".to_owned()), Value::Integer(1)),
        ])),],
        result
    );
}
