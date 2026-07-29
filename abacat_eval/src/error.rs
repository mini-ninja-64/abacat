use std::ops::Range;

use abacat_common::error::LocatableFailure;
use abacat_parser::parser::parser::{BinaryOp, UnaryOp};
use snafu::Snafu;

use crate::value::{ActualType, NumberType};

#[derive(Debug, Snafu, Clone)]
pub enum EvalError {
    #[snafu(display("Unable to convert from {from} to {to}"))]
    UnableToConvert {
        span: Option<Range<usize>>,
        from: ActualType,
        to: ActualType,
    },
    #[snafu(display("Unable to represent this {from} as a {to}, overflow?"))]
    NumberConversionFailure {
        span: Option<Range<usize>>,
        from: NumberType,
        to: NumberType,
    },
    #[snafu(display("Division by zero"))]
    DivideByZero { span: Option<Range<usize>> },
    #[snafu(display("Unable to complete {op}"))]
    BinaryOperationFailure {
        span: Option<Range<usize>>,
        op: BinaryOp,
    },
    // TODO: hmmm need to support multi highlight i guess
    #[snafu(display("Arrays with different lengths"))]
    MismatchArrayLength {
        left: Option<Range<usize>>,
        right: Option<Range<usize>>,
    },
    #[snafu(display("Unable to complete unary {op}"))]
    UnaryOperationFailure {
        span: Option<Range<usize>>,
        op: UnaryOp,
    },
    #[snafu(display("'{ident}' is not resolvable"))]
    IdentNotFound {
        span: Option<Range<usize>>,
        ident: String,
    },
    #[snafu(display("Index {index} is out of range for list of size {length}"))]
    IndexOutOfRangeError {
        span: Option<Range<usize>>,
        index: i128,
        length: usize,
    },
    #[snafu(display("Expected {expected} args, received {received}"))]
    ExpectedArgsN {
        span: Option<Range<usize>>,
        expected: usize,
        received: usize,
    },
    #[snafu(display("Expected at least {expected} args, received {received}"))]
    ExpectedAtLeastArgsN {
        span: Option<Range<usize>>,
        expected: usize,
        received: usize,
    },
    #[snafu(display("Only idents can be assigned"))]
    NonIdentAssignment { span: Option<Range<usize>> },
    #[snafu(display("Attempted to assign to an immutable variable"))]
    ImmutableAssignment { span: Option<Range<usize>> },
    #[snafu(display("Failed to parse"))]
    ParsingError,
    #[snafu(display("Implementation bug"))]
    ImplementationBug,
}

// TODO: Should eventually support more complex errors, but this is enough to start with imo
//
impl<'a> LocatableFailure<'a, String> for EvalError {
    // TODO: I should rlly make a macro for this lol
    fn span(&self) -> Option<Range<usize>> {
        match self {
            EvalError::UnableToConvert {
                span,
                from: _,
                to: _,
            } => span.clone(),
            EvalError::NumberConversionFailure {
                span,
                from: _,
                to: _,
            } => span.clone(),
            EvalError::DivideByZero { span } => span.clone(),
            EvalError::BinaryOperationFailure { span, op: _ } => span.clone(),
            EvalError::UnaryOperationFailure { span, op: _ } => span.clone(),
            EvalError::IdentNotFound { span, ident: _ } => span.clone(),
            EvalError::IndexOutOfRangeError {
                span,
                index: _,
                length: _,
            } => span.clone(),
            EvalError::ExpectedArgsN {
                span,
                expected: _,
                received: _,
            } => span.clone(),
            EvalError::ExpectedAtLeastArgsN {
                span,
                expected: _,
                received: _,
            } => span.clone(),
            EvalError::NonIdentAssignment { span } => span.clone(),
            EvalError::ImmutableAssignment { span } => span.clone(),
            EvalError::ParsingError => None,
            EvalError::ImplementationBug => None,
            EvalError::MismatchArrayLength { left, right } => left.clone(),
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

impl EvalError {
    // TODO: I should rlly make a macro for this lol
    pub fn replace_span(self, span: Option<Range<usize>>) -> Self {
        match self {
            EvalError::UnableToConvert { span: _, from, to } => {
                EvalError::UnableToConvert { span, from, to }
            }
            EvalError::NumberConversionFailure { span: _, from, to } => {
                EvalError::NumberConversionFailure { span, from, to }
            }
            EvalError::DivideByZero { span: _ } => EvalError::DivideByZero { span },
            EvalError::BinaryOperationFailure { span: _, op } => {
                EvalError::BinaryOperationFailure { span, op }
            }
            EvalError::UnaryOperationFailure { span: _, op } => {
                EvalError::UnaryOperationFailure { span, op }
            }
            EvalError::IdentNotFound { span: _, ident } => EvalError::IdentNotFound { span, ident },
            EvalError::IndexOutOfRangeError {
                span: _,
                index,
                length,
            } => EvalError::IndexOutOfRangeError {
                span,
                index,
                length,
            },
            EvalError::ExpectedArgsN {
                span: _,
                expected,
                received,
            } => EvalError::ExpectedArgsN {
                span,
                expected,
                received,
            },
            EvalError::ExpectedAtLeastArgsN {
                span: _,
                expected,
                received,
            } => EvalError::ExpectedAtLeastArgsN {
                span,
                expected,
                received,
            },
            EvalError::NonIdentAssignment { span: _ } => EvalError::NonIdentAssignment { span },
            EvalError::ImmutableAssignment { span: _ } => EvalError::ImmutableAssignment { span },
            EvalError::ParsingError => EvalError::ParsingError,
            EvalError::ImplementationBug => EvalError::ImplementationBug,
            EvalError::MismatchArrayLength { left: _, right } => {
                EvalError::MismatchArrayLength { left: span, right }
            }
        }
    }
}
