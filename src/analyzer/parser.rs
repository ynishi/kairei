use std::marker::PhantomData;
use thiserror::Error;

// 基本的なパーサー型
pub type ParseResult<O> = Result<(usize, O), ParserError>;

// パーサートレイト
pub trait Parser<I, O> {
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O>;
}

// パーサーコンビネータの基本実装
impl<I, O, F> Parser<I, O> for F
where
    F: Fn(&[I], usize) -> ParseResult<O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O> {
        self(input, pos)
    }
}

// 基本的なコンビネータ関数群
#[derive(Debug, Clone, PartialEq)]
pub struct ParserCombinator<I: PartialEq + Clone, O: PartialEq + Clone>(PhantomData<(I, O)>);

impl<I: PartialEq + Clone, O: PartialEq + Clone> ParserCombinator<I, O> {
    pub fn satisfy<F>(f: F) -> impl Parser<I, O>
    where
        F: Fn(&I) -> Option<O>,
    {
        move |input: &[I], pos: usize| {
            input
                .get(pos)
                .and_then(|x| f(x).map(|result| (pos + 1, result)))
                .ok_or(ParserError::EOF)
        }
    }

    pub fn token(token: O) -> impl Parser<I, O>
    where
        I: PartialEq<O>,
        O: Clone,
    {
        Self::satisfy(move |input| {
            if input == &token {
                Some(token.clone())
            } else {
                None
            }
        })
    }

    pub fn many<F>(parser: F) -> impl Parser<I, Vec<O>>
    where
        F: Parser<I, O>,
    {
        move |input: &[I], pos: usize| {
            let mut result = Vec::new();
            let mut pos = pos;
            while let Ok((new_pos, value)) = parser.parse(input, pos) {
                result.push(value);
                pos = new_pos;
            }
            Ok((pos, result))
        }
    }

    pub fn many1<F>(parser: F) -> impl Parser<I, Vec<O>>
    where
        F: Parser<I, O>,
    {
        move |input: &[I], pos: usize| {
            let (pos, value) = parser.parse(input, pos)?;
            let mut result = vec![value];
            let mut pos = pos;
            while let Ok((new_pos, value)) = parser.parse(input, pos) {
                result.push(value);
                pos = new_pos;
            }
            Ok((pos, result))
        }
    }

    pub fn choice<F>(parsers: Vec<F>) -> impl Parser<I, O>
    where
        F: Parser<I, O>,
    {
        move |input: &[I], pos: usize| {
            for parser in &parsers {
                if let Ok(result) = parser.parse(input, pos) {
                    return Ok(result);
                }
            }
            Err(ParserError::NoAlternative)
        }
    }

    pub fn sequence<F1, F2, O1, O2>(parser1: F1, parser2: F2) -> impl Parser<I, (O1, O2)>
    where
        F1: Parser<I, O1>,
        F2: Parser<I, O2>,
    {
        move |input: &[I], pos: usize| {
            let (pos, value1) = parser1.parse(input, pos)?;
            let (pos, value2) = parser2.parse(input, pos)?;
            Ok((pos, (value1, value2)))
        }
    }
    pub fn sequence3<P1, P2, P3, O1, O2, O3>(
        parser1: P1,
        parser2: P2,
        parser3: P3,
    ) -> impl Parser<I, (O1, O2, O3)>
    where
        P1: Parser<I, O1>,
        P2: Parser<I, O2>,
        P3: Parser<I, O3>,
    {
        move |input: &[I], pos: usize| {
            let (pos, value1) = parser1.parse(input, pos)?;
            let (pos, value2) = parser2.parse(input, pos)?;
            let (pos, value3) = parser3.parse(input, pos)?;
            Ok((pos, (value1, value2, value3)))
        }
    }

