use pin_project::pin_project;

#[pin_project]
union Union {
    //~^ ERROR may only be used on structs or enums
    x: u8,
}

fn main() {}
