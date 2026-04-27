use std::ops::Range;

use chumsky::span::SimpleSpan;

pub type SpanChumsky = SimpleSpan;
pub type SpannedChumsky<T> = (T, SpanChumsky);
pub type Spanned<T> = (T, Range<usize>);

#[derive(Debug)]
pub struct BasicFailure<M, C = ()> {
    span: Range<usize>,
    message: M,
    cause: C,
}

impl<M> BasicFailure<M, ()> {
    pub fn new(span: Range<usize>, message: M) -> BasicFailure<M, ()> {
        BasicFailure {
            span,
            message,
            cause: (),
        }
    }
}

impl<M, C> BasicFailure<M, C> {
    pub fn new_with_cause(span: Range<usize>, message: M, underlying: C) -> BasicFailure<M, C> {
        BasicFailure {
            span,
            message,
            cause: underlying,
        }
    }
}

pub trait LocatableFailure<'a, M> {
    fn span(&'a self) -> Option<Range<usize>>;
    fn message(&'a self) -> M;
}

pub trait WithCause<C> {
    fn cause(&self) -> &C;
}

impl<'a, C> LocatableFailure<'a, &'a str> for BasicFailure<String, C> {
    fn span(&self) -> Option<Range<usize>> {
        Some(self.span.clone())
    }

    fn message(&'a self) -> &'a str {
        self.message.as_str()
    }
}

impl<M, C> WithCause<C> for BasicFailure<M, C> {
    fn cause(&self) -> &C {
        &self.cause
    }
}
