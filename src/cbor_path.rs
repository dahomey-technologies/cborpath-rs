use crate::{
    builder::{self, PathBuilder},
    Error,
};
use cbor_data::{Cbor, CborBuilder, CborOwned, ItemKind, Writer};
use regex::Regex;
use std::{borrow::Cow, vec};

/// Represents a CBORPath expression
///
/// Once constructed, this structure can be used efficiently multiple times
/// to apply the CBOR Path expression on different CBOR documents.
#[derive(Debug, PartialEq)]
pub struct CborPath(AbsolutePath);

impl CborPath {
    #[inline]
    pub(crate) fn new(segments: Vec<Segment>) -> Self {
        Self(AbsolutePath::new(segments))
    }

    /// Initialize a `CborPath` instance from a [`builder`](crate::builder::PathBuilder)
    /// # Return
    /// A new `CborPath` instance
    #[inline]
    pub fn builder() -> PathBuilder {
        builder::abs_path()
    }

    /// Initialize a `CborPath` instance from a `CBOR binary buffer`
    /// # Return
    /// A new `CborPath` instance or an error if the provided buffer is neither a valid `CBOR` buffer nor a valid `CBORPath` expression.
    #[inline]
    pub fn from_bytes(cbor: &[u8]) -> Result<Self, Error>
    {
        let cbor = Cbor::checked(cbor)?;
        cbor.try_into()
    }

    /// Initialize a `CborPath` instance from a [`CBOR value`](https://docs.rs/cbor-data/latest/cbor_data/struct.Cbor.html) reference
    /// # Return
    /// A new `CborPath` instance or an error if the provided buffer is the provided [`CBOR value`] is not a valid `CBORPath` expression.
    #[inline]
    pub fn from_value(cbor: &Cbor) -> Result<Self, Error> {
        cbor.try_into()
    }

