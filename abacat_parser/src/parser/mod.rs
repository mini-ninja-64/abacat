pub mod parser;

// use std::ops::Range;

// use ariadne::{Cache, sources};
// use abacat_common::ariadne::generate_reports;

// use crate::{Spanned, lexer};

// pub fn parse<'src>(tokens: Vec<crate::Spanned<&Token<'src>>>) {}

// const SOURCE_NAME: &str = "";

// pub type IdentId = usize;
// pub type ExpressionId = usize;
// pub struct ReplParser {
//     expressions: Vec<String>,
//     sources: Vec<(IdentId, ExpressionId, Range<usize>)>,
// }

// pub struct ExpressionStore<I>(Vec<(I, String)>);

// impl ReplParser {
//     pub fn new() -> ReplParser {
//         ReplParser {
//             expressions: vec![],
//             sources: vec![],
//         }
//     }
//     pub fn parse(&'_ mut self, statement: String) -> Result<Spanned<parser::Expr<'_>>, String> {
//         self.expressions.push(statement);

//         let statement = self.expressions.last().unwrap();
//         let lex_result = lexer::lex(statement);
//         if lex_result.has_errors() {
//             for report in generate_reports(SOURCE_NAME, lex_result.errors()) {
//                 let cache = sources(vec![(SOURCE_NAME, statement)]);
//                 report.eprint(cache).unwrap();
//             }
//             return Err("problem lexing :c".to_owned());
//         }
//         let lex_result = lex_result.unwrap();
//         todo!()
//     }
// }
