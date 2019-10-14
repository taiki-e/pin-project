use auxiliary_macros::hidden_repr_cfg_any;
use pin_project::pin_project;

// `#[hidden_repr_cfg_any(packed)]` generates `#[cfg_attr(any(), repr(packed))]`.
// Since `cfg(any())` can never be true, it is okay for this to pass.
#[pin_project]
#[hidden_repr_cfg_any(packed)]
struct A {
    #[pin]
    field: u32,
}

fn main() {}
