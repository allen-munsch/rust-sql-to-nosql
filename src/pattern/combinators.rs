// pattern/combinators.rs - Core pattern matching combinators
// Pure functional combinators for matching SQL AST patterns

use std::marker::PhantomData;

/// Result of a pattern match
pub type MatchResult<T> = Result<T, ()>;

/// A pattern matcher that matches against a node of type I and produces a result of type O
pub trait Pattern<I, O> {
    /// Try to match the input and produce an output
    fn match_pattern(&self, input: &I) -> MatchResult<O>;
}

// Allow functions to be patterns
impl<I, O, F> Pattern<I, O> for F
where
    F: Fn(&I) -> MatchResult<O>,
{
    fn match_pattern(&self, input: &I) -> MatchResult<O> {
        self(input)
    }
}

/// Maps the result of a successful pattern match
pub struct Map<P, F, I, O, R> {
    pattern: P,
    f: F,
    _phantom: PhantomData<(I, O, R)>,
}

impl<P, F, I, O, R> Pattern<I, R> for Map<P, F, I, O, R>
where
    P: Pattern<I, O>,
    F: Fn(O) -> R,
{
    fn match_pattern(&self, input: &I) -> MatchResult<R> {
        self.pattern.match_pattern(input).map(&self.f)
    }
}

/// Creates a new pattern that maps the result of a successful match
pub fn map<P, F, I, O, R>(pattern: P, f: F) -> Map<P, F, I, O, R>
where
    P: Pattern<I, O>,
    F: Fn(O) -> R,
{
    Map {
        pattern,
        f,
        _phantom: PhantomData,
    }
}

/// Chains two patterns, passing the output of the first to the second
pub struct AndThen<P1, P2, I, O1, O2> {
    first: P1,
    second: P2,
    _phantom: PhantomData<(I, O1, O2)>,
}

impl<P1, P2, I, O1, O2> Pattern<I, O2> for AndThen<P1, P2, I, O1, O2>
where
    P1: Pattern<I, O1>,
    P2: Pattern<O1, O2>,
{
    fn match_pattern(&self, input: &I) -> MatchResult<O2> {
        let intermediate = self.first.match_pattern(input)?;
        self.second.match_pattern(&intermediate)
    }
}

/// Creates a new pattern that chains two patterns
pub fn and_then<P1, P2, I, O1, O2>(first: P1, second: P2) -> AndThen<P1, P2, I, O1, O2>
where
    P1: Pattern<I, O1>,
    P2: Pattern<O1, O2>,
{
    AndThen {
        first,
        second,
        _phantom: PhantomData,
    }
}

/// Tries multiple patterns in sequence, returning the first match
pub struct Or<P1, P2, I, O> {
    first: P1,
    second: P2,
    _phantom: PhantomData<(I, O)>,
}

impl<P1, P2, I, O> Pattern<I, O> for Or<P1, P2, I, O>
where
    P1: Pattern<I, O>,
    P2: Pattern<I, O>,
{
    fn match_pattern(&self, input: &I) -> MatchResult<O> {
        self.first.match_pattern(input).or_else(|_| self.second.match_pattern(input))
    }
}

/// Creates a new pattern that tries multiple patterns
pub fn or<P1, P2, I, O>(first: P1, second: P2) -> Or<P1, P2, I, O>
where
    P1: Pattern<I, O>,
    P2: Pattern<I, O>,
{
    Or {
        first,
        second,
        _phantom: PhantomData,
    }
}

impl<I: 'static, O: 'static> std::ops::BitOr for Box<dyn Pattern<I, O>> {
    type Output = Box<dyn Pattern<I, O>>;

    fn bitor(self, other: Box<dyn Pattern<I, O>>) -> Self::Output {
        // Create a new pattern that delegates to both boxed patterns
        Box::new(move |input: &I| {
            self.match_pattern(input).or_else(|_| other.match_pattern(input))
        })
    }
}

