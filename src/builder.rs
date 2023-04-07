/*!
Fluent API to build a [`CborPath`](CborPath) instance
*/
use crate::{
    AbsolutePath, BooleanExpr, CborPath, Comparable, ComparisonExpr, ComparisonOperator, Error,
    FilterSelector, Function, IndexSelector, KeySelector, FilterPath, RelativePath, Segment, Selector,
    SingularPath, SingularSegment, SliceSelector,
};
use cbor_data::{CborOwned, CborBuilder, Writer, Literal, Cbor};
use regex::Regex;

/// Represents an absolute path (beginning by a '$')
///
/// This is the format of a `CBORPath` expression.
/// It can also be passed to
/// * the [`count`](count) function
/// * the [`value`](value) function
/// * the [`filter`](SegmentBuilder::filter) selector.
/// In this case the path is used in an existence test within the filter
#[inline]
pub fn abs_path() -> PathBuilder {
    PathBuilder::new(true)
}

/// Represents a relative path (beginning by a '@')
///
/// This is the format of a `CBORPath` expression.
/// It can also be passed to
/// * the [`count`](count) function
/// * the [`value`](value) function
/// * the [`filter`](SegmentBuilder::filter) selector.
/// In this case the path is used in an existence test within the filter
#[inline]
pub fn rel_path() -> PathBuilder {
    PathBuilder::new(false)
}

/// Represents a segment of a [`path`](PathBuilder)
///
/// Used to build a segment to be passed to the [`child`](PathBuilder::child) or
/// [`descendant`](PathBuilder::descendant) functions
#[inline]
pub fn segment() -> SegmentBuilder {
    SegmentBuilder::new()
}

/// Represents a `logical AND` between two [`boolean expressions`](BooleanExprBuilder)
///
/// Note that a [`path`](PathBuilder) can be used as one of the two [`boolean expressions`](BooleanExprBuilder)
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

/// Represents a `logical OR` between two [`boolean expressions`](BooleanExprBuilder)
///
/// Note that a [`path`](PathBuilder) can be used as one of the two [`boolean expressions`](BooleanExprBuilder)
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

/// Represents a `logical NOT` on a [`boolean expression`](BooleanExprBuilder)
///
/// Note that a [`path`](PathBuilder) can be used as the [`boolean expression`](BooleanExprBuilder)
#[inline]
pub fn not<B>(expr: B) -> BooleanExprBuilder
where
    B: Into<BooleanExprBuilder>,
{
    BooleanExprBuilder::new(BooleanExpr::Not(Box::new(expr.into().build())))
}

/// Represents a `greater than or equal` comparison between two [`comparables`](ComparableBuilder)
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as one of the two [`comparables`](ComparableBuilder)
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

/// Represents a `greater than` comparison between two [`comparables`](ComparableBuilder)
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as one of the two [`comparables`](ComparableBuilder)
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

/// Represents a `equal` comparison between two [`comparables`](ComparableBuilder)
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as one of the two [`comparables`](ComparableBuilder)
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

/// Represents a `not equal` comparison between two [`comparables`](ComparableBuilder)
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as one of the two [`comparables`](ComparableBuilder)
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

/// Represents a `lesser than or equal` comparison between two [`comparables`](ComparableBuilder)
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as one of the two [`comparables`](ComparableBuilder)
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

/// Represents a `lesser than` comparison between two [`comparables`](ComparableBuilder)
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as one of the two [`comparables`](ComparableBuilder)
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

/// Represents a regular expression full match within a [`filter`](SegmentBuilder::filter).
///
/// # Arguments
/// * `comparable` - [`comparable`](ComparableBuilder)
///    on which the regular expression will be applied
/// * `regex` - Regular expression pattern in the format of the [`regex`](https://docs.rs/regex) crate.
///
/// Implementation automatically surrounds the regular expression with `^` and `$` to force the full math
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as the [`comparable`](ComparableBuilder)
#[inline]
pub fn _match<C>(comparable: C, regex: &str) -> Result<BooleanExprBuilder, Error>
where
    C: Into<ComparableBuilder>,
{
    search(comparable, &format!("^{regex}$"))
}

