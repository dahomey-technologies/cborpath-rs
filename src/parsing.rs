use crate::{
    BooleanExpr, CborPath, Comparable, ComparisonOperator, Error, Function, Path, Segment,
    Selector, SingularPath, SingularSegment,
};
use ciborium::value::Value;

pub fn parse_cbor_path(value: &Value) -> Result<CborPath, Error> {
    Ok(CborPath::new(parse_segments(value, true)?))
}

fn parse_path(value: &Value) -> Result<Path, Error> {
    match value {
        Value::Text(identifier) => match identifier.as_str() {
            "$" => Ok(Path::abs(vec![])),
            "@" => Ok(Path::rel(vec![])),
            _ => Err(Error::Parsing(
                "Expected path identifier `$` or `@`".to_owned(),
            )),
        },
        Value::Array(values) => {
            let mut iter = values.iter();

            let Some(Value::Text(identifier)) = iter.next() else {
                return Err(Error::Parsing("Expected path identifier `$` or `@`".to_owned()));
            };

            let is_absolute_path = match identifier.as_str() {
                "$" => true,
                "@" => false,
                _ => {
                    return Err(Error::Parsing(
                        "Expected path identifier `$` or `@`".to_owned(),
                    ));
                }
            };

            let mut segments = Vec::with_capacity(values.len() - 1);

            for value in iter {
                let segment = parse_segment(value)?;
                segments.push(segment.into_segment());
            }

            if is_absolute_path {
                Ok(Path::abs(segments))
            } else {
                Ok(Path::rel(segments))
            }
        }
        _ => Err(Error::Parsing(format!(
            "Cannot parse path segments from `{value:?}`"
        ))),
    }
}

fn parse_segments(value: &Value, absolute_path: bool) -> Result<Vec<Segment>, Error> {
    match value {
        Value::Text(identifier) => match identifier.as_str() {
            "$" => Ok(vec![]),
            "@" => Ok(vec![]),
            _ => Err(Error::Parsing(
                "Expected path identifier `$` or `@`".to_owned(),
            )),
        },
        Value::Array(values) => {
            let mut iter = values.iter();

            let expected_identifier = if absolute_path { "$" } else { "@" };

            let Some(Value::Text(identifier)) = iter.next() else {
                return Err(Error::Parsing(format!("Expected path identifier `{expected_identifier}`")));
            };

            if identifier != expected_identifier {
                return Err(Error::Parsing(format!(
                    "Expected path identifier `{expected_identifier}`"
                )));
            }

            let mut segments = Vec::with_capacity(values.len() - 1);

            for value in iter {
                let segment = parse_segment(value)?.into_segment();
                segments.push(segment);
            }

            Ok(segments)
        }
        _ => Err(Error::Parsing(format!(
            "Cannot parse path segments from `{value:?}`"
        ))),
    }
}

enum SegmentForParsing {
    Selector(Selector),
    Selectors(Vec<Selector>),
    Descendant(Vec<Selector>),
}

impl SegmentForParsing {
    pub fn into_segment(self) -> Segment {
        match self {
            SegmentForParsing::Selector(selector) => Segment::Child(vec![selector]),
            SegmentForParsing::Selectors(selectors) => Segment::Child(selectors),
            SegmentForParsing::Descendant(selectors) => Segment::Descendant(selectors),
        }
    }
}

