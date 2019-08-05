// compile-fail

#![deny(warnings)]

use pin_project::pin_projectable;

#[pin_projectable]
struct Struct1 {} //~ ERROR cannot be implemented for structs with zero fields

#[pin_projectable]
struct Struct2(); //~ ERROR cannot be implemented for structs with zero fields

#[pin_projectable]
struct Struct3; //~ ERROR cannot be implemented for structs with units

#[pin_projectable]
enum Enum1 {} //~ ERROR cannot be implemented for enums without variants

#[pin_projectable]
enum Enum2 {
    A = 2, //~ ERROR cannot be implemented for enums with discriminants
}

/* FIXME: cannot be implemented for enums that has no field.
#[pin_projectable]
enum Enum1 {
    A,
}
*/

fn main() {}
