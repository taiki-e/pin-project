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

impl<T, U> Unpin for Foo<T, U> where T: Unpin {}

impl<T, U> Foo<T, U> {
    fn baz(mut self: Pin<&mut Self>) {
        match self.project() {
            __FooProjection::Future(future) => {
                let _: Pin<&mut T> = future;
            }
            __FooProjection::Done(value) => {
                let _: &mut U = value;
            }
        }
    }
}
```
