use crate::{parsing::parse_cbor_path, Error};
use ciborium::{de::from_reader, value::Value, ser::into_writer};
use regex::Regex;
use serde::Deserialize;
use std::{borrow::Cow, cmp::Ordering, fmt, iter::once, vec};

#[derive(Debug, PartialEq, Deserialize)]
pub struct CborPath(AbsolutePath);

impl CborPath {
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn new(segments: Vec<Segment>) -> Self {
        Self(AbsolutePath(segments))
    }

    #[inline]
    pub fn from_reader<R: ciborium_io::Read>(reader: R) -> Result<Self, Error>
    where
        R::Error: fmt::Debug,
    {
        from_reader(reader).map_err(|e| Error::Serialization(e.to_string()))
    }

    #[inline]
    pub fn from_value(value: &Value) -> Result<Self, Error> {
        parse_cbor_path(value)
    }

    #[inline]
    pub fn evaluate<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        self.0.evaluate(value)
    }

    #[inline]
    pub fn evaluate_from_reader<R: ciborium_io::Read>(&self, reader: R) -> Result<Vec<u8>, Error>
    where
        R::Error: fmt::Debug,
    {
        let value: Value = from_reader(reader).map_err(|e| Error::Serialization(e.to_string()))?;
        let result = self.evaluate(&value);
        let mut buf = Vec::new();
        into_writer(&result, &mut buf).map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(buf)

    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AbsolutePath(Vec<Segment>);

impl AbsolutePath {
    pub(crate) fn new(segments: Vec<Segment>) -> Self {
        Self(segments)
    }

    pub fn evaluate<'a>(&self, root: &'a Value) -> Vec<&'a Value> {
        let mut current_values: Vec<&'a Value>;
        let mut iter = self.0.iter();

        if let Some(first) = iter.next() {
            current_values = first.evaluate(root, once(root));
        } else {
            return vec![root];
        }

        for segment in iter {
            current_values = segment.evaluate(root, current_values.into_iter());
        }

        current_values
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct RelativePath(Vec<Segment>);

impl RelativePath {
    pub(crate) fn new(segments: Vec<Segment>) -> Self {
        Self(segments)
    }

    pub fn evaluate<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        let mut current_values: Vec<&'a Value>;
        let mut iter = self.0.iter();

        if let Some(first) = iter.next() {
            current_values = first.evaluate(root, once(current));
        } else {
            return vec![current];
        }

        for segment in iter {
            current_values = segment.evaluate(root, current_values.into_iter());
        }

        current_values
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Path {
    /// Absolute path (begining by '$')
    Abs(AbsolutePath),
    /// Relative path (begining by '@')
    Rel(RelativePath),
}

impl Path {
    pub fn abs(segments: Vec<Segment>) -> Self {
        Self::Abs(AbsolutePath(segments))
    }

    pub fn rel(segments: Vec<Segment>) -> Self {
        Self::Rel(RelativePath(segments))
    }

    pub fn evaluate<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match self {
            Path::Abs(path) => path.evaluate(root),
            Path::Rel(path) => path.evaluate(root, current),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Segment {
    Child(Vec<Selector>),
    Descendant(Vec<Selector>),
}
impl Segment {
    fn evaluate<'a, I>(&self, root: &'a Value, current_values: I) -> Vec<&'a Value>
    where
        I: Iterator<Item = &'a Value> + Clone,
    {
        match self {
            Segment::Child(selectors) => current_values
                .flat_map(|current| selectors.iter().flat_map(|s| s.evaluate(root, current)))
                .collect(),

            Segment::Descendant(selectors) => {
                let mut descendants = Vec::new();
                for value in current_values {
                    descendants.push(value);
                    Self::fetch_descendants(&mut descendants, value);
                }

                descendants
                    .into_iter()
                    .flat_map(|current| selectors.iter().flat_map(|s| s.evaluate(root, current)))
                    .collect()
            }
        }
    }

    fn fetch_descendants<'a>(descendants: &mut Vec<&'a Value>, value: &'a Value) {
        match value {
            Value::Array(a) => {
                descendants.extend(a);
                for value in a {
                    Self::fetch_descendants(descendants, value);
                }
            }
            Value::Map(m) => {
                descendants.extend(m.iter().map(|e| &e.1));
                for value in m.iter().map(|e| &e.1) {
                    Self::fetch_descendants(descendants, value);
                }
            }
            _ => (),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Selector {
    /// Value
    Key(KeySelector),
    /// "*"
    Wildcard,
    /// {"#", idx}
    Index(IndexSelector),
    /// {":", [start, end, step]}
    Slice(SliceSelector),
    /// {"?", []}
    Filter(FilterSelector),
}

impl Selector {
    pub fn key(value: Value) -> Self {
        Self::Key(KeySelector(value))
    }

    pub fn wildcard() -> Self {
        Self::Wildcard
    }

    #[allow(dead_code)]
    pub fn index(index: isize) -> Self {
        Self::Index(IndexSelector(index))
    }

    #[allow(dead_code)]
    pub fn slice(start: isize, end: isize, step: isize) -> Self {
        Self::Slice(SliceSelector(start, end, step))
    }

    #[allow(dead_code)]
    pub fn filter(boolean_expr: BooleanExpr) -> Self {
        Self::Filter(FilterSelector(boolean_expr))
    }

    fn evaluate<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match self {
            Selector::Key(selector) => selector.evaluate(current),
            Selector::Wildcard => WildcardSelector.evaluate(current),
            Selector::Index(selector) => selector.evaluate(current),
            Selector::Slice(selector) => selector.evaluate(current),
            Selector::Filter(filter) => filter.evaluate(root, current),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct KeySelector(Value);

impl KeySelector {
    fn evaluate<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        self.evaluate_single(value)
            .map(|v| vec![v])
            .unwrap_or_else(Vec::new)
    }

    #[inline]
    fn evaluate_single<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        let Self(key) = &self;
        match value {
            Value::Map(map) => map
                .iter()
                .find_map(|(k, v)| if k == key { Some(v) } else { None }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct WildcardSelector;

impl WildcardSelector {
    fn evaluate<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        match value {
            Value::Map(map) => map.iter().map(|(_, v)| v).collect(),
            Value::Array(array) => array.iter().collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub(crate) struct IndexSelector(isize);

impl IndexSelector {
    fn evaluate<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        self.evaluate_single(value)
            .map(|v| vec![v])
            .unwrap_or_else(Vec::new)
    }

    #[inline]
    fn evaluate_single<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        let Self(index) = &self;
        match value {
            Value::Array(array) => {
                let index = normalize_index(*index, array.len()) as usize;
                array.get(index as usize)
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub(crate) struct SliceSelector(isize, isize, isize);

impl SliceSelector {
    fn evaluate<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        let SliceSelector(start, end, step) = &self;
        match value {
            Value::Array(array) => {
                let start = normalize_index(*start, array.len());
                let end = normalize_index(*end, array.len());
                let step = *step;

                if step > 0 {
                    let start = usize::min(start as usize, array.len());
                    let end = usize::min(end as usize, array.len());
                    array
                        .iter()
                        .skip(start)
                        .take(end - start)
                        .step_by(step as usize)
                        .collect()
                } else {
                    let start = array.len() - 1 - usize::min(start as usize, array.len());
                    let end = (array.len() as isize
                        - 1
                        - isize::min(isize::max(end, -1), array.len() as isize))
                        as usize;
                    array
                        .iter()
                        .rev()
                        .skip(start)
                        .take(end - start)
                        .step_by(-step as usize)
                        .collect()
                }
            }
            _ => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub(crate) struct FilterSelector(BooleanExpr);

impl FilterSelector {
    #[inline]
    fn evaluate<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        let Self(boolean_expr) = &self;
        match current {
            Value::Array(a) => a
                .iter()
                .filter(|v| boolean_expr.evaluate(root, v))
                .collect(),
            Value::Map(m) => m
                .iter()
                .filter_map(|(_, v)| {
                    if boolean_expr.evaluate(root, v) {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum BooleanExpr {
    Or(Box<BooleanExpr>, Box<BooleanExpr>),
    And(Box<BooleanExpr>, Box<BooleanExpr>),
    Not(Box<BooleanExpr>),
    Comparison(ComparisonExpr),
    /// path existence or non-existence
    Path(Path),
    Function(Function),
}

impl BooleanExpr {
    pub fn or(left: BooleanExpr, right: BooleanExpr) -> Self {
        Self::Or(Box::new(left), Box::new(right))
    }

    pub fn and(left: BooleanExpr, right: BooleanExpr) -> Self {
        Self::And(Box::new(left), Box::new(right))
    }

    pub fn not(expr: BooleanExpr) -> Self {
        Self::Not(Box::new(expr))
    }

    pub fn comparison(left: Comparable, operator: ComparisonOperator, right: Comparable) -> Self {
        Self::Comparison(ComparisonExpr(left, operator, right))
    }

    pub fn path(path: Path) -> Self {
        Self::Path(path)
    }

    pub fn function(function: Function) -> Self {
        Self::Function(function)
    }

    pub fn evaluate(&self, root: &Value, current: &Value) -> bool {
        match self {
            BooleanExpr::Or(l, r) => l.evaluate(root, current) || r.evaluate(root, current),
            BooleanExpr::And(l, r) => l.evaluate(root, current) && r.evaluate(root, current),
            BooleanExpr::Not(e) => !e.evaluate(root, current),
            BooleanExpr::Comparison(c) => c.evaluate(root, current),
            BooleanExpr::Path(p) => !p.evaluate(root, current).is_empty(),
            BooleanExpr::Function(f) => f.evaluate_as_boolean_expr(root, current),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ComparisonExpr(pub Comparable, pub ComparisonOperator, pub Comparable);
impl ComparisonExpr {
    pub fn evaluate(&self, root: &Value, current: &Value) -> bool {
        let ComparisonExpr(left, op, right) = &self;
        match op {
            ComparisonOperator::Eq => left.equals(right, root, current),
            ComparisonOperator::Neq => !left.equals(right, root, current),
            ComparisonOperator::Gt => right.lesser_than(left, root, current),
            ComparisonOperator::Gte => {
                right.lesser_than(left, root, current) || left.equals(right, root, current)
            }
            ComparisonOperator::Lt => left.lesser_than(right, root, current),
            ComparisonOperator::Lte => {
                left.lesser_than(right, root, current) || left.equals(right, root, current)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Comparable {
    Value(Value),
    SingularPath(SingularPath),
    Function(Function),
}

/// cf. https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html#name-filter-selector
impl Comparable {
    fn equals(&self, other: &Self, root: &Value, current: &Value) -> bool {
        let v1 = self.evaluate(root, current);
        let v2 = other.evaluate(root, current);

        match (&v1, &v2) {
            (None, None) => true,
            (Some(v1), Some(v2)) => value_equals(v1, v2),
            _ => false,
        }
    }

    fn lesser_than(&self, other: &Self, root: &Value, current: &Value) -> bool {
        let v1 = self.evaluate(root, current);
        let v2 = other.evaluate(root, current);

        let v1 = v1.as_ref().map(|v| v.as_ref());
        let v2 = v2.as_ref().map(|v| v.as_ref());

        match (&v1, &v2) {
            (Some(Value::Integer(i1)), Some(Value::Integer(i2))) => i1.cmp(i2) == Ordering::Less,
            (Some(Value::Float(f1)), Some(Value::Float(f2))) => f1 < f2,
            (Some(Value::Bytes(b1)), Some(Value::Bytes(b2))) => b1 < b2,
            (Some(Value::Text(t1)), Some(Value::Text(t2))) => t1 < t2,
            _ => false,
        }
    }

    fn evaluate<'a>(&'a self, root: &'a Value, current: &'a Value) -> Option<Cow<'a, Value>> {
        match self {
            Comparable::Value(value) => Some(Cow::Borrowed(value)),
            Comparable::SingularPath(path) => path.evaluate(root, current),
            Comparable::Function(function) => function
                .evaluate_as_comparable(root, current)
                .map(Cow::Owned),
        }
    }
}

fn value_equals(v1: &Value, v2: &Value) -> bool {
    match (v1, v2) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
        (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
        (Value::Float(f1), Value::Float(f2)) => f1 == f2,
        (Value::Bytes(b1), Value::Bytes(b2)) => b1.cmp(b2) == Ordering::Equal,
        (Value::Text(t1), Value::Text(t2)) => t1 == t2,
        (Value::Array(a1), Value::Array(a2)) => {
            a1.len() == a2.len() && a1.iter().zip(a2).all(|(v1, v2)| v1 == v2)
        }
        (Value::Map(m1), Value::Map(m2)) => {
            m1.len() == m2.len()
                && m1.iter().all(|(key, v1)| {
                    if let Some(v2) = m2
                        .iter()
                        .find_map(|(k, v)| if k == key { Some(v) } else { None })
                    {
                        value_equals(v1, v2)
                    } else {
                        false
                    }
                })
        }
        (Value::Tag(id1, v1), Value::Tag(id2, v2)) => {
            let cmp = id1.cmp(id2);
            if let Ordering::Equal = cmp {
                value_equals(v1, v2)
            } else {
                false
            }
        }
        _ => false,
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ComparisonOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, PartialEq)]
pub(crate) enum SingularPath {
    /// Absolute singular path (beginning by '$')
    Abs(AbsSingularPath),
    /// Relative singular path (beginning by '@')
    Rel(RelSingularPath),
}

impl SingularPath {
    pub fn abs(segments: Vec<SingularSegment>) -> Self {
        Self::Abs(AbsSingularPath(segments))
    }

    pub fn rel(segments: Vec<SingularSegment>) -> Self {
        Self::Rel(RelSingularPath(segments))
    }

    #[inline]
    pub fn evaluate<'a>(&self, root: &'a Value, current: &'a Value) -> Option<Cow<'a, Value>> {
        match self {
            SingularPath::Abs(path) => path.evaluate(root).map(Cow::Borrowed),
            SingularPath::Rel(path) => path.evaluate(current).map(Cow::Borrowed),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AbsSingularPath(Vec<SingularSegment>);

impl AbsSingularPath {
    fn evaluate<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        let mut current_value = value;
        for segment in &self.0 {
            match segment.evaluate(current_value) {
                Some(value) => current_value = value,
                None => return None,
            }
        }
        Some(current_value)
    }
}
#[derive(Debug, PartialEq)]
pub(crate) struct RelSingularPath(Vec<SingularSegment>);

impl RelSingularPath {
    fn evaluate<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        let mut current_value = value;
        for segment in &self.0 {
            match segment.evaluate(current_value) {
                Some(value) => current_value = value,
                None => return None,
            }
        }
        Some(current_value)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum SingularSegment {
    Key(KeySelector),
    Index(IndexSelector),
}

impl SingularSegment {
    pub fn key(value: Value) -> Self {
        Self::Key(KeySelector(value))
    }

    pub fn index(index: isize) -> Self {
        Self::Index(IndexSelector(index))
    }

    #[inline]
    fn evaluate<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        match self {
            SingularSegment::Key(selector) => selector.evaluate_single(value),
            SingularSegment::Index(selector) => selector.evaluate_single(value),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Function {
    Length(Box<Comparable>),
    Count(Path),
    Regex(Box<Comparable>, Regex),
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(l0), Self::Length(r0)) => l0 == r0,
            (Self::Count(l0), Self::Count(r0)) => l0 == r0,
            (Self::Regex(l0, l1), Self::Regex(r0, r1)) => l0 == r0 && l1.as_str() == r1.as_str(),
            _ => false,
        }
    }
}

impl Function {
    pub fn length(comparable: Comparable) -> Self {
        Self::Length(Box::new(comparable))
    }

    pub fn count(path: Path) -> Self {
        Self::Count(path)
    }

    pub fn _match(comparable: Comparable, regex: &str) -> Result<Self, Error> {
        Ok(Self::Regex(
            Box::new(comparable),
            Regex::new(&format!("^{regex}$"))?,
        ))
    }

    pub fn search(comparable: Comparable, regex: &str) -> Result<Self, Error> {
        Ok(Self::Regex(Box::new(comparable), Regex::new(regex)?))
    }

    fn evaluate_as_boolean_expr(&self, root: &Value, current: &Value) -> bool {
        match self {
            Function::Regex(comparable, regex) => {
                let value = comparable.evaluate(root, current);
                let value = value.as_ref().map(|v| v.as_ref());
                match value {
                    Some(Value::Text(str)) => regex.is_match(str),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn evaluate_as_comparable(&self, root: &Value, current: &Value) -> Option<Value> {
        match self {
            Function::Length(comparable) => {
                let value = comparable.evaluate(root, current);
                let value = value.as_ref().map(|v| v.as_ref());
                match value {
                    Some(Value::Array(a)) => Some(Value::Integer(a.len().into())),
                    Some(Value::Map(m)) => Some(Value::Integer(m.len().into())),
                    Some(Value::Text(t)) => Some(Value::Integer(t.len().into())),
                    Some(Value::Bytes(b)) => Some(Value::Integer(b.len().into())),
                    None => None,
                    _ => Some(Value::Integer(1.into())),
                }
            }
            Function::Count(path) => {
                Some(Value::Integer(path.evaluate(root, current).len().into()))
            }
            _ => None,
        }
    }
}

#[inline]
fn normalize_index(i: isize, len: usize) -> isize {
    if i >= 0 {
        i
    } else {
        len as isize + i
    }
}
