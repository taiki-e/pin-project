// compile-fail

#![deny(warnings, unsafe_code)]

use pin_project::pin_project;

#[pin_project]
struct Struct1 {} //~ ERROR cannot be implemented for structs with zero fields

#[pin_project]
struct Struct2(); //~ ERROR cannot be implemented for structs with zero fields

#[pin_project]
struct Struct3; //~ ERROR cannot be implemented for structs with units

#[pin_project]
enum Enum1 {} //~ ERROR cannot be implemented for enums without variants

#[pin_project]
enum Enum2 {
    A = 2, //~ ERROR cannot be implemented for enums with discriminants
}

#[pin_project]
enum Enum1 {
    A, //~ ERROR cannot be implemented for enums that have no field
    B,
}

fn main() {}
