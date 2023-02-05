use crate::{
    builder::{
        self, _match, and, eq, gt, gte, lt, lte, neq, or, rel_path, search, segment,
        sing_abs_path, sing_rel_path, val,
    },
    tests::util::{bytes_to_diag, diag_to_bytes, diag_to_value},
    CborPath, Error,
};
use ciborium::{cbor, value::Value};

#[test]
fn evaluate_root() {
    let value = diag_to_value(r#"{"k": "v"}"#);

    let cbor_path = CborPath::new(vec![]);
    let result = cbor_path.evaluate(&value);

    assert_eq!(vec![&Value::Map(vec![("k".into(), "v".into(),)])], result);
}

#[test]
fn evaluate_key() {
    let value = diag_to_value(r#"{"o": {"j j": {"k k": 3}}, "'": {"@": 2}}"#);

    let cbor_path = CborPath::builder().key("o").key("j j").key("k k").build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from(3)], result);

    let cbor_path = CborPath::builder().key("'").key("@").build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from(2)], result);
}

#[test]
fn evaluate_wildcard() -> Result<(), Error> {
    let value = diag_to_value(r#"{"o": {"j": 1, "k": 2}, "a": [5, 3]}"#);

    let cbor_path = CborPath::builder().wildcard().build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&cbor!({"j" => 1, "k" => 2})?, &cbor!([5, 3])?], result);

    let cbor_path = CborPath::builder().key("o").wildcard().build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&cbor!(1)?, &cbor!(2)?], result);

    let cbor_path = CborPath::builder().key("a").wildcard().build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&cbor!(5)?, &cbor!(3)?], result);

    Ok(())
}

#[test]
fn evaluate_index() {
    let value = diag_to_value(r#"["a", "b"]"#);

    let cbor_path = CborPath::builder().index(1).build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("b")], result);

    let cbor_path = CborPath::builder().index(-2).build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("a")], result);
}

#[test]
fn evaluate_slice() {
    let value = diag_to_value(r#"["a", "b", "c", "d", "e", "f", "g"]"#);

    let cbor_path = CborPath::builder().slice(1, 3, 1).build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("b"), &Value::from("c")], result);

    let cbor_path = CborPath::builder().slice(1, 5, 2).build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("b"), &Value::from("d")], result);

    let cbor_path = CborPath::builder().slice(5, 1, -2).build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::from("f"), &Value::from("d")], result);

    let cbor_path = CborPath::builder().slice(6, -8, -1).build();
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
    let value = diag_to_value(r#"{"obj": {"x": "y"}, "arr": [2, 3]}"#);

    // $.absent1 == $.absent2
    let comparison = eq(
        sing_abs_path().key("absent1"),
        sing_abs_path().key("absent2"),
    )
    .build();
    assert!(comparison.evaluate(&value, &value));

    // $.absent1 <= $.absent2
    let comparison = lte(
        sing_abs_path().key("absent1"),
        sing_abs_path().key("absent2"),
    )
    .build();
    assert!(comparison.evaluate(&value, &value));

    // $.absent1 == "g"
    let comparison = eq(sing_abs_path().key("absent1"), val("g")).build();
    assert!(!comparison.evaluate(&value, &value));

    // $.absent1 == $.absent2
    let comparison = neq(
        sing_abs_path().key("absent1"),
        sing_abs_path().key("absent2"),
    )
    .build();
    assert!(!comparison.evaluate(&value, &value));

    // $.absent1 != "g"
    let comparison = neq(sing_abs_path().key("absent1"), val("g")).build();
    assert!(comparison.evaluate(&value, &value));

    // 1 <= 2
    let comparison = lte(val(1), val(2)).build();
    assert!(comparison.evaluate(&value, &value));

    // 1 > 2
    let comparison = gt(val(1), val(2)).build();
    assert!(!comparison.evaluate(&value, &value));

    // 13 == "13"
    let comparison = eq(val(13), val("13")).build();
    assert!(!comparison.evaluate(&value, &value));

    // "a" <= "b"
    let comparison = lte(val("a"), val("b")).build();
    assert!(comparison.evaluate(&value, &value));

    // "a" > "b"
    let comparison = gt(val("a"), val("b")).build();
    assert!(!comparison.evaluate(&value, &value));

    // $.obj == $.arr
    let comparison = eq(sing_abs_path().key("obj"), sing_abs_path().key("arr")).build();
    assert!(!comparison.evaluate(&value, &value));

    // $.obj != $.arr
    let comparison = neq(sing_abs_path().key("obj"), sing_abs_path().key("arr")).build();
    assert!(comparison.evaluate(&value, &value));

    // $.obj == $.obj
    let comparison = eq(sing_abs_path().key("obj"), sing_abs_path().key("obj")).build();
    assert!(comparison.evaluate(&value, &value));

    // $.obj != $.obj
    let comparison = neq(sing_abs_path().key("obj"), sing_abs_path().key("obj")).build();
    assert!(!comparison.evaluate(&value, &value));

    // $.arr == $.arr
    let comparison = eq(sing_abs_path().key("arr"), sing_abs_path().key("arr")).build();
    assert!(comparison.evaluate(&value, &value));

    // $.arr != $.arr
    let comparison = neq(sing_abs_path().key("arr"), sing_abs_path().key("arr")).build();
    assert!(!comparison.evaluate(&value, &value));

    // 1 <= $.arr
    let comparison = lte(val(1), sing_abs_path().key("arr")).build();
    assert!(!comparison.evaluate(&value, &value));

    // 1 >= $.arr
    let comparison = gte(val(1), sing_abs_path().key("arr")).build();
    assert!(!comparison.evaluate(&value, &value));

    // 1 > $.arr
    let comparison = gt(val(1), sing_abs_path().key("arr")).build();
    assert!(!comparison.evaluate(&value, &value));

    // 1 < $.arr
    let comparison = lt(val(1), sing_abs_path().key("arr")).build();
    assert!(!comparison.evaluate(&value, &value));

    // true <= true
    let comparison = lte(val(true), val(true)).build();
    assert!(comparison.evaluate(&value, &value));

    // true > true
    let comparison = gt(val(true), val(true)).build();
    assert!(!comparison.evaluate(&value, &value));

    Ok(())
}

