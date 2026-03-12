// TODO: Maybe should return Option, makes harder to report exact error?
pub trait CheckedEq<T> {
    fn checked_eq(&self, val: T) -> Option<bool>;
}

pub trait CheckedAnd<T> {
    fn checked_and(&self, val: T) -> Option<bool>;
}

pub trait CheckedOr<T> {
    fn checked_or(&self, val: T) -> Option<bool>;
}
