use abacat_common::error::SpanChumsky;
use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::ValueInput,
    pratt::{infix, left, postfix, prefix, right},
    prelude::{choice, just},
    recursive::recursive,
    select,
};
use derive_more::Display;
use rust_decimal::Decimal;

use crate::{SpannedChumsky, lexer::token::Token};

#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub enum UnaryOp {
    ExclamationMark,
    Minus,
}
#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    IntDivide,
    Equal,
    EqualEqual,
    AndAnd,
    OrOr,
    Pipe,
}

// TODO: Interning, complex cos repl model means history can change
//       dont want mem leako
pub type Ident = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Base10Decimal(Decimal),
    Base10Num(u64),
    Base16Num(u64),
    Base2Num(u64),
    Base8Num(u64),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub args: SpannedChumsky<Vec<SpannedChumsky<Ident>>>,
    pub body: Box<SpannedChumsky<Expr>>,
}
impl Function {
    pub fn new<'a>(
        args: SpannedChumsky<Vec<SpannedChumsky<Ident>>>,
        body: Box<SpannedChumsky<Expr>>,
    ) -> Function {
        Function { args, body }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Binary(
        Box<SpannedChumsky<Self>>,
        BinaryOp,
        Box<SpannedChumsky<Self>>,
    ),
    Unary(UnaryOp, Box<SpannedChumsky<Self>>),
    Call(
        Box<SpannedChumsky<Self>>,
        SpannedChumsky<Vec<SpannedChumsky<Self>>>,
    ),
    Ident(Ident),
    Literal(Literal),
    List(Vec<SpannedChumsky<Self>>),
    IndexedExpr(Box<SpannedChumsky<Self>>, Box<SpannedChumsky<Self>>),
    Parenthesised(Box<SpannedChumsky<Self>>),
    NamedFunction(SpannedChumsky<Ident>, SpannedChumsky<Function>),
    AnonymousFunction(SpannedChumsky<Function>),
}

pub fn expr<'tokens, I>()
-> impl Parser<'tokens, I, SpannedChumsky<Expr>, extra::Err<Rich<'tokens, Token<'tokens>, SpanChumsky>>>
+ Clone
where
    I: ValueInput<'tokens, Token = Token<'tokens>, Span = SpanChumsky>,
{
    let ident = select! {Token::Ident(i) => i.to_string()}.map_with(|id, e| (id, e.span()));
    let literal = select! {
        Token::Bool(x) => Literal::Bool(x),
        Token::Base2Num(n) => Literal::Base2Num(n),
        Token::Base8Num(n) => Literal::Base8Num(n),
        Token::Base10Decimal(n) => Literal::Base10Decimal(n),
        Token::Base10Num(n) => Literal::Base10Num(n),
        Token::Base16Num(n) => Literal::Base16Num(n),
    };

    // TODO: Fix binding power, i think some mistakes, will be uncovered by testing
    recursive(|expr| {
        let ident_expr = ident
            .map(|(i, _)| Expr::Ident(i))
            .labelled("identifier")
            .as_context();
        let literal_expr = literal
            .map(|v| Expr::Literal(v))
            .labelled("literal")
            .as_context();
        let parenthesised_expr = expr
            .clone()
            .delimited_by(just(Token::LeftParens), just(Token::RightParens))
            .map(|expr| Expr::Parenthesised(Box::new(expr)))
            .labelled("parenthesised expression")
            .as_context();
        let list_expr = expr
            .clone()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(
                just(Token::LeftSquareBracket),
                just(Token::RightSquareBracket),
            )
            .map(Expr::List)
            .labelled("list expression")
            .as_context();
        let index_expr = expr
            .clone()
            .delimited_by(
                just(Token::LeftSquareBracket),
                just(Token::RightSquareBracket),
            )
            .labelled("index expression");
        let args_def = ident
            .clone()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LeftParens), just(Token::RightParens))
            .map_with(|args, e| (args, e.span()));

        let named_function = just(Token::Def)
            .ignore_then(ident)
            .then(
                args_def
                    .clone()
                    .then_ignore(just(Token::Equal))
                    .then(expr.clone())
                    .map_with(|(args, body), e| (Function::new(args, Box::new(body)), e.span())),
            )
            .map(|(name, func)| Expr::NamedFunction(name, func))
            .labelled("named function")
            .as_context();

        let anonymous_function = args_def
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map_with(|(args, body), e| {
                Expr::AnonymousFunction((Function::new(args, Box::new(body)), e.span()))
            })
            .labelled("anonymous function")
            .as_context();

        let expr_args = expr
            .clone()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LeftParens), just(Token::RightParens))
            .map_with(|args, e| (args, e.span()))
            .labelled("parenthesised arguments list");

        choice((
            literal_expr,
            ident_expr,
            anonymous_function,
            named_function,
            list_expr,
            parenthesised_expr,
        ))
        .map_with(|i, e| (i, e.span()))
        .pratt((
            postfix(5, index_expr, |left, index, e| {
                (Expr::IndexedExpr(Box::new(left), Box::new(index)), e.span())
            }),
            postfix(5, expr_args, |left, args, e| {
                (Expr::Call(Box::new(left), args), e.span())
            }),
            prefix(4, just(Token::Minus), |_, r, e| {
                (Expr::Unary(UnaryOp::Minus, Box::new(r)), e.span())
            }),
            prefix(4, just(Token::ExclamationMark), |_, r, e| {
                (Expr::Unary(UnaryOp::ExclamationMark, Box::new(r)), e.span())
            }),
            infix(left(0), just(Token::Pipe), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::Pipe, Box::new(r)),
                    e.span(),
                )
            }),
            infix(left(0), just(Token::Equal), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::Equal, Box::new(r)),
                    e.span(),
                )
            }),
            infix(left(1), just(Token::EqualEqual), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::EqualEqual, Box::new(r)),
                    e.span(),
                )
            }),
            infix(left(1), just(Token::AndAnd), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::AndAnd, Box::new(r)),
                    e.span(),
                )
            }),
            infix(left(1), just(Token::OrOr), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::OrOr, Box::new(r)),
                    e.span(),
                )
            }),
            infix(left(2), just(Token::Plus), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::Plus, Box::new(r)),
                    e.span(),
                )
            }),
            infix(left(2), just(Token::Minus), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::Minus, Box::new(r)),
                    e.span(),
                )
            }),
            infix(right(3), just(Token::Multiply), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::Multiply, Box::new(r)),
                    e.span(),
                )
            }),
            infix(right(3), just(Token::Divide), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::Divide, Box::new(r)),
                    e.span(),
                )
            }),
            infix(right(3), just(Token::IntDivide), |l, _, r, e| {
                (
                    Expr::Binary(Box::new(l), BinaryOp::IntDivide, Box::new(r)),
                    e.span(),
                )
            }),
        ))
    })
}
