mod invalid_argument {
    use pin_project::pin_project;

    #[pin_project]
    struct A<T> {
        #[pin()] //~ ERROR unexpected token
        field: T,
    }

    #[pin_project]
    struct B<T>(#[pin(foo)] T); //~ ERROR unexpected token

    #[pin_project]
    enum C<T> {
        A(#[pin(foo)] T), //~ ERROR unexpected token
    }

    #[pin_project]
    enum D<T> {
        A {
            #[pin(foo)] //~ ERROR unexpected token
            field: T,
        },
    }

    #[pin_project(UnsafeUnpin,,)] //~ ERROR expected identifier
    struct E<T> {
        #[pin]
        field: T,
    }

    #[pin_project(Foo)] //~ ERROR unexpected argument
    struct F<T> {
        #[pin]
        field: T,
    }
}

mod invalid_position {
    use pin_project::pin_project;

    #[pin_project]
    #[pin] //~ ERROR may only be used on fields of structs or variants
    struct Struct<T> {
        #[pin]
        field: T,
    }

    #[pin_project]
    enum Variant<T> {
        #[pin] //~ ERROR may only be used on fields of structs or variants
        A(T),
    }

    #[pin_project]
    #[pin] //~ ERROR may only be used on fields of structs or variants
    enum Enum<T> {
        A(T),
    }
}

mod duplicate_attribute {
    use pin_project::pin_project;

    #[pin_project]
    struct Field<T> {
        #[pin]
        #[pin] //~ ERROR duplicate #[pin] attribute
        field: T,
    }

    #[pin_project]
    #[pin_project] //~ ERROR duplicate #[pin_project] attribute
    struct Struct<T> {
        #[pin]
        field: T,
    }
}

mod duplicate_argument {
    use pin_project::pin_project;

    #[pin_project(UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
    struct UnsafeUnpin<T> {
        #[pin]
        field: T,
    }

    #[pin_project(PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
    struct PinnedDrop<T> {
        #[pin]
        field: T,
    }

    #[pin_project(Replace, Replace)] //~ ERROR duplicate `Replace` argument
    struct Replace<T> {
        #[pin]
        field: T,
    }

    #[pin_project(PinnedDrop, UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
    struct Duplicate3<T> {
        #[pin]
        field: T,
    }

    #[pin_project(PinnedDrop, UnsafeUnpin, PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
    struct Duplicate4<T> {
        #[pin]
        field: T,
    }

    #[pin_project(PinnedDrop, Replace)] //~ ERROR arguments `PinnedDrop` and `Replace` are mutually exclusive
    struct Duplicate5<T> {
        #[pin]
        field: T,
    }
}

mod unsupported_type {
    use pin_project::pin_project;

    #[pin_project]
    struct Struct1 {} //~ ERROR may not be used on structs with zero fields

    #[pin_project]
    struct Struct2(); //~ ERROR may not be used on structs with zero fields

    #[pin_project]
    struct Struct3; //~ ERROR may not be used on structs with units

    #[pin_project]
    enum Enum1 {} //~ ERROR may not be used on enums without variants

    #[pin_project]
    enum Enum2 {
        A = 2, //~ ERROR may not be used on enums with discriminants
    }

    #[pin_project]
    enum Enum3 {
        A, //~ ERROR may not be used on enums that have no field
        B,
    }

    #[pin_project]
    union Union {
        //~^ ERROR may only be used on structs or enums
        x: u8,
    }
}

fn main() {}
