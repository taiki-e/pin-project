#![deny(warnings)]

use pin_project::{unsafe_fields, unsafe_project};
use std::marker::Unpin;
use std::pin::Pin;

#[test]
fn test_unsafe_project() {
    #[unsafe_project]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl

    let mut foo = Foo {
        field1: 1,
        field2: 2,
    };

    let foo = Pin::new(&mut foo).project();

    let x: Pin<&mut i32> = foo.field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.field2;
    assert_eq!(*y, 2);
}

#[test]
fn test_unsafe_fields() {
    #[unsafe_fields]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl

    let mut foo = Foo {
        field1: 1,
        field2: 2,
    };

    let mut foo = Pin::new(&mut foo);

    let x: Pin<&mut i32> = foo.as_mut().field1();
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.as_mut().field2();
    assert_eq!(*y, 2);

    #[unsafe_fields]
    struct Bar<T, U> {
        #[pin]
        field1: T,
        field2: U,
        #[skip]
        _field3: (),
    }

    impl<T: Unpin, U> Unpin for Bar<T, U> {} // Conditional Unpin impl

    let mut foo = Bar {
        field1: 1,
        field2: 2,
        _field3: (),
    };

    let mut foo = Pin::new(&mut foo);

    let x: Pin<&mut i32> = foo.as_mut().field1();
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.as_mut().field2();
    assert_eq!(*y, 2);

    // let _z = foo.as_mut()._field3(); // ERROR
}