#[test]
fn filter() -> Result<(), Error> {
    let value = diag_to_value(
        r#"{
        "a": [3, 5, 1, 2, 4, 6, {"b": "j"}, {"b": "k"},
        {"b": {}}, {"b": "kilo"}],
        "o": {"p": 1, "q": 2, "r": 3, "s": 5, "t": {"u": 6}},
        "e": "f"
    }"#,
    );

    // ["$", "a", {"?": {"==": [["@", "b"], "kilo"]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(eq(sing_rel_path().key("b"), val("kilo")))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![{"b": "kilo"}]
    assert_eq!(
        vec![&Value::Map(vec![("b".into(), "kilo".into(),)]),],
        result
    );

    // ["$", "a", {"?": {">": [["@"], 3]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(gt(sing_rel_path(), val(3)))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![5, 4, 6]
    assert_eq!(
        vec![&Value::from(5), &Value::from(4), &Value::from(6)],
        result
    );

    // ["$", "a", {"?": ["@", "b"]]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(rel_path().key("b"))
        .build();
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
    let cbor_path = CborPath::builder().filter(rel_path().wildcard()).build();
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
    let cbor_path = CborPath::builder()
        .filter(rel_path().filter(rel_path().key("b")))
        .build();
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
    let cbor_path = CborPath::builder()
        .key("o")
        .child(
            segment()
                .filter(lt(sing_rel_path(), val(3)))
                .filter(lt(sing_rel_path(), val(3))),
        )
        .build();
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
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(or(
            lt(sing_rel_path(), val(2)),
            eq(sing_rel_path().key("b"), val("k")),
        ))
        .build();
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
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(_match(sing_rel_path().key("b"), "[jk]")?)
        .build();
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
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(search(sing_rel_path().key("b"), "[jk]")?)
        .build();
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

    // ["$", "o", {"?": {"&&": [{">": [["@"], 1]}, {"<": ["@", 4]}]}}]
    let cbor_path = CborPath::builder()
        .key("o")
        .filter(and(
            gt(sing_rel_path(), val(1)),
            lt(sing_rel_path(), val(4)),
        ))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![2, 3]
    assert_eq!(vec![&Value::from(2), &Value::from(3),], result);

    // ["$", "o", {"?": {"||": [["@", "u"], ["@", "x"]]}}]
    let cbor_path = CborPath::builder()
        .key("o")
        .filter(or(rel_path().key("u"), rel_path().key("x")))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![{"u", 6}]
    assert_eq!(vec![&Value::Map(vec![("u".into(), 6.into(),)]),], result);

    // ["$", "a", {"?": {"==": [["@", "b"], ["$", "x"]]}}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(eq(sing_rel_path().key("b"), sing_abs_path().key("x")))
        .build();
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
    let logical = and(eq(val(1), val(1)), neq(val(1), val(1))).build();
    assert!(!logical.evaluate(&Value::from(1), &Value::from(1)));

    let logical = and(neq(val(1), val(1)), eq(val(1), val(1))).build();
    assert!(!logical.evaluate(&Value::from(1), &Value::from(1)));

    let logical = and(neq(val(1), val(1)), neq(val(1), val(1))).build();
    assert!(!logical.evaluate(&Value::from(1), &Value::from(1)));

    let logical = and(eq(val(1), val(1)), eq(val(1), val(1))).build();
    assert!(logical.evaluate(&Value::from(1), &Value::from(1)));
}

#[test]
fn child_segment() {
    let value = diag_to_value(r#"["a", "b", "c", "d", "e", "f", "g"]"#);

    // ["$", [{"#": 0}, {"#": 3}]]
    let cbor_path = CborPath::builder()
        .child(segment().index(0).index(3))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "d"}]
    assert_eq!(vec![&Value::from("a"), &Value::from("d")], result);

    // ["$", [{":": [0, 2, 1]}, {"#": 5}]]
    let cbor_path = CborPath::builder()
        .child(segment().slice(0, 2, 1).index(5))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "b", "f"}]
    assert_eq!(
        vec![&Value::from("a"), &Value::from("b"), &Value::from("f")],
        result
    );

    // ["$", [{{"#": 0}, {"#": 0}]]
    let cbor_path = CborPath::builder()
        .child(segment().index(0).index(0))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![{"a", "b", "f"}]
    assert_eq!(vec![&Value::from("a"), &Value::from("a")], result);
}

