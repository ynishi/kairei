use super::core::ParseError;
use super::core::ParseResult;
use super::core::Parser;
use std::fmt;
use std::marker::PhantomData;

// Expected: 入力値を変換し、その結果が value と一致する場合のみ成功し、value を返す
#[derive(Clone)]
pub struct Expected<P, I, O>
where
    P: Parser<I, O>,
{
    parser: P,
    value: O,
    _phantom: PhantomData<I>,
}

impl<P, I, O> Expected<P, I, O>
where
    P: Parser<I, O>,
{
    pub fn new(parser: P, value: O) -> Self {
        Self {
            parser,
            value,
            _phantom: PhantomData,
        }
    }
}

impl<P, I, O> Parser<I, O> for Expected<P, I, O>
where
    P: Parser<I, O>,
    I: Clone,
    O: Clone + PartialEq,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        let (new_pos, parsed_value) = self.parser.parse(input, pos)?;
        if parsed_value == self.value {
            Ok((new_pos, parsed_value))
        } else {
            Err(ParseError::Fail(
                "parsed value does not equal to expected value".to_string(),
            ))
        }
    }
}

#[derive(Clone)]
pub struct Equal<I> {
    value: I,
}

impl<I> Equal<I> {
    pub fn new(value: I) -> Self {
        Self { value }
    }
}

impl<I: Clone + PartialEq + fmt::Display> Parser<I, I> for Equal<I> {
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<I> {
        if input.len() > pos {
            let (next_pos, found) = (pos + 1, input[pos].clone());
            if found == self.value {
                Ok((next_pos, found))
            } else {
                Err(ParseError::Fail(format!(
                    "expected not matched: expected: {}, found: {}, at {}",
                    self.value,
                    found,
                    pos + 1
                )))
            }
        } else {
            Err(ParseError::EOF)
        }
    }
}

// Identity: 入力を消費してそのまま返す
#[derive(Clone)]
pub struct Identity<I> {
    _phantom: PhantomData<I>,
}

impl<I> Identity<I> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl Default for Identity<char> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Clone> Parser<I, I> for Identity<I> {
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<I> {
        input
            .get(pos)
            .map(|x| (pos + 1, x.clone()))
            .ok_or(ParseError::EOF)
    }
}

// Zero: 常に zero_value を返すパーサー
#[derive(Clone)]
pub struct Zero<I, O> {
    zero_value: O,
    _phantom: PhantomData<I>,
}

impl<I, O> Zero<I, O> {
    pub fn new(zero_value: O) -> Self {
        Self {
            zero_value,
            _phantom: PhantomData,
        }
    }
}

impl<I, O: Clone> Parser<I, O> for Zero<I, O> {
    fn parse(&self, _input: &[I], pos: usize) -> ParseResult<O> {
        Ok((pos, self.zero_value.clone()))
    }
}

#[derive(Clone)]
pub struct Fail<I, O> {
    message: String,
    _phantom: PhantomData<(I, O)>,
}

impl<I, O> Fail<I, O> {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            _phantom: PhantomData,
        }
    }
}

impl<I, O> Parser<I, O> for Fail<I, O> {
    fn parse(&self, _input: &[I], _pos: usize) -> ParseResult<O> {
        Err(ParseError::Fail(self.message.clone()))
    }
}

#[derive(Clone)]
pub struct Satisfy<I, O, F> {
    f: F,
    _phantom: PhantomData<(I, O)>,
}

impl<I, O, F> Satisfy<I, O, F> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, F> Parser<I, O> for Satisfy<I, O, F>
where
    F: Fn(&I) -> Option<O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        input
            .get(pos)
            .and_then(|x| (self.f)(x).map(|result| (pos + 1, result)))
            .ok_or(ParseError::EOF)
    }
}

pub struct Choice<I, O> {
    parsers: Vec<Box<dyn Parser<I, O>>>,
}

impl<I, O> Choice<I, O> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, O>>>) -> Self {
        Self { parsers }
    }
}

impl<I, O> Parser<I, O> for Choice<I, O> {
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        for parser in &self.parsers {
            if let Ok(result) = parser.parse(input, pos) {
                return Ok(result);
            }
        }
        Err(ParseError::NoAlternative)
    }
}
#[derive(Clone)]
pub struct Preceded<P1, P2, I, O> {
    parser1: P1,
    parser2: P2,
    _phantom: PhantomData<(I, O)>,
}

