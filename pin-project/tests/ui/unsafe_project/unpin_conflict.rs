// compile-fail

#![deny(warnings)]

use pin_project::pin_projectable;
use std::pin::Pin;

// The same implementation.

#[pin_projectable] //~ ERROR E0119
struct Foo<T, U> {
    #[pin]
    future: T,
    field: U,
}

impl<T, U> Foo<T, U> {
    fn baz(mut self: Pin<&mut Self>) {
        let this = self.project();
        let _: Pin<&mut T> = this.future; // Pinned reference to the field
        let _: &mut U = this.field; // Normal reference to the field
    }
}

// conflicting implementations
impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl

// The implementation that under different conditions.

#[pin_projectable] //~ ERROR E0119
struct Bar<T, U> {
    #[pin]
    future: T,
    field: U,
}

impl<T, U> Bar<T, U> {
    fn baz(mut self: Pin<&mut Self>) {
        let this = self.project();
        let _: Pin<&mut T> = this.future; // Pinned reference to the field
        let _: &mut U = this.field; // Normal reference to the field
    }
}

// conflicting implementations
impl<T, U> Unpin for Bar<T, U> {} // Non-conditional Unpin impl

#[pin_projectable] //~ ERROR E0119
struct Baz<T, U> {
    #[pin]
    future: T,
    field: U,
}

impl<T, U> Baz<T, U> {
    fn baz(mut self: Pin<&mut Self>) {
        let this = self.project();
        let _: Pin<&mut T> = this.future; // Pinned reference to the field
        let _: &mut U = this.field; // Normal reference to the field
    }
}

// conflicting implementations
impl<T: Unpin, U: Unpin> Unpin for Baz<T, U> {} // Conditional Unpin impl

fn main() {}
