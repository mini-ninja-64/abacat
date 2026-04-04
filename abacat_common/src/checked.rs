pub trait CheckedEq<T> {
    fn checked_eq(&self, val: T) -> Option<bool>;
}

pub trait CheckedAnd<T> {
    fn checked_and(&self, val: T) -> Option<bool>;
}

pub trait CheckedOr<T> {
    fn checked_or(&self, val: T) -> Option<bool>;
}
