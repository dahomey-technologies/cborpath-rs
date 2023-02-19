use crate::{
    builder::{
        self, _match, and, eq, gt, lt, or, rel_path, search, segment, sing_abs_path, sing_rel_path,
        val,
    },
    tests::util::diag_to_bytes,
    CborPath, Error, Path,
};

#[test]
fn root() -> Result<(), Error> {
    let value = diag_to_bytes(r#"{"k": "v"}"#);

    let cbor_path = CborPath::new(vec![]);
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default()], result);

    Ok(())
}

#[test]
fn key() -> Result<(), Error> {
    let value = diag_to_bytes(r#"{"o": {"j j": {"k k": 3}}, "*": {"@": 2}}"#);

    let cbor_path = CborPath::builder().key("o").key("j j").key("k k").build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("o").key("j j").key("k k")], result);

    let cbor_path = CborPath::builder().key("*").key("@").build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("*").key("@")], result);

    Ok(())
}

#[test]
fn wildcard() -> Result<(), Error> {
    let value = diag_to_bytes(r#"{"o": {"j": 1, "k": 2}, "a": [5, 3]}"#);

    let cbor_path = CborPath::builder().wildcard().build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![Path::default().key("o"), Path::default().key("a")],
        result
    );

    let cbor_path = CborPath::builder().key("o").wildcard().build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o").key("j"),
            Path::default().key("o").key("k")
        ],
        result
    );

    let cbor_path = CborPath::builder()
        .key("o")
        .child(segment().wildcard().wildcard())
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o").key("j"),
            Path::default().key("o").key("k"),
            Path::default().key("o").key("j"),
            Path::default().key("o").key("k")
        ],
        result
    );

    let cbor_path = CborPath::builder().key("a").wildcard().build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(0),
            Path::default().key("a").idx(1),
        ],
        result
    );

    Ok(())
}

#[test]
fn index() -> Result<(), Error> {
    let value = diag_to_bytes(r#"["a", "b"]"#);

    let cbor_path = CborPath::builder().index(1).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(1)], result);

    let cbor_path = CborPath::builder().index(-2).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(0)], result);

    Ok(())
}

#[test]
fn slice() -> Result<(), Error> {
    let value = diag_to_bytes(r#"["a", "b", "c", "d", "e", "f", "g"]"#);

    let cbor_path = CborPath::builder().slice(1, 3, 1).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(1), Path::default().idx(2)], result);

    let cbor_path = CborPath::builder().slice(1, 5, 2).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(1), Path::default().idx(3)], result);

    let cbor_path = CborPath::builder().slice(5, 1, -2).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(5), Path::default().idx(3)], result);

    let cbor_path = CborPath::builder().slice(0, 7, 3).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().idx(0),
            Path::default().idx(3),
            Path::default().idx(6)
        ],
        result
    );

    let cbor_path = CborPath::builder().slice(6, -8, -3).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().idx(6),
            Path::default().idx(3),
            Path::default().idx(0)
        ],
        result
    );

    let cbor_path = CborPath::builder().slice(5, -8, -3).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(5), Path::default().idx(2)], result);

    let cbor_path = CborPath::builder().slice(6, -8, -1).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().idx(6),
            Path::default().idx(5),
            Path::default().idx(4),
            Path::default().idx(3),
            Path::default().idx(2),
            Path::default().idx(1),
            Path::default().idx(0)
        ],
        result
    );

    Ok(())
}

