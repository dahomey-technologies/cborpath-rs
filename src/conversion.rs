use crate::{
    builder, AbsolutePath, BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator,
    Error, FilterPath, FilterSelector, Function, IndexSelector, KeySelector, RelativePath, Segment,
    Selector, SingularPath, SingularSegment, SliceSelector,
};
use cbor_data::{ArrayIter, Cbor, ItemKind};

impl TryFrom<&Cbor> for CborPath {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        let segments: Segments = (value, true).try_into()?;
        Ok(CborPath::new(segments.0))
    }
}

impl TryFrom<&Cbor> for FilterPath {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        match value.kind() {
            ItemKind::Str(identifier) => match identifier.as_str() {
                Some("$") => Ok(FilterPath::Abs(AbsolutePath::new(vec![]))),
                Some("@") => Ok(FilterPath::Rel(RelativePath::new(vec![]))),
                _ => Err(Error::Conversion(
                    "Expected path identifier `$` or `@`".to_owned(),
                )),
            },
            ItemKind::Array(mut values) => {
                let Some(identifier) = values.next()else {
                    return Err(Error::Conversion("Expected path identifier `$` or `@`".to_owned()));
                };

                let ItemKind::Str(identifier) = identifier.kind() else {
                    return Err(Error::Conversion("Expected path identifier `$` or `@`".to_owned()));
                };

                let is_absolute_path = match identifier.as_str() {
                    Some("$") => true,
                    Some("@") => false,
                    _ => {
                        return Err(Error::Conversion(
                            "Expected path identifier `$` or `@`".to_owned(),
                        ));
                    }
                };

                let mut segments = if let Some(len) = values.size() {
                    Vec::with_capacity(len as usize)
                } else {
                    Vec::new()
                };

                for value in values {
                    let segment: SegmentForConversion = value.try_into()?;
                    segments.push(segment.into_segment());
                }

                if is_absolute_path {
                    Ok(FilterPath::Abs(AbsolutePath::new(segments)))
                } else {
                    Ok(FilterPath::Rel(RelativePath::new(segments)))
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse path segments from `{value:?}`"
            ))),
        }
    }
}

struct Segments(Vec<Segment>);

impl TryFrom<(&Cbor, bool)> for Segments {
    type Error = Error;

