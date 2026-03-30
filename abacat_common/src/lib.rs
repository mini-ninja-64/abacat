use chumsky::span::SimpleSpan;

pub mod ariadne;
pub mod checked;
pub mod interner;
pub mod mutability;
pub mod ui;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);
