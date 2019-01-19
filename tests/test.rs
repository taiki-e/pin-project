#![deny(warnings)]

use std::pin::Pin;

#[test]
fn test_unsafe_project() {
    use pin_project::unsafe_project;

    // struct

    #[unsafe_project(Unpin)]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    let mut foo = Foo {
        field1: 1,
        field2: 2,
    };

    let foo = Pin::new(&mut foo).project();

    let x: Pin<&mut i32> = foo.field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.field2;
    assert_eq!(*y, 2);

    // tuple struct

    #[unsafe_project(Unpin)]
    struct Bar<T, U>(#[pin] T, U);

    let mut bar = Bar(1, 2);

    let bar = Pin::new(&mut bar).project();

    let x: Pin<&mut i32> = bar.0;
    assert_eq!(*x, 1);

    let y: &mut i32 = bar.1;
    assert_eq!(*y, 2);
}

#[cfg(feature = "unsafe_fields")]
#[test]
fn test_unsafe_fields() {
    use pin_project::unsafe_fields;

    #[unsafe_fields(Unpin)]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    let mut foo = Foo {
        field1: 1,
        field2: 2,
    };

    let mut foo = Pin::new(&mut foo);

    let x: Pin<&mut i32> = foo.as_mut().field1();
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.as_mut().field2();
    assert_eq!(*y, 2);

    // skip

    #[unsafe_fields(Unpin)]
    struct Bar<T, U> {
        #[pin]
        field1: T,
        field2: U,
        #[skip]
        _field3: (),
    }

    let mut bar = Bar {
        field1: 1,
        field2: 2,
        _field3: (),
    };

    let mut bar = Pin::new(&mut bar);

    let x: Pin<&mut i32> = bar.as_mut().field1();
    assert_eq!(*x, 1);

    let y: &mut i32 = bar.as_mut().field2();
    assert_eq!(*y, 2);

    // let _z = bar.as_mut()._field3(); // ERROR
}

#[cfg(feature = "unsafe_variants")]
#[test]
fn test_unsafe_variants() {
    #![allow(dead_code)]

    use pin_project::unsafe_variants;

    #[unsafe_variants(Unpin)]
    enum Foo<A, B, C> {
        Variant1(#[pin] A, B),
        Variant2(C),
    }

    let mut foo = Foo::Variant1(1, 2);

    let mut foo = Pin::new(&mut foo);

    let x: Pin<&mut i32> = foo.as_mut().variant1().unwrap().0;
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.as_mut().variant1().unwrap().1;
    assert_eq!(*y, 2);

    let z: Option<&mut i32> = foo.as_mut().variant2();
    assert!(z.is_none());

    // skip

    #[unsafe_variants(Unpin)]
    enum Bar<A, B, C> {
        Variant1(#[pin] A, B, #[skip] ()),
        Variant2(C, #[skip] ()),
        None,
        #[skip]
        Empty(()),
    }

    let mut bar = Bar::Variant1(1, 2, ());

    let mut bar = Pin::new(&mut bar);

    let x: (Pin<&mut i32>, &mut i32) = bar.as_mut().variant1().unwrap();
    assert_eq!(*x.0, 1);
    assert_eq!(*x.1, 2);

    let y: Option<&mut i32> = bar.as_mut().variant2();
    assert!(y.is_none());

    // let _z = bar.as_mut().none(); // ERROR
    // let _z = bar.as_mut().empty(); // ERROR
}
