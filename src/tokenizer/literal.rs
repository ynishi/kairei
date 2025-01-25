use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1},
    combinator::{map_res, opt, recognize},
    sequence::{pair, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Interpolation(String),
    NewLine,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(Vec<StringPart>),
    Integer(i64),
    Float(f64),
}

// 文字列内の${...}形式の補間部分をパース
fn interpolation(input: &str) -> IResult<&str, StringPart> {
    let (input, _) = tag("${")(input)?;
    let (input, ident) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, StringPart::Interpolation(ident.to_string())))
}

// 改行をパース
fn newline(input: &str) -> IResult<&str, StringPart> {
    let (input, _) = alt((tag("\r\n"), tag("\n")))(input)?;
    Ok((input, StringPart::NewLine))
}

// 文字列リテラルの通常部分をパース
fn string_literal_part(input: &str) -> IResult<&str, StringPart> {
    let (input, content) = take_until1(|c| c == '$' || c == '\n' || c == '\r' || c == '"')(input)?;
    Ok((input, StringPart::Literal(content.to_string())))
}

// すべての文字列をパース
fn string_literal(input: &str) -> IResult<&str, Literal> {
    let (input, _) = char('"')(input)?;
    let mut parts = Vec::new();
    let mut current_input = input;

    while !current_input.starts_with('"') {
        let result = alt((interpolation, newline, string_literal_part))(current_input);

        match result {
            Ok((remaining, part)) => {
                parts.push(part);
                current_input = remaining;
            }
            Err(_) => break,
        }
    }

    let (remaining, _) = char('"')(current_input)?;
    Ok((remaining, Literal::String(parts)))
}

// 浮動小数点数をパース
fn float_literal(input: &str) -> IResult<&str, Literal> {
    map_res(
        recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
        |s: &str| s.parse::<f64>().map(Literal::Float),
    )(input)
}

// 整数をパース
fn integer_literal(input: &str) -> IResult<&str, Literal> {
    map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| {
        s.parse::<i64>().map(Literal::Integer)
    })(input)
}

// リテラル全体のパーサー
pub fn literal(input: &str) -> IResult<&str, Literal> {
    alt((string_literal, float_literal, integer_literal))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = "\"hello world\"";
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![StringPart::Literal("hello world".to_string()),])
        );
    }

    #[test]
    fn test_string_with_interpolation() {
        let input = "\"hello ${name}\"";
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![
                StringPart::Literal("hello ".to_string()),
                StringPart::Interpolation("name".to_string()),
            ])
        );
    }

    #[test]
    fn test_multiline_string() {
        let input = "\"line one\nline two\nline three\"";
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![
                StringPart::Literal("line one".to_string()),
                StringPart::NewLine,
                StringPart::Literal("line two".to_string()),
                StringPart::NewLine,
                StringPart::Literal("line three".to_string()),
            ])
        );
    }

    #[test]
    fn test_complex_string() {
        let input = "\"Hello ${name},\nYour total is: ${amount}\"";
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![
                StringPart::Literal("Hello ".to_string()),
                StringPart::Interpolation("name".to_string()),
                StringPart::Literal(",".to_string()),
                StringPart::NewLine,
                StringPart::Literal("Your total is: ".to_string()),
                StringPart::Interpolation("amount".to_string()),
            ])
        );
    }

    #[test]
    fn test_number_literals() {
        // Integer
        let (rest, result) = integer_literal("123").unwrap();
        assert_eq!(result, Literal::Integer(123));
        assert_eq!(rest, "");

        // Negative integer
        let (rest, result) = integer_literal("-123").unwrap();
        assert_eq!(result, Literal::Integer(-123));
        assert_eq!(rest, "");

        // Float
        let (rest, result) = float_literal("123.45").unwrap();
        assert_eq!(result, Literal::Float(123.45));
        assert_eq!(rest, "");

        // Negative float
        let (rest, result) = float_literal("-123.45").unwrap();
        assert_eq!(result, Literal::Float(-123.45));
        assert_eq!(rest, "");
    }
}

// 文字が $, \n, \r, " のいずれかになるまで文字列を取得
fn take_until1<'a>(
    terminals: impl Fn(char) -> bool,
) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let mut index = 0;
        let mut chars = input.chars();

        while let Some(ch) = chars.next() {
            if terminals(ch) {
                break;
            }
            index += ch.len_utf8();
        }

        if index == 0 {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TakeUntil,
            )))
        } else {
            Ok((&input[index..], &input[..index]))
        }
    }
}
