use abacat_common::mutability::MutabilityGuard;
use abacat_parser::{Spanned, parser::parser::Expr};

use crate::{
    eval::{Eval, eval},
    state::{NativeChangeset, State, StateMutation, VecChangeset},
    value::native::NATIVE_VALUES,
};

pub struct Document {
    state: State,
    exprs: Vec<Spanned<Expr>>,
}

impl Document {
    pub fn new(state: State) -> Document {
        Document {
            state,
            exprs: vec![],
        }
    }

    pub fn new_with_default_constants() -> Document {
        Document::new(State::new(NativeChangeset::new(&NATIVE_VALUES)))
    }

    pub fn next(&mut self, expr: Spanned<Expr>) -> Result<Eval, ()> {
        let ans = eval(&expr, &self.state.last())?;
        let ans_value = match &ans {
            Eval::Value(value) => value.clone(),
            Eval::ValueAssignment(_, value) => value.clone(),
        };
        let mut changeset = vec![];

        if let Eval::ValueAssignment(ident, value) = &ans {
            let mutable = self
                .state
                .last()
                .resolve_ident::<VecChangeset<_>>(&ident, None)
                .map_or(true, |state| state.is_mutable());
            if !mutable {
                return Err(()); // Attempt to mutate immutable variable
            }
            changeset.push(StateMutation::new(
                ident.clone(),
                MutabilityGuard::Mutable(value.clone()),
            ));
        }

        changeset.push(StateMutation::new(
            "ans".to_string(),
            MutabilityGuard::Immutable(ans_value),
        ));
        self.state.publish(changeset);
        return Ok(ans);
    }
}
