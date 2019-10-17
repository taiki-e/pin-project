use pin_project::pin_project;

trait Trait {}

#[pin_project]
pub struct Struct<T>
where
    Self: Trait, //~ ERROR cannot find type `Self` in this scope [E0411]
{
    x: usize,
    y: T,
}

fn main() {}
