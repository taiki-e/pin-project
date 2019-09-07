#![cfg(feature = "project_attr")]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use core::pin::Pin;
use pin_project::{pin_project, project};

#[project] // Nightly does not need a dummy attribute to the function.
#[test]
fn project_stmt_expr() {
    // struct

    #[pin_project]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    let mut foo = Foo { field1: 1, field2: 2 };
    let mut foo = Pin::new(&mut foo);

    #[project]
    let Foo { field1, field2 } = foo.project();

    let x: Pin<&mut i32> = field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = field2;
    assert_eq!(*y, 2);

    // tuple struct

    #[pin_project]
    struct Bar<T, U>(#[pin] T, U);

    let mut bar = Bar(1, 2);
    let mut bar = Pin::new(&mut bar);

    #[project]
    let Bar(x, y) = bar.project();

    let x: Pin<&mut i32> = x;
    assert_eq!(*x, 1);

    let y: &mut i32 = y;
    assert_eq!(*y, 2);

    // enum

    #[pin_project]
    enum Baz<A, B, C, D> {
        Variant1(#[pin] A, B),
        Variant2 {
            #[pin]
            field1: C,
            field2: D,
        },
        None,
    }

    let mut baz = Baz::Variant1(1, 2);

    let mut baz = Pin::new(&mut baz);
    let mut baz = baz.project();

    #[project]
    match &mut baz {
        Baz::Variant1(x, y) => {
            let x: &mut Pin<&mut i32> = x;
            assert_eq!(**x, 1);

            let y: &mut &mut i32 = y;
            assert_eq!(**y, 2);
        }
        Baz::Variant2 { field1, field2 } => {
            let _x: &mut Pin<&mut i32> = field1;
            let _y: &mut &mut i32 = field2;
        }
        Baz::None => {}
    }

    #[project]
    let val = match &mut baz {
        Baz::Variant1(_, _) => true,
        Baz::Variant2 { .. } => false,
        Baz::None => false,
    };
    assert_eq!(val, true);
}

#[test]
fn project_impl() {
    #[pin_project]
    struct HasGenerics<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    #[project]
    impl<T, U> HasGenerics<T, U> {
        fn a(self) {
            let Self { field1, field2 } = self;

            let _x: Pin<&mut T> = field1;
            let _y: &mut U = field2;
        }
    }

    #[pin_project]
    struct NoneGenerics {
        #[pin]
        field1: i32,
        field2: u32,
    }

    #[project]
    impl NoneGenerics {}

    #[pin_project]
    struct HasLifetimes<'a, T, U> {
        #[pin]
        field1: &'a mut T,
        field2: U,
    }

    #[project]
    impl<T, U> HasLifetimes<'_, T, U> {}

    #[pin_project]
    struct HasOverlappingLifetimes<'_pin, T, U> {
        #[pin]
        field1: &'_pin mut T,
        field2: U,
    }

    #[allow(single_use_lifetimes)]
    #[project]
    impl<'_pin, T, U> HasOverlappingLifetimes<'_pin, T, U> {}

    #[pin_project]
    struct HasOverlappingLifetimes2<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    #[allow(single_use_lifetimes)]
    #[project]
    impl<T, U> HasOverlappingLifetimes2<T, U> {
        fn foo<'_pin>(&'_pin self) {}
    }
}
