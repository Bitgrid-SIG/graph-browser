mod discriminant;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_derive(EnumDiscriminants, attributes(discriminant))]
pub fn derive_enum_discriminants(input: TokenStream) -> TokenStream {
    std::panic::catch_unwind(|| discriminant::derive_enum_discriminants_impl(input)).unwrap_or_else(
        |e| {
            let msg = if let Some(s) = e.downcast_ref::<&'static str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "proc-macro derive panicked".to_string()
            };
            syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into()
        },
    )
}
