use std::ops::Range;

use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

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

pub trait LocatedFailure<M> {
    fn span(&self) -> Range<usize>;
    fn message(&self) -> &M;
}

pub trait WithCause<C> {
    fn cause(&self) -> &C;
}

impl<M, C> LocatedFailure<M> for BasicFailure<M, C> {
    fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    fn message(&self) -> &M {
        &self.message
    }
}

impl<M, C> WithCause<C> for BasicFailure<M, C> {
    fn cause(&self) -> &C {
        &self.cause
    }
}
