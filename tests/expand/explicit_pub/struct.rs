// SPDX-License-Identifier: Apache-2.0 OR MIT

use pin_project::pin_project;

#[pin_project(pub project = StructProj, project_ref = StructProjRef)]
pub struct Struct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

fn main() {}
