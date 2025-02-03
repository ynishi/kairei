use super::combinators::*;
use super::core::Parser;

pub fn equal<I: Clone + PartialEq>(value: I) -> Equal<I> {
    Equal::new(value)
}

pub fn expected<P, I, O>(parser: P, value: O) -> Expected<P, I, O>
where
    P: Parser<I, O>,
    I: Clone,
    O: Clone + PartialEq,
{
    Expected::new(parser, value)
}

pub fn identity<I: Clone>() -> Identity<I> {
    Identity::new()
}

pub fn zero<I, O: Clone>(zero_value: O) -> Zero<I, O> {
    Zero::new(zero_value)
}

pub fn fail<I, O>(message: &str) -> Fail<I, O> {
    Fail::new(message)
}

pub fn satisfy<I: Clone, O, F>(f: F) -> Satisfy<I, O, F>
where
    F: Fn(&I) -> Option<O>,
{
    Satisfy::new(f)
}

pub fn choice<I, O: Clone>(parsers: Vec<Box<dyn Parser<I, O>>>) -> Choice<I, O> {
    Choice::new(parsers)
}

pub fn preceded<P1, P2, I, O1, O2>(parser1: P1, parser2: P2) -> Preceded<P1, P2, I, O1, O2>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    I: Clone,
{
    Preceded::new(parser1, parser2)
}

pub fn sequence<I, O: Clone>(parsers: Vec<Box<dyn Parser<I, O>>>) -> Sequence<I, O> {
    Sequence::new(parsers)
}

pub fn map<P, F, A, B, I>(parser: P, f: F) -> Map<P, F, A, B>
where
    P: Parser<I, A>,
    F: Fn(A) -> B,
{
    Map::new(parser, f)
}

pub fn as_unit<I, O, P>(parser: P) -> AsUnit<P, O>
where
    P: Parser<I, O>,
{
    AsUnit::new(parser)
}

pub fn many<P, I, O>(parser: P) -> Many<P, I, O>
where
    P: Parser<I, O>,
{
    Many::new(parser)
}

pub fn many1<P, I, O>(parser: P) -> Many1<P, I, O>
where
    P: Parser<I, O>,
{
    Many1::new(parser)
}

pub fn separated_list<P, S, I, O>(item_parser: P, separator_parser: S) -> SeparatedList<P, S, I, O>
where
    P: Parser<I, O>,
    S: Parser<I, ()>,
{
    SeparatedList::new(item_parser, separator_parser)
}

pub fn optional<P, I, O>(parser: P) -> Optional<P, I, O>
where
    P: Parser<I, O>,
{
    Optional::new(parser)
}

pub fn delimited<L, P, R, I, O>(left: L, parser: P, right: R) -> Delimited<L, P, R, I, O>
where
    L: Parser<I, ()>,
    P: Parser<I, O>,
    R: Parser<I, ()>,
{
    Delimited::new(left, parser, right)
}

pub fn tuple2<P1, P2, I, O1, O2>(parser1: P1, parser2: P2) -> Tuple2<P1, P2, I, O1, O2>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
{
    Tuple2::new(parser1, parser2)
}

pub fn tuple3<P1, P2, P3, I, O1, O2, O3>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
) -> Tuple3<P1, P2, P3, I, O1, O2, O3>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
{
    Tuple3::new(parser1, parser2, parser3)
}

pub fn tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
) -> Tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
{
    Tuple4::new(parser1, parser2, parser3, parser4)
}

pub fn tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    parser5: P5,
) -> Tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
    P5: Parser<I, O5>,
{
    Tuple5::new(parser1, parser2, parser3, parser4, parser5)
}

// based on practical idiom for tuple
#[allow(clippy::type_complexity)]
pub fn tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    parser5: P5,
    parser6: P6,
) -> Tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
    P5: Parser<I, O5>,
    P6: Parser<I, O6>,
{
    Tuple6::new(parser1, parser2, parser3, parser4, parser5, parser6)
}

pub fn with_context<P, I, O, C>(parser: P, c: C) -> WithContext<P, C>
where
    P: Parser<I, O>,
{
    WithContext::new(parser, c)
}

pub fn lazy<I, O, F, P>(f: F) -> Lazy<F>
where
    F: Fn() -> P,
    P: Parser<I, O>,
{
    Lazy::new(f)
}
