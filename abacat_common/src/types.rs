pub trait PossiblyRef<T> {
    fn as_ref(&self) -> &T;
    fn as_owned(self) -> T;
}

impl<T> PossiblyRef<T> for T {
    fn as_ref(&self) -> &T {
        self
    }

    fn as_owned(self) -> T {
        self
    }
}
impl<'a, T> PossiblyRef<T> for &'a T
where
    T: Clone,
{
    fn as_ref(&self) -> &T {
        self
    }

    fn as_owned(self) -> T {
        self.clone()
    }
}
