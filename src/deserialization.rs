use crate::{
    AbsolutePath, BooleanExpr, Comparable, ComparisonOperator, FilterSelector, Function,
    IndexSelector, Path, RelativePath, Segment, Selector, SingularPath, SingularSegment,
    SliceSelector,
};
use serde::{
    de::{self, value::SeqAccessDeserializer},
    Deserialize, Deserializer,
};
use std::fmt;

const ROOT_NODE_IDENTIFIER: &str = "$";
const CURRENT_NODE_IDENTIFIER: &str = "@";
const DESCENDANT_SEGMENT_IDENTIFIER: &str = "..";
const INDEX_IDENTIFIER: &str = "#";
const SLICE_IDENTIFIER: &str = ":";
const FILTER_IDENTIFIER: &str = "?";
const WILDCARD_IDENTIFIER: &str = "*";

impl<'de> Deserialize<'de> for AbsolutePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(AbsolutePath::new(deserialize_path_segments(
            deserializer,
            true,
        )?))
    }
}

impl<'de> Deserialize<'de> for RelativePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(RelativePath::new(deserialize_path_segments(
            deserializer,
            false,
        )?))
    }
}

fn deserialize_path_segments<'de, D: Deserializer<'de>>(
    deserializer: D,
    absolute_path: bool,
) -> Result<Vec<Segment>, D::Error> {
    struct Visitor {
        absolute_path: bool,
    }

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Vec<Segment>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Vec<Segment>")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                ROOT_NODE_IDENTIFIER | CURRENT_NODE_IDENTIFIER => Ok(vec![]),
                _ => Err(de::Error::invalid_value(
                    de::Unexpected::Str(v),
                    &"`$`or `@`",
                )),
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let Some(identifier) = seq.next_element::<String>()? else {
                return Err(de::Error::invalid_length(0, &"more elements in sequence"));
            };

            let expected_identifier = if self.absolute_path {
                ROOT_NODE_IDENTIFIER
            } else {
                CURRENT_NODE_IDENTIFIER
            };

            if identifier != expected_identifier {
                return Err(de::Error::invalid_value(
                    de::Unexpected::Str(&identifier),
                    &expected_identifier,
                ));
            }

            let mut segments = match seq.size_hint() {
                Some(size) if size > 1 => Vec::with_capacity(size),
                _ => Vec::new(),
            };

            while let Some(segment) = seq.next_element::<SegmentForDeserialization>()? {
                segments.push(segment.into_segment())
            }

            Ok(segments)
        }
    }

    deserializer.deserialize_any(Visitor { absolute_path })
}

enum SegmentForDeserialization {
    Selector(Selector),
    Selectors(Vec<Selector>),
    Descendant(Vec<Selector>),
}

impl SegmentForDeserialization {
    pub fn into_segment(self) -> Segment {
        match self {
            SegmentForDeserialization::Selector(selector) => Segment::Child(vec![selector]),
            SegmentForDeserialization::Selectors(selectors) => Segment::Child(selectors),
            SegmentForDeserialization::Descendant(selectors) => Segment::Descendant(selectors),
        }
    }
}

