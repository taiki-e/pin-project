mod pin_argument {
    use pin_project::pin_project;

    #[pin_project]
    struct Struct {
        #[pin()] //~ ERROR unexpected token
        field: (),
    }

    #[pin_project]
    struct TupleStruct(#[pin(foo)] ()); //~ ERROR unexpected token

    #[pin_project]
    enum EnumTuple {
        V(#[pin(foo)] ()), //~ ERROR unexpected token
    }

    #[pin_project]
    enum EnumStruct {
        V {
            #[pin(foo)] //~ ERROR unexpected token
            field: (),
        },
    }
}

mod pin_attribute {
    use pin_project::pin_project;

    #[pin_project]
    struct DuplicateStruct {
        #[pin]
        #[pin] //~ ERROR duplicate #[pin] attribute
        field: (),
    }

    #[pin_project]
    struct DuplicateTupleStruct(
        #[pin]
        #[pin]
        (),
        //~^^ ERROR duplicate #[pin] attribute
    );

    #[pin_project]
    enum DuplicateEnumTuple {
        V(
            #[pin]
            #[pin]
            (),
            //~^^ ERROR duplicate #[pin] attribute
        ),
    }

    #[pin_project]
    enum DuplicateEnumStruct {
        V {
            #[pin]
            #[pin] //~ ERROR duplicate #[pin] attribute
            field: (),
        },
    }
}

mod pin_item {
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
}

mod pin_project_argument {
    use pin_project::pin_project;

    #[pin_project(UnsafeUnpin,,)] //~ ERROR expected identifier
    struct Unexpected1(#[pin] ());

    #[pin_project(Foo)] //~ ERROR unexpected argument
    struct Unexpected2(#[pin] ());

    #[pin_project(,UnsafeUnpin)] //~ ERROR expected identifier
    struct Unexpected3(#[pin] ());

    #[pin_project()] // Ok
    struct Unexpected4(#[pin] ());

    #[pin_project(PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
    struct DuplicatePinnedDrop(#[pin] ());

    #[pin_project(Replace, Replace)] //~ ERROR duplicate `Replace` argument
    struct DuplicateReplace(#[pin] ());

    #[pin_project(UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
    struct DuplicateUnsafeUnpin(#[pin] ());

    #[pin_project(!Unpin, !Unpin)] //~ ERROR duplicate `!Unpin` argument
    struct DuplicateNotUnpin(#[pin] ());

    #[pin_project(PinnedDrop, UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
    struct Duplicate3(#[pin] ());

    #[pin_project(PinnedDrop, UnsafeUnpin, PinnedDrop, UnsafeUnpin)] //~ ERROR duplicate `PinnedDrop` argument
    struct Duplicate4(#[pin] ());

    #[pin_project(project = A, project = B)] //~ ERROR duplicate `project` argument
    struct DuplicateProject(#[pin] ());

    #[pin_project(project_ref = A, project_ref = B)] //~ ERROR duplicate `project_ref` argument
    struct DuplicateProjectRef(#[pin] ());

    #[pin_project(project_replace = A, project_replace = B)] //~ ERROR duplicate `project_replace` argument
    struct DuplicateProjectReplace(#[pin] ());

    #[pin_project(project_replace = A)] //~ ERROR `project_replace` argument can only be used together with `Replace` argument
    struct ProjectReplaceWithoutReplace(#[pin] ());

    #[pin_project(PinnedDrop, Replace)] //~ ERROR arguments `PinnedDrop` and `Replace` are mutually exclusive
    struct PinnedDropWithReplace1(#[pin] ());

    #[pin_project(Replace, UnsafeUnpin, PinnedDrop)] //~ ERROR arguments `PinnedDrop` and `Replace` are mutually exclusive
    struct PinnedDropWithReplace2(#[pin] ());

    #[pin_project(UnsafeUnpin, !Unpin)] //~ ERROR arguments `UnsafeUnpin` and `!Unpin` are mutually exclusive
    struct UnsafeUnpinWithNotUnpin1(#[pin] ());

    #[pin_project(!Unpin, PinnedDrop, UnsafeUnpin)] //~ ERROR arguments `UnsafeUnpin` and `!Unpin` are mutually exclusive
    struct UnsafeUnpinWithNotUnpin2(#[pin] ());

    #[pin_project(!)] //~ ERROR unexpected end of input, expected `Unpin`
    struct NotUnpin1(#[pin] ());

    #[pin_project(Unpin)] //~ ERROR unexpected argument
    struct NotUnpin2(#[pin] ());

    #[pin_project(project)] //~ ERROR expected `=`
    struct Project1(#[pin] ());

    #[pin_project(project = )] //~ ERROR unexpected end of input, expected identifier
    struct Project2(#[pin] ());

    #[pin_project(project_ref)] //~ ERROR expected `=`
    struct ProjectRef1(#[pin] ());

    #[pin_project(project_ref = )] //~ ERROR unexpected end of input, expected identifier
    struct ProjectRef2(#[pin] ());

    #[pin_project(project_replace)] //~ ERROR expected `=`
    struct ProjectReplace1(#[pin] ());

    #[pin_project(project_replace = )] //~ ERROR unexpected end of input, expected identifier
    struct ProjectReplace2(#[pin] ());
}

mod pin_project_attribute {
    use pin_project::pin_project;

    #[pin_project]
    #[pin_project] //~ ERROR duplicate #[pin_project] attribute
    struct Duplicate(#[pin] ());
}

mod pin_project_item {
    use pin_project::pin_project;

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

// #[repr(packed)] is always detected first, even on unsupported structs.
mod pin_project_item_packed {
    use pin_project::pin_project;

    #[pin_project]
    #[repr(packed)]
    struct Struct {} //~ ERROR may not be used on #[repr(packed)] types

    #[pin_project]
    #[repr(packed)]
    struct TupleStruct(); //~ ERROR may not be used on #[repr(packed)] types

    #[pin_project]
    #[repr(packed)]
    struct UnitStruct; //~ ERROR may not be used on #[repr(packed)] types
}

fn main() {}
