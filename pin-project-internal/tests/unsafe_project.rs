#![recursion_limit = "128"]
#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use core::pin::Pin;
use pin_project_internal::{pin_project, pin_projectable};

#[test]
fn test_pin_projectable() {
    // struct

    #[pin_projectable]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    let mut foo = Foo { field1: 1, field2: 2 };

    let foo = Pin::new(&mut foo).project();

    let x: Pin<&mut i32> = foo.field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.field2;
    assert_eq!(*y, 2);

    // tuple struct

    #[pin_projectable]
    struct Bar<T, U>(#[pin] T, U);

    let mut bar = Bar(1, 2);

    let bar = Pin::new(&mut bar).project();

    let x: Pin<&mut i32> = bar.0;
    assert_eq!(*x, 1);

    let y: &mut i32 = bar.1;
    assert_eq!(*y, 2);

    // enum

    #[pin_projectable]
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

    let baz = Pin::new(&mut baz).project();

    match baz {
        __BazProjection::Variant1(x, y) => {
            let x: Pin<&mut i32> = x;
            assert_eq!(*x, 1);

            let y: &mut i32 = y;
            assert_eq!(*y, 2);
        }
        __BazProjection::Variant2 { field1, field2 } => {
            let _x: Pin<&mut i32> = field1;
            let _y: &mut i32 = field2;
        }
        __BazProjection::None => {}
    }

    let mut baz = Baz::Variant2 { field1: 3, field2: 4 };

    let mut baz = Pin::new(&mut baz).project();

    match &mut baz {
        __BazProjection::Variant1(x, y) => {
            let _x: &mut Pin<&mut i32> = x;
            let _y: &mut &mut i32 = y;
        }
        __BazProjection::Variant2 { field1, field2 } => {
            let x: &mut Pin<&mut i32> = field1;
            assert_eq!(**x, 3);

            let y: &mut &mut i32 = field2;
            assert_eq!(**y, 4);
        }
        __BazProjection::None => {}
    }

    if let __BazProjection::Variant2 { field1, field2 } = baz {
        let x: Pin<&mut i32> = field1;
        assert_eq!(*x, 3);

        let y: &mut i32 = field2;
        assert_eq!(*y, 4);
    }
}

#[test]
fn where_clause_and_associated_type_fields() {
    // struct

    #[pin_projectable]
    struct Foo<I>
    where
        I: Iterator,
    {
        #[pin]
        field1: I,
        field2: I::Item,
    }

    // enum

    #[pin_projectable]
    enum Baz<I>
    where
        I: Iterator,
    {
        Variant1(#[pin] I),
        Variant2(I::Item),
    }
}

#[test]
fn trait_bounds_on_type_generics() {
    // struct

    #[pin_projectable]
    pub struct Foo<'a, T: ?Sized> {
        field: &'a mut T,
    }

    // tuple struct
    #[pin_projectable]
    pub struct Bar<'a, T: ?Sized>(&'a mut T);

    // enum

    #[pin_projectable]
    enum Baz<'a, T: ?Sized> {
        Variant(&'a mut T),
    }
}

pin_project! {
    #[pin_projectable]
    pub struct Foo<'a> {
        was_dropped: &'a mut bool,
        #[pin] field_2: u8
    }

    #[pinned_drop]
    fn do_drop(foo: Pin<&mut Foo<'_>>) {
        **foo.project().was_dropped = true;
    }
}

#[test]
fn safe_project() {
    let mut was_dropped = false;
    drop(Foo { was_dropped: &mut was_dropped, field_2: 42 });
    assert!(was_dropped);
}