/// Represents a regular expression substring match within a [`filter`](SegmentBuilder::filter).
///
/// # Arguments
/// * `comparable` - [`comparable`](ComparableBuilder)
///    on which the regular expression will be applied
/// * `regex` - Regular expression pattern in the format of the [`regex`](https://docs.rs/regex) crate.
///
/// Note that a [`singular path`](SingularPathBuilder) can be used as the [`comparable`](ComparableBuilder)
#[inline]
pub fn search<C>(comparable: C, regex: &str) -> Result<BooleanExprBuilder, Error>
where
    C: Into<ComparableBuilder>,
{
    Ok(BooleanExprBuilder::new(BooleanExpr::Function(
        Function::Regex(Box::new(comparable.into().build()), Regex::new(regex)?),
    )))
}

/// Represents an absolute [`singular path`](SingularPathBuilder) (beginning by a '$')
#[inline]
pub fn sing_abs_path() -> SingularPathBuilder {
    SingularPathBuilder::new(true)
}

/// Represents a relative [`singular path`](SingularPathBuilder) (beginning by a '@')
#[inline]
pub fn sing_rel_path() -> SingularPathBuilder {
    SingularPathBuilder::new(false)
}

/// Represents a simple `CBOR` value within a filter.
///
/// Can be used wherever a [`comparable`](ComparableBuilder) is expected
#[inline]
pub fn val<V: IntoCborOwned>(v: V) -> ComparableBuilder {
    ComparableBuilder::new(Comparable::Value(v.into()))
}

/// Represents the `length` function within a [`filter`](SegmentBuilder::filter)
///
/// The `length` function extension provides a way to compute the length of a value
/// and make that available for further processing in the filter expression:
/// ```json
/// ["$", {"?": {">=": [{"length": ["@", "authors"]}, 5]}}]
/// ```
///
/// Its only argument is a [`comparable`](ComparableBuilder)
/// (possibly taken from a [`singular path`](SingularPathBuilder) as in the example above).
///
/// The result also is a [`comparable`](ComparableBuilder), an unsigned integer.
/// * If the argument value is a `text string`, the result is the number of UTF8 characters in the string.
/// * If the argument value is a `byte string`, the result is the size of the byte buffer.
/// * If the argument value is an `array`, the result is the number of elements in the array.
/// * If the argument value is a `map`, the result is the number of items in the map.
/// * For any other argument value, the result is `1`.
///
/// Can be used wherever a [`comparable`](ComparableBuilder) is expected
#[inline]
pub fn length<C>(comparable: C) -> ComparableBuilder
where
    C: Into<ComparableBuilder>,
{
    ComparableBuilder::new(Comparable::Function(Function::Length(Box::new(
        comparable.into().comparable,
    ))))
}

/// Represents the `count` function within a [`filter`](SegmentBuilder::filter)
///
/// The `count` function extension provides a way to obtain the number of nodes in a [`path`](PathBuilder)
/// and make that available for further processing in the filter expression:
/// ```json
/// ["$", {"?": {">=": [{"count": ["@", "authors"]}, 5]}}]
/// ```
///
/// The result is a [`comparable`](ComparableBuilder), an unsigned integer,
/// that gives the number of nodes in the [`path`](PathBuilder).
///
/// Can be used wherever a [`comparable`](ComparableBuilder) is expected.
#[inline]
pub fn count(path: PathBuilder) -> ComparableBuilder {
    ComparableBuilder::new(Comparable::Function(Function::Count(path.build_path())))
}

/// Represents the `vakye` function within a [`filter`](SegmentBuilder::filter)
///
/// The `value` function extension provides a way to convert a [`path`](PathBuilder)
/// to a value and make that available for further processing in the filter expression:
/// ```json
/// ["$", {"?": {"==": [{"value": ["@", "color"]}, "red"]}}]
/// ```
///
/// The result is a [`comparable`](ComparableBuilder).
/// - If the argument contains a single node, the result is the value of the node.
/// - If the argument is an empty node list or contains multiple nodes, the result is `None`.
///
/// Can be used wherever a [`comparable`](ComparableBuilder) is expected.
#[inline]
pub fn value(path: PathBuilder) -> ComparableBuilder {
    ComparableBuilder::new(Comparable::Function(Function::Value(path.build_path())))
}

