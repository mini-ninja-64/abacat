use abacat_common::ariadne::generate_reports;
use ariadne::sources;
use chumsky::{Parser, input::Input, span::SimpleSpan};

use crate::parser::parser::Expr;

pub mod lexer;
pub mod parser;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

// TODO: Proper error types in a way we can still get the ariadne info
pub fn parse<'a>(expr: &'a str) -> Result<(Expr, SimpleSpan), String> {
    let _source_name = "";

    // if expr.chars().all(|c| c.is_whitespace()) {
    //     return Ok((Expr::Nothing, SimpleSpan::new(0usize, expr.len())));
    // }

    let lex_result: chumsky::ParseResult<
        Vec<(lexer::token::Token<'_>, SimpleSpan)>,
        chumsky::prelude::Rich<'_, char>,
    > = lexer::lex(expr);

    if lex_result.has_errors() {
        // for report in generate_reports(source_name, lex_result.errors()) {
        //     let cache = sources(vec![(source_name, expr)]);
        //     report.eprint(cache).unwrap();
        // }
        return Err("problem lexing :c".to_owned());
    }

    let a = lex_result.unwrap();
    let result = crate::parser::parser::expr()
        .map_with(|e, i| (e, i.span()))
        .parse(
            a.as_slice()
                .map((expr.len()..expr.len()).into(), |(t, s)| (t, s)),
        );
    if result.has_errors() {
        // for report in generate_reports(source_name, result.errors()) {
        //     let cache = sources(vec![(source_name, expr)]);
        //     report.eprint(cache).unwrap();
        // }
        return Err("problem parsing :c".to_owned());
    }
    let (result, _) = result.unwrap();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_works() {
        // let x = parse("45+12-35*(88+*2)");
        let r = parse("").unwrap();
        println!("{:?}", r);
        // for report in generate_reports(source_name, result.errors()) {
        //     let cache = sources(vec![(source_name, expr)]);
        //     report.eprint(cache).unwrap();
        // }
    }

    #[test]
    fn it_works() {
        // let x = parse("45+12-35*(88+*2)");
        let r = parse("12.34+45+0xff+0b100+0123").unwrap();
        println!("{:?}", r);
        // for report in generate_reports(source_name, result.errors()) {
        //     let cache = sources(vec![(source_name, expr)]);
        //     report.eprint(cache).unwrap();
        // }
    }
}
