use pin_project::pin_project;

#[pin_project(Replace, project = Proj, project_ref = ProjRef, project_replace = ProjOwn)]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {}
