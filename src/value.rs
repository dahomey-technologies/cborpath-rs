use crate::{
    BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, Error, Function,
    IndexSelector, Path, Segment, Selector, SingularPath, SingularSegment, SliceSelector,
};
use ciborium::value::Value;

impl TryFrom<&Value> for CborPath {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let segments: Segments = (value, true).try_into()?;
        Ok(CborPath::new(segments.0))
    }
}

impl TryFrom<&Value> for Path {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Text(identifier) => match identifier.as_str() {
                "$" => Ok(Path::abs(vec![])),
                "@" => Ok(Path::rel(vec![])),
                _ => Err(Error::Conversion(
                    "Expected path identifier `$` or `@`".to_owned(),
                )),
            },
            Value::Array(values) => {
                let mut iter = values.iter();

                let Some(Value::Text(identifier)) = iter.next() else {
                return Err(Error::Conversion("Expected path identifier `$` or `@`".to_owned()));
            };

                let is_absolute_path = match identifier.as_str() {
                    "$" => true,
                    "@" => false,
                    _ => {
                        return Err(Error::Conversion(
                            "Expected path identifier `$` or `@`".to_owned(),
                        ));
                    }
                };

                let mut segments = Vec::with_capacity(values.len() - 1);

                for value in iter {
                    let segment: SegmentForConversion = value.try_into()?;
                    segments.push(segment.into_segment());
                }

                if is_absolute_path {
                    Ok(Path::abs(segments))
                } else {
                    Ok(Path::rel(segments))
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse path segments from `{value:?}`"
            ))),
        }
    }
}

struct Segments(Vec<Segment>);

impl TryFrom<(&Value, bool)> for Segments {
    type Error = Error;

    fn try_from(value: (&Value, bool)) -> Result<Self, Self::Error> {
        let (value, absolute_path) = value;

        match value {
            Value::Text(identifier) => match identifier.as_str() {
                "$" => Ok(Segments(vec![])),
                "@" => Ok(Segments(vec![])),
                _ => Err(Error::Conversion(
                    "Expected path identifier `$` or `@`".to_owned(),
                )),
            },
            Value::Array(values) => {
                let mut iter = values.iter();

                let expected_identifier = if absolute_path { "$" } else { "@" };

                let Some(Value::Text(identifier)) = iter.next() else {
                return Err(Error::Conversion(format!("Expected path identifier `{expected_identifier}`")));
            };

                if identifier != expected_identifier {
                    return Err(Error::Conversion(format!(
                        "Expected path identifier `{expected_identifier}`"
                    )));
                }

                let mut segments = Vec::with_capacity(values.len() - 1);

                for value in iter {
                    let segment: SegmentForConversion = value.try_into()?;
                    segments.push(segment.into_segment());
                }

                Ok(Segments(segments))
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse path segments from `{value:?}`"
            ))),
        }
    }
}

enum SegmentForConversion {
    Selector(Selector),
    Selectors(Vec<Selector>),
    Descendant(Vec<Selector>),
}

impl SegmentForConversion {
    pub fn into_segment(self) -> Segment {
        match self {
            SegmentForConversion::Selector(selector) => Segment::Child(vec![selector]),
            SegmentForConversion::Selectors(selectors) => Segment::Child(selectors),
            SegmentForConversion::Descendant(selectors) => Segment::Descendant(selectors),
        }
    }
}

