[![Crate](https://img.shields.io/crates/v/cborpath.svg)](https://crates.io/crates/cborpath)
[![docs.rs](https://docs.rs/cborpath/badge.svg)](https://docs.rs/cborpath)
[![Build](https://github.com/dahomey-technologies/cborpath-rs/actions/workflows/compile_and_test.yml/badge.svg)](https://github.com/dahomey-technologies/cborpath-rs/actions/workflows/compile_and_test.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

# cborpath-rs
cborpath is a CBORPath engine written in Rust.

# CBORPath
CBORPath is an adaptation of JSONPath to [CBOR](https://www.rfc-editor.org/rfc/rfc8949.html) 
based on the [JSONPath RFC Draft](https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html)

## Syntax summary
### Path
A `path` expression is a `CBOR Array` which, when applied to a `CBOR` value, the
*argument*, selects zero or more nodes of the argument and output these nodes as a nodelist.
 
A `path` always begins by an identifier
* a root identifier (`$`) for absolute paths,
* a current node identifier (`@`) for relative paths. relative path are always used in a filter context.
 
A `path` is then followed by one or more `segments`.

| Syntax                                        | Description                                                                                                             |
|-----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `["$", <segments>]`                           | an absolute path composed by an array of segments<br> and which always begins by a root identifier (`$`)                |
| `["@", <segments>]`                           | a relative path composed by an array of segments<br> and which always begins by a current node identifier (`@`)         |

### Segment
`Segments` apply one or more `selectors` to an input value and concatenate the results into a single nodelist.

| Syntax                                        | Description                                                                                                             |
|-----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `[<selectors>]`                               | a `child segment`, composed by one ore more `selectors`                                                                 |
| `<selector>`                                  | shortcut for a `child segment`, composed by a unique `selector`                                                         |
| `{"..": [<selectors>]}`                       | a `descendant segment`, composed by one ore more `selectors`                                                            |
| `{"..": <selector>}`                          | shortcut for a `descendant segment`, composed by a unique `selector`                                                    |

### Selector
A selector produces a nodelist consisting of zero or more children of the input value.

| Syntax                                        | Description                                                                                                             |
|-----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `<CBOR Text>`<br>`<CBOR Bytes>`<br>`<CBOR Integer>`<br>`<CBOR Float>`<br>`<CBOR Boolean>`<br>`<CBOR Null>` | `key selector`: selects a child of a CBOR Map based on the child key |
| `{"*": 1}`                                    | `wildcard selector`: selects all children of a node                                                                     |
| `{"#": <index> }`                             | `index selector`: selects an indexed child of an array (from 0)                                                         |
| `{":": [<start>, <end>, <step>]}`             | `array slice selector`: selects a subset of the elements of an array<br>(between `start` and `end` with a `step`)       |
| `{"?": <boolean-expr>}`                       | `filter selector`: selects particular children using a boolean expression                                               |

### Boolean expression
A boolean expression returns `true` or `false` and is used by a `filter selector` to filter array elements or map items.

| Syntax                                        | Description                                                                                                             |
|-----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `{"&&": [<boolean-expr>, <boolean-expr>]}`    | logical `AND`                                                                                                           |
| `{"\|\|": [<boolean-expr>, <boolean-expr>]}`  | logical `OR`                                                                                                            |
| `{"!": <boolean-expr>}`                       | logical `NOT`                                                                                                           |
| `{"<=": [<comparable>, <comparable>]}`        | comparison `lesser than or equal                                                                                        |
| `{"<": [<comparable>, <comparable>]}`         | comparison `lesser than`                                                                                                |
| `{"==": [<comparable>, <comparable>]}`        | comparison `equal`                                                                                                      |
| `{"!=": [<comparable>, <comparable>]}`        | comparison `not equal`                                                                                                  |
| `{">": [<comparable>, <comparable>]}`         | comparison `greater than`                                                                                               |
| `{">=": [<comparable>, <comparable>]}`        | comparison `greater than or equal`                                                                                      |
| `{"match": [<comparable>, <regex>]}`          | match function to compute a regular expression full match.<br>returns a boolean                                         |
| `{"search": [<comparable>, <regex>]}`         | length function to compute a regular expression substring match.<br>returns a boolean                                   |

### Comparable
A `comparable` is an operand of a `filter` comparison or an argument of a function.

| Syntax                                        | Description                                                                                                             |
|-----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `<CBOR Text>`<br>`<CBOR Bytes>`<br>`<CBOR Integer>`<br>`<CBOR Float>`<br>`<CBOR Boolean>`<br>`<CBOR Null>` | a `CBOR` value                                             |
| `["$", <singular-segments>]`<br>`["@", <singular-segments>]` | a singular path (path which procudes a nodelist containing at most one node)                             |
| `{"length": <comparable>}`                    | length function to compute the length of a value.<br>returns an unsigned integer                                        |
| `{"count": <path>}`                           | count function to compute the number of nodes in a path.<br>returns an unsigned integer                                 |

### Singular Segment
A `singular segment` produces a nodelist containing at most one node.

| Syntax                                        | Description                                                                                                             |
|-----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `<CBOR Text>`<br>`<CBOR Bytes>`<br>`<CBOR Integer>`<br>`<CBOR Float>`<br>`<CBOR Boolean>`<br>`<CBOR Null>` | `key selector`: selects a child of a CBOR Map based on the child key |
| `{"#": <index> }`                             | `index selector`: selects an indexed child of an array (from 0)                                                         |

## Examples

This section is informative. It provides examples of CBORPath expressions.

The examples are based on the simple CBOR value representing a bookstore (that also has a bicycle).

```json
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
}
```

This table shows some CBORPath queries that might be applied to this example and their intended results.

| JSONPath                        | CBORPath                                             | Intended result                                                 |
|---------------------------------|------------------------------------------------------|-----------------------------------------------------------------|
| `$.store.book[*].author`        | `["$", "store", "book", {"*": 1}, "author"]`         | the authors of all books in the store                           |
| `$..author`                     | `["$", {"..": "author"}]`                            | all authors                                                     |
| `$.store.*`                     | `["$", "store", {"*": 1}]`                           | all things in store, which are some books and a red bicycle     |
| `$.store..price`                | `["$", "store", {"..": "price"}]`                    | the prices of everything in the store                           |
| `$..book[2]`                    | `["$", {"..": "book"}, {"#": 2}]  `                  | the third book                                                  |
| `$..book[-1]`                   | `["$", {"..": "book"}, {"#": -1}]`                   | the last book in order                                          |
| `$..book[0,1]`<br>`$..book[:2]` | `["$", {"..": "book"}, [{"#": 0}, {"#": 1}]]`<br>`["$", {"..": "book"}, {":": [0, 2, 1]}]` | the first two books       |
| `$..book[?(@.isbn)]`            | `["$", {"..": "book"}, {"?": ["@", "isbn"]}]`        | all books with an ISBN number                                   |
| `$..book[?(@.price<10)]`        | `["$", {"..": "book"}, {"?": {"<": [["@", "price"], 10.0]}}]`  | all books cheaper than 10                             |
| `$..*`                          | `["$", {"..": {"*": 1}}]`                            | all map item values and array elements contained in input value |

# Library Usage

These are a few samples of code based on the examples of the previous section.

```rust
use cborpath::{CborPath, builder};
use ciborium::{cbor, value::Value};

let value = cbor!({ "store" => {
    "book" => [
      { "category" => "reference",
        "author" => "Nigel Rees",
        "title" => "Sayings of the Century",
        "price" => 8.95
      },
      { "category" => "fiction",
        "author" => "Evelyn Waugh",
        "title" => "Sword of Honour",
        "price" => 12.99
      },
      { "category" => "fiction",
        "author" => "Herman Melville",
        "title" => "Moby Dick",
        "isbn" => "0-553-21311-3",
        "price" => 8.99
      },
      { "category" => "fiction",
        "author" => "J. R. R. Tolkien",
        "title" => "The Lord of the Rings",
        "isbn" => "0-395-19395-8",
        "price" => 22.99
      }
    ],
    "bicycle" => {
      "color" => "red",
      "price" => 399
    }
  }
}).unwrap();

// the authors of all books in the store
// ["$", "store", "book", {"*": 1}, "author"]
let cbor_path = CborPath::builder()
  .key("store")
  .key("book")
  .wildcard()
  .key("author")
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!("Nigel Rees").unwrap(),
  &cbor!("Evelyn Waugh").unwrap(),
  &cbor!("Herman Melville").unwrap(),
  &cbor!("J. R. R. Tolkien").unwrap()
], results);

// all authors
// ["$", {"..": "author"}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("author"))
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!("Nigel Rees").unwrap(),
  &cbor!("Evelyn Waugh").unwrap(),
  &cbor!("Herman Melville").unwrap(),
  &cbor!("J. R. R. Tolkien").unwrap()
], results);

// all things in store, which are some books and a red bicycle
// ["$", "store", {"*": 1}]
let cbor_path = CborPath::builder()
  .key("store")
  .wildcard()
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!([
    { "category" => "reference",
      "author" => "Nigel Rees",
      "title" => "Sayings of the Century",
      "price" => 8.95
    },
    { "category" => "fiction",
      "author" => "Evelyn Waugh",
      "title" => "Sword of Honour",
      "price" => 12.99
    },
    { "category" => "fiction",
      "author" => "Herman Melville",
      "title" => "Moby Dick",
      "isbn" => "0-553-21311-3",
      "price" => 8.99
    },
    { "category" => "fiction",
      "author" => "J. R. R. Tolkien",
      "title" => "The Lord of the Rings",
      "isbn" => "0-395-19395-8",
      "price" => 22.99
    }
  ]).unwrap(),
  &cbor!({
    "color" => "red",
    "price" => 399
  }).unwrap()
], results);

