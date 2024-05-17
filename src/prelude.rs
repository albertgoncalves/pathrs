#[macro_export]
macro_rules! size_of_field {
    ($ty:ty, $field:ident $(,)?) => {{
        const fn infer<T>(_: *const T) -> usize {
            mem::size_of::<T>()
        }
        let r#struct = mem::MaybeUninit::<$ty>::uninit();
        let field = ptr::addr_of!((*r#struct.as_ptr()).$field);
        infer(field)
    }};
}
