use crate::{
    AbsolutePath, BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, Error,
    FilterSelector, Function, IndexSelector, KeySelector, Path, RelativePath, Segment, Selector,
    SingularPath, SingularSegment, SliceSelector,
};
use ciborium::value::Value;
use regex::Regex;

#[inline]
pub fn abs_path() -> PathBuilder {
    PathBuilder::new(true)
}

#[inline]
pub fn rel_path() -> PathBuilder {
    PathBuilder::new(false)
}

#[inline]
pub fn segment() -> SegmentBuilder {
    SegmentBuilder::new()
}

#[inline]
pub fn and<B1, B2>(left: B1, right: B2) -> BooleanExprBuilder
where
    B1: Into<BooleanExprBuilder>,
    B2: Into<BooleanExprBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::And(
        Box::new(left.into().build()),
        Box::new(right.into().build()),
    ))
}

#[inline]
pub fn or<B1, B2>(left: B1, right: B2) -> BooleanExprBuilder
where
    B1: Into<BooleanExprBuilder>,
    B2: Into<BooleanExprBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Or(
        Box::new(left.into().build()),
        Box::new(right.into().build()),
    ))
}

#[inline]
pub fn not<B>(expr: B) -> BooleanExprBuilder
where
    B: Into<BooleanExprBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Not(Box::new(expr.into().build())))
}

#[inline]
pub fn gte<C1, C2>(left: C1, right: C2) -> BooleanExprBuilder
where
    C1: Into<ComparableBuilder>,
    C2: Into<ComparableBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Comparison(ComparisonExpr::new(
        left.into().build(),
        ComparisonOperator::Gte,
        right.into().build(),
    )))
}

#[inline]
pub fn gt<C1, C2>(left: C1, right: C2) -> BooleanExprBuilder
where
    C1: Into<ComparableBuilder>,
    C2: Into<ComparableBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Comparison(ComparisonExpr::new(
        left.into().build(),
        ComparisonOperator::Gt,
        right.into().build(),
    )))
}

#[inline]
pub fn eq<C1, C2>(left: C1, right: C2) -> BooleanExprBuilder
where
    C1: Into<ComparableBuilder>,
    C2: Into<ComparableBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Comparison(ComparisonExpr::new(
        left.into().build(),
        ComparisonOperator::Eq,
        right.into().build(),
    )))
}

#[inline]
pub fn neq<C1, C2>(left: C1, right: C2) -> BooleanExprBuilder
where
    C1: Into<ComparableBuilder>,
    C2: Into<ComparableBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Comparison(ComparisonExpr::new(
        left.into().build(),
        ComparisonOperator::Neq,
        right.into().build(),
    )))
}

#[inline]
pub fn lte<C1, C2>(left: C1, right: C2) -> BooleanExprBuilder
where
    C1: Into<ComparableBuilder>,
    C2: Into<ComparableBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Comparison(ComparisonExpr::new(
        left.into().build(),
        ComparisonOperator::Lte,
        right.into().build(),
    )))
}

#[inline]
pub fn lt<C1, C2>(left: C1, right: C2) -> BooleanExprBuilder
where
    C1: Into<ComparableBuilder>,
    C2: Into<ComparableBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Comparison(ComparisonExpr::new(
        left.into().build(),
        ComparisonOperator::Lt,
        right.into().build(),
    )))
}

#[inline]
pub fn _match<C>(comparable: C, regex: &str) -> Result<BooleanExprBuilder, Error>
where
    C: Into<ComparableBuilder>,
{
    search(comparable, &format!("^{regex}$"))
}

#[inline]
pub fn search<C>(comparable: C, regex: &str) -> Result<BooleanExprBuilder, Error>
where
    C: Into<ComparableBuilder>,
{
    Ok(BooleanExprBuilder::new(BooleanExpr::Function(
        Function::Regex(Box::new(comparable.into().build()), Regex::new(regex)?),
    )))
}

/// Create an absolute SingularPathBuilder
#[inline]
pub fn sing_abs_path() -> SingularPathBuilder {
    SingularPathBuilder::new(true)
}

/// Create a relative SingularPathBuilder
#[inline]
pub fn sing_rel_path() -> SingularPathBuilder {
    SingularPathBuilder::new(false)
}

#[inline]
pub fn val<V: Into<Value>>(v: V) -> ComparableBuilder {
    ComparableBuilder::new(Comparable::Value(v.into()))
}

#[inline]
pub fn length<C>(comparable: C) -> ComparableBuilder
where
    C: Into<ComparableBuilder>,
{
    ComparableBuilder::new(Comparable::Function(Function::Length(Box::new(
        comparable.into().comparable,
    ))))
}

#[inline]
pub fn count(path: PathBuilder) -> ComparableBuilder {
    ComparableBuilder::new(Comparable::Function(Function::Count(path.build_path())))
}

pub struct PathBuilder {
    is_absolute: bool,
    segments: Vec<Segment>,
}

impl PathBuilder {
    #[inline]
    pub(crate) fn new(is_absolute: bool) -> Self {
        Self {
            is_absolute,
            segments: vec![],
        }
    }

