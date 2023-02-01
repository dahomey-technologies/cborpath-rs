use crate::{
    BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, Error, Function, Path,
    Segment, Selector, SingularPath, SingularSegment,
};
use cbor_diag::{parse_diag, parse_bytes};
use ciborium::{cbor, value::Value};

#[test]
fn evaluate_root() {
    let value = Value::Map(vec![("k".into(), "v".into())]);

    let cbor_path = CborPath::new(vec![]);
    let result = cbor_path.evaluate(&value);

    assert_eq!(vec![&Value::Map(vec![("k".into(), "v".into(),)])], result);
}

#[test]
fn evaluate_key() {
    let value = Value::Map(vec![
        (
            "o".into(),
            Value::Map(vec![(
                "j j".into(),
                Value::Map(vec![("k k".into(), 3.into())]),
            )]),
        ),
        ("'".into(), Value::Map(vec![("@".into(), 2.into())])),
    ]);

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("o".into())]),
        Segment::Child(vec![Selector::key("j j".into())]),
        Segment::Child(vec![Selector::key("k k".into())]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from(3)], result);

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("'".into())]),
        Segment::Child(vec![Selector::key("@".into())]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from(2)], result);
}

#[test]
fn evaluate_wildcard() -> Result<(), Error> {
    let value = cbor!(
    {
        "o" => {"j" => 1, "k" => 2},
        "a" => [5, 3]
    })?;

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::Wildcard])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&cbor!({"j" => 1, "k" => 2})?, &cbor!([5, 3])?], result);

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("o".into())]),
        Segment::Child(vec![Selector::Wildcard]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&cbor!(1)?, &cbor!(2)?], result);

    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::Wildcard]),
    ]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&cbor!(5)?, &cbor!(3)?], result);

    Ok(())
}

#[test]
fn evaluate_index() {
    let value = Value::Array(vec!["a".into(), "b".into()]);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::index(1)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("b")], result);

    let value = Value::Array(vec!["a".into(), "b".into()]);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::index(-2)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("a")], result);
}

#[test]
fn evaluate_slice() {
    let value = Value::Array(vec![
        "a".into(),
        "b".into(),
        "c".into(),
        "d".into(),
        "e".into(),
        "f".into(),
        "g".into(),
    ]);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(1, 3, 1)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("b"), &Value::from("c")], result);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(1, 5, 2)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("b"), &Value::from("d")], result);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(5, 1, -2)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("f"), &Value::from("d")], result);

    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::slice(6, -8, -1)])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![
            &Value::from("g"),
            &Value::from("f"),
            &Value::from("e"),
            &Value::from("d"),
            &Value::from("c"),
            &Value::from("b"),
            &Value::from("a"),
        ],
        result
    );
}

#[test]
fn comparison() -> Result<(), Error> {
    let value = Value::Map(vec![
        ("obj".into(), Value::Map(vec![("x".into(), "y".into())])),
        ("arr".into(), Value::Array(vec![2.into(), 3.into()])),
    ]);

    // $.absent1 == $.absent2
    let comparison = ComparisonExpr::new(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(cbor!(
            "absent1"
        )?)])),
        ComparisonOperator::Eq,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(cbor!(
            "absent2"
        )?)])),
    );
    assert!(comparison.evaluate(&value, &value));

    // $.absent1 <= $.absent2
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Eq,
        Comparable::Value("g".into()),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.absent1 != $.absent2
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "absent1".to_owned(),
        ))])),
        ComparisonOperator::Neq,
        Comparable::Value("g".into()),
    );
    assert!(comparison.evaluate(&value, &value));

    // 1 <= 2
    let comparison = ComparisonExpr::new(
        Comparable::Value(1.into()),
        ComparisonOperator::Lte,
        Comparable::Value(2.into()),
    );
    assert!(comparison.evaluate(&value, &value));

    // 1 > 2
    let comparison = ComparisonExpr::new(
        Comparable::Value(1.into()),
        ComparisonOperator::Gt,
        Comparable::Value(2.into()),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 13 == "13"
    let comparison = ComparisonExpr::new(
        Comparable::Value(13.into()),
        ComparisonOperator::Eq,
        Comparable::Value("13".into()),
    );
    assert!(!comparison.evaluate(&value, &value));

    // "a" <= "b"
    let comparison = ComparisonExpr::new(
        Comparable::Value("a".into()),
        ComparisonOperator::Lte,
        Comparable::Value("b".into()),
    );
    assert!(comparison.evaluate(&value, &value));

    // "a" > "b"
    let comparison = ComparisonExpr::new(
        Comparable::Value("a".into()),
        ComparisonOperator::Gt,
        Comparable::Value("b".into()),
    );
    assert!(!comparison.evaluate(&value, &value));

    // $.obj == $.arr
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
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
    let comparison = ComparisonExpr::new(
        Comparable::Value(1.into()),
        ComparisonOperator::Lte,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 >= $.arr
    let comparison = ComparisonExpr::new(
        Comparable::Value(1.into()),
        ComparisonOperator::Gte,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 > $.arr
    let comparison = ComparisonExpr::new(
        Comparable::Value(1.into()),
        ComparisonOperator::Gt,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // 1 < $.arr
    let comparison = ComparisonExpr::new(
        Comparable::Value(1.into()),
        ComparisonOperator::Lt,
        Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key(Value::Text(
            "arr".to_owned(),
        ))])),
    );
    assert!(!comparison.evaluate(&value, &value));

    // true <= true
    let comparison = ComparisonExpr::new(
        Comparable::Value(Value::Bool(true)),
        ComparisonOperator::Lte,
        Comparable::Value(Value::Bool(true)),
    );
    assert!(comparison.evaluate(&value, &value));

    // true > true
    let comparison = ComparisonExpr::new(
        Comparable::Value(Value::Bool(true)),
        ComparisonOperator::Gt,
        Comparable::Value(Value::Bool(true)),
    );
    assert!(!comparison.evaluate(&value, &value));

    Ok(())
}