#[test]
fn filter() -> Result<(), Error> {
    let value = diag_to_bytes(
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
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("a").idx(9)], result);

    // ["$", "a", {"?": {">": [["@"], 3]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(gt(sing_rel_path(), val(3)))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(1),
            Path::default().key("a").idx(4),
            Path::default().key("a").idx(5),
        ],
        result
    );

    // ["$", "a", {"?": ["@", "b"]]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(rel_path().key("b"))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(6),
            Path::default().key("a").idx(7),
            Path::default().key("a").idx(8),
            Path::default().key("a").idx(9),
        ],
        result
    );

    // ["$", {"?": ["@", "*"]]
    let cbor_path = CborPath::builder().filter(rel_path().wildcard()).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![Path::default().key("a"), Path::default().key("o")],
        result
    );

    // ["$", {"?": ["@", {"?": ["@", "b"]}]]
    let cbor_path = CborPath::builder()
        .filter(rel_path().filter(rel_path().key("b")))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("a")], result);

    // ["$", "o", [{"?": {"<", [["@"], 3]}}, {"?": {"<", [["@"], 3]}}]]
    let cbor_path = CborPath::builder()
        .key("o")
        .child(
            segment()
                .filter(lt(sing_rel_path(), val(3)))
                .filter(lt(sing_rel_path(), val(3))),
        )
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o").key("p"),
            Path::default().key("o").key("q"),
            Path::default().key("o").key("p"),
            Path::default().key("o").key("q")
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
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(2),
            Path::default().key("a").idx(7)
        ],
        result
    );

    // ["$", "a", {"?": {"match": [["@", "b"], "[jk]"]}}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(_match(sing_rel_path().key("b"), "[jk]")?)
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(6),
            Path::default().key("a").idx(7)
        ],
        result
    );

    // ["$", "a", {"?": {"search": [["@", "b"], "[jk]"]}}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(search(sing_rel_path().key("b"), "[jk]")?)
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(6),
            Path::default().key("a").idx(7),
            Path::default().key("a").idx(9)
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
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o").key("q"),
            Path::default().key("o").key("r")
        ],
        result
    );

    // ["$", "o", {"?": {"||": [["@", "u"], ["@", "x"]]}}]
    let cbor_path = CborPath::builder()
        .key("o")
        .filter(or(rel_path().key("u"), rel_path().key("x")))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("o").key("t")], result);

    // ["$", "a", {"?": {"==": [["@", "b"], ["$", "x"]]}}]
    let cbor_path = CborPath::builder()
        .key("a")
        .filter(eq(sing_rel_path().key("b"), sing_abs_path().key("x")))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(0),
            Path::default().key("a").idx(1),
            Path::default().key("a").idx(2),
            Path::default().key("a").idx(3),
            Path::default().key("a").idx(4),
            Path::default().key("a").idx(5)
        ],
        result
    );

    Ok(())
}

#[test]
fn child_segment() -> Result<(), Error> {
    let value = diag_to_bytes(r#"["a", "b", "c", "d", "e", "f", "g"]"#);

    // ["$", [{"#": 0}, {"#": 3}]]
    let cbor_path = CborPath::builder()
        .child(segment().index(0).index(3))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(0), Path::default().idx(3)], result);

    // ["$", [{":": [0, 2, 1]}, {"#": 5}]]
    let cbor_path = CborPath::builder()
        .child(segment().slice(0, 2, 1).index(5))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().idx(0),
            Path::default().idx(1),
            Path::default().idx(5)
        ],
        result
    );

    // ["$", [{{"#": 0}, {"#": 0}]]
    let cbor_path = CborPath::builder()
        .child(segment().index(0).index(0))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().idx(0), Path::default().idx(0)], result);

    Ok(())
}

#[test]
fn descendant_segment() -> Result<(), Error> {
    let value = diag_to_bytes(r#"{"o": {"j": 1, "k": 2}, "a": [5, 3, [{"j": 4}, {"k": 6}]]}"#);

    // ["$", {"..": "j"}]
    let cbor_path = CborPath::builder().descendant(segment().key("j")).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o").key("j"),
            Path::default().key("a").idx(2).idx(0).key("j")
        ],
        result
    );

    // ["$", {"..": {"#": 0}}]
    let cbor_path = CborPath::builder().descendant(segment().index(0)).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(0),
            Path::default().key("a").idx(2).idx(0)
        ],
        result
    );

    // ["$", {"..": "*"}]
    let cbor_path = CborPath::builder().descendant(segment().wildcard()).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o"),
            Path::default().key("a"),
            Path::default().key("o").key("j"),
            Path::default().key("o").key("k"),
            Path::default().key("a").idx(0),
            Path::default().key("a").idx(1),
            Path::default().key("a").idx(2),
            Path::default().key("a").idx(2).idx(0),
            Path::default().key("a").idx(2).idx(1),
            Path::default().key("a").idx(2).idx(0).key("j"),
            Path::default().key("a").idx(2).idx(1).key("k")
        ],
        result
    );

    // ["$", {"..": "o"}]
    let cbor_path = CborPath::builder().descendant(segment().key("o")).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("o")], result);

    // ["$", "o", {"..": [{"*": 1}, {"*": 1}]}]
    let cbor_path = CborPath::builder()
        .key("o")
        .descendant(segment().wildcard().wildcard())
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("o").key("j"),
            Path::default().key("o").key("k"),
            Path::default().key("o").key("j"),
            Path::default().key("o").key("k"),
        ],
        result
    );

    // ["$", "a", {"..": [{"#": 0}, {"#": 1}]}]
    let cbor_path = CborPath::builder()
        .key("a")
        .descendant(segment().index(0).index(1))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("a").idx(0),
            Path::default().key("a").idx(1),
            Path::default().key("a").idx(2).idx(0),
            Path::default().key("a").idx(2).idx(1)
        ],
        result
    );

    Ok(())
}

