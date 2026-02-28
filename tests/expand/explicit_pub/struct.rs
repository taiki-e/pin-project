// SPDX-License-Identifier: Apache-2.0 OR MIT

use pin_project::pin_project;

/// Test Struct
#[pin_project(pub project = StructProj, project_ref = StructProjRef)]
pub struct Struct<T, U> {
    /// Pinned field
    #[pin]
    pub pinned: T,
    /// UnPinned field
    pub unpinned: U,
}

fn main() {}
