pub struct Defer<F: FnMut()>(pub F);

impl<F: FnMut()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

#[macro_export]
macro_rules! defer {
    ($($tt:tt)*) => {
        let __defer__ = Defer(|| {
            $($tt)*
        });
    };
}
