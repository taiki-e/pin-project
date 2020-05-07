mod argument {
    use pin_project::pin_project;

    #[pin_project]
    struct Unexpected1 {
        #[pin()] //~ ERROR unexpected token
        field: (),
    }

    #[pin_project]
    struct Unexpected2(#[pin(foo)] ()); //~ ERROR unexpected token

    #[pin_project]
    enum Unexpected3 {
        V(#[pin(foo)] ()), //~ ERROR unexpected token
    }

    #[pin_project]
    enum Unexpected4 {
        V {
            #[pin(foo)] //~ ERROR unexpected token
            field: (),
        },
    }

    #[pin_project(UnsafeUnpin,,)] //~ ERROR expected identifier
    struct Unexpected5 {
        #[pin]
        field: (),
    }

    #[pin_project(Foo)] //~ ERROR unexpected argument
    struct Unexpected6 {
        #[pin]
        field: (),
    }

    #[pin_project()] // Ok
    struct Unexpected7 {
        #[pin]
        field: (),
    }

    #[pin_project(UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
    struct UnsafeUnpin {
        #[pin]
        field: (),
    }

    #[pin_project(PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
    struct PinnedDrop {
        #[pin]
        field: (),
    }

    #[pin_project(Replace, Replace)] //~ ERROR duplicate `Replace` argument
    struct Replace {
        #[pin]
        field: (),
    }

    #[pin_project(PinnedDrop, UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
    struct Duplicate3 {
        #[pin]
        field: (),
    }

    #[pin_project(PinnedDrop, UnsafeUnpin, PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
    struct Duplicate4 {
        #[pin]
        field: (),
    }

    #[pin_project(PinnedDrop, Replace)] //~ ERROR arguments `PinnedDrop` and `Replace` are mutually exclusive
    struct Duplicate5 {
        #[pin]
        field: (),
    }
}

mod pin_attribute {
    use pin_project::pin_project;

    #[pin_project]
    #[pin] //~ ERROR may only be used on fields of structs or variants
    struct Struct {
        #[pin]
        field: (),
    }

    #[pin_project]
    enum Variant {
        #[pin] //~ ERROR may only be used on fields of structs or variants
        V(()),
    }

    #[pin_project]
    #[pin] //~ ERROR may only be used on fields of structs or variants
    enum Enum {
        V(()),
    }

    #[pin_project]
    struct DuplicateField1 {
        #[pin]
        #[pin] //~ ERROR duplicate #[pin] attribute
        field: (),
    }

    #[pin_project]
    struct DuplicateField2(
        #[pin]
        #[pin]
        (),
        //~^^ ERROR duplicate #[pin] attribute
    );

    #[pin_project]
    enum DuplicateField3 {
        V {
            #[pin]
            #[pin] //~ ERROR duplicate #[pin] attribute
            field: (),
        },
    }

    #[pin_project]
    enum DuplicateField4 {
        V(
            #[pin]
            #[pin]
            (),
            //~^^ ERROR duplicate #[pin] attribute
        ),
    }
}

mod pin_project_attribute {
    use pin_project::pin_project;

    #[pin_project]
    #[pin_project] //~ ERROR duplicate #[pin_project] attribute
    struct Duplicate {
        #[pin]
        field: (),
    }

    #[pin_project]
    struct Struct {} //~ ERROR may not be used on structs with zero fields

    #[pin_project]
    struct TupleStruct(); //~ ERROR may not be used on structs with zero fields

    #[pin_project]
    struct UnitStruct; //~ ERROR may not be used on structs with zero fields

    #[pin_project]
    enum EnumEmpty {} //~ ERROR may not be used on enums without variants

    #[pin_project]
    enum EnumDiscriminant {
        V = 2, //~ ERROR may not be used on enums with discriminants
    }

    #[pin_project]
    enum EnumZeroFields {
        Unit, //~ ERROR may not be used on enums with zero fields
        Tuple(),
        Struct {},
    }

    #[pin_project]
    union Union {
        //~^ ERROR may only be used on structs or enums
        f: (),
    }
}

fn main() {}
