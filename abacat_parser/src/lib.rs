use std::ops::Range;

use abacat_common::error::{BasicFailure, LocatableFailure, SpannedChumsky};
use chumsky::{Parser, input::Input, span::SimpleSpan};

use crate::{lexer::token::Token, parser::parser::Expr};

pub mod lexer;
pub mod parser;

// TODO: Should update to handle contexts nicely or something, this will do for now
#[derive(Debug)]
pub enum ParsingError {
    LexError(BasicFailure<String>),
    ParseError(BasicFailure<String>),
    Empty(Range<usize>),
    Unknown,
}

impl<'a> LocatableFailure<'a, &'a str> for ParsingError {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            ParsingError::LexError(basic_failure) => basic_failure.span(),
            ParsingError::ParseError(basic_failure) => basic_failure.span(),
            ParsingError::Empty(range) => Some(range.clone()),
            ParsingError::Unknown => None,
        }
    }

    fn message(&'a self) -> &'a str {
        match self {
            ParsingError::LexError(basic_failure) => basic_failure.message(),
            ParsingError::ParseError(basic_failure) => basic_failure.message(),
            ParsingError::Empty(_) => "Tried to parse an empty expression",
            ParsingError::Unknown => "Unknown error occurred",
        }
    }
}

pub type ParserResult = Result<SpannedChumsky<Expr>, ParsingError>;

pub fn parse<'a>(expr: &'a str) -> ParserResult {
    let lex_result: chumsky::ParseResult<
        Vec<(lexer::token::Token<'a>, SimpleSpan)>,
        chumsky::prelude::Rich<'a, char>,
    > = lexer::lex(expr);

    if expr.len() == 0 || expr.chars().all(|x| x.is_whitespace()) {
        return Err(ParsingError::Empty(0..expr.len()));
    }
    if lex_result.has_errors() {
        let err = lex_result.errors().next().unwrap();

        return Err(ParsingError::LexError(BasicFailure::new(
            err.span().into_range(),
            err.reason().to_string(),
        )));
    }

    let a: Vec<(Token<'a>, SimpleSpan)> = lex_result.unwrap();
    let result = crate::parser::parser::expr()
        .map_with(|e, i| (e, i.span()))
        .parse(a.map((expr.len()..expr.len()).into(), |(t, s)| (t, s)));
    if result.has_errors() {
        let err = result.errors().next().unwrap();

        return Err(ParsingError::ParseError(BasicFailure::new(
            err.span().into_range(),
            err.reason().to_string(),
        )));
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
    fn wierd() {
        let r = parse("1 / true").unwrap();
        println!("{:?}", r);
    }

    #[test]
    fn it_works() {
        let r = parse("12.34+45+0xff+0b100+0o123").unwrap();
        println!("{:?}", r);
    }
}
