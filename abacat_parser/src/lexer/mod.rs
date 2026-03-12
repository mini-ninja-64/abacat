use chumsky::{ParseResult, Parser};

pub mod token;

pub fn lex(
    expr: &str,
) -> ParseResult<Vec<(token::Token<'_>, crate::Span)>, chumsky::prelude::Rich<'_, char>> {
    return token::lexer().parse(expr);
}
