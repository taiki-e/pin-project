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

// Automatically create the appropriate conditional Unpin implementation.
impl<T, U> Unpin for Foo<T, U> where T: Unpin {}

// Ensure that struct does not implement `Drop`.
// There are two possible cases:
// 1. The user type does not implement Drop. In this case,
// the first blanked impl will not apply to it. This code
// will compile, as there is only one impl of MustNotImplDrop for the user type
// 2. The user type does impl Drop. This will make the blanket impl applicable,
// which will then comflict with the explicit MustNotImplDrop impl below.
// This will result in a compilation error, which is exactly what we want.
trait FooMustNotImplDrop {}
impl<T: Drop> FooMustNotImplDrop for T {}
impl<T, U> FooMustNotImplDrop for Foo<T, U> {}
```
