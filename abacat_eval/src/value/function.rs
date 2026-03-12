pub struct Args {}

impl Args {
    #[inline]
    pub fn exactly<T>(count: usize, args: &Vec<T>) -> Result<(), ()> {
        if args.len() == count {
            return Ok(());
        }
        Err(())
    }

    #[inline]
    pub fn at_least<T>(count: usize, args: &Vec<T>) -> Result<(), ()> {
        if args.len() >= count {
            return Ok(());
        }
        Err(())
    }
}
