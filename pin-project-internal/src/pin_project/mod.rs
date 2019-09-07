use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, *};

mod attribute;
mod derive;

/// The annotation for pinned type.
const PIN: &str = "pin";

type Variants = Punctuated<Variant, token::Comma>;

pub(crate) fn attribute(args: TokenStream, input: Item) -> TokenStream {
    attribute::parse_attribute(args, input).unwrap_or_else(|e| e.to_compile_error())
}

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    derive::parse_derive(input).unwrap_or_else(|e| e.to_compile_error())
}

// We need to check this in both proc-macro-attribute and proc-macro-derive for the following reasons:
// * `cfg` may reduce fields after being checked by proc-macro-attribute.
// * If we check this only on proc-macro-derive, it may generate unhelpful error messages.
fn validate_struct(ident: &Ident, fields: &Fields) -> Result<()> {
    match fields {
        Fields::Named(FieldsNamed { named: f, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: f, .. })
            if f.is_empty() =>
        {
            Err(error!(
                fields,
                "#[pin_project] attribute may not be used on structs with zero fields"
            ))
        }
        Fields::Unit => {
            Err(error!(ident, "#[pin_project] attribute may not be used on structs with units"))
        }
        _ => Ok(()),
    }
}

fn validate_enum(brace_token: token::Brace, variants: &Variants) -> Result<()> {
    if variants.is_empty() {
        return Err(syn::Error::new(
            brace_token.span,
            "#[pin_project] attribute may not be used on enums without variants",
        ));
    }
    let has_field = variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "#[pin_project] attribute may not be used on enums with discriminants"))
        } else if let Fields::Unit = v.fields {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if has_field {
        Ok(())
    } else {
        Err(error!(
            variants,
            "#[pin_project] attribute may not be used on enums that have no field"
        ))
    }
}
