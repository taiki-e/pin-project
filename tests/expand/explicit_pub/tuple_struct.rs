// SPDX-License-Identifier: Apache-2.0 OR MIT

use pin_project::pin_project;

#[pin_project(pub project = TupleStructProj)]
pub struct TupleStruct<T, U>(#[pin] pub T, pub U);

fn main() {}