impl TryFrom<&Value> for SegmentForConversion {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(_)
            | Value::Bytes(_)
            | Value::Float(_)
            | Value::Bool(_)
            | Value::Null => Ok(SegmentForConversion::Selector(Selector::key(value.clone()))),
            Value::Text(_) => Ok(SegmentForConversion::Selector(Selector::key(value.clone()))),
            Value::Array(a) => {
                let selectors = a
                    .iter()
                    .map(|v| match v.try_into()? {
                        SegmentForConversion::Selector(selector) => Ok(selector),
                        _ => Err(Error::Conversion("Expected a single selector".to_owned())),
                    })
                    .collect::<Result<Vec<Selector>, Error>>()?;
                Ok(SegmentForConversion::Selectors(selectors))
            }
            Value::Map(m) => {
                let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Conversion("Expected a single element map".to_owned()));
            };

                match identifier.as_str() {
                    ".." => match value.try_into()? {
                        SegmentForConversion::Selectors(selectors) => {
                            Ok(SegmentForConversion::Descendant(selectors))
                        }
                        _ => Err(Error::Conversion(
                            "Expected selector or array of selectors in a descendant segment"
                                .to_owned(),
                        )),
                    },
                    "*" => match value {
                        Value::Integer(i) if Into::<i128>::into(*i) == 1i128 => {
                            Ok(SegmentForConversion::Selector(Selector::Wildcard))
                        }
                        _ => Err(Error::Conversion("Cannot parse wildcard".to_owned())),
                    },
                    "#" => Ok(SegmentForConversion::Selector(Selector::Index(
                        value.try_into()?,
                    ))),
                    ":" => Ok(SegmentForConversion::Selector(Selector::Slice(
                        value.try_into()?,
                    ))),
                    "?" => Ok(SegmentForConversion::Selector(Selector::filter(
                        value.try_into()?,
                    ))),
                    _ => Err(Error::Conversion(
                        "Expected identifier `..`, `#`, `:` or `?`".to_owned(),
                    )),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse segments from `{value:?}`"
            ))),
        }
    }
}

impl TryFrom<&Value> for IndexSelector {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Value::Integer(index) = value {
            let index: i64 = (*index).try_into()?;
            Ok(IndexSelector::new(index as isize))
        } else {
            Err(Error::Conversion("Expected integer".to_owned()))
        }
    }
}

impl TryFrom<&Value> for SliceSelector {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Value::Array(a) = value {
            match &a[..] {
                [Value::Integer(start), Value::Integer(end), Value::Integer(step)] => {
                    let start: i64 = (*start).try_into()?;
                    let end: i64 = (*end).try_into()?;
                    let step: i64 = (*step).try_into()?;
                    Ok(SliceSelector::new(
                        start as isize,
                        end as isize,
                        step as isize,
                    ))
                }
                _ => Err(Error::Conversion("Expected 3-elements array".to_owned())),
            }
        } else {
            Err(Error::Conversion("Expected array".to_owned()))
        }
    }
}

impl TryFrom<&Value> for BooleanExpr {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(_) => Ok(BooleanExpr::path(value.try_into()?)),
            Value::Map(m) => {
                let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Conversion("Expected a single element map".to_owned()));
            };

                match identifier.as_str() {
                    "&&" => {
                        if let Value::Array(a) = value {
                            match &a[..] {
                                [left, right] => {
                                    Ok(BooleanExpr::and(left.try_into()?, right.try_into()?))
                                }
                                _ => Err(Error::Conversion(format!(
                                    "Cannot parse boolean expression from `{value:?}`"
                                ))),
                            }
                        } else {
                            Err(Error::Conversion(format!(
                                "Cannot parse boolean expression from `{value:?}`"
                            )))
                        }
                    }
                    "||" => {
                        if let Value::Array(a) = value {
                            match &a[..] {
                                [left, right] => {
                                    Ok(BooleanExpr::and(left.try_into()?, right.try_into()?))
                                }
                                _ => Err(Error::Conversion(format!(
                                    "Cannot parse boolean expression from `{value:?}`"
                                ))),
                            }
                        } else {
                            Err(Error::Conversion(format!(
                                "Cannot parse boolean expression from `{value:?}`"
                            )))
                        }
                    }
                    "!" => Ok(BooleanExpr::not(value.try_into()?)),
                    "<" | "<=" | "==" | "!=" | ">=" | ">" => Ok(BooleanExpr::Comparison(
                        (identifier.as_str(), value).try_into()?,
                    )),
                    "match" => {
                        if let Value::Array(a) = value {
                            match &a[..] {
                                [comparable, Value::Text(regex)] => Ok(BooleanExpr::function(
                                    Function::_match(comparable.try_into()?, regex.as_str())?,
                                )),
                                _ => Err(Error::Conversion(format!(
                                    "Cannot parse match function from `{value:?}`"
                                ))),
                            }
                        } else {
                            Err(Error::Conversion(format!(
                                "Cannot parse match function from `{value:?}`"
                            )))
                        }
                    }
                    "search" => {
                        if let Value::Array(a) = value {
                            match &a[..] {
                                [comparable, Value::Text(regex)] => Ok(BooleanExpr::function(
                                    Function::search(comparable.try_into()?, regex.as_str())?,
                                )),
                                _ => Err(Error::Conversion(format!(
                                    "Cannot parse search function from `{value:?}`"
                                ))),
                            }
                        } else {
                            Err(Error::Conversion(format!(
                                "Cannot parse search function from `{value:?}`"
                            )))
                        }
                    }
                    _ => Err(Error::Conversion(format!(
                        "Cannot parse boolean expression from `{value:?}`"
                    ))),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse boolean expression from `{value:?}`"
            ))),
        }
    }
}

