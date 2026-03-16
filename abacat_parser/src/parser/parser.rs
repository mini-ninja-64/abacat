use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::ValueInput,
    pratt::{infix, left, prefix, right},
    prelude::{choice, just},
    recursive::recursive,
    select,
};
use rust_decimal::Decimal;

use crate::{Span, Spanned, lexer::token::Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    ExclamationMark,
    Minus,
}
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub args: Vec<Spanned<Ident>>,
    pub body: Box<Spanned<Expr>>,
}
impl Function {
    pub fn new<'a>(args: Vec<Spanned<Ident>>, body: Box<Spanned<Expr>>) -> Function {
        Function { args, body }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Unary(UnaryOp, Box<Spanned<Self>>),
    Call(Spanned<Ident>, Vec<Spanned<Self>>),
    Ident(Spanned<Ident>),
    Literal(Literal), // List(Box<>)
    Parenthesised(Box<Spanned<Self>>),
    NamedFunction(Spanned<Ident>, Spanned<Function>),
    AnonymousFunction(Spanned<Function>),
}

pub fn expr<'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Expr>, extra::Err<Rich<'tokens, Token<'tokens>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'tokens>, Span = Span>,
{
    let ident = select! {Token::Ident(i) => i}.map_with(|i, e| (i.to_string(), e.span()));
    let literal = select! {
        Token::Bool(x) => Literal::Bool(x),
        Token::Base2Num(n) => Literal::Base2Num(n),
        Token::Base8Num(n) => Literal::Base8Num(n),
        Token::Base10Decimal(n) => Literal::Base10Decimal(n),
        Token::Base10Num(n) => Literal::Base10Num(n),
        Token::Base16Num(n) => Literal::Base16Num(n),
    };

    // TODO: Fix binding power, its wrong ooops
    recursive(|expr| {
        let ident_expr = ident.map(|i| Expr::Ident(i)).labelled("Ident").as_context();
        let literal_expr = literal
            .map(|v| Expr::Literal(v))
            .labelled("Literal")
            .as_context();
        let parenthesised_expr = expr
            .clone()
            .delimited_by(just(Token::LeftParens), just(Token::RightParens))
            .map(|e| Expr::Parenthesised(Box::new(e)))
            .labelled("Parenthesised")
            .as_context();

        let expr_args = expr
            .clone()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LeftParens), just(Token::RightParens));
        let call_expr = ident
            .then(expr_args)
            .map(|(ident, args)| Expr::Call(ident, args))
            .labelled("Function call")
            .as_context();

        let args_def = ident
            .clone()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LeftParens), just(Token::RightParens));

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
            .labelled("Named function declaration")
            .as_context();

        let anonymous_function = args_def
            .then_ignore(just(Token::Arrow))
            .then(expr)
            .map_with(|(args, body), e| {
                Expr::AnonymousFunction((Function::new(args, Box::new(body)), e.span()))
            })
            .labelled("Anonymous function declaration")
            .as_context();
        // .map_with(|x, e| Expr::NamedFunction((x, e.span())));
        choice((
            literal_expr,
            call_expr,
            ident_expr,
            anonymous_function,
            named_function,
            parenthesised_expr,
        ))
        .map_with(|i, e| (i, e.span()))
        .pratt((
            prefix(4, just(Token::Minus), |_, r, e| {
                (Expr::Unary(UnaryOp::Minus, Box::new(r)), e.span())
            }),
            prefix(4, just(Token::ExclamationMark), |_, r, e| {
                (Expr::Unary(UnaryOp::ExclamationMark, Box::new(r)), e.span())
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

// pub fn expr<'tokens, 'src: 'tokens, I>() -> impl Parser<
//     'tokens,
//     I,
//     Vec<Spanned<Expr<'src>>>,
//     extra::Err<Rich<'tokens, Token<'src>, Span>>,
// > {
// }
