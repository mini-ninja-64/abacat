use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    prelude::{choice, just},
    text,
};
use derive_more::Display;
use rust_decimal::Decimal;

pub type IdentIndex = usize;

#[derive(PartialEq, Clone, Debug, Display)]
pub enum Token<'src> {
    // Literals
    Bool(bool),
    Ident(&'src str),
    // Str(&'src str),
    // Char(u8),
    Base10Decimal(Decimal),
    Base10Num(u64),
    Base16Num(u64),
    Base2Num(u64),
    Base8Num(u64),

    // Operators
    ExclamationMark,
    Plus,
    Minus,
    Multiply,
    Divide,
    IntDivide,
    Equal,
    Arrow,

    // Comparators
    EqualEqual,
    AndAnd,
    OrOr,

    // Symbols
    LeftParens,
    RightParens,
    Comma,

    // Keywords
    Def,
}

pub fn lexer<'src>() -> impl Parser<
    'src,
    &'src str,
    Vec<crate::Spanned<Token<'src>>>,
    extra::Err<Rich<'src, char, crate::Span>>,
> {
    // TODO: improve this
    let base10_num = text::int(10)
        .then(just('.').ignore_then(text::digits(10).to_slice()).or_not())
        .to_slice()
        .try_map(|slice: &str, span| {
            if slice.contains(".") {
                let decimal = slice
                    .parse::<Decimal>()
                    .map_err(|e| Rich::custom(span, e))?;
                return Ok(Token::Base10Decimal(decimal));
            }
            let integral = slice.parse::<u64>().map_err(|e| Rich::custom(span, e))?;
            return Ok(Token::Base10Num(integral));
        });

    let base16_num = just("0x").ignore_then(text::digits(16).to_slice().try_map(|n, span| {
        let num = u64::from_str_radix(n, 16).map_err(|e| Rich::custom(span, e))?;
        Ok(Token::Base16Num(num))
    }));

    let base2_num = just("0b").ignore_then(text::digits(2).to_slice().try_map(|n, span| {
        let num = u64::from_str_radix(n, 2).map_err(|e| Rich::custom(span, e))?;
        Ok(Token::Base2Num(num))
    }));

    let base8_num = just("0").ignore_then(text::digits(8).to_slice().try_map(|n, span| {
        let num = u64::from_str_radix(n, 8).map_err(|e| Rich::custom(span, e))?;
        Ok(Token::Base8Num(num))
    }));

    let number = choice((base16_num, base2_num, base8_num, base10_num));

    let token = choice((
        number,
        just("def").map(|_| Token::Def),
        just("true").map(|_| Token::Bool(true)),
        just("false").map(|_| Token::Bool(false)),
        text::ascii::ident().map(|s| Token::Ident(s)),
        just("!").map(|_| Token::ExclamationMark),
        just("+").map(|_| Token::Plus),
        just("-").map(|_| Token::Minus),
        just("*").map(|_| Token::Multiply),
        just("//").map(|_| Token::IntDivide),
        just("/").map(|_| Token::Divide),
        just("(").map(|_| Token::LeftParens),
        just(")").map(|_| Token::RightParens),
        just(",").map(|_| Token::Comma),
        just("&&").map(|_| Token::AndAnd),
        just("||").map(|_| Token::OrOr),
        just("=>").map(|_| Token::Arrow),
        just("==").map(|_| Token::EqualEqual),
        just("=").map(|_| Token::Equal),
    ));

    return token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .repeated()
        .collect();
}
