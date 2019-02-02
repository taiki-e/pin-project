#![recursion_limit = "128"]
#![deny(warnings)]
#![allow(dead_code)]

use pin_project::unsafe_project;
use std::pin::Pin;

#[test]
fn test_unsafe_project() {
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

    // enum

    #[unsafe_project(Unpin)]
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

    let mut baz = Baz::Variant2 {
        field1: 3,
        field2: 4,
    };

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

#[cfg(feature = "project_attr")]
use pin_project::project;

#[cfg(feature = "project_attr")]
#[project] // Nightly does not need a dummy attribute to the function.
#[test]
fn test_project_attr() {
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

    #[project]
    let Foo { field1, field2 } = Pin::new(&mut foo).project();

    let x: Pin<&mut i32> = field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = field2;
    assert_eq!(*y, 2);

    // tuple struct

    #[unsafe_project(Unpin)]
    struct Bar<T, U>(#[pin] T, U);

    let mut bar = Bar(1, 2);

    #[project]
    let Bar(x, y) = Pin::new(&mut bar).project();

    let x: Pin<&mut i32> = x;
    assert_eq!(*x, 1);

    let y: &mut i32 = y;
    assert_eq!(*y, 2);

    // enum

    #[unsafe_project(Unpin)]
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

    let mut baz = Pin::new(&mut baz).project();

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
    {
        if let Baz::Variant1(x, y) = baz {
            let x: Pin<&mut i32> = x;
            assert_eq!(*x, 1);

            let y: &mut i32 = y;
            assert_eq!(*y, 2);
        } else if let Option::Some(_) = Some(1) {
            // Check that don't replace different types by mistake
        }
    }
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