// the prices of everything in the store  
// ["$", "store", {"..": "price"}]
let cbor_path = CborPath::builder()
  .key("store")
  .descendant(builder::segment().key("price"))
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!(399).unwrap(),
  &cbor!(8.95).unwrap(),
  &cbor!(12.99).unwrap(),
  &cbor!(8.99).unwrap(),
  &cbor!(22.99).unwrap()
], results);

// the third book
// ["$", {"..": "book"}, {"#": 2}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("book"))
  .index(2)
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({ 
    "category" => "fiction",
    "author" => "Herman Melville",
    "title" => "Moby Dick",
    "isbn" => "0-553-21311-3",
    "price" => 8.99
  }).unwrap()
], results);

// the last book in order
// ["$", {"..": "book"}, {"#": -1}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("book"))
  .index(-1)
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({ 
    "category" => "fiction",
    "author" => "J. R. R. Tolkien",
    "title" => "The Lord of the Rings",
    "isbn" => "0-395-19395-8",
    "price" => 22.99
  }).unwrap()
], results);

// the first two books
// ["$", {"..": "book"}, [{"#": 0}, {"#": 1}]]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("book"))
  .child(builder::segment().index(0).index(1))
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({
    "category" => "reference",
    "author" => "Nigel Rees",
    "title" => "Sayings of the Century",
    "price" => 8.95
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "Evelyn Waugh",
    "title" => "Sword of Honour",
    "price" => 12.99
  }).unwrap()
], results);