    #[inline]
    pub fn child(mut self, segment: SegmentBuilder) -> Self {
        self.segments.push(segment.build(true));
        self
    }

    #[inline]
    pub fn descendant(mut self, segment: SegmentBuilder) -> Self {
        self.segments.push(segment.build(false));
        self
    }

    #[inline]
    pub fn key<V: Into<Value>>(mut self, v: V) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Key(KeySelector::new(
                v.into(),
            ))]));
        self
    }

    #[inline]
    pub fn wildcard(mut self) -> Self {
        self.segments.push(Segment::Child(vec![Selector::Wildcard]));
        self
    }

    #[inline]
    pub fn index(mut self, index: isize) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Index(IndexSelector::new(
                index,
            ))]));
        self
    }

    #[inline]
    pub fn slice(mut self, start: isize, end: isize, step: isize) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Slice(SliceSelector::new(
                start, end, step,
            ))]));
        self
    }

    #[inline]
    pub fn filter<B: Into<BooleanExprBuilder>>(mut self, boolean_expr: B) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Filter(FilterSelector::new(
                boolean_expr.into().build(),
            ))]));
        self
    }

    #[inline]
    pub fn build(self) -> CborPath {
        CborPath::new(self.segments)
    }

    #[inline]
    pub(crate) fn build_path(self) -> Path {
        if self.is_absolute {
            Path::Abs(AbsolutePath::new(self.segments))
        } else {
            Path::Rel(RelativePath::new(self.segments))
        }
    }
}

pub struct SegmentBuilder {
    selectors: Vec<Selector>,
}

impl SegmentBuilder {
    #[inline]
    pub(crate) fn new() -> Self {
        Self { selectors: vec![] }
    }

    #[inline]
    pub fn key<V: Into<Value>>(mut self, v: V) -> Self {
        self.selectors
            .push(Selector::Key(KeySelector::new(v.into())));
        self
    }

    #[inline]
    pub fn wildcard(mut self) -> Self {
        self.selectors.push(Selector::Wildcard);
        self
    }

    #[inline]
    pub fn index(mut self, index: isize) -> Self {
        self.selectors
            .push(Selector::Index(IndexSelector::new(index)));
        self
    }

    #[inline]
    pub fn slice(mut self, start: isize, end: isize, step: isize) -> Self {
        self.selectors
            .push(Selector::Slice(SliceSelector::new(start, end, step)));
        self
    }

    #[inline]
    pub fn filter(mut self, boolean_expr: BooleanExprBuilder) -> Self {
        self.selectors
            .push(Selector::Filter(FilterSelector::new(boolean_expr.build())));
        self
    }

    #[inline]
    pub(crate) fn build(self, is_child: bool) -> Segment {
        if is_child {
            Segment::Child(self.selectors)
        } else {
            Segment::Descendant(self.selectors)
        }
    }
}

pub struct BooleanExprBuilder {
    boolean_expr: BooleanExpr,
}

impl BooleanExprBuilder {
    #[inline]
    pub(crate) fn new(boolean_expr: BooleanExpr) -> Self {
        Self { boolean_expr }
    }

    #[inline]
    pub(crate) fn build(self) -> BooleanExpr {
        self.boolean_expr
    }
}

impl From<PathBuilder> for BooleanExprBuilder {
    #[inline]
    fn from(path: PathBuilder) -> Self {
        BooleanExprBuilder {
            boolean_expr: BooleanExpr::Path(path.build_path()),
        }
    }
}

pub struct ComparableBuilder {
    comparable: Comparable,
}

impl ComparableBuilder {
    #[inline]
    pub(crate) fn new(comparable: Comparable) -> Self {
        Self { comparable }
    }

    #[inline]
    pub(crate) fn build(self) -> Comparable {
        self.comparable
    }
}

impl From<SingularPathBuilder> for ComparableBuilder {
    #[inline]
    fn from(singular_path: SingularPathBuilder) -> Self {
        ComparableBuilder {
            comparable: Comparable::SingularPath(singular_path.build()),
        }
    }
}

impl From<Comparable> for ComparableBuilder {
    #[inline]
    fn from(comparable: Comparable) -> Self {
        ComparableBuilder { comparable }
    }
}

pub struct SingularPathBuilder {
    is_absolute: bool,
    segments: Vec<SingularSegment>,
}

impl SingularPathBuilder {
    #[inline]
    pub(crate) fn new(is_absolute: bool) -> Self {
        Self {
            is_absolute,
            segments: vec![],
        }
    }

    #[inline]
    pub fn key<V: Into<Value>>(mut self, v: V) -> SingularPathBuilder {
        self.segments
            .push(SingularSegment::Key(KeySelector::new(v.into())));
        self
    }

    #[inline]
    pub fn index(mut self, index: isize) -> SingularPathBuilder {
        self.segments
            .push(SingularSegment::Index(IndexSelector::new(index)));
        self
    }

    #[inline]
    pub(crate) fn build(self) -> SingularPath {
        if self.is_absolute {
            SingularPath::Abs(self.segments)
        } else {
            SingularPath::Rel(self.segments)
        }
    }
}
