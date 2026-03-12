use std::marker::PhantomData;

use abacat_common::mutability::MutabilityGuard;
use abacat_parser::parser::parser::Ident;

use crate::value::{native::NativeValue, Function, Value};

// TODO: MutabilityGuard could be genericised, e.g. make value V
pub struct StateMutation<I> {
    ident: I,
    value: MutabilityGuard<Value>,
}

impl<I> StateMutation<I> {
    pub fn new(ident: I, value: MutabilityGuard<Value>) -> StateMutation<I> {
        return StateMutation {
            ident: ident,
            value: value,
        };
    }
}

pub trait StateSource<I> {
    fn resolve_ident(&self, ident: &I) -> Option<MutabilityGuard<Value>>;
}

pub type VecChangeset<I> = Vec<StateMutation<I>>;

impl<I> StateSource<I> for VecChangeset<I>
where
    I: std::cmp::PartialEq,
{
    fn resolve_ident(&self, id: &I) -> Option<MutabilityGuard<Value>> {
        self.iter().rev().find_map(|mutation: &StateMutation<I>| {
            if mutation.ident == *id {
                Some(mutation.value.clone())
            } else {
                None
            }
        })
    }
}

pub struct NativeChangeset {
    phf: &'static phf::Map<&'static str, NativeValue>,
}
impl NativeChangeset {
    pub fn new(phf: &'static phf::Map<&'static str, NativeValue>) -> NativeChangeset {
        NativeChangeset { phf }
    }
}

impl StateSource<String> for NativeChangeset {
    fn resolve_ident(&self, ident: &String) -> Option<MutabilityGuard<Value>> {
        self.phf.get(ident).map(|native_value| {
            MutabilityGuard::Immutable(match native_value {
                NativeValue::Function(func) => Value::function(Function::Native(*func)),
                NativeValue::Value(value) => value.clone(),
            })
        })
    }
}

pub struct StateSnapshot<'a, I, O, IN, S>
where
    O: StateSource<I>,
    IN: StateSource<I>,
    S: StateSource<I>,
{
    overrides: Option<&'a O>,
    initial: &'a IN,
    changesets: &'a [S],
    phantom: PhantomData<I>,
}
impl<I, O, IN, S> StateSnapshot<'_, I, O, IN, S>
where
    IN: StateSource<I>,
    S: StateSource<I>,
    O: StateSource<I>,
{
    pub fn with<'a: 'b, 'b>(&'a self, overrides: &'b O) -> StateSnapshot<'b, I, O, IN, S> {
        StateSnapshot {
            overrides: Some(overrides),
            initial: self.initial,
            changesets: self.changesets,
            phantom: self.phantom,
        }
    }
    pub fn new<'a>(initial: &'a IN, changesets: &'a [S]) -> StateSnapshot<'a, I, O, IN, S> {
        StateSnapshot {
            initial: initial,
            changesets: changesets,
            overrides: None,
            phantom: PhantomData::default(),
        }
    }
    pub fn resolve_ident(&self, id: &I) -> Option<MutabilityGuard<Value>> {
        self.resolve_ident_and_source(id).map(|(_, val)| val)
    }

    pub fn resolve_ident_and_source<'a: 'b, 'b>(
        &'a self,
        id: &I,
    ) -> Option<(StateSnapshot<'b, I, O, IN, S>, MutabilityGuard<Value>)> {
        self.overrides
            .and_then(|overrides| {
                overrides
                    .resolve_ident(id)
                    .map(|resolved| (StateSnapshot::new(self.initial, &self.changesets), resolved))
            })
            .or_else(|| {
                self.changesets
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(pos, source)| source.resolve_ident(&id).map(move |val| (pos, val)))
                    .map(|(pos, value)| {
                        (
                            StateSnapshot::new(self.initial, &self.changesets[0..pos]),
                            value,
                        )
                    })
                    .or_else(|| {
                        self.initial
                            .resolve_ident(&id)
                            .map(|val| (StateSnapshot::new(self.initial, &[]), val))
                    })
            })
    }
}

pub struct State<I = Ident, O = VecChangeset<I>, IN = NativeChangeset, S = VecChangeset<I>>
where
    IN: StateSource<I>,
    S: StateSource<I> + Sized, // T: StateSource + 'a,
    O: StateSource<I>,
{
    initial: IN,
    changesets: Vec<S>,
    phantom: PhantomData<(I, O)>,
}

impl<'a: 'b, 'b, I, O, IN, S> State<I, O, IN, S>
where
    O: StateSource<I>,
    IN: StateSource<I>,
    S: StateSource<I> + Sized,
{
    pub fn new(initial: IN) -> State<I, O, IN, S> {
        State {
            initial,
            changesets: Vec::new(),
            phantom: PhantomData::default(),
        }
    }
    pub fn len(&self) -> usize {
        self.changesets.len()
    }
    pub fn at(&'a self, expr_index: usize) -> Option<StateSnapshot<'b, I, O, IN, S>> {
        if expr_index > self.len() {
            None
        } else {
            Some(StateSnapshot::new(
                &self.initial,
                &self.changesets[0..expr_index],
            ))
        }
    }
    pub fn last(&'a self) -> StateSnapshot<'b, I, O, IN, S> {
        self.at(self.len()).unwrap()
    }
    pub fn publish(&mut self, changeset: S) {
        self.changesets.push(changeset);
    }
}

#[cfg(test)]
mod tests {
    use abacat_common::{checked::CheckedEq, mutability::MutabilityGuard};
    use rstest::rstest;
    use rust_decimal::Decimal;

    use crate::{
        state::{NativeChangeset, State, StateMutation},
        value::{native::NATIVE_VALUES, Value},
    };

    // TODO: Better test name && structure && should check mutability guard
    #[rstest]
    fn test() {
        let mut state: State = State::new(NativeChangeset::new(&NATIVE_VALUES));
        let value = state
            .last()
            .resolve_ident(&"DECIMAL_MIN".to_string())
            .unwrap();
        assert!(value
            .consume()
            .checked_eq(&Value::decimal(Decimal::MIN))
            .unwrap());

        state.publish(vec![StateMutation::new(
            "test1".to_string(),
            MutabilityGuard::Mutable(Value::boolean(true)),
        )]);
        let value = state.last().resolve_ident(&"test1".to_string()).unwrap();
        assert!(value.consume().checked_eq(&Value::boolean(true)).unwrap());

        state.publish(vec![StateMutation::new(
            "test2".to_string(),
            MutabilityGuard::Immutable(Value::boolean(false)),
        )]);

        let value = state.last().resolve_ident(&"test2".to_string()).unwrap();
        assert!(value.consume().checked_eq(&Value::boolean(false)).unwrap());

        let value = state.last().resolve_ident(&"test1".to_string()).unwrap();
        assert!(value.consume().checked_eq(&Value::boolean(true)).unwrap());

        let value = state
            .last()
            .resolve_ident(&"DECIMAL_MIN".to_string())
            .unwrap();
        assert!(value
            .consume()
            .checked_eq(&Value::decimal(Decimal::MIN))
            .unwrap());
    }
}
