use pin_project::pin_project;

#[pin_project(Replace)]
struct TupleStruct<T, U>(#[pin] T, U);

fn main() {}
