pub trait Interner<'a, T, I = usize>
where
    T: PartialEq<T>,
    I: InternedId<I> + Clone,
{
    fn intern(&mut self, value: T) -> I;
    fn find_value(&self, value: &T) -> Option<I>;
}

// TODO: Should make this checked
pub trait InternedId<I> {
    fn generate(previous: Option<&I>) -> I;
}

pub struct VecInterner<T, I> {
    bucket: Vec<(I, T)>,
}

impl<T, I> VecInterner<T, I>
where
    T: PartialEq<T>,
    I: InternedId<I> + Clone,
{
    pub const fn new() -> VecInterner<T, I> {
        VecInterner { bucket: vec![] }
    }
}

impl<'a, T, I> Interner<'a, T, I> for VecInterner<T, I>
where
    T: PartialEq<T>,
    I: InternedId<I> + Clone,
{
    fn intern(&mut self, new_value: T) -> I {
        let id = self.find_value(&new_value);
        if let Some(id) = id {
            return id.clone();
        } else {
            let previous_id = self.bucket.last().map(|(id, _)| id);
            let new_id = I::generate(previous_id);
            self.bucket.push((new_id.clone(), new_value));
            new_id
        }
    }

    fn find_value(&self, desired: &T) -> Option<I> {
        self.bucket.iter().find_map(|(id, value)| {
            if value == desired {
                Some(id.clone())
            } else {
                None
            }
        })
    }
}

impl<T> InternedId<T> for T
where
    T: num_traits::PrimInt,
{
    #[inline]
    fn generate(previous: Option<&T>) -> T {
        previous.map_or(T::min_value(), |p| *p + T::one())
    }
}
