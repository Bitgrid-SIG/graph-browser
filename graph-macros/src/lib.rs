mod builder_wrap;

use proc_macro::TokenStream;
use quote::quote;

// use builder_wrap::impl_builder_macro;
#[proc_macro_attribute]
pub fn builder_wrap_derive(_args: TokenStream, _input: TokenStream) -> TokenStream {
    // let args = syn::parse_macro_input!(args);
    // let input_struct = syn::parse_macro_input!(input);

    // impl_builder_macro(&ast)

    todo!()
}