/// Represents a `path`
///
/// A `path` expression is a `CBOR Array` which, when applied to a `CBOR` value, the
/// *argument*, selects zero or more nodes of the argument and output these nodes as a nodelist.
///
/// A `path` always begins by an identifier
/// * `$` for absolute paths,
/// * `@` for relative paths.
/// This identifier is automatically included by the [`abs_path`] and the [`rel_path`] function.
///
/// A `path` is then followed by one or more [`segments`](SegmentBuilder)
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

    /// Adds a `child` [`segments`](SegmentBuilder) to the `path`
    ///
    /// A child segment contains a sequence of selectors, each of which selects zero or more children of the input value.
    ///
    /// Selectors of different kinds may be combined within a single child segment.
    /// Most of a time a segment contains a unique selector
    #[inline]
    pub fn child(mut self, segment: SegmentBuilder) -> Self {
        self.segments.push(segment.build(true));
        self
    }

    /// Adds a `descendant` [`segments`](SegmentBuilder) to the `path`
    ///
    /// A descendant segment produces zero or more descendants of the input value.
    ///
    /// A descendant selector visits the input value and each of its descendants such that:
    /// * nodes of any array are visited in array order, and
    /// * nodes are visited before their descendants.
    ///
    /// Selectors of different kinds may be combined within a single descendant segment.
    /// Most of a time a segment contains a unique selector
    #[inline]
    pub fn descendant(mut self, segment: SegmentBuilder) -> Self {
        self.segments.push(segment.build(false));
        self
    }

    /// Shortcut for a [`child`](PathBuilder::child) segment with a unique [`key`](SegmentBuilder::key) selector.
    #[inline]
    pub fn key<V: IntoCborOwned>(mut self, v: V) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Key(KeySelector::new(v.into()))]));
        self
    }

    /// Shortcut for a [`child`](PathBuilder::child) segment with a unique [`wildcard`](SegmentBuilder::wildcard) selector.
    #[inline]
    pub fn wildcard(mut self) -> Self {
        self.segments.push(Segment::Child(vec![Selector::Wildcard]));
        self
    }

    /// Shortcut for a [`child`](PathBuilder::child) segment with a unique [`index`](SegmentBuilder::index) selector.
    #[inline]
    pub fn index(mut self, index: isize) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Index(IndexSelector::new(
                index,
            ))]));
        self
    }

    /// Shortcut for a [`child`](PathBuilder::child) segment with a unique [`slice`](SegmentBuilder::slice) selector.
    #[inline]
    pub fn slice(mut self, start: isize, end: isize, step: isize) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Slice(SliceSelector::new(
                start, end, step,
            ))]));
        self
    }

    /// Shortcut for a [`child`](PathBuilder::child) segment with a unique [`filter`](SegmentBuilder::filter) selector.
    #[inline]
    pub fn filter<B: Into<BooleanExprBuilder>>(mut self, boolean_expr: B) -> Self {
        self.segments
            .push(Segment::Child(vec![Selector::Filter(FilterSelector::new(
                boolean_expr.into().build(),
            ))]));
        self
    }

    /// Build a [`CborPath`] from the builder
    #[inline]
    pub fn build(self) -> CborPath {
        CborPath::new(self.segments)
    }

    #[inline]
    pub(crate) fn build_path(self) -> FilterPath {
        if self.is_absolute {
            FilterPath::Abs(AbsolutePath::new(self.segments))
        } else {
            FilterPath::Rel(RelativePath::new(self.segments))
        }
    }
}

/// Represents a [`path`](PathBuilder) segment.
///
/// Segments apply one or more selectors to an input value and concatenate the results into a single nodelist.
pub struct SegmentBuilder {
    selectors: Vec<Selector>,
}

impl SegmentBuilder {
    #[inline]
    pub(crate) fn new() -> Self {
        Self { selectors: vec![] }
    }

