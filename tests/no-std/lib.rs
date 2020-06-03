#![no_std]
#![warn(rust_2018_idioms, single_use_lifetimes)]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop, UnsafeUnpin};

include!("../include/basic.rs");
