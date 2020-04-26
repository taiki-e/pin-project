#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use pin_project::{pin_project, project_replace};
use std::{marker::PhantomData, pin::Pin};

#[project_replace] // Nightly does not need a dummy attribute to the function.
#[test]
fn project_replace_stmt_expr() {
    // struct

    #[pin_project(Replace)]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    let mut foo = Foo { field1: 1, field2: 2 };

    #[project_replace]
    let Foo { field1, field2 } = Pin::new(&mut foo).project_replace(Foo { field1: 42, field2: 43 });

    let _x: PhantomData<i32> = field1;

    let y: i32 = field2;
    assert_eq!(y, 2);

    // tuple struct

    #[pin_project(Replace)]
    struct Bar<T, U>(#[pin] T, U);

    let mut bar = Bar(1, 2);

    #[project_replace]
    let Bar(x, y) = Pin::new(&mut bar).project_replace(Bar(42, 43));

    let _x: PhantomData<i32> = x;
    let y: i32 = y;
    assert_eq!(y, 2);

    // enum

    #[pin_project(Replace)]
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

    let baz = Pin::new(&mut baz).project_replace(Baz::None);

    #[project_replace]
    match baz {
        Baz::Variant1(x, y) => {
            let _x: PhantomData<i32> = x;
            let y: i32 = y;
            assert_eq!(y, 2);
        }
        Baz::Variant2 { field1, field2 } => {
            let _x: PhantomData<i32> = field1;
            let _y: i32 = field2;
            panic!()
        }
        Baz::None => panic!(),
    }
}
