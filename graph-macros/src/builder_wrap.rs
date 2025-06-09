use crate::{TokenStream, quote};

#[allow(dead_code)]
pub(crate) fn impl_builder_macro(ast: &syn::DeriveInput) -> TokenStream {
    let _structname = &ast.ident;
    let bgen = quote! {};
    bgen.into()
}
