use abacat_common::{error::Spanned, types::PossiblyRef};

use crate::error::EvalError;

pub struct Args {}

impl Args {
    #[inline]
    pub fn exactly<T>(count: usize, (args, span): &Spanned<Vec<T>>) -> Result<(), EvalError> {
        if args.len() == count {
            return Ok(());
        }
        Err(EvalError::ExpectedArgsN {
            span: span.as_owned(),
            expected: count,
            received: args.len(),
        })
    }

    #[inline]
    pub fn at_least<T>(count: usize, (args, span): &Spanned<Vec<T>>) -> Result<(), EvalError> {
        if args.len() >= count {
            return Ok(());
        }
        Err(EvalError::ExpectedAtLeastArgsN {
            span: span.as_owned(),
            expected: count,
            received: args.len(),
        })
    }
}