impl<P1, P2, I, O> Preceded<P1, P2, I, O> {
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
            _phantom: PhantomData,
        }
    }
}

impl<P1, P2, I, O> Parser<I, O> for Preceded<P1, P2, I, O>
where
    P1: Parser<I, ()>,
    P2: Parser<I, O>,
    I: Clone,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        let (pos, _) = self.parser1.parse(input, pos)?;
        let (pos, result) = self.parser2.parse(input, pos)?;
        Ok((pos, result))
    }
}

pub struct Sequence<I, O> {
    parsers: Vec<Box<dyn Parser<I, O>>>,
}

impl<I, O> Sequence<I, O> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, O>>>) -> Self {
        Self { parsers }
    }
}

impl<I, O: Clone> Parser<I, Vec<O>> for Sequence<I, O> {
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Vec<O>> {
        let mut results = Vec::new();
        let mut current_pos = pos;
        for parser in &self.parsers {
            let (new_pos, result) = parser.parse(input, current_pos)?;
            results.push(result);
            current_pos = new_pos;
        }
        Ok((current_pos, results))
    }
}

#[derive(Clone)]
pub struct Map<P, F, A, B> {
    parser: P,
    f: F,
    _phantom: PhantomData<(A, B)>,
}

impl<P, F, A, B> Map<P, F, A, B> {
    pub fn new(parser: P, f: F) -> Self {
        Self {
            parser,
            f,
            _phantom: PhantomData,
        }
    }
}

impl<I, A, B, P, F> Parser<I, B> for Map<P, F, A, B>
where
    P: Parser<I, A>,
    F: Fn(A) -> B,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<B> {
        self.parser
            .parse(input, pos)
            .map(|(pos, value)| (pos, (self.f)(value)))
    }
}

#[derive(Clone)]
pub struct AsUnit<P, O> {
    // Oを追加
    parser: P,
    _phantom: PhantomData<O>, // Oを保持
}

impl<P, O> AsUnit<P, O> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<I, P, O> Parser<I, ()> for AsUnit<P, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<()> {
        self.parser.parse(input, pos).map(|(pos, _)| (pos, ()))
    }
}

#[derive(Clone)]
pub struct Many<P, I, O> {
    parser: P,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> Many<P, I, O> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, P> Parser<I, Vec<O>> for Many<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Vec<O>> {
        let mut results = Vec::new();
        let mut current_pos = pos;

        while let Ok((new_pos, value)) = self.parser.parse(input, current_pos) {
            results.push(value);
            current_pos = new_pos;
        }

        Ok((current_pos, results))
    }
}

#[derive(Clone)]
pub struct Many1<P, I, O> {
    parser: P,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> Many1<P, I, O> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, P> Parser<I, Vec<O>> for Many1<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Vec<O>> {
        let (pos, first) = self.parser.parse(input, pos)?;
        let mut result = vec![first];
        let mut current_pos = pos;

        while let Ok((new_pos, value)) = self.parser.parse(input, current_pos) {
            result.push(value);
            current_pos = new_pos;
        }

        Ok((current_pos, result))
    }
}

pub struct SeparatedList<P, S, I, O> {
    item_parser: P,
    separator_parser: S,
    _phantom: PhantomData<(I, O)>,
}

impl<P, S, I, O> SeparatedList<P, S, I, O> {
    pub fn new(item_parser: P, separator_parser: S) -> Self {
        Self {
            item_parser,
            separator_parser,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, P, S> Parser<I, Vec<O>> for SeparatedList<P, S, I, O>
where
    I: Clone,
    P: Parser<I, O>,
    S: Parser<I, ()>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Vec<O>> {
        let mut results = Vec::new();
        let mut current_pos = pos;

        // 最初の要素をパース（失敗したら空のリストを返す）
        if let Ok((new_pos, value)) = self.item_parser.parse(input, current_pos) {
            results.push(value);
            current_pos = new_pos;

            // 残りの要素を繰り返しパース
            while let Ok((sep_pos, _)) = self.separator_parser.parse(input, current_pos) {
                current_pos = sep_pos;
                // カンマの後の要素をパース（失敗したら終了）
                if let Ok((new_pos, value)) = self.item_parser.parse(input, current_pos) {
                    results.push(value);
                    current_pos = new_pos;
                } else {
                    break;
                }
            }
        } else if let Ok((sep_pos, _)) = self.separator_parser.parse(input, current_pos) {
            // カンマのみの場合は位置を更新
            current_pos = sep_pos;
        }

        Ok((current_pos, results))
    }
}

#[derive(Clone)]
pub struct Optional<P, I, O> {
    parser: P,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> Optional<P, I, O> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, P> Parser<I, Option<O>> for Optional<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Option<O>> {
        match self.parser.parse(input, pos) {
            Ok((new_pos, value)) => Ok((new_pos, Some(value))),
            Err(_) => Ok((pos, None)),
        }
    }
}

#[derive(Clone)]
pub struct Tuple2<P1, P2, I, O1, O2> {
    parser1: P1,
    parser2: P2,
    _phantom: PhantomData<(I, O1, O2)>,
}

impl<P1, P2, I, O1, O2> Tuple2<P1, P2, I, O1, O2> {
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
            _phantom: PhantomData,
        }
    }
}

