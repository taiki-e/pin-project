use pin_project::pin_project;

#[pin_project(project = A, project_ref = A)] //~ ERROR E0428,E0308
struct Struct(#[pin] ());

fn main() {}
