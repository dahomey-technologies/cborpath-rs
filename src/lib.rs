#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
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

| Syntax                                                                                      | Intended result                                                 |
|---------------------------------------------------------------------------------------------|-----------------------------------------------------------------|
|  `["$", "store", "book", {"*": 1}, "author"]`                                               | the authors of all books in the store                           |
|  `["$", {"..": "author"}]`                                                                  | all authors                                                     |
|  `["$", "store", {"*": 1}]`                                                                 | all things in store, which are some books<br> and a red bicycle |
|  `["$", "store", {"..": "price"}]`                                                          | the prices of everything in the store                           |
|  `["$", {"..": "book"}, {"#": 2}]  `                                                        | the third book                                                  |
|  `["$", {"..": "book"}, {"#": -1}]`                                                         | the last book in order                                          |
|  `["$", {"..": "book"}, [{"#": 0}, {"#": 1}]]`<br>or<br>`["$", {"..": "book"}, {":": [0, 2, 1]}]` | the first two books                                       |
|  `["$", {"..": "book"}, {"?": ["@", "isbn"]}]`                                              | all books with an ISBN number                                   |
|  `["$", {"..": "book"}, {"?": {"<": [["@", "price"], 10.0]}}]`                              | all books cheaper than 10                                       |
|  `["$", {"..": {"*": 1}}]`                                                              | all map item values and array elements<br> contained in input value |

# Library Usage

These are a few samples of code based on the examples of the previous section.

```
use cborpath::{CborPath, builder, Error};
use cbor_diag::parse_diag;

pub fn diag_to_bytes(cbor_diag_str: &str) -> Vec<u8> {
    parse_diag(cbor_diag_str).unwrap().to_bytes()
}

fn main() -> Result<(), Error> {
  let value = diag_to_bytes(
  r#"{
    "store": {
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
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
              "Nigel Rees",
              "Evelyn Waugh",
              "Herman Melville",
              "J. R. R. Tolkien"
          ]"#
      ),
      results
  );

  // all authors
  // ["$", {"..": "author"}]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().key("author"))
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
              "Nigel Rees",
              "Evelyn Waugh",
              "Herman Melville",
              "J. R. R. Tolkien"
          ]"#
      ),
      results
  );

  // all things in store, which are some books and a red bicycle
  // ["$", "store", {"*": 1}]
  let cbor_path = CborPath::builder().key("store").wildcard().build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[[
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
              {
                  "color": "red",
                  "price": 399
              }]"#
      ),
      results
  );

  // the prices of everything in the store
  // ["$", "store", {"..": "price"}]
  let cbor_path = CborPath::builder()
      .key("store")
      .descendant(builder::segment().key("price"))
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
              399,
              8.95,
              12.99,
              8.99,
              22.99
          ]"#
      ),
      results
  );

  // the third book
  // ["$", {"..": "book"}, {"#": 2}]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().key("book"))
      .index(2)
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[{
              "category": "fiction",
              "author": "Herman Melville",
              "title": "Moby Dick",
              "isbn": "0-553-21311-3",
              "price": 8.99
          }]"#
      ),
      results
  );

  // the last book in order
  // ["$", {"..": "book"}, {"#": -1}]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().key("book"))
      .index(-1)
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[{
              "category": "fiction",
              "author": "J. R. R. Tolkien",
              "title": "The Lord of the Rings",
              "isbn": "0-395-19395-8",
              "price": 22.99
          }]"#
      ),
      results
  );

  // the first two books
  // ["$", {"..": "book"}, [{"#": 0}, {"#": 1}]]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().key("book"))
      .child(builder::segment().index(0).index(1))
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
          {
            "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          }]"#
      ),
      results
  );

  // the first two books
  // ["$", {"..": "book"}, {":": [0, 2, 1]}]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().key("book"))
      .slice(0, 2, 1)
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
          {
            "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          }]"#
      ),
      results
  );

  // all books with an ISBN number
  // ["$", {"..": "book"}, {"?": ["@", "isbn"]}]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().key("book"))
      .filter(builder::rel_path().key("isbn"))
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
          {
            "category": "fiction",
            "author": "Herman Melville",
            "title": "Moby Dick",
            "isbn": "0-553-21311-3",
            "price": 8.99
          },
          {
            "category": "fiction",
            "author": "J. R. R. Tolkien",
            "title": "The Lord of the Rings",
            "isbn": "0-395-19395-8",
            "price": 22.99
          }]"#
      ),
      results
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
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[
          {
            "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          {
            "category": "fiction",
            "author": "Herman Melville",
            "title": "Moby Dick",
            "isbn": "0-553-21311-3",
            "price": 8.99
          }]"#
      ),
      results
  );

  // all map item values and array elements contained in input value
  // ["$", {"..": {"*": 1}}]
  let cbor_path = CborPath::builder()
      .descendant(builder::segment().wildcard())
      .build();
  let results = cbor_path.read_from_bytes(&value)?;
  assert_eq!(
      diag_to_bytes(
          r#"[{
            "book": [
              {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
              },
              {
                "category": "fiction",
                "author": "Evelyn Waugh",
                "title": "Sword of Honour",
                "price": 12.99
              },
              {
                "category": "fiction",
                "author": "Herman Melville",
                "title": "Moby Dick",
                "isbn": "0-553-21311-3",
                "price": 8.99
              },
              {
                "category": "fiction",
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
          },
          [
            {
              "category": "reference",
              "author": "Nigel Rees",
              "title": "Sayings of the Century",
              "price": 8.95
            },
            {
              "category": "fiction",
              "author": "Evelyn Waugh",
              "title": "Sword of Honour",
              "price": 12.99
            },
            {
              "category": "fiction",
              "author": "Herman Melville",
              "title": "Moby Dick",
              "isbn": "0-553-21311-3",
              "price": 8.99
            },
            {
              "category": "fiction",
              "author": "J. R. R. Tolkien",
              "title": "The Lord of the Rings",
              "isbn": "0-395-19395-8",
              "price": 22.99
            }
          ],
          {
            "color": "red",
            "price": 399
          },
          {
            "category": "reference",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          },
          {
            "category": "fiction",
            "author": "Herman Melville",
            "title": "Moby Dick",
            "isbn": "0-553-21311-3",
            "price": 8.99
          },
          {
            "category": "fiction",
            "author": "J. R. R. Tolkien",
            "title": "The Lord of the Rings",
            "isbn": "0-395-19395-8",
            "price": 22.99
          },
          "red",
          399,
          "reference",
          "Nigel Rees",
          "Sayings of the Century",
          8.95,
          "fiction",
          "Evelyn Waugh",
          "Sword of Honour",
          12.99,
          "fiction",
          "Herman Melville",
          "Moby Dick",
          "0-553-21311-3",
          8.99,
          "fiction",
          "J. R. R. Tolkien",
          "The Lord of the Rings",
          "0-395-19395-8",
          22.99
          ]"#
      ),
      results
  );

  Ok(())
}
```
*/

pub mod builder;
mod cbor_path;
mod conversion;
mod error;
mod write_visitor;

pub use cbor_path::*;
pub use error::*;

#[cfg(test)]
mod tests;
