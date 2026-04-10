//! Derive macro for `eon_schema::EonSchema`.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, GenericParam,
    Generics, Lit, LitStr, Meta, Type, parse_macro_input, parse_quote,
};

/// Derive `eon_schema::EonSchema` for named structs and enums.
#[proc_macro_derive(EonSchema, attributes(serde, deprecated))]
pub fn derive_eon_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let generics = add_schema_bounds(&input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let schema = match &input.data {
        Data::Struct(data) => expand_struct(input, data)?,
        Data::Enum(data) => expand_enum(input, data)?,
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "EonSchema cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics ::eon_schema::EonSchema for #ident #ty_generics #where_clause {
            fn schema() -> ::eon_schema::SchemaNode {
                #schema
            }
        }
    })
}

fn add_schema_bounds(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(ty) = param {
            ty.bounds.push(parse_quote!(::eon_schema::EonSchema));
        }
    }
    generics
}

fn expand_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<proc_macro2::TokenStream> {
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &data.fields,
            "EonSchema currently supports named-field structs",
        ));
    };

    let container = parse_container_attrs(&input.attrs)?;
    let name = container.rename.unwrap_or_else(|| input.ident.to_string());
    let name = lit(&name);
    let docs = lit(&doc_string(&input.attrs));
    let field_tokens = named_field_tokens(fields.named.iter())?;

    Ok(quote! {
        ::eon_schema::SchemaNode::Object(::eon_schema::ObjectSchema {
            name: #name,
            docs: #docs,
            fields: vec![#(#field_tokens),*],
            open: false,
            extensions: Vec::new(),
        })
    })
}

fn expand_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let container = parse_container_attrs(&input.attrs)?;
    let name = container.rename.unwrap_or_else(|| input.ident.to_string());
    let name = lit(&name);
    let docs = lit(&doc_string(&input.attrs));
    let mut variants = Vec::new();

    for variant in &data.variants {
        let attrs = parse_serde_attrs(&variant.attrs)?;
        if attrs.skip || attrs.skip_deserializing {
            continue;
        }

        let variant_name = attrs.rename.unwrap_or_else(|| variant.ident.to_string());
        let variant_name = lit(&variant_name);
        let variant_docs = lit(&doc_string(&variant.attrs));
        let deprecated = deprecated_tokens(&variant.attrs)?;
        let payload = match &variant.fields {
            Fields::Unit => quote!(::eon_schema::VariantPayload::Unit),
            Fields::Unnamed(fields) => {
                let values = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote!(<#ty as ::eon_schema::EonSchema>::schema())
                });
                quote!(::eon_schema::VariantPayload::Tuple(vec![#(#values),*]))
            }
            Fields::Named(fields) => {
                let fields = named_field_tokens(fields.named.iter())?;
                quote!(::eon_schema::VariantPayload::Struct(vec![#(#fields),*]))
            }
        };

        variants.push(quote! {
            ::eon_schema::VariantSchema {
                name: #variant_name,
                docs: #variant_docs,
                payload: #payload,
                deprecated: #deprecated,
                extensions: Vec::new(),
            }
        });
    }

    Ok(quote! {
        ::eon_schema::SchemaNode::Enum(::eon_schema::EnumSchema {
            name: #name,
            docs: #docs,
            variants: vec![#(#variants),*],
            extensions: Vec::new(),
        })
    })
}

fn named_field_tokens<'a>(
    fields: impl Iterator<Item = &'a Field>,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut out = Vec::new();
    for field in fields {
        if let Some(tokens) = field_token(field)? {
            out.push(tokens);
        }
    }
    Ok(out)
}

fn field_token(field: &Field) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let attrs = parse_serde_attrs(&field.attrs)?;
    if attrs.skip || attrs.skip_deserializing {
        return Ok(None);
    }

    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "expected a named field"))?;
    let field_name = attrs.rename.unwrap_or_else(|| ident.to_string());
    let field_name = lit(&field_name);
    let field_docs = lit(&doc_string(&field.attrs));
    let ty = &field.ty;
    let required = !attrs.default && !is_option_type(ty);
    let default = attrs.default;
    let deprecated = deprecated_tokens(&field.attrs)?;

    Ok(Some(quote! {
        ::eon_schema::FieldSchema {
            name: #field_name,
            docs: #field_docs,
            ty: <#ty as ::eon_schema::EonSchema>::schema(),
            required: #required,
            default: #default,
            deprecated: #deprecated,
            extensions: Vec::new(),
        }
    }))
}

#[derive(Default)]
struct ContainerAttrs {
    rename: Option<String>,
}

fn parse_container_attrs(attrs: &[Attribute]) -> syn::Result<ContainerAttrs> {
    let serde = parse_serde_attrs(attrs)?;
    Ok(ContainerAttrs {
        rename: serde.rename,
    })
}

#[derive(Default)]
struct SerdeAttrs {
    rename: Option<String>,
    default: bool,
    skip: bool,
    skip_deserializing: bool,
}

fn parse_serde_attrs(attrs: &[Attribute]) -> syn::Result<SerdeAttrs> {
    let mut out = SerdeAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                out.rename = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("default") {
                out.default = true;
                Ok(())
            } else if meta.path.is_ident("skip") {
                out.skip = true;
                Ok(())
            } else if meta.path.is_ident("skip_deserializing") {
                out.skip_deserializing = true;
                Ok(())
            } else {
                Ok(())
            }
        })?;
    }
    Ok(out)
}

fn deprecated_tokens(attrs: &[Attribute]) -> syn::Result<proc_macro2::TokenStream> {
    for attr in attrs {
        if !attr.path().is_ident("deprecated") {
            continue;
        }

        let mut note = "deprecated".to_owned();
        if let Meta::List(_) = &attr.meta {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("note") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    note = lit.value();
                }
                Ok(())
            })?;
        }
        let note = lit(&note);
        return Ok(quote!(Some(#note)));
    }

    Ok(quote!(None))
}

fn doc_string(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        let Meta::NameValue(name_value) = &attr.meta else {
            continue;
        };
        let Expr::Lit(ExprLit {
            lit: Lit::Str(doc), ..
        }) = &name_value.value
        else {
            continue;
        };
        lines.push(doc.value().trim().to_owned());
    }
    lines.join("\n")
}

fn is_option_type(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Option")
}

fn lit(value: &str) -> LitStr {
    LitStr::new(value, Span::call_site())
}
