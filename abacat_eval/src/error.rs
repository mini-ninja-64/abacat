use std::ops::Range;

use abacat_common::error::LocatableFailure;
use abacat_parser::parser::parser::{BinaryOp, UnaryOp};
use snafu::Snafu;

use crate::value::{ActualType, NumberType};

#[derive(Debug, Snafu, Clone)]
pub enum EvalError {
    #[snafu(display("Unable to convert from {from} to {to}"))]
    UnableToConvert {
        span: Range<usize>,
        from: ActualType,
        to: ActualType,
    },
    #[snafu(display("Unable to represent this {from} as a {to}, overflow?"))]
    NumberConversionFailure {
        span: Range<usize>,
        from: NumberType,
        to: NumberType,
    },
    #[snafu(display("Unable to complete {op}"))]
    BinaryOperationFailure { span: Range<usize>, op: BinaryOp },
    #[snafu(display("Unable to complete unary {op}"))]
    UnaryOperationFailure { span: Range<usize>, op: UnaryOp },
    #[snafu(display("'{ident}' is not resolvable"))]
    IdentNotFound { span: Range<usize>, ident: String },
    #[snafu(display("Expected {expected} args, received {received}"))]
    ExpectedArgsN {
        span: Range<usize>,
        expected: usize,
        received: usize,
    },
    #[snafu(display("Expected at least {expected} args, received {received}"))]
    ExpectedAtLeastArgsN {
        span: Range<usize>,
        expected: usize,
        received: usize,
    },
    #[snafu(display("Only idents can be assigned"))]
    NonIdentAssignment { span: Range<usize> },
    #[snafu(display("Attempted to assign to an immutable variable"))]
    ImmutableAssignment { span: Range<usize> },
    #[snafu(display("Failed to parse"))]
    ParsingError,
    #[snafu(display("Implementation bug"))]
    ImplementationBug,
}

// TODO: Should eventually support more complex errors, but this is enough to start with imo
impl EvalError {
    pub fn span(&self) -> Option<Range<usize>> {
        match self {
            EvalError::UnableToConvert {
                span,
                from: _,
                to: _,
            } => Some(span.clone()),
            EvalError::NumberConversionFailure {
                span,
                from: _,
                to: _,
            } => Some(span.clone()),
            EvalError::BinaryOperationFailure { span, op: _ } => Some(span.clone()),
            EvalError::UnaryOperationFailure { span, op: _ } => Some(span.clone()),
            EvalError::IdentNotFound { span, ident: _ } => Some(span.clone()),
            EvalError::ExpectedArgsN {
                span,
                expected: _,
                received: _,
            } => Some(span.clone()),
            EvalError::ExpectedAtLeastArgsN {
                span,
                expected: _,
                received: _,
            } => Some(span.clone()),
            EvalError::NonIdentAssignment { span } => Some(span.clone()),
            EvalError::ImmutableAssignment { span } => Some(span.clone()),
            EvalError::ParsingError => None,
            EvalError::ImplementationBug => None,
        }
    }
}
