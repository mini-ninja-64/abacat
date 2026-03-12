use std::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub enum MutabilityGuard<T> {
    Mutable(T),
    Immutable(T),
}

impl<T> MutabilityGuard<T> {
    pub fn is_mutable(&self) -> bool {
        match &self {
            MutabilityGuard::Mutable(_) => true,
            _ => false,
        }
    }
    pub fn consume(self) -> T {
        match self {
            MutabilityGuard::Mutable(val) => val,
            MutabilityGuard::Immutable(val) => val,
        }
    }
}

impl<T> Deref for MutabilityGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MutabilityGuard::Mutable(val) => val,
            MutabilityGuard::Immutable(val) => val,
        }
    }
}
