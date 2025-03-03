use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute,
    Data,
    DeriveInput,
    Field,
    Fields,
    FieldsNamed,
    FieldsUnnamed,
    Generics,
    Ident,
    Meta,
    Result,
    Token,
    Type,
    WherePredicate,
    braced,
    bracketed,
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Bracket, Comma, Paren},
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BraceType {
    Paren,
    Curly,
}

#[derive(Debug, Clone)]
struct CallVariant<'a> {
    variant_ident: &'a Ident,
    brace_type: BraceType,
    field_ident: Ident,
    ty: &'a Type,
    flatten_tys: Option<Punctuated<Type, Comma>>,
}

impl<'a> CallVariant<'a> {
    pub fn new(
        variant_ident: &'a Ident,
        variant_attrs: &'a [Attribute],
        field: &'a Field,
    ) -> Result<Self> {
        let mut flatten_tys = None;
        for attribute in variant_attrs {
            if let Some(attr) = Attr::from_meta(&attribute.meta)? {
                match attr {
                    Attr::Flatten(tys) => {
                        flatten_tys = Some(tys);
                    },
                }
            }
        }

        let ty = &field.ty;

        let field_ident = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new("__field", field.span()));

        let brace_type = if field.ident.is_some() {
            BraceType::Curly
        } else {
            BraceType::Paren
        };

        Ok(Self { variant_ident, brace_type, ty, field_ident, flatten_tys })
    }

    pub fn reply_error_tokens(&self) -> TokenStream {
        let variant_ident = self.variant_ident;
        let field_ident = &self.field_ident;
        let field_tokens = match self.brace_type {
            BraceType::Paren => quote! { ( #field_ident ) },
            BraceType::Curly => quote! { { #field_ident } },
        };
        quote! {
            Self::#variant_ident #field_tokens =>
                ::spalhad_actor::CallSuperset::reply_error(
                    #field_ident,
                    error
                ),
        }
    }

    pub fn inject_tokens(
        &self,
        super_ident: &Ident,
        generics: &Generics,
    ) -> TokenStream {
        match &self.flatten_tys {
            Some(tys) => {
                let mut tokens = quote! {};
                for ty in tys {
                    let curr_tokens =
                        self.injection_tokens_for(ty, super_ident, generics);
                    tokens = quote! {
                        #tokens
                        #curr_tokens
                    };
                }
                tokens
            },
            None => self.injection_tokens_for(self.ty, super_ident, generics),
        }
    }

    fn injection_tokens_for(
        &self,
        call_ty: &Type,
        super_ident: &Ident,
        generics: &Generics,
    ) -> TokenStream {
        let variant_ident = self.variant_ident;
        let field_ty = self.ty;
        let params = &generics.params;
        let where_clause = &generics.where_clause;

        quote! {
            impl<#params>
                ::spalhad_actor::CallInjection<#call_ty>
                for #super_ident<#params>
            #where_clause
            {
                fn inject(
                    call: #call_ty,
                ) -> Self {
                    Self::#variant_ident(
                        <
                            #field_ty as
                            ::spalhad_actor::CallInjection::<#call_ty>
                        >::inject(call)
                    )
                }
            }
        }
    }
}

#[derive(Debug)]
enum Attr {
    Flatten(Punctuated<Type, Comma>),
}

impl Attr {
    pub fn from_meta(meta: &Meta) -> Result<Option<Self>> {
        if *meta.path() != parse_quote!(spalhad) {
            return Ok(None);
        }
        let meta_list = meta.require_list()?;
        syn::parse2(meta_list.tokens.clone()).map(Some)
    }
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "flatten" {
            let mut types = None;
            if input.peek(Paren) {
                let content;
                parenthesized!(
                    content in input
                );
                types = Some(content.parse_terminated(Type::parse, Token![,])?);
            } else if input.peek(Brace) {
                let content;
                braced!(
                    content in input
                );
                types = Some(content.parse_terminated(Type::parse, Token![,])?);
            } else if input.peek(Bracket) {
                let content;
                bracketed!(
                    content in input
                );
                types = Some(content.parse_terminated(Type::parse, Token![,])?);
            }
            let Some(types) = types else {
                Err(syn::Error::new(ident.span(), "missing type list"))?
            };
            Ok(Self::Flatten(types))
        } else {
            Err(syn::Error::new(ident.span(), "unknown attribute name"))
        }
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let Data::Enum(data_enum) = &input.data else {
        Err(syn::Error::new(input.span(), "Only enums are supported"))?
    };

    let mut cases = quote! {};
    let mut injections = quote! {};
    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;

        let call_variant = match &variant.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let Some(field) = named.first().filter(|_| named.len() == 1)
                else {
                    Err(syn::Error::new(
                        named.span(),
                        "Only exactly one field per variant is supported",
                    ))?
                };
                CallVariant::new(variant_ident, &variant.attrs, field)?
            },

            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let Some(field) =
                    unnamed.first().filter(|_| unnamed.len() == 1)
                else {
                    Err(syn::Error::new(
                        unnamed.span(),
                        "Only exactly one field per variant is supported",
                    ))?
                };
                CallVariant::new(variant_ident, &variant.attrs, field)?
            },

            Fields::Unit => Err(syn::Error::new(
                variant.fields.span(),
                "Only exactly one field per variant is supported",
            ))?,
        };

        let reply_error_tokens = call_variant.reply_error_tokens();
        cases = quote! { #cases #reply_error_tokens };

        let inject_tokens =
            call_variant.inject_tokens(&input.ident, &input.generics);
        injections = quote! { #injections #inject_tokens };
    }

    let params = &input.generics.params;
    let ty_ident = &input.ident;
    let where_clause = &input.generics.where_clause;

    let mut from_params = params.clone();
    from_params.push(parse_quote! { __I });
    from_params.push(parse_quote! { __O });

    let from_where_predicate: WherePredicate = parse_quote! {
        #ty_ident<#params>:
            ::spalhad_actor::CallInjection<::spalhad_actor::ActorCall<__I, __O>>
    };
    let mut from_where_clause = where_clause.clone();
    from_where_clause
        .get_or_insert_with(|| parse_quote! { where })
        .predicates
        .push(from_where_predicate);

    let tokens = quote! {
        impl<#params> ::spalhad_actor::CallSuperset for #ty_ident<#params>
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

        impl<#from_params> From<::spalhad_actor::ActorCall<__I, __O>>
            for #ty_ident<#params>
        #from_where_clause
        {
            fn from(call: ::spalhad_actor::ActorCall<__I, __O>) -> Self {
                <
                    Self as
                    ::spalhad_actor::CallInjection<
                        ::spalhad_actor::ActorCall<__I, __O>
                    >
                >::inject(call)
            }
        }

        #injections
    };

    Ok(tokens)
}