fn parse_segment(value: &Value) -> Result<SegmentForParsing, Error> {
    match value {
        Value::Integer(_) | Value::Bytes(_) | Value::Float(_) | Value::Bool(_) | Value::Null => {
            Ok(SegmentForParsing::Selector(Selector::key(value.clone())))
        }
        Value::Text(s) => {
            if s == "*" {
                Ok(SegmentForParsing::Selector(Selector::wildcard()))
            } else {
                Ok(SegmentForParsing::Selector(Selector::key(value.clone())))
            }
        }
        Value::Array(a) => {
            let selectors = a
                .iter()
                .map(|v| match parse_segment(v)? {
                    SegmentForParsing::Selector(selector) => Ok(selector),
                    _ => Err(Error::Parsing("Expected a single selector".to_owned())),
                })
                .collect::<Result<Vec<Selector>, Error>>()?;
            Ok(SegmentForParsing::Selectors(selectors))
        }
        Value::Map(m) => {
            let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Parsing("Expected a single element map".to_owned()));
            };

            match identifier.as_str() {
                ".." => match parse_segment(value)? {
                    SegmentForParsing::Selectors(selectors) => {
                        Ok(SegmentForParsing::Descendant(selectors))
                    }
                    _ => Err(Error::Parsing(
                        "Expected selector or array of selectors in a descendant segment"
                            .to_owned(),
                    )),
                },
                "#" => Ok(SegmentForParsing::Selector(Selector::index(parse_index(
                    value,
                )?))),
                ":" => Ok(SegmentForParsing::Selector(parse_slice_selector(value)?)),
                "?" => Ok(SegmentForParsing::Selector(Selector::filter(
                    parse_boolean_expr(value)?,
                ))),
                _ => Err(Error::Parsing(
                    "Expected identifier `..`, `#`, `:` or `?`".to_owned(),
                )),
            }
        }
        _ => Err(Error::Parsing(format!(
            "Cannot parse segments from `{value:?}`"
        ))),
    }
}

fn parse_index(value: &Value) -> Result<isize, Error> {
    if let Value::Integer(index) = value {
        let index: i64 = (*index).try_into()?;
        Ok(index as isize)
    } else {
        Err(Error::Parsing("Expected integer".to_owned()))
    }
}

fn parse_slice_selector(value: &Value) -> Result<Selector, Error> {
    if let Value::Array(a) = value {
        match &a[..] {
            [Value::Integer(start), Value::Integer(end), Value::Integer(step)] => {
                let start: i64 = (*start).try_into()?;
                let end: i64 = (*end).try_into()?;
                let step: i64 = (*step).try_into()?;
                Ok(Selector::slice(start as isize, end as isize, step as isize))
            }
            _ => Err(Error::Parsing("Expected 3-elements array".to_owned())),
        }
    } else {
        Err(Error::Parsing("Expected array".to_owned()))
    }
}

fn parse_boolean_expr(value: &Value) -> Result<BooleanExpr, Error> {
    match value {
        Value::Array(_) => Ok(BooleanExpr::path(parse_path(value)?)),
        Value::Map(m) => {
            let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Parsing("Expected a single element map".to_owned()));
            };

            match identifier.as_str() {
                "&&" => {
                    if let Value::Array(a) = value {
                        match &a[..] {
                            [left, right] => Ok(BooleanExpr::and(
                                parse_boolean_expr(left)?,
                                parse_boolean_expr(right)?,
                            )),
                            _ => Err(Error::Parsing(format!(
                                "Cannot parse boolean expression from `{value:?}`"
                            ))),
                        }
                    } else {
                        Err(Error::Parsing(format!(
                            "Cannot parse boolean expression from `{value:?}`"
                        )))
                    }
                }
                "||" => {
                    if let Value::Array(a) = value {
                        match &a[..] {
                            [left, right] => Ok(BooleanExpr::and(
                                parse_boolean_expr(left)?,
                                parse_boolean_expr(right)?,
                            )),
                            _ => Err(Error::Parsing(format!(
                                "Cannot parse boolean expression from `{value:?}`"
                            ))),
                        }
                    } else {
                        Err(Error::Parsing(format!(
                            "Cannot parse boolean expression from `{value:?}`"
                        )))
                    }
                }
                "!" => Ok(BooleanExpr::not(parse_boolean_expr(value)?)),
                "<" | "<=" | "==" | "!=" | ">=" | ">" => parse_comparison(identifier, value),
                "match" => Ok(BooleanExpr::function(parse_match(value)?)),
                "search" => Ok(BooleanExpr::function(parse_search(value)?)),
                _ => Err(Error::Parsing(format!(
                    "Cannot parse boolean expression from `{value:?}`"
                ))),
            }
        }
        _ => Err(Error::Parsing(format!(
            "Cannot parse boolean expression from `{value:?}`"
        ))),
    }
}