    pub fn map<P, F, O1, O2>(parser: P, f: F) -> impl Parser<I, O2>
    where
        P: Parser<I, O1>,
        F: Fn(O1) -> O2,
        O1: Clone, // 必要に応じて
        O2: Clone, // 必要に応じて
    {
        move |input: &[I], pos: usize| {
            let (new_pos, value) = parser.parse(input, pos)?;
            Ok((new_pos, f(value)))
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParserError {
    #[error("Parse error: {message}")]
    ParseError {
        message: String,
        found: String,
        position: (usize, usize),
    },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("EOF")]
    EOF,
    #[error("Unexpected")]
    Unexpected,
    #[error("No alternative")]
    NoAlternative,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 入力トークン
    #[derive(Debug, Clone, PartialEq)]
    enum TestToken {
        Number(i32),
        Plus,
        Minus,
        LeftParen,
        RightParen,
        Error,
    }

    // パース結果の型
    #[derive(Debug, Clone, PartialEq)]
    enum Expr {
        Number(i32),
        BinaryOp(Box<Expr>, Op, Box<Expr>),
        Paren(Box<Expr>),
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Op {
        Plus,
        Minus,
    }

    #[test]
    fn test_basic_parsing() {
        let input = vec![
            TestToken::Number(42),
            TestToken::Plus,
            TestToken::Number(10),
        ];

        // 数値のパース
        let number_parser = ParserCombinator::<TestToken, Expr>::satisfy(|token| match token {
            TestToken::Number(n) => Some(Expr::Number(*n)),
            _ => None,
        });

        let numeber_parser_outer =
            ParserCombinator::<TestToken, Expr>::satisfy(|token| match token {
                TestToken::Number(n) => Some(Expr::Number(*n)),
                _ => None,
            });

        // 演算子のパース
        let op_parser = ParserCombinator::<TestToken, Op>::satisfy(|token| match token {
            TestToken::Plus => Some(Op::Plus),
            TestToken::Minus => Some(Op::Minus),
            _ => None,
        });

        // シーケンスのテスト
        let expr_parser = ParserCombinator::<TestToken, (Expr, (Op, Expr))>::sequence(
            numeber_parser_outer,
            ParserCombinator::<TestToken, (Op, Expr)>::sequence(op_parser, number_parser),
        );

        let result = expr_parser.parse(&input, 0);
        assert!(result.is_ok());
        let (pos, (first, (op, second))) = result.unwrap();
        assert_eq!(pos, 3);
        assert_eq!(first, Expr::Number(42));
        assert_eq!(op, Op::Plus);
        assert_eq!(second, Expr::Number(10));
    }

    #[test]
    fn test_many() {
        let input = vec![
            TestToken::Number(1),
            TestToken::Number(2),
            TestToken::Number(3),
            TestToken::Plus,
        ];

        let number_parser = ParserCombinator::<TestToken, Expr>::satisfy(|token| match token {
            TestToken::Number(n) => Some(Expr::Number(*n)),
            _ => None,
        });

        let many_numbers = ParserCombinator::many(number_parser);
        let result = many_numbers.parse(&input, 0);

        assert!(result.is_ok());
        let (pos, numbers) = result.unwrap();
        assert_eq!(pos, 3);
        assert_eq!(numbers.len(), 3);
    }

    #[test]
    fn test_choice() {
        let input = vec![TestToken::Plus, TestToken::Minus];

        let op_parser = ParserCombinator::<TestToken, Op>::satisfy(|token| match token {
            TestToken::Plus => Some(Op::Plus),
            TestToken::Minus => Some(Op::Minus),
            _ => None,
        });

        let parsers = vec![op_parser];
        let choice_parser = ParserCombinator::<TestToken, _>::choice(parsers);

        let result = choice_parser.parse(&input, 0);
        assert!(result.is_ok());
        let (pos, op) = result.unwrap();
        assert_eq!(pos, 1);
        assert_eq!(op, Op::Plus);
    }

    #[test]
    fn test_complex_sequence() {
        let input = vec![
            TestToken::Number(1),
            TestToken::Plus,
            TestToken::Number(2),
            TestToken::Plus,
            TestToken::Number(3),
        ];

        // 数値パーサー
        let number = ParserCombinator::satisfy(|token| match token {
            TestToken::Number(n) => Some(*n),
            _ => None,
        });

        let number_outer = ParserCombinator::satisfy(|token| match token {
            TestToken::Number(n) => Some(*n),
            _ => None,
        });

        // 演算子パーサー
        let operator = ParserCombinator::satisfy(|token| match token {
            TestToken::Plus => Some(Op::Plus),
            _ => None,
        });

        // 式パーサー: number (operator number)*
        let expr = ParserCombinator::<TestToken, (Expr, (Op, Expr))>::sequence(
            number_outer,
            ParserCombinator::many(ParserCombinator::<TestToken, (Op, Expr)>::sequence(
                operator, number,
            )),
        );

        let result = expr.parse(&input, 0);
        assert!(result.is_ok());
        let (pos, (first, operations)) = result.unwrap();
        assert_eq!(pos, 5); // 全てのトークンを消費
        assert_eq!(first, 1);
        assert_eq!(operations, vec![(Op::Plus, 2), (Op::Plus, 3)]);
    }
}