#[test]
fn null() -> Result<(), Error> {
    let value = diag_to_bytes(r#"{"a": null, "b": [null], "c": [{}], "null": 1}"#);

    // ["$", "a"]
    let cbor_path = CborPath::builder().key("a").build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("a")], result);

    // ["$", "a", {"#": 0}]
    let cbor_path = CborPath::builder().key("a").index(0).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(Vec::<Path>::new(), result);

    // ["$", "a", "d"]
    let cbor_path = CborPath::builder().key("a").key("d").build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(Vec::<Path>::new(), result);

    // ["$", "b", {"#": 0}]
    let cbor_path = CborPath::builder().key("b").index(0).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("b").idx(0)], result);

    // ["$", "b", "*"]
    let cbor_path = CborPath::builder().key("b").wildcard().build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("b").idx(0)], result);

    // ["$", "b", {"?": "@"}]
    let cbor_path = CborPath::builder().key("b").filter(rel_path()).build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("b").idx(0)], result);

    // ["$", "b", {"?": {"==": ["@", null]}}]
    let cbor_path = CborPath::builder()
        .key("b")
        .filter(eq(sing_rel_path(), val(())))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("b").idx(0)], result);

    // ["$", "b", {"?": {"==": [["@", "d"], null]}}]
    let cbor_path = CborPath::builder()
        .key("b")
        .filter(eq(sing_rel_path().key("d"), val(())))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(Vec::<Path>::new(), result);

    // ["$", "null"]
    let cbor_path = CborPath::builder().key("null").build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("null")], result);

    Ok(())
}

#[test]
fn count() -> Result<(), Error> {
    let value = diag_to_bytes(
        r#"{
        "o": {"j": 1, "k": 2},
        "a": [5, 3, [{"j": 4}, {"k": 6}]]
    }"#,
    );

    // ["$", {"?": {"==" : [{"count": ["@", "*"]}, 2]}}]
    let cbor_path = CborPath::builder()
        .filter(eq(builder::count(rel_path().wildcard()), val(2)))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("o")], result);

    Ok(())
}

