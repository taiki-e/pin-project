mod argument {
    use pin_project::{pin_project, project};

    #[pin_project]
    struct A<T>(#[pin] T);

    #[project]
    fn unexpected_stmt1() {
        let mut x = A(0);
        #[project()] //~ ERROR unexpected token
        let A(_) = Pin::new(&mut x).project();
    }

    #[project]
    fn unexpected_stmt2() {
        let mut x = A(0);
        #[project(foo)] //~ ERROR unexpected token
        let A(_) = Pin::new(&mut x).project();
    }

    #[project()] // Ok
    fn unexpected_fn1() {}

    #[project(foo)] //~ ERROR unexpected token
    fn unexpected_fn2() {}
}

mod attribute {
    use pin_project::{pin_project, project};

    #[pin_project]
    struct A<T>(#[pin] T);

    #[project]
    fn duplicate_stmt() {
        let mut x = A(0);
        #[project]
        #[project] //~ ERROR duplicate #[project] attribute
        let A(_) = Pin::new(&mut x).project();
    }

    // FIXME: Using #[project] on a function that doesn't contain #[project] is no-op,
    //        but, ideally, it should be detected.
    #[project]
    #[project] // Ok
    fn duplicate_fn() {}
}

fn main() {}
