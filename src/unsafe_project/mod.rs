mod enums;
mod structs;

use proc_macro2::TokenStream;
use syn::Item;

use crate::utils::compile_err;

/// The attribute name.
const NAME: &str = "unsafe_project";
/// The annotation for pinned type.
const PIN: &str = "pin";

pub(super) fn unsafe_project(args: TokenStream, input: TokenStream) -> TokenStream {
    match syn::parse2(input) {
        Ok(Item::Struct(item)) => structs::unsafe_project(args, item),
        Ok(Item::Enum(item)) => enums::unsafe_project(args, item),
        _ => compile_err("`unsafe_project` may only be used on structs or enums"),
    }
}
