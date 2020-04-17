use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

fn self_in_macro_def() {
    #[pin_project(PinnedDrop)]
    pub struct Struct {
        x: usize,
    }

    #[pinned_drop]
    impl PinnedDrop for Struct {
        fn drop(self: Pin<&mut Self>) {
            macro_rules! t {
                () => {{
                    let _ = self; //~ ERROR can't capture dynamic environment in a fn item

                    fn f(self: ()) {
                        //~^ ERROR `self` parameter is only allowed in associated functions
                        let _ = self;
                    }
                }};
            }
            t!();
        }
    }
}

fn main() {}