/// Optionally matches a pattern, returning None if no match
pub struct Optional<P, I, O> {
    pattern: P,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> Pattern<I, Option<O>> for Optional<P, I, O>
where
    P: Pattern<I, O>,
{
    fn match_pattern(&self, input: &I) -> MatchResult<Option<O>> {
        match self.pattern.match_pattern(input) {
            Ok(o) => Ok(Some(o)),
            Err(_) => Ok(None),
        }
    }
}

/// Creates a new pattern that optionally matches
pub fn optional<P, I, O>(pattern: P) -> Optional<P, I, O>
where
    P: Pattern<I, O>,
{
    Optional {
        pattern,
        _phantom: PhantomData,
    }
}

/// Matches against a pair of patterns
pub struct Pair<P1, P2, I, O1, O2> {
    first: P1,
    second: P2,
    _phantom: PhantomData<(I, O1, O2)>,
}

impl<P1, P2, I, O1, O2> Pattern<I, (O1, O2)> for Pair<P1, P2, I, O1, O2>
where
    P1: Pattern<I, O1>,
    P2: Pattern<I, O2>,
{
    fn match_pattern(&self, input: &I) -> MatchResult<(O1, O2)> {
        let first_result = self.first.match_pattern(input)?;
        let second_result = self.second.match_pattern(input)?;
        Ok((first_result, second_result))
    }
}

/// Creates a new pattern that matches against a pair of patterns
pub fn pair<P1, P2, I, O1, O2>(first: P1, second: P2) -> Pair<P1, P2, I, O1, O2>
where
    P1: Pattern<I, O1>,
    P2: Pattern<I, O2>,
{
    Pair {
        first,
        second,
        _phantom: PhantomData,
    }
}

/// A pattern that always matches
pub struct Always<I, O> {
    value: O,
    _phantom: PhantomData<I>,
}

impl<I, O: Clone> Pattern<I, O> for Always<I, O> {
    fn match_pattern(&self, _input: &I) -> MatchResult<O> {
        Ok(self.value.clone())
    }
}

/// Creates a pattern that always matches with a given value
pub fn always<I, O: Clone>(value: O) -> Always<I, O> {
    Always {
        value,
        _phantom: PhantomData,
    }
}

/// A pattern that never matches
pub struct Never<I, O> {
    _phantom: PhantomData<(I, O)>,
}

impl<I, O> Pattern<I, O> for Never<I, O> {
    fn match_pattern(&self, _input: &I) -> MatchResult<O> {
        Err(())
    }
}

/// Creates a pattern that never matches
pub fn never<I, O>() -> Never<I, O> {
    Never {
        _phantom: PhantomData,
    }
}

/// Matches against a predicate
pub struct Predicate<F, I> {
    predicate: F,
    _phantom: PhantomData<I>,
}

impl<F, I> Pattern<I, ()> for Predicate<F, I>
where
    F: Fn(&I) -> bool,
{
    fn match_pattern(&self, input: &I) -> MatchResult<()> {
        if (self.predicate)(input) {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Creates a pattern that matches if a predicate is true
pub fn predicate<F, I>(predicate: F) -> Predicate<F, I>
where
    F: Fn(&I) -> bool,
{
    Predicate {
        predicate,
        _phantom: PhantomData,
    }
}

/// Extracts a value from the input
pub struct Extract<F, I, O> {
    extract: F,
    _phantom: PhantomData<(I, O)>,
}

impl<F, I, O> Pattern<I, O> for Extract<F, I, O>
where
    F: Fn(&I) -> Option<O>,
{
    fn match_pattern(&self, input: &I) -> MatchResult<O> {
        (self.extract)(input).ok_or(())
    }
}

/// Creates a pattern that extracts a value
pub fn extract<F, I, O>(extract: F) -> Extract<F, I, O>
where
    F: Fn(&I) -> Option<O>,
{
    Extract {
        extract,
        _phantom: PhantomData,
    }
}