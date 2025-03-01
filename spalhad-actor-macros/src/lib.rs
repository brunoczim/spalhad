use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data,
    DeriveInput,
    Fields,
    FieldsNamed,
    FieldsUnnamed,
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_derive(CallSuperSet)]
pub fn derive_call_super_set(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(input.span(), "Only enums are supported")
            .into_compile_error()
            .into();
    };

    let params = &input.generics.params;
    let ident = &input.ident;
    let where_clause = &input.generics.where_clause;

    let mut cases = quote! {};
    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;

        cases = match &variant.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let Some(field) = named.first().filter(|_| named.len() == 1)
                else {
                    return syn::Error::new(
                        named.span(),
                        "Only exactly one field per variant is supported",
                    )
                    .into_compile_error()
                    .into();
                };
                let field_ident = &field.ident;
                quote! {
                    #cases
                    Self::#variant_ident { #field_ident } =>
                        ::spalhad_actor::CallSuperSet::reply_error(
                            #field_ident,
                            error
                        ),
                }
            },

            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                if unnamed.len() != 1 {
                    return syn::Error::new(
                        unnamed.span(),
                        "Only exactly one field per variant is supported",
                    )
                    .into_compile_error()
                    .into();
                }
                quote! {
                    #cases
                    Self::#variant_ident ( field_ident ) =>
                        ::spalhad_actor::CallSuperSet::reply_error(
                            field_ident,
                            error,
                        ),
                }
            },

            Fields::Unit => {
                return syn::Error::new(
                    variant.fields.span(),
                    "Only exactly one field per variant is supported",
                )
                .into_compile_error()
                .into();
            },
        };
    }

    let tokens = quote! {
        impl<#params> ::spalhad_actor::CallSuperSet for #ident<#params>
        #where_clause
        {
            fn reply_error<__ErrorType>(self, error: __ErrorType) -> bool
            where
                __ErrorType: Into<::anyhow::Error>,
            {
                match self {
                    #cases
                }
            }
        }
    };

    tokens.into()
}