#[test]
fn descendant_segment() -> Result<(), Error> {
    let value = diag_to_value(r#"{"o": {"j": 1, "k": 2}, "a": [5, 3, [{"j": 4}, {"k": 6}]]}"#);

    // ["$", {"..": "j"}]
    let cbor_path = CborPath::builder().descendant(segment().key("j")).build();
    let result = cbor_path.evaluate(&value);
    // vec![1, 4]
    assert_eq!(vec![&cbor!(1)?, &cbor!(4)?], result);

    // ["$", {"..": {"#": 0}}]
    let cbor_path = CborPath::builder().descendant(segment().index(0)).build();
    let result = cbor_path.evaluate(&value);
    // vec![5, {"j": 4}]
    assert_eq!(
        vec![&Value::from(5), &Value::Map(vec![("j".into(), 4.into()),])],
        result
    );

    // ["$", {"..": "*"}]
    let cbor_path = CborPath::builder()
        .descendant(segment().wildcard())
        .build();
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
    let cbor_path = CborPath::builder().descendant(segment().key("o")).build();
    let result = cbor_path.evaluate(&value);
    // vec![{"j": 1, "k": 2}]
    assert_eq!(
        vec![&Value::Map(vec![
            ("j".into(), 1.into()),
            ("k".into(), 2.into()),
        ]),],
        result
    );

    // ["$", "o", {"..": [{"*": 1}, {"*": 1}]}]
    let cbor_path = CborPath::builder()
        .key("o")
        .descendant(segment().wildcard().wildcard())
        .build();
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

    // ["$", "a", {"..": [{"#": 0}, {"#": 1}]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .descendant(segment().index(0).index(1))
        .build();
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
    let value = diag_to_value(r#"{"a": null, "b": [null], "c": [{}], "null": 1}"#);

    // ["$", "a"]
    let cbor_path = CborPath::builder().key("a").build();
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "a", {"#": 0}]
    let cbor_path = CborPath::builder().key("a").index(0).build();
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "a", "d"]
    let cbor_path = CborPath::builder().key("a").key("d").build();
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "b", {"#": 0}]
    let cbor_path = CborPath::builder().key("b").index(0).build();
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", "*"]
    let cbor_path = CborPath::builder().key("b").wildcard().build();
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": "@"}]
    let cbor_path = CborPath::builder().key("b").filter(rel_path()).build();
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": {"==": ["@", null]}}]
    let cbor_path = CborPath::builder()
        .key("b")
        .filter(eq(sing_rel_path(), val(Value::Null)))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![null]
    assert_eq!(vec![&Value::Null], result);

    // ["$", "b", {"?": {"==": [["@", "d"], null]}}]
    let cbor_path = CborPath::builder()
        .key("b")
        .filter(eq(sing_rel_path().key("d"), val(Value::Null)))
        .build();
    let result = cbor_path.evaluate(&value);
    // vec![]
    assert!(result.is_empty());

    // ["$", "null"]
    let cbor_path = CborPath::builder().key("null").build();
    let result = cbor_path.evaluate(&value);
    // vec![1]
    assert_eq!(vec![&Value::from(1)], result);
}

#[test]
fn count() {
    let value = diag_to_value(
        r#"{
        "o": {"j": 1, "k": 2},
        "a": [5, 3, [{"j": 4}, {"k": 6}]]
    }"#,
    );

    // ["$", {"?": {"==" : [{"count": ["@", "*"]}, 2]}}]
    let cbor_path = CborPath::builder()
        .filter(eq(builder::count(rel_path().wildcard()), val(2)))
        .build();
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
    let value = diag_to_value(
        r#"{
        "a": {"k": 1},
        "b": {"k": 3},
        "c": 2
    }"#,
    );

    // ["$", {"..": {"?": {"<": [["@", "k"], ["$", "c""]]}}}]
    let cbor_path = CborPath::builder()
        .descendant(segment().filter(lt(sing_rel_path().key("k"), sing_abs_path().key("c"))))
        .build();
    let result = cbor_path.evaluate(&value);
    assert_eq!(vec![&Value::Map(vec![("k".into(), 1.into()),]),], result);
}

#[test]
fn evaluate_from_reader() -> Result<(), Error> {
    let buf = diag_to_bytes(r#"{"a": {"k": 1}, "b": {"k": 3}, "c": 2 }"#);

    // ["$", {"..": {"?": {"<": [["@", "k"], ["$", "c""]]}}}]
    let cbor_path = CborPath::builder()
        .descendant(segment().filter(lt(sing_rel_path().key("k"), sing_abs_path().key("c"))))
        .build();

    let result = cbor_path.evaluate_from_reader(buf.as_slice())?;
    assert_eq!(r#"[{"k":1}]"#, bytes_to_diag(&result));

    Ok(())
}
