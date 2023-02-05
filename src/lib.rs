#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
cborpath is a CBORPath engine written in Rust.

# CBORPath
CBORPath is an adaptation of JSONPath to [CBOR](https://www.rfc-editor.org/rfc/rfc8949.html) based on the [JSONPath RFC Draft](https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html)

## Syntax summary

| JSONPath            | CBORPath                | Description                                                                                                             |
|---------------------|-------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `$`                 | `"$"`                   | root node identifier                                                                                                    |
| `@`                 | `"@"`                   | current node identifier (valid only within filter selectors)                                                            |
| `[<selectors>]`     | `[<selectors>]`         | child segment selects zero or more children of a node; contains one or more selectors, separated by commas              |
| `..[<selectors>]`   | `{"..": [<selectors>]}` | descendant segment: selects zero or more descendants of a node; contains one or more selectors, separated by commas     |
| `'name'`            | `<CBOR Text>`<br>`<CBOR Bytes>`<br>`<CBOR Integer>`<br>`<CBOR Float>`<br>`<CBOR Boolean>`<br>`<CBOR Null>` | key selector: selects a child of a CBOR Map based on the child key |
| `*`                 | `{"*": 1}`              | wildcard selector: selects all children of a node                                                                       |
| `3`                 | `{"#": <index> }`       | index selector: selects an indexed child of an array (from 0)                                                           |
| `0:100:5`           | `{":": [<start>, <end>, <step>]}` | array slice selector: start:end:step for arrays                                                               |
| `?<expr>`           | `{"?": <expr>}`         | filter selector: selects particular children using a boolean expression                                                 |
| `length(@.foo)`     | `{"length": ["@", "foo"]}` | function extension: invokes a function in a filter expression                                                        |

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

```
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