    /// Adds a `key selector` to the segment.
    ///
    /// Applying the `key selector` to a `CBOR map` node selects an item value whose `key`
    /// equals the item key from the selector, or selects nothing if there is no such item value.
    /// Nothing is selected from a value that is not a `CBOR map`.
    #[inline]
    pub fn key<V: IntoCborOwned>(mut self, key: V) -> Self {
        self.selectors
            .push(Selector::Key(KeySelector::new(key.into())));
        self
    }

    /// Adds a `wildcard selector` to the segment.
    ///
    /// A `wildcard selector` selects the nodes of all children of a map or array.
    ///
    /// The wildcard selector selects nothing from a primitive `CBOR` value (that is, a number, a string, a boolean, or null).
    #[inline]
    pub fn wildcard(mut self) -> Self {
        self.selectors.push(Selector::Wildcard);
        self
    }

    /// Adds an `index selector` to the segment.
    ///
    /// A non-negative `index selector` applied to an array selects an array element using a zero-based index.
    ///
    /// A negative `index selector` counts from the array end. For example, the selector `-1` selects the last
    /// and the selector `-2` selects the penultimate element of an array with at least two elements.
    ///
    /// Nothing is selected if index is out of bounds or if the value is not an array.
    #[inline]
    pub fn index(mut self, index: isize) -> Self {
        self.selectors
            .push(Selector::Index(IndexSelector::new(index)));
        self
    }

    /// Adds a `slice selector` to the segment.
    ///
    /// A slice expression selects a subset of the elements of the input array,
    /// in the same order as the array or the reverse order,
    /// depending on the sign of the step parameter.
    /// It selects no nodes from a node that is not an array.
    ///
    /// A slice is defined by the two slice parameters, `start` and `end`, and an iteration delta, `step`.
    ///
    /// # See
    /// <https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html#name-array-slice-selector>
    #[inline]
    pub fn slice(mut self, start: isize, end: isize, step: isize) -> Self {
        self.selectors
            .push(Selector::Slice(SliceSelector::new(start, end, step)));
        self
    }

    /// Adds a `filter selector` to the segment.
    ///
    /// The `filter selector` works with arrays and maps exclusively.
    /// Its result is a list of *zero*, *one*, *multiple* or *all* of their array elements or map item values, respectively.
    /// Applied to primitive values, it will select nothing.
    ///
    /// # Existence Tests
    /// A [`path`](PathBuilder) by itself in a Boolean context is an existence test which yields `true`
    /// if the path selects at least one node and yields `false` if the path does not select any nodes.
    ///
    /// # Comparisons
    /// The comparison operators `==` and `<` are defined first and then these are used to define `!=`, `<=`, `>`, and `>=`.
    ///
    /// When a path resulting in an empty nodelist appears on either side of a comparison:
    /// * a comparison using the operator `==` yields true if and only if the comparison
    /// is between two paths each of which result in an empty nodelist.
    /// * a comparison using the operator `<` yields false.
    ///
    /// When any path on either side of a comparison results in a nodelist consisting of a single node, each such path is
    /// replaced by the value of its node and then:
    /// * a comparison using the operator `==` yields true if and only if the comparison
    /// is between:
    ///     * equal primitive values,
    ///     * equal arrays, that is arrays of the same length where each element of the first array is equal to the corresponding
    ///       element of the second array, or
    ///     * equal maps with no duplicate keys, that is where:
    ///         * both maps have the same collection of keys (with no duplicates), and
    ///         * for each of those keys, the values associated with the key by the maps are equal.
    /// * a comparison using the operator `<` yields true if and only if
    /// the comparison is between values which are both numbers or both strings and which satisfy the comparison:
    ///     * numbers compare using the normal mathematical ordering;
    ///     * the empty string compares less than any non-empty string
    ///     * a non-empty string compares less than another non-empty string if and only if the first string starts with a
    ///       lower Unicode character value than the second string or if both strings start with the same Unicode character value and
    ///       the remainder of the first string compares less than the remainder of the second string.
    ///
    /// Note that comparisons using the operator `<` yield false if either value being
    /// compared is a map, array, boolean, or `null`.
    ///
    /// `!=`, `<=`, `>`, and `>=` are defined in terms of the other comparison operators. For any `a` and `b`:
    /// * The comparison `a != b` yields true if and only if `a == b` yields false.
    /// * The comparison `a <= b` yields true if and only if `a < b` yields true or `a == b` yields true.
    /// * The comparison `a > b` yields true if and only if `b < a` yields true.
    /// * The comparison `a >= b` yields true if and only if `b < a` yields true or `a == b` yields true.    
    ///
    /// # Boolean Operators
    /// The logical [`AND`](and), [`OR`](or), and [`NOT`](not) operators have the normal semantics of Boolean algebra and
    /// obey its laws.
    ///
    /// # Functions
    /// Filter selectors may use any of the following functions:
    /// * [`match`](_match)
    /// * [`search`]
    /// * [`length`]
    /// * [`count`]
    /// * [`value`]
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

/// Represents a `boolean expression` used within a [`filter`](SegmentBuilder::filter)
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

/// Represents a `comparable` used as an operand of a [`filter`](SegmentBuilder::filter) comparison.
///
/// A `comparable` can be
/// * A [`value`](val)
/// * A [`singular path`](SingularPathBuilder)
/// * A `function` ([`match`](_match), [`search`], [`length`], [`count`] or [`value`])
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

/// Represents a `singular path`.
///
/// A `singular path` is a CBORPath expression built from segments each of which,
/// regardless of the input value, produces a nodelist containing at most one node.
///
/// A `singular path` can be absolute (beginning with a `$`) or relative (beginning with a `@`)
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

