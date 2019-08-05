```rust
struct Foo<T, U> {
    future: T,
    field: U,
}

struct __FooProjection<'__a, T, U> {
    future: ::core::pin::Pin<&'__a mut T>,
    field: &'__a mut U,
}

impl<T, U> Foo<T, U> {
    fn project<'__a>(self: ::core::pin::Pin<&'__a mut Self>) -> __FooProjection<'__a, T, U> {
        unsafe {
            let this = ::core::pin::Pin::get_unchecked_mut(self);
            __FooProjection {
                future: ::core::pin::Pin::new_unchecked(&mut this.future),
                field: &mut this.field,
            }
        }
    }
}

// Automatically create the appropriate conditional Unpin implementation (optional).
impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
```