#[test]
fn filter() -> Result<(), Error> {
    // {
    //     "a": [3, 5, 1, 2, 4, 6, {"b": "j"}, {"b": "k"},
    //     {"b": {}}, {"b": "kilo"}],
    //     "o": {"p": 1, "q": 2, "r": 3, "s": 5, "t": {"u": 6}},
    //     "e": "f"
    // }
    let value = Value::Map(vec![
        (
            "a".into(),
            Value::Array(vec![
                3.into(),
                5.into(),
                1.into(),
                2.into(),
                4.into(),
                6.into(),
                Value::Map(vec![("b".into(), "j".into())]),
                Value::Map(vec![("b".into(), "k".into())]),
                Value::Map(vec![("b".into(), Value::Map(vec![]))]),
                Value::Map(vec![("b".into(), "kilo".into())]),
            ]),
        ),
        (
            "o".into(),
            Value::Map(vec![
                ("p".into(), 1.into()),
                ("q".into(), 2.into()),
                ("r".into(), 3.into()),
                ("s".into(), 5.into()),
                ("t".into(), Value::Map(vec![("u".into(), 6.into())])),
            ]),
        ),
        ("e".into(), "f".into()),
    ]);

    // ["$", "a", {"?": {"==": [["@", "b"], "kilo"]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key(Value::Text(
                "b".to_owned(),
            ))])),
            ComparisonOperator::Eq,
            Comparable::Value("kilo".into()),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "kilo"}]
    assert_eq!(
        vec![&Value::Map(vec![("b".into(), "kilo".into(),)]),],
        result
    );

    // ["$", "a", {"?": {">": [["@"], 3]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![])),
            ComparisonOperator::Gt,
            Comparable::Value(3.into()),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![5, 4, 6]
    assert_eq!(
        vec![&Value::from(5), &Value::from(4), &Value::from(6)],
        result
    );

    // ["$", "a", {"?": ["@", "b"]]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::path(Path::rel(vec![
            Segment::Child(vec![Selector::key("b".into())]),
        ])))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "j"}, {"b": "k"}, {"b": {}}, {"b": "kilo"}]
    assert_eq!(
        vec![
            &Value::Map(vec![("b".into(), "j".into(),)]),
            &Value::Map(vec![("b".into(), "k".into(),)]),
            &Value::Map(vec![("b".into(), Value::Map(vec![]),)]),
            &Value::Map(vec![("b".into(), "kilo".into(),)]),
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
                3.into(),
                5.into(),
                1.into(),
                2.into(),
                4.into(),
                6.into(),
                Value::Map(vec![("b".into(), "j".into(),)]),
                Value::Map(vec![("b".into(), "k".into(),)]),
                Value::Map(vec![("b".into(), Value::Map(vec![]),)]),
                Value::Map(vec![("b".into(), "kilo".into(),)]),
            ]),
            &Value::Map(vec![
                ("p".into(), 1.into()),
                ("q".into(), 2.into()),
                ("r".into(), 3.into()),
                ("s".into(), 5.into()),
                ("t".into(), Value::Map(vec![("u".into(), 6.into(),)]),),
            ])
        ],
        result
    );

    // ["$", {"?": ["@", {"?": ["@", "b"]}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::filter(
        BooleanExpr::path(Path::rel(vec![Segment::Child(vec![Selector::filter(
            BooleanExpr::path(Path::rel(vec![Segment::Child(vec![Selector::key(
                "b".into(),
            )])])),
        )])])),
    )])]);
    let result = cbor_path.evaluate(&value);
    // vec![[3, 5, 1, 2, 4, 6, {"b": "j"}, {"b": "k"}, {"b": {}}, {"b": "kilo"}]]
    assert_eq!(
        vec![&Value::Array(vec![
            3.into(),
            5.into(),
            1.into(),
            2.into(),
            4.into(),
            6.into(),
            Value::Map(vec![("b".into(), "j".into(),)]),
            Value::Map(vec![("b".into(), "k".into(),)]),
            Value::Map(vec![("b".into(), Value::Map(vec![]),)]),
            Value::Map(vec![("b".into(), "kilo".into(),)]),
        ])],
        result
    );

    // ["$", "o", [{"?": {"<", [["@"], 3]}}, {"?": {"<", [["@"], 3]}}]]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("o".into())]),
        Segment::Child(vec![
            Selector::filter(BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(3.into()),
            )),
            Selector::filter(BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(3.into()),
            )),
        ]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![1, 2, 1, 2]
    assert_eq!(
        vec![
            &Value::from(1),
            &Value::from(2),
            &Value::from(1),
            &Value::from(2)
        ],
        result
    );

    // ["$", "a", {"?": {'||': [{"<": [["@"], 2]}, {"==": [["@", "b"], "k"]}]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::or(
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(2.into()),
            ),
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key("b".into())])),
                ComparisonOperator::Eq,
                Comparable::Value("k".into()),
            ),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![1, {"b": "k"}]
    assert_eq!(
        vec![
            &Value::from(1),
            &Value::Map(vec![("b".into(), "k".into(),)]),
        ],
        result
    );

    // ["$", "a", {"?": {"match": [["@", "b"], "[jk]"]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::function(
            Function::_match(
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key("b".into())])),
                "[jk]",
            )?,
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "j"}, {"b": "k"}]
    assert_eq!(
        vec![
            &Value::Map(vec![("b".into(), "j".into(),)]),
            &Value::Map(vec![("b".into(), "k".into(),)]),
        ],
        result
    );

    // ["$", "a", {"?": {"search": [["@", "b"], "[jk]"]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::function(
            Function::search(
                Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key("b".into())])),
                "[jk]",
            )?,
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "j"}, {"b": "k"}]
    assert_eq!(
        vec![
            &Value::Map(vec![("b".into(), "j".into(),)]),
            &Value::Map(vec![("b".into(), "k".into(),)]),
            &Value::Map(vec![("b".into(), "kilo".into(),)]),
        ],
        result
    );

    // ["$", "o", {"?": {"&&": [{">": [["@"], 1]}, {"<": [["@", 4]]}]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("o".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::and(
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Gt,
                Comparable::Value(1.into()),
            ),
            BooleanExpr::comparison(
                Comparable::SingularPath(SingularPath::rel(vec![])),
                ComparisonOperator::Lt,
                Comparable::Value(4.into()),
            ),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![2, 3]
    assert_eq!(vec![&Value::from(2), &Value::from(3),], result);

    // ["$", "o", {"?": {"&&": [{">": [["@"], 1]}, {"<": [["@", 4]]}]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("o".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::or(
            BooleanExpr::Path(Path::rel(vec![Segment::Child(vec![Selector::key(
                "u".into(),
            )])])),
            BooleanExpr::Path(Path::rel(vec![Segment::Child(vec![Selector::key(
                "x".into(),
            )])])),
        ))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![{"u", 6}]
    assert_eq!(vec![&Value::Map(vec![("u".into(), 6.into(),)]),], result);

    // ["$", "a", {"?": {"==": [["@", "b"], ["$", "x"]]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
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
            &Value::from(3),
            &Value::from(5),
            &Value::from(1),
            &Value::from(2),
            &Value::from(4),
            &Value::from(6),
        ],
        result
    );

    Ok(())
}

