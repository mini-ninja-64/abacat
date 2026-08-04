use abacat_common::error::Spanned;

use crate::{
    error::EvalError,
    value::{ActualType, Value},
};

pub struct Args {}

impl Args {
    pub fn exactly<T>(count: usize, (args, span): &Spanned<Vec<T>>) -> Result<(), EvalError> {
        if args.len() == count {
            return Ok(());
        }
        Err(EvalError::ExpectedArgsN {
            span: Some(span.clone()),
            expected: count,
            received: args.len(),
        })
    }

    pub fn typed(
        index: usize,
        expected_ty: ActualType,
        (args, span): &Spanned<Vec<Spanned<Value>>>,
    ) -> Result<&Spanned<Value>, EvalError> {
        let val = args.get(index).ok_or_else(|| EvalError::ExpectedArgsN {
            span: Some(span.clone()),
            expected: index,
            received: args.len(),
        })?;

        let actual_ty = val.0.val.actual_type();
        if actual_ty != expected_ty {
            return Err(EvalError::UnableToConvert {
                span: Some(val.1.clone()),
                from: actual_ty,
                to: expected_ty,
            });
        }

        Ok(val)
    }

    pub fn at_least<T>(count: usize, (args, span): &Spanned<Vec<T>>) -> Result<(), EvalError> {
        if args.len() >= count {
            return Ok(());
        }
        Err(EvalError::ExpectedAtLeastArgsN {
            span: Some(span.clone()),
            expected: count,
            received: args.len(),
        })
    }
}