#[test]
fn filter_root_current() -> Result<(), Error> {
    let value = diag_to_bytes(
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
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(vec![Path::default().key("a")], result);

    Ok(())
}

#[test]
fn store() -> Result<(), Error> {
    let value = diag_to_bytes(
        r#"
    { "store": {
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

    // the authors of all books in the store
    // ["$", "store", "book", {"*": 1}, "author"]
    let cbor_path = CborPath::builder()
        .key("store")
        .key("book")
        .wildcard()
        .key("author")
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default()
                .key("store")
                .key("book")
                .idx(0)
                .key("author"),
            Path::default()
                .key("store")
                .key("book")
                .idx(1)
                .key("author"),
            Path::default()
                .key("store")
                .key("book")
                .idx(2)
                .key("author"),
            Path::default()
                .key("store")
                .key("book")
                .idx(3)
                .key("author")
        ],
        result
    );

    // all authors
    // ["$", {"..": "author"}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("author"))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default()
                .key("store")
                .key("book")
                .idx(0)
                .key("author"),
            Path::default()
                .key("store")
                .key("book")
                .idx(1)
                .key("author"),
            Path::default()
                .key("store")
                .key("book")
                .idx(2)
                .key("author"),
            Path::default()
                .key("store")
                .key("book")
                .idx(3)
                .key("author")
        ],
        result
    );

    // all things in store, which are some books and a red bicycle
    // ["$", "store", {"*": 1}]
    let cbor_path = CborPath::builder().key("store").wildcard().build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store").key("book"),
            Path::default().key("store").key("bicycle")
        ],
        result
    );

    // the prices of everything in the store
    // ["$", "store", {"..": "price"}]
    let cbor_path = CborPath::builder()
        .key("store")
        .descendant(builder::segment().key("price"))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store").key("bicycle").key("price"),
            Path::default().key("store").key("book").idx(0).key("price"),
            Path::default().key("store").key("book").idx(1).key("price"),
            Path::default().key("store").key("book").idx(2).key("price"),
            Path::default().key("store").key("book").idx(3).key("price")
        ],
        result
    );

    // the third book
    // ["$", {"..": "book"}, {"#": 2}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("book"))
        .index(2)
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![Path::default().key("store").key("book").idx(2)],
        result
    );

    // the last book in order
    // ["$", {"..": "book"}, {"#": -1}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("book"))
        .index(-1)
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![Path::default().key("store").key("book").idx(3)],
        result
    );

    // the first two books
    // ["$", {"..": "book"}, [{"#": 0}, {"#": 1}]]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("book"))
        .child(builder::segment().index(0).index(1))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store").key("book").idx(0),
            Path::default().key("store").key("book").idx(1)
        ],
        result
    );

    // the first two books
    // ["$", {"..": "book"}, {":": [0, 2, 1]}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("book"))
        .slice(0, 2, 1)
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store").key("book").idx(0),
            Path::default().key("store").key("book").idx(1)
        ],
        result
    );

    // all books with an ISBN number
    // ["$", {"..": "book"}, {"?": ["@", "isbn"]}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("book"))
        .filter(builder::rel_path().key("isbn"))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store").key("book").idx(2),
            Path::default().key("store").key("book").idx(3)
        ],
        result
    );

    // all books cheaper than 10
    // ["$", {"..": "book"}, {"?": {"<": [["@", "price"], 10.0]}}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().key("book"))
        .filter(builder::lt(
            builder::sing_rel_path().key("price"),
            builder::val(10.),
        ))
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store").key("book").idx(0),
            Path::default().key("store").key("book").idx(2)
        ],
        result
    );

    // all map item values and array elements contained in input value
    // ["$", {"..": {"*": 1}}]
    let cbor_path = CborPath::builder()
        .descendant(builder::segment().wildcard())
        .build();
    let result = cbor_path.get_paths_from_bytes(&value)?;
    assert_eq!(
        vec![
            Path::default().key("store"),
            Path::default().key("store").key("book"),
            Path::default().key("store").key("bicycle"),
            Path::default().key("store").key("book").idx(0),
            Path::default().key("store").key("book").idx(1),
            Path::default().key("store").key("book").idx(2),
            Path::default().key("store").key("book").idx(3),
            Path::default().key("store").key("bicycle").key("color"),
            Path::default().key("store").key("bicycle").key("price"),
            Path::default()
                .key("store")
                .key("book")
                .idx(0)
                .key("category"),
            Path::default()
                .key("store")
                .key("book")
                .idx(0)
                .key("author"),
            Path::default().key("store").key("book").idx(0).key("title"),
            Path::default().key("store").key("book").idx(0).key("price"),
            Path::default()
                .key("store")
                .key("book")
                .idx(1)
                .key("category"),
            Path::default()
                .key("store")
                .key("book")
                .idx(1)
                .key("author"),
            Path::default().key("store").key("book").idx(1).key("title"),
            Path::default().key("store").key("book").idx(1).key("price"),
            Path::default()
                .key("store")
                .key("book")
                .idx(2)
                .key("category"),
            Path::default()
                .key("store")
                .key("book")
                .idx(2)
                .key("author"),
            Path::default().key("store").key("book").idx(2).key("title"),
            Path::default().key("store").key("book").idx(2).key("isbn"),
            Path::default().key("store").key("book").idx(2).key("price"),
            Path::default()
                .key("store")
                .key("book")
                .idx(3)
                .key("category"),
            Path::default()
                .key("store")
                .key("book")
                .idx(3)
                .key("author"),
            Path::default().key("store").key("book").idx(3).key("title"),
            Path::default().key("store").key("book").idx(3).key("isbn"),
            Path::default().key("store").key("book").idx(3).key("price")
        ],
        result
    );

    Ok(())
}
