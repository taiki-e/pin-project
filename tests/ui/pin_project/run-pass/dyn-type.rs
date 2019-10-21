use pin_project::pin_project;

#[pin_project]
struct Struct1 {
    a: i32,
    f: dyn std::fmt::Debug,
}

#[pin_project]
struct Struct2 {
    a: i32,
    #[pin]
    f: dyn std::fmt::Debug,
}

#[pin_project]
struct Struct3 {
    a: i32,
    f: dyn std::fmt::Debug + Send,
}

#[pin_project]
struct Struct4 {
    a: i32,
    #[pin]
    f: dyn std::fmt::Debug + Send,
}

fn main() {}