// the first two books
// ["$", {"..": "book"}, {":": [0, 2, 1]}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("book"))
  .slice(0, 2, 1)
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({
    "category" => "reference",
    "author" => "Nigel Rees",
    "title" => "Sayings of the Century",
    "price" => 8.95
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "Evelyn Waugh",
    "title" => "Sword of Honour",
    "price" => 12.99
  }).unwrap()
], results);

// all books with an ISBN number
// ["$", {"..": "book"}, {"?": ["@", "isbn"]}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("book"))
  .filter(builder::rel_path().key("isbn"))
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({
    "category" => "fiction",
    "author" => "Herman Melville",
    "title" => "Moby Dick",
    "isbn" => "0-553-21311-3",
    "price" => 8.99
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "J. R. R. Tolkien",
    "title" => "The Lord of the Rings",
    "isbn" => "0-395-19395-8",
    "price" => 22.99
  }).unwrap()
], results);

// all books cheaper than 10
// ["$", {"..": "book"}, {"?": {"<": [["@", "price"], 10.0]}}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().key("book"))
  .filter(builder::lt(builder::sing_rel_path().key("price"), builder::val(10.)))
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({
    "category" => "reference",
    "author" => "Nigel Rees",
    "title" => "Sayings of the Century",
    "price" => 8.95
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "Herman Melville",
    "title" => "Moby Dick",
    "isbn" => "0-553-21311-3",
    "price" => 8.99
  }).unwrap()
], results);

