```rust
enum Foo<T, U> {
    Future(T),
    Done(U),
}

enum __FooProjection<'__a, T, U> {
    Future(::core::pin::Pin<&'__a mut T>),
    Done(&'__a mut U),
}

impl<T, U> Foo<T, U> {
    fn project<'__a>(self: ::core::pin::Pin<&'__a mut Self>) -> __FooProjection<'__a, T, U> {
        unsafe {
            match ::core::pin::Pin::get_unchecked_mut(self) {
                Foo::Future(_x0) => __FooProjection::Future(::core::pin::Pin::new_unchecked(_x0)),
                Foo::Done(_x0) => __FooProjection::Done(_x0),
            }
        }
    }
}

// Automatically create the Drop implementation.
impl<T, U> Drop for Foo<T, U> {
    fn drop(&mut self) {
        // Do nothing. The precense of this Drop
        // impl ensures that the user can't provide one of their own
    }
}

// Automatically create the appropriate conditional Unpin implementation.
impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
```