#[test]
fn logical() {
    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Eq,
            Comparable::Value(1.into()),
        ),
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Neq,
            Comparable::Value(1.into()),
        ),
    );
    assert!(!logical.evaluate(&Value::from(1), &Value::from(1)));

    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Neq,
            Comparable::Value(1.into()),
        ),
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Eq,
            Comparable::Value(1.into()),
        ),
    );
    assert!(!logical.evaluate(&Value::from(1), &Value::from(1)));

    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Neq,
            Comparable::Value(1.into()),
        ),
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Neq,
            Comparable::Value(1.into()),
        ),
    );
    assert!(!logical.evaluate(&Value::from(1), &Value::from(1)));

    let logical = BooleanExpr::and(
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Eq,
            Comparable::Value(1.into()),
        ),
        BooleanExpr::comparison(
            Comparable::Value(1.into()),
            ComparisonOperator::Eq,
            Comparable::Value(1.into()),
        ),
    );
    assert!(logical.evaluate(&Value::from(1), &Value::from(1)));
}

#[test]
fn child_segment() {
    // ["a", "b", "c", "d", "e", "f", "g"]
    let value = Value::Array(vec![
        "a".into(),
        "b".into(),
        "c".into(),
        "d".into(),
        "e".into(),
        "f".into(),
        "g".into(),
    ]);

    // ["$", [{"#": 0}, {"#": 3}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![
        Selector::index(0),
        Selector::index(3),
    ])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "d"}]
    assert_eq!(vec![&Value::from("a"), &Value::from("d")], result);

    // ["$", [{":": [0, 2, 1]}, {"#": 5}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![
        Selector::slice(0, 2, 1),
        Selector::index(5),
    ])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "b", "f"}]
    assert_eq!(
        vec![&Value::from("a"), &Value::from("b"), &Value::from("f")],
        result
    );

    // ["$", [{{"#": 0}, {"#": 0}]]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![
        Selector::index(0),
        Selector::index(0),
    ])]);
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "b", "f"}]
    assert_eq!(vec![&Value::from("a"), &Value::from("a")], result);
}