// all map item values and array elements contained in input value
// ["$", {"..": {"*": 1}}]
let cbor_path = CborPath::builder()
  .descendant(builder::segment().wildcard())
  .build();
let results = cbor_path.evaluate(&value);
assert_eq!(vec![
  &cbor!({
    "book" => [
      {
        "category" => "reference",
        "author" => "Nigel Rees",
        "title" => "Sayings of the Century",
        "price" => 8.95
      },
      {
        "category" => "fiction",
        "author" => "Evelyn Waugh",
        "title" => "Sword of Honour",
        "price" => 12.99
      },
      {
        "category" => "fiction",
        "author" => "Herman Melville",
        "title" => "Moby Dick",
        "isbn" => "0-553-21311-3",
        "price" => 8.99
      },
      {
        "category" => "fiction",
        "author" => "J. R. R. Tolkien",
        "title" => "The Lord of the Rings",
        "isbn" => "0-395-19395-8",
        "price" => 22.99
      }
    ],
    "bicycle" => {
      "color" => "red",
      "price" => 399
    }
  }).unwrap(),
  &cbor!([
    {
      "category" => "reference",
      "author" => "Nigel Rees",
      "title" => "Sayings of the Century",
      "price" => 8.95
    },
    {
      "category" => "fiction",
      "author" => "Evelyn Waugh",
      "title" => "Sword of Honour",
      "price" => 12.99
    },
    {
      "category" => "fiction",
      "author" => "Herman Melville",
      "title" => "Moby Dick",
      "isbn" => "0-553-21311-3",
      "price" => 8.99
    },
    {
      "category" => "fiction",
      "author" => "J. R. R. Tolkien",
      "title" => "The Lord of the Rings",
      "isbn" => "0-395-19395-8",
      "price" => 22.99
    }
  ]).unwrap(),
  &cbor!({
    "color" => "red",
    "price" => 399
  }).unwrap(),
  &cbor!({
    "category" => "reference",
    "author" => "Nigel Rees",
    "title" => "Sayings of the Century",
    "price" => 8.95
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "Evelyn Waugh",
    "title" => "Sword of Honour",
    "price" => 12.99
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "Herman Melville",
    "title" => "Moby Dick",
    "isbn" => "0-553-21311-3",
    "price" => 8.99
  }).unwrap(),
  &cbor!({
    "category" => "fiction",
    "author" => "J. R. R. Tolkien",
    "title" => "The Lord of the Rings",
    "isbn" => "0-395-19395-8",
    "price" => 22.99
  }).unwrap(),
  &cbor!("red").unwrap(),
  &cbor!(399).unwrap(),
  &cbor!("reference").unwrap(),
  &cbor!("Nigel Rees").unwrap(),
  &cbor!("Sayings of the Century").unwrap(),
  &cbor!(8.95).unwrap(),
  &cbor!("fiction").unwrap(),
  &cbor!("Evelyn Waugh").unwrap(),
  &cbor!("Sword of Honour").unwrap(),
  &cbor!(12.99).unwrap(),
  &cbor!("fiction").unwrap(),
  &cbor!("Herman Melville").unwrap(),
  &cbor!("Moby Dick").unwrap(),
  &cbor!("0-553-21311-3").unwrap(),
  &cbor!(8.99).unwrap(),
  &cbor!("fiction").unwrap(),
  &cbor!("J. R. R. Tolkien").unwrap(),
  &cbor!("The Lord of the Rings").unwrap(),
  &cbor!("0-395-19395-8").unwrap(),
  &cbor!(22.99).unwrap()
], results);
```