impl TryFrom<(&str, &Value)> for ComparisonExpr {
    type Error = Error;

    fn try_from(value: (&str, &Value)) -> Result<Self, Self::Error> {
        let (identifier, value) = value;

        let operator = match identifier {
            "<" => ComparisonOperator::Lt,
            "<=" => ComparisonOperator::Lte,
            "==" => ComparisonOperator::Eq,
            "!=" => ComparisonOperator::Neq,
            ">=" => ComparisonOperator::Gte,
            ">" => ComparisonOperator::Gt,
            _ => {
                return Err(Error::Conversion(format!(
                    "Cannot parse comparison operator from `{identifier:?}`"
                )))
            }
        };

        if let Value::Array(a) = value {
            match &a[..] {
                [left, right] => Ok(ComparisonExpr::new(
                    left.try_into()?,
                    operator,
                    right.try_into()?,
                )),
                _ => Err(Error::Conversion(format!(
                    "Cannot parse comparison from `{value:?}`"
                ))),
            }
        } else {
            Err(Error::Conversion(format!(
                "Cannot parse comparison from `{value:?}`"
            )))
        }
    }
}

impl TryFrom<&Value> for Comparable {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(_)
            | Value::Bytes(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Bool(_)
            | Value::Null => Ok(Comparable::Value(value.clone())),
            Value::Array(a) => Ok(Comparable::SingularPath(a.try_into()?)),
            Value::Map(m) => {
                let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Conversion("Expected a single element map".to_owned()));
            };

                match identifier.as_str() {
                    "length" => Ok(Comparable::Function(Function::length(value.try_into()?))),
                    "count" => Ok(Comparable::Function(Function::count(value.try_into()?))),
                    _ => Err(Error::Conversion(
                        "Expected `length` or `count` function".to_owned(),
                    )),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse comparable from `{value:?}`"
            ))),
        }
    }
}

impl TryFrom<&Vec<Value>> for SingularPath {
    type Error = Error;

    fn try_from(values: &Vec<Value>) -> Result<Self, Self::Error> {
        let mut iter = values.iter();

        let Some(Value::Text(identifier)) = iter.next() else {
        return Err(Error::Conversion("Expected singular path identifier `$` or `@`".to_owned()));
    };

        let is_absolute_path = match identifier.as_str() {
            "$" => true,
            "@" => false,
            _ => {
                return Err(Error::Conversion(
                    "Expected singular path identifier `$` or `@`".to_owned(),
                ));
            }
        };

        let mut segments = Vec::with_capacity(values.len() - 1);

        for value in iter {
            let segment = value.try_into()?;
            segments.push(segment);
        }

        if is_absolute_path {
            Ok(SingularPath::abs(segments))
        } else {
            Ok(SingularPath::rel(segments))
        }
    }
}

impl TryFrom<&Value> for SingularSegment {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(_)
            | Value::Bytes(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Bool(_)
            | Value::Null => Ok(SingularSegment::key(value.clone())),
            Value::Map(m) => {
                let Some((Value::Text(identifier), value)) = m.first() else {
                return Err(Error::Conversion("Expected a single element map".to_owned()));
            };

                match identifier.as_str() {
                    "#" => Ok(SingularSegment::Index(value.try_into()?)),
                    _ => Err(Error::Conversion("Expected identifier `#`".to_owned())),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse singular segment from `{value:?}`"
            ))),
        }
    }
}
