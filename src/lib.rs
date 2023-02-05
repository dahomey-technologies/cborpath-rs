#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
cborpath is a CBORPath engine written in Rust.
CBORPath is an adaptation of JSONPath to [CBOR](https://www.rfc-editor.org/rfc/rfc8949.html) based on the [JsonPATH RFC Draft](https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html)

# CBORPath
## Syntax summary

| JSONPath            | CBORPath                | Description                                                                                                             |
|---------------------|-------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `$`                 | `"$"`                   | root node identifier                                                                               |
| `@`                 | `"@"`                   | current node identifier (valid only within filter selectors)                                        |
| `[<selectors>]`     | `[<selectors>]`         | child segment selects zero or more children of a node; contains one or more selectors, separated by commas |
| `..[<selectors>]`   | `{"..": [<selectors>]}` | descendant segment: selects zero or more descendants of a node; contains one or more selectors, separated by commas |
| `'name'`            | `<CBOR Text>`<br>`<CBOR Bytes>`<br>`<CBOR Integer>`<br>`<CBOR Float>`<br>`<CBOR Boolean>`<br>`<CBOR Null>` | key selector: selects a child of a CBOR Map based on the child key |
| `*`                 | `{"*": 1}`                   | wildcard selector: selects all children of a node                                                      |
| `3`                 | `{"#": <index> }`       | index selector: selects an indexed child of an array (from 0)                                        |
| `0:100:5`           | `{":": [<start>, <end>, <step>]}` | array slice selector: start:end:step for arrays                                                     |
| `?<expr>`           | `{"?": <expr>}`         | filter selector: selects particular children using a boolean expression                             |
| `length(@.foo)`     | `{"length": ["@", "foo"]}` | function extension: invokes a function in a filter expression                                               |

## Examples

This section is informative. It provides examples of CBORPath expressions.

The examples are based on the simple CBOR value representing a bookstore (that also has a bicycle).

~~~~cbor
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
~~~~

This table shows some CBORPath queries that might be applied to this example and their intended results.

| JSONPath                        | CBORPath                       | Intended result                                                                     |
|---------------------------------|------------------------------------------------------|---------------------------------------------------------------|
| `$.store.book[*].author`        | `["$", "store", "book", {"*": 1}, "author"]`              | the authors of all books in the store                         |
| `$..author`                     | `["$", {"..": "author"}]`                            | all authors                                                   |
| `$.store.*`                     | `["$", "store", {"*": 1}]`                                | all things in store, which are some books and a red bicycle   |
| `$.store..price`                | `["$", "store", {"..": "price"}]`                    | the prices of everything in the store                         |
| `$..book[2]`                    | `["$", {"..": ["book", {"#": 2}]}]`                  | the third book                                                |
| `$..book[-1]`                   | `["$", {"..": ["book", {"#": -1}]}]`                 | the last book in order                                        |
| `$..book[0,1]`<br>`$..book[:2]` | `["$", {"..": ["book", [{"#": 0}, {"#": 1}]]}]`<br>`["$", {"..": ["book", {":": [0, 1, 1]}]}]` | the first two books |
| `$..book[?(@.isbn)]`            | `["$", {"..": {"?": ["@", "isbn"]}}]`                | all books with an ISBN number                                 |
| `$..book[?(@.price<10)]`        | `["$", {"..": {"?": {"<": [["@", "price"], 10]}}}]`  | all books cheaper than 10                                     |
| `$..*`                          | `["$": {"..": {"*": 1}}]`                                   | all member values and array elements contained in input value |

*/

pub mod builder;
mod cbor_path;
mod deserialization;
mod error;
mod value;

pub use cbor_path::*;
pub use error::*;

#[cfg(test)]
mod tests;
