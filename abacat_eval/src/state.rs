use std::marker::PhantomData;

use abacat_common::mutability::MutabilityGuard;
use abacat_parser::parser::parser::Ident;

use crate::value::{Function, Value, native::NativeValue};

// TODO: MutabilityGuard could be genericised, e.g. make value V
#[derive(Debug, Clone)]
pub struct StateMutation<I> {
    pub ident: I,
    pub value: MutabilityGuard<Value>,
}

impl<I> StateMutation<I> {
    pub fn new(ident: I, value: MutabilityGuard<Value>) -> StateMutation<I> {
        return StateMutation {
            ident: ident,
            value: value,
        };
    }
    pub fn map_ident<I2>(self, mapper: fn(I) -> I2) -> StateMutation<I2> {
        StateMutation {
            ident: mapper(self.ident),
            value: self.value,
        }
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

pub struct StateSnapshot<'a, I, IN, S>
where
    IN: StateSource<I>,
    S: StateSource<I>,
{
    initial: &'a IN,
    changesets: &'a [S],
    phantom: PhantomData<I>,
}
impl<'a: 'b, 'b, I, IN, S> StateSnapshot<'a, I, IN, S>
where
    IN: StateSource<I>,
    S: StateSource<I>,
{
    // pub fn with(mut self, overrides: O) -> StateSnapshot<'a, I, O, IN, S>{
    //     StateSnapshot {
    //         overrides: self.overrides,
    //         initial: self.initial,
    //         changesets: self.changesets,
    //         phantom: self.phantom,
    //     }
    // }
    pub fn new(initial: &'a IN, changesets: &'a [S]) -> StateSnapshot<'a, I, IN, S> {
        StateSnapshot {
            initial: initial,
            changesets: changesets,
            phantom: PhantomData::default(),
        }
    }
    pub fn resolve_ident<O: StateSource<I>>(
        &self,
        id: &I,
        overrides: Option<&O>,
    ) -> Option<MutabilityGuard<Value>> {
        self.resolve_ident_and_source(id, overrides)
            .map(|(_, val)| val)
    }

    pub fn resolve_ident_and_source<O: StateSource<I>>(
        &'a self,
        id: &I,
        overrides: Option<&O>,
    ) -> Option<(StateSnapshot<'b, I, IN, S>, MutabilityGuard<Value>)> {
        overrides
            .and_then(|ov| ov.resolve_ident(&id))
            .map(|val| (StateSnapshot::new(self.initial, self.changesets), val))
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

pub struct State<I = Ident, IN = NativeChangeset, S = VecChangeset<I>>
where
    IN: StateSource<I>,
    S: StateSource<I> + Sized, // T: StateSource + 'a,
{
    initial: IN,
    changesets: Vec<S>,
    phantom: PhantomData<I>,
}

impl<'a: 'b, 'b, I, IN, S> State<I, IN, S>
where
    IN: StateSource<I>,
    S: StateSource<I> + Sized,
{
    pub fn new(initial: IN) -> State<I, IN, S> {
        State {
            initial,
            changesets: Vec::new(),
            phantom: PhantomData::default(),
        }
    }
    pub fn len(&self) -> usize {
        self.changesets.len()
    }
    pub fn at(&'a self, expr_index: usize) -> Option<StateSnapshot<'b, I, IN, S>> {
        if expr_index > self.len() {
            None
        } else {
            Some(StateSnapshot::new(
                &self.initial,
                &self.changesets[0..expr_index],
            ))
        }
    }
    pub fn last(&'a self) -> StateSnapshot<'b, I, IN, S> {
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
        value::{Value, native::NATIVE_VALUES},
    };

    // TODO: Better test name && structure && should check mutability guard
    #[rstest]
    fn test() {
        let mut state: State = State::new(NativeChangeset::new(&NATIVE_VALUES));
        let value = state
            .last()
            .resolve_ident::<NativeChangeset>(&"DECIMAL_MIN".to_string(), None)
            .unwrap();
        assert!(
            value
                .consume()
                .checked_eq(&Value::decimal(Decimal::MIN))
                .unwrap()
        );

        state.publish(vec![StateMutation::new(
            "test1".to_string(),
            MutabilityGuard::Mutable(Value::boolean(true)),
        )]);
        let value = state
            .last()
            .resolve_ident::<NativeChangeset>(&"test1".to_string(), None)
            .unwrap();
        assert!(value.consume().checked_eq(&Value::boolean(true)).unwrap());

        state.publish(vec![StateMutation::new(
            "test2".to_string(),
            MutabilityGuard::Immutable(Value::boolean(false)),
        )]);

        let value = state
            .last()
            .resolve_ident::<NativeChangeset>(&"test2".to_string(), None)
            .unwrap();
        assert!(value.consume().checked_eq(&Value::boolean(false)).unwrap());

        let value = state
            .last()
            .resolve_ident::<NativeChangeset>(&"test1".to_string(), None)
            .unwrap();
        assert!(value.consume().checked_eq(&Value::boolean(true)).unwrap());

        let value = state
            .last()
            .resolve_ident::<NativeChangeset>(&"DECIMAL_MIN".to_string(), None)
            .unwrap();
        assert!(
            value
                .consume()
                .checked_eq(&Value::decimal(Decimal::MIN))
                .unwrap()
        );
    }
}