fn parse_comparison(identifier: &str, value: &Value) -> Result<BooleanExpr, Error> {
    let operator = match identifier {
        "<" => ComparisonOperator::Lt,
        "<=" => ComparisonOperator::Lte,
        "==" => ComparisonOperator::Eq,
        "!=" => ComparisonOperator::Neq,
        ">=" => ComparisonOperator::Gte,
        ">" => ComparisonOperator::Gt,
        _ => {
            return Err(Error::Parsing(format!(
                "Cannot parse comparison operator from `{identifier:?}`"
            )))
        }
    };

    if let Value::Array(a) = value {
        match &a[..] {
            [left, right] => Ok(BooleanExpr::comparison(
                parse_comparable(left)?,
                operator,
                parse_comparable(right)?,
            )),
            _ => Err(Error::Parsing(format!(
                "Cannot parse comparison from `{value:?}`"
            ))),
        }
    } else {
        Err(Error::Parsing(format!(
            "Cannot parse comparison from `{value:?}`"
        )))
    }
}

fn parse_comparable(value: &Value) -> Result<Comparable, Error> {
    match value {
        Value::Integer(_)
        | Value::Bytes(_)
        | Value::Float(_)
        | Value::Text(_)
        | Value::Bool(_)
        | Value::Null => Ok(Comparable::Value(value.clone())),
        Value::Array(a) => Ok(Comparable::SingularPath(parse_singular_path(a)?)),
        Value::Map(m) => {
            let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Parsing("Expected a single element map".to_owned()));
            };

            match identifier.as_str() {
                "length" => Ok(Comparable::Function(parse_length(value)?)),
                "count" => Ok(Comparable::Function(parse_count(value)?)),
                _ => Err(Error::Parsing(
                    "Expected `length` or `count` function".to_owned(),
                )),
            }
        }
        _ => Err(Error::Parsing(format!(
            "Cannot parse comparable from `{value:?}`"
        ))),
    }
}

fn parse_match(value: &Value) -> Result<Function, Error> {
    if let Value::Array(a) = value {
        match &a[..] {
            [comparable, Value::Text(regex)] => {
                Function::_match(parse_comparable(comparable)?, regex.as_str())
            }
            _ => Err(Error::Parsing(format!(
                "Cannot parse match function from `{value:?}`"
            ))),
        }
    } else {
        Err(Error::Parsing(format!(
            "Cannot parse match function from `{value:?}`"
        )))
    }
}

fn parse_search(value: &Value) -> Result<Function, Error> {
    if let Value::Array(a) = value {
        match &a[..] {
            [comparable, Value::Text(regex)] => {
                Function::search(parse_comparable(comparable)?, regex.as_str())
            }
            _ => Err(Error::Parsing(format!(
                "Cannot parse search function from `{value:?}`"
            ))),
        }
    } else {
        Err(Error::Parsing(format!(
            "Cannot parse search function from `{value:?}`"
        )))
    }
}

fn parse_length(value: &Value) -> Result<Function, Error> {
    Ok(Function::length(parse_comparable(value)?))
}

fn parse_count(value: &Value) -> Result<Function, Error> {
    Ok(Function::count(parse_path(value)?))
}

fn parse_singular_path(values: &[Value]) -> Result<SingularPath, Error> {
    let mut iter = values.iter();

    let Some(Value::Text(identifier)) = iter.next() else {
        return Err(Error::Parsing("Expected singular path identifier `$` or `@`".to_owned()));
    };

    let is_absolute_path = match identifier.as_str() {
        "$" => true,
        "@" => false,
        _ => {
            return Err(Error::Parsing(
                "Expected singular path identifier `$` or `@`".to_owned(),
            ));
        }
    };

    let mut segments = Vec::with_capacity(values.len() - 1);

    for value in iter {
        let segment = parse_singular_segment(value)?;
        segments.push(segment);
    }

    if is_absolute_path {
        Ok(SingularPath::abs(segments))
    } else {
        Ok(SingularPath::rel(segments))
    }
}

fn parse_singular_segment(value: &Value) -> Result<SingularSegment, Error> {
    match value {
        Value::Integer(_)
        | Value::Bytes(_)
        | Value::Float(_)
        | Value::Text(_)
        | Value::Bool(_)
        | Value::Null => Ok(SingularSegment::key(value.clone())),
        Value::Map(m) => {
            let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Parsing("Expected a single element map".to_owned()));
            };

            match identifier.as_str() {
                "#" => Ok(SingularSegment::index(parse_index(value)?)),
                _ => Err(Error::Parsing("Expected identifier `#`".to_owned())),
            }
        }
        _ => Err(Error::Parsing(format!(
            "Cannot parse singular segment from `{value:?}`"
        ))),
    }
}
