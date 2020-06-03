include!("basic-safe-part.rs");

unsafe impl<T: Unpin, U: Unpin> UnsafeUnpin for UnsafeUnpinStruct<T, U> {}
unsafe impl<T: Unpin, U: Unpin> UnsafeUnpin for UnsafeUnpinTupleStruct<T, U> {}
unsafe impl<T: Unpin, U: Unpin> UnsafeUnpin for UnsafeUnpinEnum<T, U> {}
