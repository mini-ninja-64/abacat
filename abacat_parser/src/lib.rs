use std::ops::Range;

use abacat_common::error::{BasicFailure, LocatedFailure, Spanned};
use chumsky::{Parser, input::Input, span::SimpleSpan};

use crate::{lexer::token::Token, parser::parser::Expr};

pub mod lexer;
pub mod parser;

const EMPTY_STRING: &String = &String::new();

#[derive(Debug)]
pub enum ParsingError {
    LexError(Vec<BasicFailure<String>>),
    ParseError(Vec<BasicFailure<String>>),
    Empty(Range<usize>),
}

pub type ParserResult = Result<Spanned<Expr>, ParsingError>;

pub fn parse<'a>(expr: &'a str) -> ParserResult {
    let lex_result: chumsky::ParseResult<
        Vec<(lexer::token::Token<'a>, SimpleSpan)>,
        chumsky::prelude::Rich<'a, char>,
    > = lexer::lex(expr);

    if expr.len() == 0 || expr.chars().all(|x| x.is_whitespace()) {
        return Err(ParsingError::Empty(0..expr.len()));
    }
    if lex_result.has_errors() {
        return Err(ParsingError::LexError(
            lex_result
                .into_errors()
                .into_iter()
                .map(|e| BasicFailure::new(e.span().into_range(), e.reason().to_string()))
                .collect(),
        ));
    }

    let a: Vec<(Token<'a>, SimpleSpan)> = lex_result.unwrap();
    let result = crate::parser::parser::expr()
        .map_with(|e, i| (e, i.span()))
        .parse(a.map((expr.len()..expr.len()).into(), |(t, s)| (t, s)));
    if result.has_errors() {
        return Err(ParsingError::ParseError(
            result
                .errors()
                .map(|e| BasicFailure::new(e.span().into_range(), e.reason().to_string()))
                .collect(),
        ));
    }
    let (result, _) = result.unwrap();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_doesnt_work() {
        let r = parse("12.34  =+a= d= =d daf").unwrap_err();
        println!("{:?}", r);
    }

    #[test]
    fn it_works() {
        let r = parse("12.34+45+0xff+0b100+0o123").unwrap();
        println!("{:?}", r);
    }
}