#[test]
fn descendant_segment() -> Result<(), Error> {
    let value = cbor!(
    {
        "o" => {"j" => 1, "k" => 2},
        "a" => [5, 3, [{"j" => 4}, {"k" => 6}]]
    })
    .unwrap();

    // ["$", {"..": "j"}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::key(Value::Text(
        "j".to_owned(),
    ))])]);
    let result = cbor_path.evaluate(&value);
    // vec![1, 4]
    assert_eq!(vec![&cbor!(1)?, &cbor!(4)?], result);

    // ["$", {"..": {"#": 0}}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::index(0)])]);
    let result = cbor_path.evaluate(&value);
    // vec![5, {"j": 4}]
    assert_eq!(
        vec![&Value::from(5), &Value::Map(vec![("j".into(), 4.into()),])],
        result
    );

    // ["$", {"..": "*"}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::wildcard()])]);
    let result = cbor_path.evaluate(&value);
    // vec![[5, 3, [{"j": 4}, {"k": 6}]], {"j": 1, "k": 2}, 5, 3, [{"j": 4}, {"k": 6}], 1, 2, {"j": 4}, {"k": 6}, 4, 6]
    assert_eq!(
        vec![
            &cbor!({"j" => 1, "k" => 2})?,
            &cbor!([5, 3, [{"j" => 4}, {"k" => 6}]])?,
            &cbor!(1)?,
            &cbor!(2)?,
            &cbor!(5)?,
            &cbor!(3)?,
            &cbor!([{"j" => 4}, {"k" => 6}])?,
            &cbor!({"j" => 4})?,
            &cbor!({"k" => 6})?,
            &cbor!(4)?,
            &cbor!(6)?,
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
        vec![&Value::Map(vec![
            ("j".into(), 1.into()),
            ("k".into(), 2.into()),
        ]),],
        result
    );

    // ["$", "o", {"..": ["*", "*"]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("o".into())]),
        Segment::Descendant(vec![Selector::wildcard(), Selector::wildcard()]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![1, 2, 1, 2]
    assert_eq!(
        vec![
            &Value::from(1),
            &Value::from(2),
            &Value::from(1),
            &Value::from(2),
        ],
        result
    );

    // ["$", "a", {"..": [0, 1]}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Descendant(vec![Selector::index(0), Selector::index(1)]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![5, 3, {"j": 4}, {"k": 6}]
    assert_eq!(
        vec![
            &Value::from(5),
            &Value::from(3),
            &Value::Map(vec![("j".into(), 4.into(),)]),
            &Value::Map(vec![("k".into(), 6.into(),)]),
        ],
        result
    );

    Ok(())
}

#[test]
fn null() {
    // {"a": null, "b": [null], "c": [{}], "null": 1}
    let value = Value::Map(vec![
        ("a".into(), Value::Null),
        ("b".into(), Value::Array(vec![Value::Null])),
        ("c".into(), Value::Array(vec![Value::Map(vec![])])),
        ("null".into(), 1.into()),
    ]);

    // ["$", "a"]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::key(Value::Text(
        "a".to_owned(),
    ))])]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "a", {"#": 0}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::index(0)]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "a", "d"]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("a".into())]),
        Segment::Child(vec![Selector::key("d".into())]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "b", {"#": 0}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("b".into())]),
        Segment::Child(vec![Selector::index(0)]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", "*"]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("b".into())]),
        Segment::Child(vec![Selector::wildcard()]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": "@"}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("b".into())]),
        Segment::Child(vec![Selector::filter(BooleanExpr::Path(Path::rel(vec![])))]),
    ]);
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": {"==": ["@", null]}}]
    let cbor_path = CborPath::new(vec![
        Segment::Child(vec![Selector::key("b".into())]),
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
        Segment::Child(vec![Selector::key("b".into())]),
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
    assert_eq!(vec![&Value::from(1)], result);
}

#[test]
fn count() {
    // {
    //   "o": {"j": 1, "k": 2},
    //   "a": [5, 3, [{"j": 4}, {"k": 6}]]
    // }
    let value = Value::Map(vec![
        (
            "o".into(),
            Value::Map(vec![("j".into(), 1.into()), ("k".into(), 2.into())]),
        ),
        (
            "a".into(),
            Value::Array(vec![
                5.into(),
                3.into(),
                Value::Array(vec![
                    Value::Map(vec![("j".into(), 4.into())]),
                    Value::Map(vec![("k".into(), 6.into())]),
                ]),
            ]),
        ),
    ]);

    // ["$", {"?": {"==" : [{"count": ["@", "*"]}, 2]}}]
    let cbor_path = CborPath::new(vec![Segment::Child(vec![Selector::filter(
        BooleanExpr::comparison(
            Comparable::Function(Function::count(Path::rel(vec![Segment::Child(vec![
                Selector::wildcard(),
            ])]))),
            ComparisonOperator::Eq,
            Comparable::Value(2.into()),
        ),
    )])]);
    let result = cbor_path.evaluate(&value);
    assert_eq!(
        vec![&Value::Map(vec![
            ("j".into(), 1.into()),
            ("k".into(), 2.into()),
        ]),],
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
    let value = Value::Map(vec![
        ("a".into(), Value::Map(vec![("k".into(), 1.into())])),
        ("b".into(), Value::Map(vec![("k".into(), 3.into())])),
        ("c".into(), 2.into()),
    ]);

    // ["$", {"..": {"?": {"<": [["@", "k"], ["$", "c""]]}}}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::filter(
        BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key("k".into())])),
            ComparisonOperator::Lt,
            Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key("c".into())])),
        ),
    )])]);

    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Map(vec![("k".into(), 1.into()),]),], result);
}

#[test]
fn evaluate_from_reader() -> Result<(), Error> {
    let cbor = r#"{"a": {"k": 1}, "b": {"k": 3}, "c": 2 }"#;
    let buf = parse_diag(cbor).unwrap().to_bytes();

    // ["$", {"..": {"?": {"<": [["@", "k"], ["$", "c""]]}}}]
    let cbor_path = CborPath::new(vec![Segment::Descendant(vec![Selector::filter(
        BooleanExpr::comparison(
            Comparable::SingularPath(SingularPath::rel(vec![SingularSegment::key("k".into())])),
            ComparisonOperator::Lt,
            Comparable::SingularPath(SingularPath::abs(vec![SingularSegment::key("c".into())])),
        ),
    )])]);

    let result = cbor_path.evaluate_from_reader(buf.as_slice())?;
    assert_eq!(r#"[{"k":1}]"#, parse_bytes(&result).unwrap().to_diag());

    Ok(())
}