impl<P1, P2, I, O1, O2> Parser<I, (O1, O2)> for Tuple2<P1, P2, I, O1, O2>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<(O1, O2)> {
        let (pos, result1) = self.parser1.parse(input, pos)?;
        let (pos, result2) = self.parser2.parse(input, pos)?;
        Ok((pos, (result1, result2)))
    }
}

#[derive(Clone)]
pub struct Tuple3<P1, P2, P3, I, O1, O2, O3> {
    parser1: P1,
    parser2: P2,
    parser3: P3,
    _phantom: PhantomData<(I, O1, O2, O3)>,
}

impl<P1, P2, P3, I, O1, O2, O3> Tuple3<P1, P2, P3, I, O1, O2, O3> {
    pub fn new(parser1: P1, parser2: P2, parser3: P3) -> Self {
        Self {
            parser1,
            parser2,
            parser3,
            _phantom: PhantomData,
        }
    }
}

impl<P1, P2, P3, I, O1, O2, O3> Parser<I, (O1, O2, O3)> for Tuple3<P1, P2, P3, I, O1, O2, O3>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<(O1, O2, O3)> {
        let (pos, result1) = self.parser1.parse(input, pos)?;
        let (pos, result2) = self.parser2.parse(input, pos)?;
        let (pos, result3) = self.parser3.parse(input, pos)?;
        Ok((pos, (result1, result2, result3)))
    }
}

// Tuple4
#[derive(Clone)]
pub struct Tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4> {
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    _phantom: PhantomData<(I, O1, O2, O3, O4)>,
}

impl<P1, P2, P3, P4, I, O1, O2, O3, O4> Tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4> {
    pub fn new(parser1: P1, parser2: P2, parser3: P3, parser4: P4) -> Self {
        Self {
            parser1,
            parser2,
            parser3,
            parser4,
            _phantom: PhantomData,
        }
    }
}

impl<P1, P2, P3, P4, I, O1, O2, O3, O4> Parser<I, (O1, O2, O3, O4)>
    for Tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<(O1, O2, O3, O4)> {
        let (pos, result1) = self.parser1.parse(input, pos)?;
        let (pos, result2) = self.parser2.parse(input, pos)?;
        let (pos, result3) = self.parser3.parse(input, pos)?;
        let (pos, result4) = self.parser4.parse(input, pos)?;
        Ok((pos, (result1, result2, result3, result4)))
    }
}

// Tuple5
#[derive(Clone)]
pub struct Tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5> {
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    parser5: P5,
    _phantom: PhantomData<(I, O1, O2, O3, O4, O5)>,
}

impl<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5> Tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5> {
    pub fn new(parser1: P1, parser2: P2, parser3: P3, parser4: P4, parser5: P5) -> Self {
        Self {
            parser1,
            parser2,
            parser3,
            parser4,
            parser5,
            _phantom: PhantomData,
        }
    }
}

impl<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5> Parser<I, (O1, O2, O3, O4, O5)>
    for Tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
    P5: Parser<I, O5>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<(O1, O2, O3, O4, O5)> {
        let (pos, result1) = self.parser1.parse(input, pos)?;
        let (pos, result2) = self.parser2.parse(input, pos)?;
        let (pos, result3) = self.parser3.parse(input, pos)?;
        let (pos, result4) = self.parser4.parse(input, pos)?;
        let (pos, result5) = self.parser5.parse(input, pos)?;
        Ok((pos, (result1, result2, result3, result4, result5)))
    }
}

