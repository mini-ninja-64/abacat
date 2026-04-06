use abacat_common::error::SpanChumsky;
use chumsky::{ParseResult, Parser};

pub mod token;

pub fn lex(
    expr: &str,
) -> ParseResult<Vec<(token::Token<'_>, SpanChumsky)>, chumsky::prelude::Rich<'_, char>> {
    return token::lexer().parse(expr);
}
