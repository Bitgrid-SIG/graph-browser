use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};

use syn::{
    Data, DeriveInput, Expr, Fields, Ident, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

type Csv<T> = Punctuated<T, Token![,]>;

#[derive(Debug)]
struct FieldValue {
    name: Ident,
    _colon: Token![:],
    expr: Expr,
}

struct ExprStructInline {
    _brace: syn::token::Brace,
    fields: Csv<FieldValue>,
}

enum EnumFields {
    SingleDefault(syn::Lit),
    Unnamed(Csv<Expr>),
    Named(ExprStructInline),
}

enum DiscriminantAttrField {
    Name(Ident),
    Fields(EnumFields),
}

struct DiscriminantAttrs(Csv<DiscriminantAttrField>);

fn camel_to_screaming(s: &Ident) -> Ident {
    let mut result = String::new();

    let mut prev_upper = false;
    for (i, ch) in s.to_string().chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 && !prev_upper {
                result.push('_');
            }
            for up in ch.to_uppercase() {
                result.push(up);
            }
            prev_upper = true;
        } else {
            result.push(ch.to_ascii_uppercase());
            prev_upper = false;
        }
    }

    format_ident!("{result}", span = s.span())
}

fn extract_repr(attrs: &[syn::Attribute]) -> Option<Ident> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("repr"))
        .and_then(|attr| {
            attr.parse_args_with(Csv::<syn::Ident>::parse_terminated)
                .ok()
        })
        .and_then(|idents| {
            idents.into_iter().find(|ident| {
                matches!(
                    ident.to_string().as_str(),
                    "u8" | "u16" | "u32" | "u64" | "u128"
                )
            })
        })
}

pub fn derive_enum_discriminants_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        abort! {
            enum_ident, "#[derive(EnumDiscriminants)] only works on enums"
        };
    };

    if extract_repr(&input.attrs).is_none() {
        abort! {
            enum_ident.span(), "No unsigned int repr found.";
            help = format!("#[repr(u8 | u16 | u32 | u64 | u128)]\nenum {enum_ident} {{...}}")
        };
    }

    let consts = data_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_is_upper = variant_ident.to_string().chars().all(|c| c.is_uppercase());

        // find and parse #[discriminant(...)]
        let discriminant_cfg_parsed = variant.attrs.iter()
            .find(|a| a.path().is_ident("discriminant"))
            .map(|a| a.parse_args::<DiscriminantAttrs>())
            .transpose();

        let discriminant_cfg = match discriminant_cfg_parsed {
            Ok(opt) => opt,
            Err(e) => return e.to_compile_error(),
        };

        match (&variant.fields, &discriminant_cfg) {
            // === errored unit variants ===
            (Fields::Unit, Some(cfg)) if cfg.default().is_some() => {
                abort! { variant_ident, "`default = ...` is not allowed on unit variants" };
            }

            (Fields::Unit, Some(cfg)) if cfg.defaults_named().is_some() || cfg.defaults_unnamed().is_some() => {
                abort! { variant_ident, "`defaults = ...` is not allowed on unit variants" };
            }

            // === valid unit variants ===
            (Fields::Unit, cfg_o) => {
                let const_ident = if let Some(cfg) = cfg_o {
                    match cfg.name().cloned() {
                        Some(name) => name,
                        None => camel_to_screaming(variant_ident),
                    }
                } else {
                    if variant_is_upper {
                        abort! {
                            variant_ident,
                            "variants that are already in all uppercase must have #[discriminant(name = ...)]"
                        };
                    }
                    camel_to_screaming(variant_ident)
                };
                let constructor = quote! { #enum_ident::#variant_ident };

                quote! {
                    pub const #const_ident: std::mem::Discriminant<#enum_ident>
                        = std::mem::discriminant(&#constructor);
                }
            }

            (fields, Some(cfg)) => {
                let field_count = match (fields, cfg.defaults()) {
                    (Fields::Named(fin), EnumFields::Named(_)) => fin.named.len(),
                    (Fields::Unnamed(fun), EnumFields::Unnamed(_) | EnumFields::SingleDefault(_)) => fun.unnamed.len(),
                    _ => abort! { variant_ident, "mismatch between number of types in defaults and number of types in variant" },
                };

                // === non-unit variants with non-matching fields ===
                if cfg.check_field_count(field_count) {
                    abort! { variant_ident, "mismatch between number of elements in defaults and number of elements in variant" };
                }

                // === non-unit variants with valid #[discriminant(...)] ===
                let expr = cfg.defaults();

                let const_ident = match cfg.name().cloned() {
                    Some(name) => name,
                    None if variant_ident.to_string().chars().all(|c| c.is_uppercase()) => abort! {
                        variant_ident,
                        "non-unit variants that are already in all uppercase must have #[discriminant(name = ...)]"
                    },
                    None => camel_to_screaming(variant_ident),
                };

                let constructor = match fields {
                    Fields::Unnamed(_) => quote! { #enum_ident::#variant_ident (#expr) },
                    Fields::Named(_) => quote! { #enum_ident::#variant_ident #expr },
                    Fields::Unit => unreachable!(),
                };

                quote! {
                    pub const #const_ident: std::mem::Discriminant<#enum_ident> =
                        std::mem::discriminant(&#constructor);
                }
            }

            // === non-unit variants with missing #[discriminant(...)] ===
            (_, None) if variant_ident.to_string().chars().all(|c| c.is_uppercase()) => {
                abort! {
                    variant_ident,
                    "variants that are already in all uppercase must have #[discriminant(name = ...)]"
                }
            }

            (_, None) => {
                abort! {
                    variant_ident,
                    "non-unit variants must have #[discriminant(default = ...)] or #[discriminant(defaults = (default1, ...))] or #[discriminant(defaults = {field1: default1, ...})]"
                };
            }
        }
    });

    let expanded = quote! {
        impl #enum_ident {
            #(#consts)*

            #[inline(always)]
            pub const fn as_discriminant(&self) -> core::mem::Discriminant<Self> {
                core::mem::discriminant(self)
            }
        }

        impl core::cmp::PartialEq<core::mem::Discriminant<#enum_ident>> for #enum_ident {
            fn eq(&self, other: &core::mem::Discriminant<#enum_ident>) -> bool {
                self.as_discriminant() == *other
            }
        }
    };

    expanded.into()
}

