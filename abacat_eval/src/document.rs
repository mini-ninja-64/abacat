use abacat_common::mutability::MutabilityGuard;
use abacat_parser::{
    Spanned,
    parser::parser::{Expr, Ident},
};

use crate::{
    eval::{Eval, Snapshot, eval},
    state::{NativeChangeset, StateMutation, StateSnapshot, StateSource, VecChangeset},
    value::{Value, native::NATIVE_VALUES},
};

#[derive(Debug)]
pub struct Document {
    initial_state: NativeChangeset,
    exprs: Vec<(Result<Spanned<Expr>, ()>, Result<Eval, ()>)>,
}

impl StateSource<Ident> for (Result<Spanned<Expr>, ()>, Result<Eval, ()>) {
    fn resolve_ident(&self, ident: &Ident) -> Option<MutabilityGuard<Value>> {
        self.1
            .as_ref()
            .ok()
            .and_then(|eval| match (ident.as_str(), eval) {
                ("ans", eval) => Some(MutabilityGuard::Immutable(eval.value().clone())),
                (ident, Eval::ValueAssignment(assignment, value))
                    if ident == assignment.as_str() =>
                {
                    Some(MutabilityGuard::Mutable(value.clone()))
                }
                _ => None,
            })
    }
}

impl Document {
    pub fn new(state: NativeChangeset) -> Document {
        Document {
            // state,
            exprs: vec![],
            initial_state: state,
        }
    }

    pub fn new_with_default_constants() -> Document {
        Document::new(NativeChangeset::new(&NATIVE_VALUES))
    }

    pub fn with_expr(mut self, expr: Result<Spanned<Expr>, ()>) -> Document {
        let _ = self.next(expr);
        self
    }

    fn calculate(expr: &Result<Spanned<Expr>, ()>, state: &Snapshot) -> Result<Eval, ()> {
        let expr = expr.as_ref().map_err(|x| *x)?;
        let ans = eval(expr, state)?;
        let ans_value = ans.value().clone();
        let mut changeset = vec![];

        if let Eval::ValueAssignment(ident, value) = &ans {
            let mutable = state
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
        return Ok(ans);
    }

    pub fn replace_at(&mut self, to_replace: usize, expr: Result<Spanned<Expr>, ()>) {
        self.exprs[to_replace].0 = expr;
        for index in to_replace..self.exprs.len() {
            let (expr, _) = &self.exprs[index];
            let snapshot = self.snapshot_at(index).unwrap();
            let new_eval = Self::calculate(expr, &snapshot);
            self.exprs[index].1 = new_eval;
        }
    }

    pub fn snapshot_at(&'_ self, index: usize) -> Option<Snapshot<'_>> {
        Some(StateSnapshot::new(
            &self.initial_state,
            &self.exprs[0..index],
        ))
    }
    pub fn current_snapshot(&self) -> Snapshot<'_> {
        StateSnapshot::new(&self.initial_state, &self.exprs[0..])
    }

    pub fn insert_at(&mut self, to_insert: usize, new_expr: Result<Spanned<Expr>, ()>) {
        self.exprs.insert(to_insert, (new_expr, Err(())));
        for index in to_insert..self.exprs.len() {
            let (expr, _) = &self.exprs[index];
            let snapshot = self.snapshot_at(index).unwrap();
            let new_eval = Self::calculate(expr, &snapshot);
            self.exprs[index].1 = new_eval;
        }
    }

    pub fn next(&mut self, expr: Result<Spanned<Expr>, ()>) -> Result<Eval, ()> {
        let state = self.current_snapshot();
        let eval = Self::calculate(&expr, &state);

        self.exprs.push((expr, eval.clone()));
        eval
    }

    pub fn history(&self) -> &[(Result<Spanned<Expr>, ()>, Result<Eval, ()>)] {
        &self.exprs
    }
    pub fn history_at(
        &self,
        index: usize,
    ) -> Option<&(Result<Spanned<Expr>, ()>, Result<Eval, ()>)> {
        self.exprs.get(index)
    }
    pub fn history_len(&self) -> usize {
        self.exprs.len()
    }
}
