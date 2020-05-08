mod pub_ {
    use pin_project::pin_project;

    #[pin_project]
    pub struct Default(());

    #[pin_project(Replace)]
    pub struct Replace(());
}
pub mod pub_use {
    #[rustfmt::skip]
    pub use crate::pub_::__DefaultProjection; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::__DefaultProjectionRef; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::__ReplaceProjection; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::__ReplaceProjectionOwned; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::__ReplaceProjectionRef; //~ ERROR E0365

    // Confirm that the visibility of the original type is not changed.
    pub use crate::pub_::{Default, Replace};
}
pub mod pub_use2 {
    // Ok
    #[allow(unused_imports)]
    pub(crate) use crate::pub_::{
        __DefaultProjection, __DefaultProjectionRef, __ReplaceProjection, __ReplaceProjectionOwned,
        __ReplaceProjectionRef,
    };
}

mod pub_crate {
    use pin_project::pin_project;

    #[pin_project]
    pub(crate) struct Default(());

    #[pin_project(Replace)]
    pub(crate) struct Replace(());
}
pub mod pub_crate_use {
    // Ok
    #[allow(unused_imports)]
    pub(crate) use crate::pub_crate::{
        __DefaultProjection, __DefaultProjectionRef, __ReplaceProjection, __ReplaceProjectionOwned,
        __ReplaceProjectionRef,
    };
}

fn main() {}
