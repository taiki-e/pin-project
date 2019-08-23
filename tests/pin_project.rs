#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop, UnsafeUnpin};

#[test]
fn test_pin_project() {
    // struct

    #[pin_project]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    let mut foo = Foo { field1: 1, field2: 2 };

    let mut foo_orig = Pin::new(&mut foo);
    let foo = foo_orig.project();

    let x: Pin<&mut i32> = foo.field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.field2;
    assert_eq!(*y, 2);

    assert_eq!(foo_orig.as_ref().field1, 1);
    assert_eq!(foo_orig.as_ref().field2, 2);

    let mut foo = Foo { field1: 1, field2: 2 };

    let mut foo = Pin::new(&mut foo);
    let foo = foo.project();

    let __FooProjection { field1, field2 } = foo;
    let _: Pin<&mut i32> = field1;
    let _: &mut i32 = field2;

    // tuple struct

    #[pin_project]
    struct Bar<T, U>(#[pin] T, U);

    let mut bar = Bar(1, 2);

    let mut bar = Pin::new(&mut bar);
    let bar = bar.project();

    let x: Pin<&mut i32> = bar.0;
    assert_eq!(*x, 1);

    let y: &mut i32 = bar.1;
    assert_eq!(*y, 2);

    // enum

    #[pin_project]
    #[derive(Eq, PartialEq, Debug)]
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

    let mut baz_orig = Pin::new(&mut baz);
    let baz = baz_orig.project();

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

    assert_eq!(Pin::into_ref(baz_orig).get_ref(), &Baz::Variant1(1, 2));

    let mut baz = Baz::Variant2 { field1: 3, field2: 4 };

    let mut baz = Pin::new(&mut baz);
    let mut baz = baz.project();

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
fn enum_project_set() {

    #[pin_project]
    #[derive(Eq, PartialEq, Debug)]
    enum Bar {
        Variant1(#[pin] u8),
        Variant2(bool)
    }

    let mut bar = Bar::Variant1(25);
    let mut bar_orig = Pin::new(&mut bar);
    let bar_proj = bar_orig.project();

    match bar_proj {
        __BarProjection::Variant1(val) => {
            let new_bar = Bar::Variant2(val.as_ref().get_ref() == &25);
            bar_orig.set(new_bar);
        },
        _ => unreachable!()
    }

    assert_eq!(bar, Bar::Variant2(true));
}

#[test]
fn where_clause_and_associated_type_fields() {
    // struct

    #[pin_project]
    struct Foo<I>
    where
        I: Iterator,
    {
        #[pin]
        field1: I,
        field2: I::Item,
    }

    // enum

    #[pin_project]
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

    #[pin_project]
    pub struct Foo<'a, T: ?Sized> {
        field: &'a mut T,
    }

    // tuple struct
    #[pin_project]
    pub struct Bar<'a, T: ?Sized>(&'a mut T);

    // enum

    #[pin_project]
    enum Baz<'a, T: ?Sized> {
        Variant(&'a mut T),
    }
}

#[test]
fn overlapping_lifetime_names() {
    #[pin_project]
    pub struct Foo<'_pin, T> {
        field: &'_pin mut T,
    }
}

#[test]
fn combine() {
    #[pin_project(PinnedDrop, UnsafeUnpin)]
    pub struct Foo<T> {
        field_1: u8,
        #[pin]
        field_2: T,
    }

    #[pinned_drop]
    fn do_drop<T>(_: Pin<&mut Foo<T>>) {}

    #[allow(unsafe_code)]
    unsafe impl<T: Unpin> UnsafeUnpin for Foo<T> {}
}