    /// Add a key segment to the singular path.
    ///
    /// Works exactly as the [`key selector`](SegmentBuilder::key)
    #[inline]
    pub fn key<V: IntoCborOwned>(mut self, v: V) -> SingularPathBuilder {
        self.segments
            .push(SingularSegment::Key(KeySelector::new(v.into())));
        self
    }

    /// Add an index segment to the singular path.
    ///
    /// Works exactly as the [`index selector`](SegmentBuilder::index)
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

pub trait IntoCborOwned {
    fn into(self) -> CborOwned;
}

impl IntoCborOwned for u64 {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_pos(self, None)
    }
}

impl IntoCborOwned for i64 {
    fn into(self) -> CborOwned {
        if self >= 0 {
            CborBuilder::new().write_pos(self as u64, None)
        } else {
            CborBuilder::new().write_neg((-1 -self) as u64, None)
        }
    }
}

impl IntoCborOwned for u32 {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_pos(self as u64, None)
    }
}

impl IntoCborOwned for i32 {
    fn into(self) -> CborOwned {
        if self >= 0 {
            CborBuilder::new().write_pos(self as u64, None)
        } else {
            CborBuilder::new().write_neg((-1 -self) as u64, None)
        }
    }
}

impl IntoCborOwned for u16 {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_pos(self as u64, None)
    }
}

impl IntoCborOwned for i16 {
    fn into(self) -> CborOwned {
        if self >= 0 {
            CborBuilder::new().write_pos(self as u64, None)
        } else {
            CborBuilder::new().write_neg((-1 -self) as u64, None)
        }
    }
}

impl IntoCborOwned for u8 {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_pos(self as u64, None)
    }
}

impl IntoCborOwned for i8 {
    fn into(self) -> CborOwned {
        if self >= 0 {
            CborBuilder::new().write_pos(self as u64, None)
        } else {
            CborBuilder::new().write_neg((-1 -self) as u64, None)
        }
    }
}

impl IntoCborOwned for f64 {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_lit(Literal::L8(self.to_bits()), None)
    }
}

impl IntoCborOwned for f32 {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_lit(Literal::L4(self.to_bits()), None)
    }
}

impl IntoCborOwned for bool {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_bool(self, None)
    }
}

impl IntoCborOwned for &str {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_str(self, None)
    }
}

impl IntoCborOwned for &[u8] {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_bytes(self, None)
    }
}

impl IntoCborOwned for () {
    fn into(self) -> CborOwned {
        CborBuilder::new().write_null(None)
    }
}

impl<'a> IntoCborOwned for &'a Cbor {
    fn into(self) -> CborOwned {
        self.to_owned()
    }
}
