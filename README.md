# pin-project

[![Build Status](https://travis-ci.org/taiki-e/pin-project.svg?branch=master)](https://travis-ci.org/taiki-e/pin-project)
[![version](https://img.shields.io/crates/v/pin-project.svg)](https://crates.io/crates/pin-project/)
[![documentation](https://docs.rs/pin-project/badge.svg)](https://docs.rs/pin-project/)
[![license](https://img.shields.io/crates/l/pin-project.svg)](https://crates.io/crates/pin-project/)

An attribute that would create a projection struct covering all the fields.

[Documentation](https://docs.rs/pin-project/)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
pin-project = "0.1.8"
```

Now, you can use pin-project:

```rust
use pin_project::unsafe_project;
```

The current version of pin-project requires Rust 1.33 or later.

## Examples

For structs:

```rust
use pin_project::unsafe_project;
use std::pin::Pin;

#[unsafe_project(Unpin)]
struct Foo<T, U> {
    #[pin]
    future: T,
    field: U,
}

impl<T, U> Foo<T, U> {
    fn baz(mut self: Pin<&mut Self>) {
        let this = self.project();
        let _: Pin<&mut T> = this.future; // Pinned reference to the field
        let _: &mut U = this.field; // Normal reference to the field
    }
}

// Automatically create the appropriate conditional Unpin implementation.
// impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
```

For enums:

```rust
use pin_project::{project, unsafe_project};
use std::pin::Pin;

#[unsafe_project(Unpin)]
enum Foo<T, U> {
    Future(#[pin] T),
    Done(U),
}

impl<T, U> Foo<T, U> {
    #[project] // Nightly does not need a dummy attribute to the function.
    fn baz(mut self: Pin<&mut Self>) {
        #[project]
        match self.project() {
            Foo::Future(future) => {
                let _: Pin<&mut T> = future;
            }
            Foo::Done(value) => {
                let _: &mut U = value;
            }
        }
    }
}

// Automatically create the appropriate conditional Unpin implementation.
// impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
```

See the [documentation](https://docs.rs/pin-project/) for more details.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