// Tuple6
#[derive(Clone)]
pub struct Tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6> {
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    parser5: P5,
    parser6: P6,
    _phantom: PhantomData<(I, O1, O2, O3, O4, O5, O6)>,
}

impl<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>
    Tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>
{
    pub fn new(
        parser1: P1,
        parser2: P2,
        parser3: P3,
        parser4: P4,
        parser5: P5,
        parser6: P6,
    ) -> Self {
        Self {
            parser1,
            parser2,
            parser3,
            parser4,
            parser5,
            parser6,
            _phantom: PhantomData,
        }
    }
}

impl<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6> Parser<I, (O1, O2, O3, O4, O5, O6)>
    for Tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
    P5: Parser<I, O5>,
    P6: Parser<I, O6>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<(O1, O2, O3, O4, O5, O6)> {
        let (pos, result1) = self.parser1.parse(input, pos)?;
        let (pos, result2) = self.parser2.parse(input, pos)?;
        let (pos, result3) = self.parser3.parse(input, pos)?;
        let (pos, result4) = self.parser4.parse(input, pos)?;
        let (pos, result5) = self.parser5.parse(input, pos)?;
        let (pos, result6) = self.parser6.parse(input, pos)?;
        Ok((pos, (result1, result2, result3, result4, result5, result6)))
    }
}

#[derive(Clone)]
pub struct Delimited<L, P, R, I, O> {
    left: L,
    parser: P,
    right: R,
    _phantom: PhantomData<(I, O)>,
}

impl<L, P, R, I, O> Delimited<L, P, R, I, O> {
    pub fn new(left: L, parser: P, right: R) -> Self {
        Self {
            left,
            parser,
            right,
            _phantom: PhantomData,
        }
    }
}

impl<I, O, L, P, R> Parser<I, O> for Delimited<L, P, R, I, O>
where
    L: Parser<I, ()>,
    P: Parser<I, O>,
    R: Parser<I, ()>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        let (pos, _) = self.left.parse(input, pos)?;
        let (pos, value) = self.parser.parse(input, pos)?;
        let (pos, _) = self.right.parse(input, pos)?;
        Ok((pos, value))
    }
}

#[derive(Clone)]
pub struct WithContext<P, C> {
    parser: P,
    context: C,
}

impl<P, C> WithContext<P, C> {
    pub fn new(parser: P, context: C) -> Self {
        Self { parser, context }
    }
}

impl<I, O, P, C: ToString> Parser<I, O> for WithContext<P, C>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        self.parser
            .parse(input, pos)
            .map_err(|e| ParseError::WithContext {
                message: self.context.to_string(),
                inner: Box::new(e),
            })
    }
}

#[derive(Clone)]
pub struct Lazy<F> {
    f: F,
}