impl<'de> Deserialize<'de> for SegmentForDeserialization {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SegmentForDeserialization;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SegmentForDeserialization")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SegmentForDeserialization::Selector(Selector::key(v.into())))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SegmentForDeserialization::Selector(Selector::key(v.into())))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SegmentForDeserialization::Selector(Selector::key(v.into())))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SegmentForDeserialization::Selector(Selector::key(v.into())))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SegmentForDeserialization::Selector(Selector::key(v.into())))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SegmentForDeserialization::Selector(Selector::key(v.into())))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let Some(identifier) = map.next_key::<String>()? else {
                    return Err(de::Error::invalid_length(0, &"1 element in map"));
                };

                match identifier.as_str() {
                    DESCENDANT_SEGMENT_IDENTIFIER => {
                        let descendant: SegmentForDeserialization = map.next_value()?;
                        match descendant {
                            SegmentForDeserialization::Selectors(selectors) => {
                                Ok(SegmentForDeserialization::Descendant(selectors))
                            }
                            _ => Err(de::Error::custom(
                                "Expected selector or array of selectors in a descendant segment",
                            )),
                        }
                    },
                    WILDCARD_IDENTIFIER => match map.next_value::<i32>() {
                        Ok(i) if i == 1 => Ok(SegmentForDeserialization::Selector(Selector::wildcard())),
                        _ => Err(de::Error::custom("Expected value `1`")),
                    }
                    INDEX_IDENTIFIER => Ok(SegmentForDeserialization::Selector(Selector::Index(
                        map.next_value::<IndexSelector>()?,
                    ))),
                    SLICE_IDENTIFIER => Ok(SegmentForDeserialization::Selector(Selector::Slice(
                        map.next_value::<SliceSelector>()?,
                    ))),
                    FILTER_IDENTIFIER => Ok(SegmentForDeserialization::Selector(Selector::Filter(
                        map.next_value::<FilterSelector>()?,
                    ))),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Str(&identifier),
                        &format!(
                            "`{DESCENDANT_SEGMENT_IDENTIFIER}`, `{INDEX_IDENTIFIER}`, `{SLICE_IDENTIFIER}`, or `{FILTER_IDENTIFIER}`"
                        )
                        .as_str(),
                    )),
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut selectors = match seq.size_hint() {
                    Some(size) => Vec::with_capacity(size),
                    _ => Vec::new(),
                };

                while let Some(selector) = seq.next_element()? {
                    match selector {
                        SegmentForDeserialization::Selector(selector) => selectors.push(selector),
                        _ => {
                            return Err(de::Error::invalid_value(
                                de::Unexpected::Seq,
                                &"a single selector",
                            ))
                        }
                    }
                }

                Ok(SegmentForDeserialization::Selectors(selectors))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl<'de> Deserialize<'de> for BooleanExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = BooleanExpr;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("BooleanExpr")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let Some(identifier) = map.next_key::<String>()? else {
                    return Err(de::Error::invalid_length(0, &"1 element in map"));
                };

                match identifier.as_str() {
                    "&&" => {
                        let (right, left) = map.next_value::<(BooleanExpr, BooleanExpr)>()?;
                        Ok(BooleanExpr::and(right, left))
                    }
                    "||" => {
                        let (right, left) = map.next_value::<(BooleanExpr, BooleanExpr)>()?;
                        Ok(BooleanExpr::or(right, left))
                    }
                    "!" => {
                        let expr = map.next_value::<BooleanExpr>()?;
                        Ok(BooleanExpr::not(expr))
                    }
                    "<" => {
                        let (right, left) = map.next_value::<(Comparable, Comparable)>()?;
                        Ok(BooleanExpr::comparison(right, ComparisonOperator::Lt, left))
                    }
                    "<=" => {
                        let (right, left) = map.next_value::<(Comparable, Comparable)>()?;
                        Ok(BooleanExpr::comparison(
                            right,
                            ComparisonOperator::Lte,
                            left,
                        ))
                    }
                    "==" => {
                        let (right, left) = map.next_value::<(Comparable, Comparable)>()?;
                        Ok(BooleanExpr::comparison(right, ComparisonOperator::Eq, left))
                    }
                    "!=" => {
                        let (right, left) = map.next_value::<(Comparable, Comparable)>()?;
                        Ok(BooleanExpr::comparison(
                            right,
                            ComparisonOperator::Neq,
                            left,
                        ))
                    }
                    ">=" => {
                        let (right, left) = map.next_value::<(Comparable, Comparable)>()?;
                        Ok(BooleanExpr::comparison(
                            right,
                            ComparisonOperator::Gte,
                            left,
                        ))
                    }
                    ">" => {
                        let (right, left) = map.next_value::<(Comparable, Comparable)>()?;
                        Ok(BooleanExpr::comparison(right, ComparisonOperator::Gt, left))
                    }
                    "match" => {
                        let (comparable, regex) = map.next_value::<(Comparable, String)>()?;
                        Ok(BooleanExpr::function(
                            Function::_match(comparable, &regex).map_err(de::Error::custom)?,
                        ))
                    }
                    "search" => {
                        let (comparable, regex) = map.next_value::<(Comparable, String)>()?;
                        Ok(BooleanExpr::function(
                            Function::search(comparable, &regex).map_err(de::Error::custom)?,
                        ))
                    }
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Str(identifier.as_str()),
                        &"logical expression, comparison expression or function",
                    )),
                }
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let path = Path::deserialize(SeqAccessDeserializer::new(seq))?;
                Ok(BooleanExpr::path(path))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Path;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Path")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let Some(identifier) = seq.next_element::<String>()? else {
                    return Err(de::Error::invalid_length(0, &"more elements in sequence"));
                };

                let is_absolute_path = match identifier.as_str() {
                    ROOT_NODE_IDENTIFIER => true,
                    CURRENT_NODE_IDENTIFIER => false,
                    _ => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(identifier.as_str()),
                            &"`$`or `@`",
                        ))
                    }
                };

                let mut segments = match seq.size_hint() {
                    Some(size) if size > 1 => Vec::with_capacity(size),
                    _ => Vec::new(),
                };

                while let Some(segment) = seq.next_element::<SegmentForDeserialization>()? {
                    segments.push(segment.into_segment())
                }

                if is_absolute_path {
                    Ok(Path::abs(segments))
                } else {
                    Ok(Path::rel(segments))
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl<'de> Deserialize<'de> for Comparable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Comparable;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Comparable")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Comparable::Value(v.into()))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Comparable::Value(v.into()))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Comparable::Value(v.into()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Comparable::Value(v.into()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Comparable::Value(v.into()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Comparable::Value(v.into()))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let path = SingularPath::deserialize(SeqAccessDeserializer::new(seq))?;
                Ok(Comparable::SingularPath(path))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let Some(identifier) = map.next_key::<String>()? else {
                    return Err(de::Error::invalid_length(0, &"1 element in map"));
                };

                match identifier.as_str() {
                    "length" => Ok(Comparable::Function(Function::length(map.next_value()?))),
                    "count" => Ok(Comparable::Function(Function::count(map.next_value()?))),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Str(&identifier),
                        &"`length` or `count`",
                    )),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl<'de> Deserialize<'de> for SingularPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SingularPath;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SingularPath")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let Some(identifier) = seq.next_element::<String>()? else {
                    return Err(de::Error::invalid_length(0, &"more elements in sequence"));
                };

                let is_absolute_path = match identifier.as_str() {
                    ROOT_NODE_IDENTIFIER => true,
                    CURRENT_NODE_IDENTIFIER => false,
                    _ => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(identifier.as_str()),
                            &"`$`or `@`",
                        ))
                    }
                };

                let mut segments = match seq.size_hint() {
                    Some(size) if size > 1 => Vec::with_capacity(size),
                    _ => Vec::new(),
                };

                while let Some(segment) = seq.next_element()? {
                    segments.push(segment)
                }

                if is_absolute_path {
                    Ok(SingularPath::abs(segments))
                } else {
                    Ok(SingularPath::rel(segments))
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl<'de> Deserialize<'de> for SingularSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SingularSegment;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SingularSegment")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SingularSegment::key(v.into()))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SingularSegment::key(v.into()))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SingularSegment::key(v.into()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SingularSegment::key(v.into()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SingularSegment::key(v.into()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SingularSegment::key(v.into()))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let Some(identifier) = map.next_key::<String>()? else {
                    return Err(de::Error::invalid_length(0, &"1 element in map"));
                };

                match identifier.as_str() {
                    INDEX_IDENTIFIER => Ok(SingularSegment::index(map.next_value()?)),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Str(&identifier),
                        &format!("`{INDEX_IDENTIFIER}`").as_str(),
                    )),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