    /// Applies the CBORPath expression to the input `CBOR` document
    /// # Return
    /// A binarized `CBOR` document.
    ///
    /// The returned value is always in a the form of a `CBOR Array` which contains all the results `CBOR` nodes
    ///
    /// The evaluation in itself does not raise any error: 
    /// if the CBORPath expression does not match the input value, an empty `CBOR Array` will be returned
    #[inline]
    pub fn evaluate<'a>(&self, cbor: &'a Cbor) -> Vec<&'a Cbor> {
        self.0.evaluate(cbor)
    }

    /// Applies the CBORPath expression to the input `CBOR` document
    /// # Return
    /// A binarized `CBOR` document.
    ///
    /// The returned value is always in a the form of a `CBOR Array` which contains all the results `CBOR` nodes
    ///
    /// The evaluation in itself does not raise any error:
    /// if the CBORPath expression does not match the input value, an empty `CBOR Array` will be returned
    ///
    /// Errors can only occur if the input buffer is not a valid `CBOR` document.
    #[inline]
    pub fn evaluate_from_bytes(&self, cbor: &[u8]) -> Result<Vec<u8>, Error> {
        let cbor = Cbor::checked(cbor)?;
        let result = self.evaluate(cbor);
        let ouput = CborBuilder::new().write_array(None, |builder| {
            for cbor in result {
                builder.write_item(cbor);
            }
        });

        Ok(ouput.into_vec())
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AbsolutePath(Vec<Segment>);

impl AbsolutePath {
    #[inline]
    pub(crate) fn new(segments: Vec<Segment>) -> Self {
        Self(segments)
    }

    pub fn evaluate<'a>(&self, root: &'a Cbor) -> Vec<&'a Cbor> {
        let mut current_values: Vec<&'a Cbor>;
        let mut iter = self.0.iter();

        if let Some(first) = iter.next() {
            current_values = first.evaluate(root, &[root]);
        } else {
            return vec![root];
        }

        for segment in iter {
            current_values = segment.evaluate(root, &current_values);
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

    pub fn evaluate<'a>(&self, root: &'a Cbor, current: &'a Cbor) -> Vec<&'a Cbor> {
        let mut current_values: Vec<&'a Cbor>;
        let mut iter = self.0.iter();

        if let Some(first) = iter.next() {
            current_values = first.evaluate(root, &[current]);
        } else {
            return vec![current];
        }

        for segment in iter {
            current_values = segment.evaluate(root, &current_values);
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
    #[inline]
    pub fn evaluate<'a>(&self, root: &'a Cbor, current: &'a Cbor) -> Vec<&'a Cbor> {
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
    fn evaluate<'a>(&self, root: &'a Cbor, current_values: &[&'a Cbor]) -> Vec<&'a Cbor> {
        match self {
            Segment::Child(selectors) => current_values
                .iter()
                .flat_map(|current| selectors.iter().flat_map(|s| s.evaluate(root, current)))
                .collect(),

            Segment::Descendant(selectors) => {
                let mut descendants = Vec::new();
                for value in current_values.iter() {
                    descendants.push(*value);
                    Self::fetch_descendants(&mut descendants, value);
                }

                descendants
                    .into_iter()
                    .flat_map(|current| selectors.iter().flat_map(|s| s.evaluate(root, current)))
                    .collect()
            }
        }
    }

    fn fetch_descendants<'a>(descendants: &mut Vec<&'a Cbor>, value: &'a Cbor) {
        match value.kind() {
            ItemKind::Array(a) => {
                descendants.extend(a);
                for value in a {
                    Self::fetch_descendants(descendants, value);
                }
            }
            ItemKind::Dict(d) => {
                descendants.extend(d.map(|(_k, v)| v));
                for value in d.map(|(_k, v)| v) {
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
    fn evaluate<'a>(&self, root: &'a Cbor, current: &'a Cbor) -> Vec<&'a Cbor> {
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
pub(crate) struct KeySelector(CborOwned);

impl KeySelector {
    #[inline]
    pub fn new(value: CborOwned) -> Self {
        Self(value)
    }

    #[inline]
    fn evaluate<'a>(&self, value: &'a Cbor) -> Vec<&'a Cbor> {
        self.evaluate_single(value)
            .map(|v| vec![v])
            .unwrap_or_else(Vec::new)
    }

    #[inline]
    fn evaluate_single<'a>(&self, value: &'a Cbor) -> Option<&'a Cbor> {
        let Self(key) = &self;
        match value.kind() {
            ItemKind::Dict(mut d) => d.find_map(|(k, v)| {
                if value_equals(k, key) {
                    Some(v)
                } else {
                    None
                }
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct WildcardSelector;

impl WildcardSelector {
    #[inline]
    fn evaluate<'a>(&self, value: &'a Cbor) -> Vec<&'a Cbor> {
        match value.kind() {
            ItemKind::Dict(d) => d.map(|(_, v)| v).collect(),
            ItemKind::Array(array) => array.collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct IndexSelector(isize);

impl IndexSelector {
    #[inline]
    pub fn new(index: isize) -> Self {
        Self(index)
    }

    #[inline]
    fn evaluate<'a>(&self, value: &'a Cbor) -> Vec<&'a Cbor> {
        self.evaluate_single(value)
            .map(|v| vec![v])
            .unwrap_or_else(Vec::new)
    }

    #[inline]
    fn evaluate_single<'a>(&self, value: &'a Cbor) -> Option<&'a Cbor> {
        let Self(index) = &self;
        match value.kind() {
            ItemKind::Array(mut array) => {
                let len = array.size().unwrap_or(array.count() as u64) as usize;
                let index = normalize_index(*index, len) as usize;
                array.nth(index as usize)
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct SliceSelector(isize, isize, isize);

impl SliceSelector {
    #[inline]
    pub fn new(start: isize, end: isize, step: isize) -> Self {
        Self(start, end, step)
    }

    fn evaluate<'a>(&self, value: &'a Cbor) -> Vec<&'a Cbor> {
        let SliceSelector(start, end, step) = &self;
        match value.kind() {
            ItemKind::Array(array) => {
                let len = array.size().unwrap_or(array.count() as u64) as usize;
                let start = normalize_index(*start, len);
                let end = normalize_index(*end, len);

                if *step > 0 {
                    let start = usize::min(start as usize, len);
                    let end = usize::min(end as usize, len);
                    array
                        .skip(start)
                        .take(end - start)
                        .step_by(*step as usize)
                        .collect()
                } else {
                    let start = len - 1 - usize::min(start as usize, len);
                    let end =
                        (len as isize - 1 - isize::min(isize::max(end, -1), len as isize)) as usize;
                    array
                        .collect::<Vec<_>>()
                        .into_iter()
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

#[derive(Debug, PartialEq)]
pub(crate) struct FilterSelector(BooleanExpr);

impl FilterSelector {
    #[inline]
    pub fn new(boolean_expr: BooleanExpr) -> Self {
        Self(boolean_expr)
    }

    #[inline]
    fn evaluate<'a>(&self, root: &'a Cbor, current: &'a Cbor) -> Vec<&'a Cbor> {
        let Self(boolean_expr) = &self;
        match current.kind() {
            ItemKind::Array(a) => a.filter(|v| boolean_expr.evaluate(root, v)).collect(),
            ItemKind::Dict(d) => d
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
    #[inline]
    pub fn evaluate(&self, root: &Cbor, current: &Cbor) -> bool {
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
pub(crate) struct ComparisonExpr(Comparable, ComparisonOperator, Comparable);

impl ComparisonExpr {
    #[inline]
    pub fn new(left: Comparable, operator: ComparisonOperator, right: Comparable) -> Self {
        Self(left, operator, right)
    }

    pub fn evaluate(&self, root: &Cbor, current: &Cbor) -> bool {
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
    Value(CborOwned),
    SingularPath(SingularPath),
    Function(Function),
}

/// cf. https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html#name-filter-selector
impl Comparable {
    fn equals(&self, other: &Self, root: &Cbor, current: &Cbor) -> bool {
        let v1 = self.evaluate(root, current);
        let v2 = other.evaluate(root, current);

        match (&v1, &v2) {
            (None, None) => true,
            (Some(v1), Some(v2)) => value_equals(v1, v2),
            _ => false,
        }
    }

    fn lesser_than(&self, other: &Self, root: &Cbor, current: &Cbor) -> bool {
        let v1 = self.evaluate(root, current);
        let v2 = other.evaluate(root, current);

        let v1 = v1.as_ref().map(|v| v.as_ref());
        let v2 = v2.as_ref().map(|v| v.as_ref());

        match (
            v1.map(|v| v.kind()),
            v2.map(|v| v.kind()),
        ) {
            (Some(ItemKind::Pos(v1)), Some(ItemKind::Pos(v2))) => v1 < v2,
            (Some(ItemKind::Neg(v1)), Some(ItemKind::Neg(v2))) => v1 < v2,
            (Some(ItemKind::Float(v1)), Some(ItemKind::Float(v2))) => v1 < v2,
            (Some(ItemKind::Bytes(v1)), Some(ItemKind::Bytes(v2))) => v1 < v2,
            (Some(ItemKind::Str(v1)), Some(ItemKind::Str(v2))) => v1 < v2,
            _ => false,
        }
    }

    fn evaluate<'a>(&'a self, root: &'a Cbor, current: &'a Cbor) -> Option<Cow<'a, Cbor>> {
        match self {
            Comparable::Value(value) => Some(Cow::Borrowed(value)),
            Comparable::SingularPath(path) => path.evaluate(root, current),
            Comparable::Function(function) => function
                .evaluate_as_comparable(root, current)
                .map(Cow::Owned),
        }
    }
}

fn value_equals(v1: &Cbor, v2: &Cbor) -> bool {
    match (v1.kind(), v2.kind()) {
        (ItemKind::Pos(v1), ItemKind::Pos(v2)) => v1 == v2,
        (ItemKind::Neg(v1), ItemKind::Neg(v2)) => v1 == v2,
        (ItemKind::Bool(v1), ItemKind::Bool(v2)) => v1 == v2,
        (ItemKind::Simple(v1), ItemKind::Simple(v2)) => v1 == v2,
        (ItemKind::Float(v1), ItemKind::Float(v2)) => v1 == v2,
        (ItemKind::Bytes(v1), ItemKind::Bytes(v2)) => v1 == v2,
        (ItemKind::Str(v1), ItemKind::Str(v2)) => v1 == v2,
        (ItemKind::Null, ItemKind::Null) => true,
        (ItemKind::Array(a1), ItemKind::Array(a2)) => {
            let len1 = a1.size().unwrap_or(a1.count() as u64);
            let len2 = a2.size().unwrap_or(a2.count() as u64);
            len1 == len2 && a1.zip(a2).all(|(v1, v2)| v1 == v2)
        }
        (ItemKind::Dict(mut d1), ItemKind::Dict(mut d2)) => {
            let len1 = d1.size().unwrap_or(d1.count() as u64);
            let len2 = d2.size().unwrap_or(d2.count() as u64);
            len1 == len2
                && d1.all(|(key, v1)| {
                    if let Some(v2) = d2.find_map(|(k, v)| if k == key { Some(v) } else { None }) {
                        value_equals(v1, v2)
                    } else {
                        false
                    }
                })
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
    Abs(Vec<SingularSegment>),
    /// Relative singular path (beginning by '@')
    Rel(Vec<SingularSegment>),
}

impl SingularPath {
    #[inline]
    pub fn evaluate<'a>(&self, root: &'a Cbor, current: &'a Cbor) -> Option<Cow<'a, Cbor>> {
        match self {
            SingularPath::Abs(segments) => Self::evaluate_impl(segments, root),
            SingularPath::Rel(segments) => Self::evaluate_impl(segments, current),
        }
    }

    fn evaluate_impl<'a>(
        segments: &Vec<SingularSegment>,
        value: &'a Cbor,
    ) -> Option<Cow<'a, Cbor>> {
        let mut current_value = value;
        for segment in segments {
            match segment.evaluate(current_value) {
                Some(value) => current_value = value,
                None => return None,
            }
        }
        Some(Cow::Borrowed(current_value))
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum SingularSegment {
    Key(KeySelector),
    Index(IndexSelector),
}

impl SingularSegment {
    #[inline]
    fn evaluate<'a>(&self, value: &'a Cbor) -> Option<&'a Cbor> {
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
    fn evaluate_as_boolean_expr(&self, root: &Cbor, current: &Cbor) -> bool {
        match self {
            Function::Regex(comparable, regex) => {
                let value = comparable.evaluate(root, current);
                let value = value.as_ref().map(|v| v.as_ref());
                match value.map(|v| v.kind()) {
                    Some(ItemKind::Str(str)) => match str.as_str() {
                        Some(str) => regex.is_match(str),
                        None => false,
                    },
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn evaluate_as_comparable(&self, root: &Cbor, current: &Cbor) -> Option<CborOwned> {
        match self {
            Function::Length(comparable) => {
                let value = comparable.evaluate(root, current);
                let value = value.as_ref().map(|v| v.as_ref().kind());
                match value {
                    Some(ItemKind::Array(a)) => {
                        let len = a.size().unwrap_or(a.count() as u64);
                        Some(CborBuilder::new().write_pos(len, None))
                    }
                    Some(ItemKind::Dict(d)) => {
                        let len = d.size().unwrap_or(d.count() as u64);
                        Some(CborBuilder::new().write_pos(len, None))
                    }
                    Some(ItemKind::Str(s)) => {
                        Some(CborBuilder::new().write_pos(s.len() as u64, None))
                    }
                    Some(ItemKind::Bytes(b)) => {
                        Some(CborBuilder::new().write_pos(b.len() as u64, None))
                    }
                    None => None,
                    _ => Some(CborBuilder::new().write_pos(1, None)),
                }
            }
            Function::Count(path) => {
                Some(CborBuilder::new().write_pos(path.evaluate(root, current).len() as u64, None))
            }
            _ => None,
        }
    }
}

#[inline]
pub(crate) fn normalize_index(i: isize, len: usize) -> isize {
    if i >= 0 {
        i
    } else {
        len as isize + i
    }
}