impl<F> Lazy<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<I, O, F, P> Parser<I, O> for Lazy<F>
where
    F: Fn() -> P,
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        (self.f)().parse(input, pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        let input = vec![1, 2, 3, 4, 5];

        // 成功するケース
        let parser = Expected::new(Satisfy::new(|x: &i32| Some(*x)), 3);
        assert_eq!(parser.parse(&input, 2), Ok((3, 3)));

        // 失敗するケース (パース結果が一致しない)
        let parser = Expected::new(Satisfy::new(|x: &i32| Some(*x)), 4);
        assert_eq!(
            parser.parse(&input, 2),
            Err(ParseError::Fail(
                "parsed value does not equal to expected value".to_string()
            ))
        );

        // 失敗するケース (入力範囲外)
        let parser = Expected::new(Satisfy::new(|x: &i32| Some(*x)), 3);
        assert_eq!(parser.parse(&input, 5), Err(ParseError::EOF));

        let input = vec!['a', 'b', 'c', 'd'];

        // 成功するケース
        let parser = Expected::new(Satisfy::new(|x: &char| Some(*x)), 'c');
        assert_eq!(parser.parse(&input, 2), Ok((3, 'c')));

        // 失敗するケース (クロージャがNoneを返す)
        let parser = Expected::new(
            Satisfy::new(|x: &char| if *x == 'b' { None } else { Some(*x) }),
            'c',
        );
        assert_eq!(parser.parse(&input, 1), Err(ParseError::EOF));
    }

    #[test]
    fn test_identity() {
        let input = vec!['a', 'b', 'c'];

        // 成功するケース
        let parser = Identity::new();
        assert_eq!(parser.parse(&input, 0), Ok((1, 'a')));
        assert_eq!(parser.parse(&input, 1), Ok((2, 'b')));

        // 失敗するケース (入力範囲外)
        let parser = Identity::new();
        assert_eq!(parser.parse(&input, 3), Err(ParseError::EOF));
    }

    #[test]
    fn test_zero() {
        let input = vec![1, 2, 3];

        // 成功するケース
        let parser = Zero::new("hello");
        assert_eq!(parser.parse(&input, 0), Ok((0, "hello")));
        assert_eq!(parser.parse(&input, 2), Ok((2, "hello")));

        // 成功するケース (空入力)
        let input: Vec<i32> = vec![];
        let parser = Zero::new(42);
        assert_eq!(parser.parse(&input, 0), Ok((0, 42)));
    }

    #[test]
    fn test_fail() {
        let input = vec![1, 2, 3];

        // 常に失敗するケース
        let parser = Fail::<i32, i32>::new("error message");
        assert_eq!(
            parser.parse(&input, 0),
            Err(ParseError::Fail("error message".to_string()))
        );
        assert_eq!(
            parser.parse(&input, 2),
            Err(ParseError::Fail("error message".to_string()))
        );

        // 失敗するケース (空入力)
        let input: Vec<i32> = vec![];
        let parser = Fail::<i32, i32>::new("error message");
        assert_eq!(
            parser.parse(&input, 0),
            Err(ParseError::Fail("error message".to_string()))
        );
    }

    #[test]
    fn test_satisfy() {
        let input = vec![1, 2, 3, 4, 5];

        // 成功するケース (条件を満たす)
        let parser = Satisfy::new(|x: &i32| if *x % 2 == 0 { Some(*x) } else { None });
        assert_eq!(parser.parse(&input, 1), Ok((2, 2)));
        assert_eq!(parser.parse(&input, 3), Ok((4, 4)));

        // 失敗するケース (条件を満たさない)
        let parser = Satisfy::new(|x: &i32| if *x % 2 == 0 { Some(*x) } else { None });
        assert_eq!(parser.parse(&input, 0), Err(ParseError::EOF));
        assert_eq!(parser.parse(&input, 2), Err(ParseError::EOF));

        // 失敗するケース (入力範囲外)
        let parser = Satisfy::new(|x: &i32| Some(*x));
        assert_eq!(parser.parse(&input, 5), Err(ParseError::EOF));
    }
    #[test]
    fn test_choice() {
        let input = vec![1, 2, 3];
        let parser1 = Satisfy::new(|x: &i32| if *x == 1 { Some(*x) } else { None });
        let parser2 = Satisfy::new(|x: &i32| if *x == 2 { Some(*x) } else { None });
        let parser3 = Satisfy::new(|x: &i32| if *x == 3 { Some(*x) } else { None });

        // 成功するケース (最初のパーサーが成功)
        let choice_parser = Choice::new(vec![
            Box::new(parser1.clone()),
            Box::new(parser2.clone()),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(choice_parser.parse(&input, 0), Ok((1, 1)));

        // 成功するケース (2番目のパーサーが成功)
        let choice_parser = Choice::new(vec![
            Box::new(parser2.clone()),
            Box::new(parser1.clone()),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(choice_parser.parse(&input, 1), Ok((2, 2)));

        // 成功するケース (最後のパーサーが成功)
        let choice_parser = Choice::new(vec![
            Box::new(Fail::new("fail")),
            Box::new(Fail::new("fail")),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(choice_parser.parse(&input, 2), Ok((3, 3)));

        // 失敗するケース (すべてのパーサーが失敗)
        let choice_parser = Choice::new(vec![
            Box::new(Fail::<i32, i32>::new("fail")),
            Box::new(Fail::<i32, i32>::new("fail")),
            Box::new(Fail::<i32, i32>::new("fail")),
        ]);
        assert_eq!(
            choice_parser.parse(&input, 0),
            Err(ParseError::NoAlternative)
        );

        // 失敗するケース (入力範囲外)
        let choice_parser = Choice::new(vec![
            Box::new(parser1.clone()),
            Box::new(parser2.clone()),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(
            choice_parser.parse(&input, 3),
            Err(ParseError::NoAlternative)
        );
    }

    #[test]
    fn test_sequence() {
        let input = vec![1, 2, 3, 4];
        let parser1 = Satisfy::new(|x: &i32| if *x == 1 { Some(*x) } else { None });
        let parser2 = Satisfy::new(|x: &i32| if *x == 2 { Some(*x) } else { None });
        let parser3 = Satisfy::new(|x: &i32| if *x == 3 { Some(*x) } else { None });

        // 成功するケース
        let sequence_parser = Sequence::new(vec![
            Box::new(parser1.clone()),
            Box::new(parser2.clone()),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(sequence_parser.parse(&input, 0), Ok((3, vec![1, 2, 3])));

        // 失敗するケース (途中で失敗)
        let sequence_parser = Sequence::new(vec![
            Box::new(parser1.clone()),
            Box::new(Fail::new("fail")),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(
            sequence_parser.parse(&input, 0),
            Err(ParseError::Fail("fail".to_string()))
        );

        // 失敗するケース (入力範囲外)
        let sequence_parser = Sequence::new(vec![
            Box::new(parser1.clone()),
            Box::new(parser2.clone()),
            Box::new(parser3.clone()),
        ]);
        assert_eq!(sequence_parser.parse(&input, 2), Err(ParseError::EOF));
    }

    #[test]
    fn test_map() {
        let input = vec![1, 2, 3];
        let parser = Satisfy::new(|x: &i32| Some(*x));

        // 成功するケース
        let map_parser = Map::new(parser.clone(), |x| x * 2);
        assert_eq!(map_parser.parse(&input, 0), Ok((1, 2)));
        assert_eq!(map_parser.parse(&input, 2), Ok((3, 6)));

        // 失敗するケース (元のパーサーが失敗)
        let map_parser = Map::new(Fail::new("fail"), |x: i32| x * 2);
        assert_eq!(
            map_parser.parse(&input, 0),
            Err(ParseError::Fail("fail".to_string()))
        );

        // 失敗するケース (入力範囲外)
        let map_parser = Map::new(parser.clone(), |x| x * 2);
        assert_eq!(map_parser.parse(&input, 3), Err(ParseError::EOF));
    }

    #[test]
    fn test_many() {
        let input = vec![1, 1, 1, 2, 3];
        let parser = Satisfy::new(|x: &i32| if *x == 1 { Some(*x) } else { None });

        // 成功するケース (複数回成功)
        let many_parser = Many::new(parser.clone());
        assert_eq!(many_parser.parse(&input, 0), Ok((3, vec![1, 1, 1])));

        // 成功するケース (0回成功)
        let many_parser = Many::new(parser.clone());
        assert_eq!(many_parser.parse(&input, 3), Ok((3, vec![])));

        // 失敗しない (入力範囲外でも空のベクタを返す)
        let many_parser = Many::new(parser.clone());
        assert_eq!(many_parser.parse(&input, 5), Ok((5, vec![])));
    }

    #[test]
    fn test_many1() {
        let input = vec![1, 1, 1, 2, 3];
        let parser = Satisfy::new(|x: &i32| if *x == 1 { Some(*x) } else { None });

        // 成功するケース (複数回成功)
        let many1_parser = Many1::new(parser.clone());
        assert_eq!(many1_parser.parse(&input, 0), Ok((3, vec![1, 1, 1])));

        // 失敗するケース (0回成功)
        let many1_parser = Many1::new(parser.clone());
        assert_eq!(many1_parser.parse(&input, 3), Err(ParseError::EOF));

        // 失敗するケース (入力範囲外)
        let many1_parser = Many1::new(parser.clone());
        assert_eq!(many1_parser.parse(&input, 5), Err(ParseError::EOF));
    }
    #[test]
    fn test_separated_list() {
        let item_parser = Satisfy::new(|x: &char| if *x != ',' { Some(*x) } else { None });
        let separator_parser = Satisfy::new(|x: &char| if *x == ',' { Some(()) } else { None });
        let parser = SeparatedList::new(item_parser, separator_parser);

        // Case 1: 空のリスト "[]" -> OK
        let input: Vec<char> = vec![];
        assert_eq!(parser.parse(&input, 0), Ok((0, vec![])));

        // Case 2: 単一要素 "[a]" -> OK
        let input: Vec<char> = vec!['a'];
        assert_eq!(parser.parse(&input, 0), Ok((1, vec!['a'])));

        // Case 3: 複数要素 "[a,b,c]" -> OK
        let input: Vec<char> = vec!['a', ',', 'b', ',', 'c'];
        assert_eq!(parser.parse(&input, 0), Ok((5, vec!['a', 'b', 'c'])));

        // Case 4: 末尾カンマあり "[a,b,]" -> OK
        let input: Vec<char> = vec!['a', ',', 'b', ','];
        assert_eq!(parser.parse(&input, 0), Ok((4, vec!['a', 'b'])));

        // Case 5: カンマのみ "[,]" -> OK (空のリストとして扱う)
        let input: Vec<char> = vec![','];
        assert_eq!(parser.parse(&input, 0), Ok((1, vec![])));
    }

    #[test]
    fn test_optional() {
        let input = vec![1, 2, 3];
        let parser = Satisfy::new(|x: &i32| if *x == 1 { Some(*x) } else { None });

        // 成功するケース (パーサーが成功)
        let optional_parser = Optional::new(parser.clone());
        assert_eq!(optional_parser.parse(&input, 0), Ok((1, Some(1))));

        // 成功するケース (パーサーが失敗)
        let optional_parser = Optional::new(parser.clone());
        assert_eq!(optional_parser.parse(&input, 1), Ok((1, None)));

        // 成功するケース (入力範囲外)
        let optional_parser = Optional::new(parser.clone());
        assert_eq!(optional_parser.parse(&input, 3), Ok((3, None)));
    }

    #[test]
    fn test_tuple3() {
        let input = vec![1, 2, 3, 4];
        let parser1 = Satisfy::new(|x: &i32| if *x == 1 { Some(*x) } else { None });
        let parser2 = Satisfy::new(|x: &i32| if *x == 2 { Some(*x) } else { None });
        let parser3 = Satisfy::new(|x: &i32| if *x == 3 { Some(*x) } else { None });

        // 成功するケース
        let sequence3_parser = Tuple3::new(parser1.clone(), parser2.clone(), parser3.clone());
        assert_eq!(sequence3_parser.parse(&input, 0), Ok((3, (1, 2, 3))));

        // 失敗するケース (途中で失敗)
        let sequence3_parser = Tuple3::new(
            parser1.clone(),
            Fail::<i32, i32>::new("fail"),
            parser3.clone(),
        );
        assert_eq!(
            sequence3_parser.parse(&input, 0),
            Err(ParseError::Fail("fail".to_string()))
        );

        // 失敗するケース (入力範囲外)
        let sequence3_parser = Tuple3::new(parser1.clone(), parser2.clone(), parser3.clone());
        assert_eq!(sequence3_parser.parse(&input, 2), Err(ParseError::EOF));
    }

    #[test]
    fn test_delimited() {
        let input = vec!['(', '1', ')', '(', '2', ')'];
        let left = Satisfy::new(|x: &char| if *x == '(' { Some(()) } else { None });
        let parser = Satisfy::new(|x: &char| {
            if x.is_digit(10) {
                x.to_digit(10).map(|n| n as i32)
            } else {
                None
            }
        });
        let right = Satisfy::new(|x: &char| if *x == ')' { Some(()) } else { None });

        // 成功するケース
        let delimited_parser = Delimited::new(left.clone(), parser.clone(), right.clone());
        assert_eq!(delimited_parser.parse(&input, 0), Ok((3, 1)));

        // 失敗するケース (左デリミタが失敗)
        let delimited_parser = Delimited::new(Fail::new("fail"), parser.clone(), right.clone());
        assert_eq!(
            delimited_parser.parse(&input, 0),
            Err(ParseError::Fail("fail".to_string()))
        );

        // 失敗するケース (パーサーが失敗)
        let delimited_parser =
            Delimited::new(left.clone(), Fail::<char, i32>::new("fail"), right.clone());
        assert_eq!(
            delimited_parser.parse(&input, 0),
            Err(ParseError::Fail("fail".to_string()))
        );

        // 失敗するケース (右デリミタが失敗)
        let delimited_parser = Delimited::new(left.clone(), parser.clone(), Fail::new("fail"));
        assert_eq!(
            delimited_parser.parse(&input, 0),
            Err(ParseError::Fail("fail".to_string()))
        );

        // 失敗するケース (入力範囲外)
        let delimited_parser = Delimited::new(left.clone(), parser.clone(), right.clone());
        assert_eq!(delimited_parser.parse(&input, 6), Err(ParseError::EOF));
    }
}