    fn try_from(value: (&Cbor, bool)) -> Result<Self, Self::Error> {
        let (value, absolute_path) = value;

        match value.kind() {
            ItemKind::Str(identifier) => match identifier.as_str() {
                Some("$") => Ok(Segments(vec![])),
                Some("@") => Ok(Segments(vec![])),
                _ => Err(Error::Conversion(
                    "Expected path identifier `$` or `@`".to_owned(),
                )),
            },
            ItemKind::Array(mut values) => {
                let expected_identifier = if absolute_path { "$" } else { "@" };

                let Some(identifier) = values.next() else {
                    return Err(Error::Conversion(format!("Expected path identifier `{expected_identifier}`")));
                };

                let ItemKind::Str(identifier) = identifier.kind() else {
                    return Err(Error::Conversion(format!("Expected path identifier `{expected_identifier}`")));
                };

                let Some(identifier) = identifier.as_str() else {
                    return Err(Error::Conversion(format!("Expected path identifier `{expected_identifier}`")));
                };

                if identifier != expected_identifier {
                    return Err(Error::Conversion(format!(
                        "Expected path identifier `{expected_identifier}`"
                    )));
                }

                let mut segments = if let Some(len) = values.size() {
                    Vec::with_capacity(len as usize)
                } else {
                    Vec::new()
                };

                for value in values {
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

impl TryFrom<&Cbor> for SegmentForConversion {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        match value.kind() {
            ItemKind::Pos(_)
            | ItemKind::Neg(_)
            | ItemKind::Float(_)
            | ItemKind::Str(_)
            | ItemKind::Bytes(_)
            | ItemKind::Bool(_)
            | ItemKind::Simple(_)
            | ItemKind::Null => Ok(SegmentForConversion::Selector(Selector::Key(
                KeySelector::new(value.to_owned()),
            ))),
            ItemKind::Array(a) => {
                let selectors = a
                    .map(|v| match v.try_into()? {
                        SegmentForConversion::Selector(selector) => Ok(selector),
                        _ => Err(Error::Conversion("Expected a single selector".to_owned())),
                    })
                    .collect::<Result<Vec<Selector>, Error>>()?;
                Ok(SegmentForConversion::Selectors(selectors))
            }
            ItemKind::Dict(mut d) => {
                let (Some((identifier, value)), None) = (d.next(), d.next()) else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                let ItemKind::Str(identifier) = identifier.kind() else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                match identifier.as_str() {
                    Some("..") => match value.try_into()? {
                        SegmentForConversion::Selectors(selectors) => {
                            Ok(SegmentForConversion::Descendant(selectors))
                        }
                        SegmentForConversion::Selector(selector) => {
                            Ok(SegmentForConversion::Descendant(vec![selector]))
                        }
                        _ => Err(Error::Conversion(
                            "Expected selector or array of selectors in a descendant segment"
                                .to_owned(),
                        )),
                    },
                    Some("*") => match value.kind() {
                        ItemKind::Pos(i) if i == 1 => {
                            Ok(SegmentForConversion::Selector(Selector::Wildcard))
                        }
                        _ => Err(Error::Conversion("Cannot parse wildcard".to_owned())),
                    },
                    Some("#") => Ok(SegmentForConversion::Selector(Selector::Index(
                        value.try_into()?,
                    ))),
                    Some(":") => Ok(SegmentForConversion::Selector(Selector::Slice(
                        value.try_into()?,
                    ))),
                    Some("?") => Ok(SegmentForConversion::Selector(Selector::Filter(
                        FilterSelector::new(value.try_into()?),
                    ))),
                    _ => Err(Error::Conversion(
                        "Expected identifier `..`, `#`, `:` or `?`".to_owned(),
                    )),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse segments from `{value}`"
            ))),
        }
    }
}

impl TryFrom<&Cbor> for IndexSelector {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        match value.kind() {
            ItemKind::Pos(index) => Ok(IndexSelector::new(index as isize)),
            ItemKind::Neg(index) => Ok(IndexSelector::new(-1 - (index as isize))),
            _ => Err(Error::Conversion("Expected integer".to_owned())),
        }
    }
}

impl TryFrom<&Cbor> for SliceSelector {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        if let ItemKind::Array(mut a) = value.kind() {
            match (a.next(), a.next(), a.next(), a.next()) {
                (Some(start), Some(end), Some(step), None) => {
                    let start = match start.kind() {
                        ItemKind::Pos(index) => index as isize,
                        ItemKind::Neg(index) => -1 - (index as isize),
                        _ => return Err(Error::Conversion("Expected integer".to_owned())),
                    };

                    let end = match end.kind() {
                        ItemKind::Pos(index) => index as isize,
                        ItemKind::Neg(index) => -1 - (index as isize),
                        _ => return Err(Error::Conversion("Expected integer".to_owned())),
                    };

                    let step = match step.kind() {
                        ItemKind::Pos(index) => index as isize,
                        ItemKind::Neg(index) => -1 - (index as isize),
                        _ => return Err(Error::Conversion("Expected integer".to_owned())),
                    };

                    Ok(SliceSelector::new(start, end, step))
                }
                _ => Err(Error::Conversion("Expected 3-elements array".to_owned())),
            }
        } else {
            Err(Error::Conversion("Expected array".to_owned()))
        }
    }
}

impl TryFrom<&Cbor> for BooleanExpr {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        match value.kind() {
            ItemKind::Array(_) => Ok(BooleanExpr::Path(value.try_into()?)),
            ItemKind::Dict(mut d) => {
                let (Some((identifier, value)), None) = (d.next(), d.next()) else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                let ItemKind::Str(identifier) = identifier.kind() else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                let Some(identifier) = identifier.as_str() else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                match identifier {
                    "&&" => {
                        if let ItemKind::Array(mut a) = value.kind() {
                            match (a.next(), a.next(), a.next()) {
                                (Some(left), Some(right), None) => Ok(BooleanExpr::And(
                                    Box::new(left.try_into()?),
                                    Box::new(right.try_into()?),
                                )),
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
                        if let ItemKind::Array(mut a) = value.kind() {
                            match (a.next(), a.next(), a.next()) {
                                (Some(left), Some(right), None) => Ok(BooleanExpr::Or(
                                    Box::new(left.try_into()?),
                                    Box::new(right.try_into()?),
                                )),
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
                    "!" => Ok(BooleanExpr::Not(Box::new(value.try_into()?))),
                    "<" | "<=" | "==" | "!=" | ">=" | ">" => {
                        Ok(BooleanExpr::Comparison((identifier, value).try_into()?))
                    }
                    "match" => {
                        if let ItemKind::Array(mut a) = value.kind() {
                            match (a.next(), a.next(), a.next()) {
                                (Some(comparable), Some(regex), None) => {
                                    let ItemKind::Str(regex) = regex.kind() else {
                                        return Err(Error::Conversion(format!(
                                            "Cannot parse match function from `{value:?}`"
                                        )));
                                    };
                                    let Some(regex) = regex.as_str() else {
                                        return Err(Error::Conversion(format!(
                                            "Cannot parse match function from `{value:?}`"
                                        )));
                                    };

                                    Ok(builder::_match(
                                        TryInto::<Comparable>::try_into(comparable)?,
                                        regex,
                                    )?
                                    .build())
                                }
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
                        if let ItemKind::Array(mut a) = value.kind() {
                            match (a.next(), a.next(), a.next()) {
                                (Some(comparable), Some(regex), None) => {
                                    let ItemKind::Str(regex) = regex.kind() else {
                                        return Err(Error::Conversion(format!(
                                            "Cannot parse search function from `{value:?}`"
                                        )));
                                    };
                                    let Some(regex) = regex.as_str() else {
                                        return Err(Error::Conversion(format!(
                                            "Cannot parse search function from `{value:?}`"
                                        )));
                                    };

                                    Ok(builder::search(
                                        TryInto::<Comparable>::try_into(comparable)?,
                                        regex,
                                    )?
                                    .build())
                                }
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

impl TryFrom<(&str, &Cbor)> for ComparisonExpr {
    type Error = Error;

    fn try_from(value: (&str, &Cbor)) -> Result<Self, Self::Error> {
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

        if let ItemKind::Array(mut a) = value.kind() {
            match (a.next(), a.next(), a.next()) {
                (Some(left), Some(right), None) => Ok(ComparisonExpr::new(
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

impl TryFrom<&Cbor> for Comparable {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        match value.kind() {
            ItemKind::Pos(_)
            | ItemKind::Neg(_)
            | ItemKind::Float(_)
            | ItemKind::Str(_)
            | ItemKind::Bytes(_)
            | ItemKind::Bool(_)
            | ItemKind::Simple(_)
            | ItemKind::Null => Ok(Comparable::Value(value.to_owned())),
            ItemKind::Array(a) => Ok(Comparable::SingularPath(a.try_into()?)),
            ItemKind::Dict(mut d) => {
                let (Some((identifier, value)), None) = (d.next(), d.next()) else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                let ItemKind::Str(identifier) = identifier.kind() else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                match identifier.as_str() {
                    Some("length") => Ok(Comparable::Function(Function::Length(Box::new(
                        value.try_into()?,
                    )))),
                    Some("count") => Ok(Comparable::Function(Function::Count(value.try_into()?))),
                    Some("value") => Ok(Comparable::Function(Function::Value(value.try_into()?))),
                    _ => Err(Error::Conversion(
                        "Expected `length`, `count` or `value` function".to_owned(),
                    )),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse comparable from `{value:?}`"
            ))),
        }
    }
}

impl<'a> TryFrom<ArrayIter<'a>> for SingularPath {
    type Error = Error;

    fn try_from(mut values: ArrayIter<'a>) -> Result<Self, Self::Error> {
        let Some(identifier) = values.next() else {
            return Err(Error::Conversion("Expected singular path identifier `$` or `@`".to_owned()));
        };

        let ItemKind::Str(identifier) = identifier.kind() else {
            return Err(Error::Conversion("Expected singular path identifier `$` or `@`".to_owned()));
        };

        let is_absolute_path = match identifier.as_str() {
            Some("$") => true,
            Some("@") => false,
            _ => {
                return Err(Error::Conversion(
                    "Expected singular path identifier `$` or `@`".to_owned(),
                ));
            }
        };

        let mut segments = if let Some(len) = values.size() {
            Vec::with_capacity(len as usize)
        } else {
            Vec::new()
        };

        for value in values {
            let segment = value.try_into()?;
            segments.push(segment);
        }

        if is_absolute_path {
            Ok(SingularPath::Abs(segments))
        } else {
            Ok(SingularPath::Rel(segments))
        }
    }
}

impl TryFrom<&Cbor> for SingularSegment {
    type Error = Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        match value.kind() {
            ItemKind::Pos(_)
            | ItemKind::Neg(_)
            | ItemKind::Float(_)
            | ItemKind::Str(_)
            | ItemKind::Bytes(_)
            | ItemKind::Bool(_)
            | ItemKind::Simple(_)
            | ItemKind::Null => Ok(SingularSegment::Key(KeySelector::new(value.to_owned()))),
            ItemKind::Dict(mut d) => {
                let (Some((identifier, value)), None) = (d.next(), d.next()) else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                let ItemKind::Str(identifier) = identifier.kind() else {
                    return Err(Error::Conversion("Expected a single element map".to_owned()));
                };

                match identifier.as_str() {
                    Some("#") => Ok(SingularSegment::Index(value.try_into()?)),
                    _ => Err(Error::Conversion("Expected identifier `#`".to_owned())),
                }
            }
            _ => Err(Error::Conversion(format!(
                "Cannot parse singular segment from `{value:?}`"
            ))),
        }
    }
}
