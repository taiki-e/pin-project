// compile-fail

#![deny(warnings)]

use pin_project::unsafe_project;

#[unsafe_project]
struct Struct1 {} //~ ERROR cannot be implemented for structs with zero fields

#[unsafe_project]
struct Struct2(); //~ ERROR cannot be implemented for structs with zero fields

#[unsafe_project]
struct Struct3; //~ ERROR cannot be implemented for structs with units

#[unsafe_project]
enum Enum1 {} //~ ERROR cannot be implemented for enums without variants

#[unsafe_project]
enum Enum2 {
    A = 2, //~ ERROR cannot be implemented for enums with discriminants
}

/* FIXME: cannot be implemented for enums that has no field.
#[unsafe_project]
enum Enum1 {
    A,
}
*/

fn main() {}