impl EnumFields {
    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            EnumFields::SingleDefault(_) => 1,
            EnumFields::Named(fields) => fields.len(),
            EnumFields::Unnamed(fields) => fields.len(),
        }
    }
}

impl core::ops::Deref for ExprStructInline {
    type Target = Csv<FieldValue>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl DiscriminantAttrField {
    #[inline(always)]
    fn name(&self) -> Option<&Ident> {
        match self {
            DiscriminantAttrField::Name(ident) => Some(ident),
            DiscriminantAttrField::Fields(_) => None,
        }
    }

    #[inline(always)]
    fn fields(&self) -> Option<&EnumFields> {
        match self {
            DiscriminantAttrField::Name(_) => None,
            DiscriminantAttrField::Fields(fields) => Some(fields),
        }
    }
}

impl DiscriminantAttrs {
    #[inline(always)]
    fn name(&self) -> Option<&Ident> {
        self.0.iter().find_map(|daf| daf.name())
    }

    #[inline]
    fn default(&self) -> Option<&syn::Lit> {
        let Some(EnumFields::SingleDefault(default)) = self.0.iter().find_map(|daf| daf.fields())
        else {
            return None;
        };

        Some(default)
    }

    #[inline]
    fn defaults_named(&self) -> Option<&ExprStructInline> {
        let Some(EnumFields::Named(defaults_named)) = self.0.iter().find_map(|daf| daf.fields())
        else {
            return None;
        };

        Some(defaults_named)
    }

    #[inline]
    fn defaults_unnamed(&self) -> Option<&Csv<Expr>> {
        let Some(EnumFields::Unnamed(defaults_unnamed)) =
            self.0.iter().find_map(|daf| daf.fields())
        else {
            return None;
        };

        Some(defaults_unnamed)
    }

    fn check_field_count(&self, count: usize) -> bool {
        self.num_defaults() != count
    }

    fn num_defaults(&self) -> usize {
        self.0
            .iter()
            .find_map(|daf| daf.fields().map(|ef| ef.len()))
            .unwrap_or(0)
    }

    #[inline(always)]
    fn defaults(&self) -> &EnumFields {
        self.0
            .iter()
            .filter_map(|daf| daf.fields())
            .collect::<Vec<_>>()[0]
    }
}

impl Parse for DiscriminantAttrField {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        match key.to_string().as_str() {
            "default" => {
                let expr: syn::Lit = input.parse()?;
                Ok(DiscriminantAttrField::Fields(EnumFields::SingleDefault(
                    expr,
                )))
            }
            "defaults" => {
                if let Ok(tup) = input.parse::<syn::ExprTuple>() {
                    Ok(DiscriminantAttrField::Fields(EnumFields::Unnamed(
                        tup.elems,
                    )))
                } else if let Ok(sti) = input.parse::<ExprStructInline>() {
                    Ok(DiscriminantAttrField::Fields(EnumFields::Named(sti)))
                } else {
                    Err(input.error("#[discriminant(...)] key `defaults` expects a tuple of values. Did you mean key `default = ...` or `named_defaults = (...)`?"))
                }
            }
            "name" => {
                let id: Ident = input.parse::<Ident>()?;
                let val = id.to_string();
                if !val
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
                {
                    Err(input.error("`name` must be UPPER_SNAKE_CASE"))
                } else {
                    Ok(DiscriminantAttrField::Name(id))
                }
            }
            _ => Err(input.error("unexpected key in #[discriminant(...)]")),
        }
    }
}

impl Parse for DiscriminantAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let punctuated = input.parse_terminated(DiscriminantAttrField::parse, Token![,])?;
        Ok(Self(punctuated))
    }
}

impl Parse for FieldValue {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl Parse for ExprStructInline {
    fn parse(input: ParseStream) -> Result<Self> {
        let block;
        let brace = syn::braced!(block in input);

        let fields = Csv::<FieldValue>::parse_terminated(&block)?;

        Ok(Self {
            _brace: brace,
            fields,
        })
    }
}

impl quote::ToTokens for FieldValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let expr = &self.expr;
        tokens.extend(quote! { #name: #expr });
    }
}

impl quote::ToTokens for ExprStructInline {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = &self.fields;
        let ts = quote! { { #fields } };
        tokens.extend(ts);
    }
}

impl quote::ToTokens for EnumFields {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            EnumFields::Named(fields) => quote! { #fields },
            EnumFields::Unnamed(fields) => quote! { #fields },
            EnumFields::SingleDefault(field) => quote! { #field },
        });
    }
}
