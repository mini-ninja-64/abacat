use abacat_common::{error::Spanned, mutability::MutabilityGuard};
use abacat_parser::{
    ParserResult,
    parser::parser::{Expr, Ident},
};

use crate::{
    eval::{Eval, Snapshot, eval},
    state::{NativeChangeset, StateMutation, StateSnapshot, StateSource, VecChangeset},
    value::{Value, native::NATIVE_VALUES},
};

#[derive(Debug)]

pub struct ParserEvalPair(pub ParserResult, pub Result<Eval, ()>);

#[derive(Debug)]
pub struct Document {
    initial_state: NativeChangeset,
    exprs: Vec<ParserEvalPair>,
}

impl StateSource<Ident> for ParserEvalPair {
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

    pub fn with_expr(mut self, expr: ParserResult) -> Document {
        let _ = self.next(expr);
        self
    }

    fn calculate(expr: &ParserResult, state: &Snapshot) -> Result<Eval, ()> {
        let expr = expr.as_ref().map_err(|_| ())?;
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

    pub fn replace_at(&mut self, to_replace: usize, expr: ParserResult) {
        self.exprs[to_replace].0 = expr;
        for index in to_replace..self.exprs.len() {
            let ParserEvalPair(expr, _) = &self.exprs[index];
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

    pub fn insert_at(&mut self, to_insert: usize, new_expr: ParserResult) {
        self.exprs
            .insert(to_insert, ParserEvalPair(new_expr, Err(())));
        for index in to_insert..self.exprs.len() {
            let ParserEvalPair(expr, _) = &self.exprs[index];
            let snapshot = self.snapshot_at(index).unwrap();
            let new_eval = Self::calculate(expr, &snapshot);
            self.exprs[index].1 = new_eval;
        }
    }

    pub fn next(&mut self, expr: ParserResult) -> Result<Eval, ()> {
        let state = self.current_snapshot();
        let eval = Self::calculate(&expr, &state);

        self.exprs.push(ParserEvalPair(expr, eval.clone()));
        eval
    }

    pub fn history(&self) -> &[ParserEvalPair] {
        &self.exprs
    }
    pub fn history_at(&self, index: usize) -> Option<&ParserEvalPair> {
        self.exprs.get(index)
    }
    pub fn history_len(&self) -> usize {
        self.exprs.len()
    }
}
