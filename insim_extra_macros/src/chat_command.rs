// darling's expansion of `#[darling(multiple)]` trips this lint; it cannot be
// allowed on the struct.
#![allow(clippy::redundant_field_names)]

use darling::{FromDeriveInput, FromField, FromVariant, ast};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(chat), supports(enum_unit, enum_named))]
pub(crate) struct Receiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<Variant, ()>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(chat), forward_attrs(doc))]
struct Variant {
    ident: syn::Ident,
    fields: ast::Fields<Field>,
    attrs: Vec<syn::Attribute>,
    #[darling(default)]
    rename: Option<String>,
    #[darling(multiple)]
    alias: Vec<String>,
    #[darling(default)]
    admin: bool,
}

#[derive(Debug, FromField)]
struct Field {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

/// How one field is read from the arguments.
enum Kind {
    /// One word, parsed with `FromStr`.
    Required,
    /// One word if there is one, parsed with `FromStr`.
    Optional,
    /// The rest of the line, which must not be empty. Last field only.
    Rest,
    /// The rest of the line, if any. Last field only.
    RestOptional,
}

/// The last path segment of `ty`, if it is a plain path.
fn last_segment(ty: &syn::Type) -> Option<&syn::PathSegment> {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() => p.path.segments.last(),
        _ => None,
    }
}

/// `T` if `ty` is written as `Option<T>`.
fn option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    let seg = last_segment(ty)?;
    if seg.ident != "Option" {
        return None;
    }
    match &seg.arguments {
        syn::PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
            match args.args.first()? {
                syn::GenericArgument::Type(inner) => Some(inner),
                _ => None,
            }
        },
        _ => None,
    }
}

fn is_string(ty: &syn::Type) -> bool {
    last_segment(ty).is_some_and(|seg| seg.ident == "String" && seg.arguments.is_none())
}

/// `CamelCase` to `snake_case`.
fn snake_case(ident: &str) -> String {
    let mut out = String::with_capacity(ident.len() + 4);
    for (i, c) in ident.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Doc comment lines, joined into one.
fn doc_text(attrs: &[syn::Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
        {
            let line = s.value();
            let line = line.trim();
            if !line.is_empty() {
                lines.push(line.to_owned());
            }
        }
    }
    lines.join(" ")
}

impl Receiver {
    pub(crate) fn expand(&self) -> darling::Result<TokenStream> {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let variants = match &self.data {
            ast::Data::Enum(variants) => variants,
            ast::Data::Struct(_) => unreachable!("darling only accepts enums"),
        };

        let mut errors = darling::Error::accumulator();
        let mut arms = Vec::new();
        let mut usages = Vec::new();
        let mut seen: Vec<(String, &syn::Ident)> = Vec::new();

        for variant in variants {
            let Some((arm, usage)) = errors.handle(variant.expand(&mut seen)) else {
                continue;
            };
            arms.push(arm);
            usages.push(usage);
        }
        errors.finish()?;

        Ok(quote! {
            impl #impl_generics ::core::str::FromStr for #ident #ty_generics #where_clause {
                type Err = ::insim_extra::chat::ChatParseError;

                fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                    let mut __args = ::insim_extra::chat::__private::Args::new(s);
                    let __name = __args.command()?;
                    #(#arms)*
                    ::core::result::Result::Err(
                        ::insim_extra::chat::ChatParseError::Unknown(
                            ::std::string::ToString::to_string(__name),
                        ),
                    )
                }
            }

            impl #impl_generics ::insim_extra::chat::ChatCommand for #ident #ty_generics #where_clause {
                const COMMANDS: &'static [::insim_extra::chat::CommandUsage] = &[
                    #(#usages),*
                ];
            }
        })
    }
}

impl Variant {
    fn expand<'a>(
        &'a self,
        seen: &mut Vec<(String, &'a syn::Ident)>,
    ) -> darling::Result<(TokenStream, TokenStream)> {
        let ident = &self.ident;
        let name = self
            .rename
            .clone()
            .unwrap_or_else(|| snake_case(&ident.to_string()));

        let mut errors = darling::Error::accumulator();
        for n in std::iter::once(&name).chain(&self.alias) {
            if n.is_empty() || n.contains(char::is_whitespace) {
                errors.push(
                    darling::Error::custom(format!(
                        "command name `{n}` must be non-empty with no whitespace"
                    ))
                    .with_span(ident),
                );
            }
            let lower = n.to_lowercase();
            if let Some((_, other)) = seen.iter().find(|(s, _)| *s == lower) {
                errors.push(
                    darling::Error::custom(format!(
                        "command name `{n}` is already used by `{other}`"
                    ))
                    .with_span(ident),
                );
            }
            seen.push((lower, ident));
        }

        let fields = &self.fields.fields;
        let mut kinds = Vec::with_capacity(fields.len());
        let mut seen_optional = false;
        for (i, field) in fields.iter().enumerate() {
            let last = i + 1 == fields.len();
            let kind = match option_inner(&field.ty) {
                Some(inner) if last && is_string(inner) => Kind::RestOptional,
                Some(_) => Kind::Optional,
                None if last && is_string(&field.ty) => Kind::Rest,
                None => Kind::Required,
            };
            let optional = matches!(kind, Kind::Optional | Kind::RestOptional);
            if seen_optional && !optional {
                errors.push(
                    darling::Error::custom(
                        "required arguments cannot follow optional (`Option<_>`) ones",
                    )
                    .with_span(&field.ty),
                );
            }
            seen_optional |= optional;
            kinds.push(kind);
        }
        errors.finish()?;

        let reads = fields.iter().zip(&kinds).map(|(field, kind)| {
            let field_ident = field.ident.as_ref().expect("named fields");
            let arg = field_ident.to_string();
            let ty = &field.ty;
            match kind {
                Kind::Required => quote! { let #field_ident: #ty = __args.required(#name, #arg)?; },
                Kind::Optional => quote! { let #field_ident: #ty = __args.optional(#name, #arg)?; },
                Kind::Rest => quote! { let #field_ident: #ty = __args.rest(#name, #arg)?; },
                Kind::RestOptional => quote! { let #field_ident: #ty = __args.rest_optional(); },
            }
        });
        let field_idents = fields
            .iter()
            .map(|f| f.ident.as_ref().expect("named fields"));
        let construct = match self.fields.style {
            ast::Style::Unit => quote! { Self::#ident },
            _ => quote! { Self::#ident { #(#field_idents),* } },
        };

        let names = std::iter::once(&name).chain(&self.alias);
        let arm = quote! {
            if #(__name.eq_ignore_ascii_case(#names))||* {
                #(#reads)*
                __args.finish(#name)?;
                return ::core::result::Result::Ok(#construct);
            }
        };

        let usage_args = fields
            .iter()
            .zip(&kinds)
            .map(|(field, kind)| {
                let arg = field.ident.as_ref().expect("named fields").to_string();
                match kind {
                    Kind::Required => format!("<{arg}>"),
                    Kind::Optional => format!("[{arg}]"),
                    Kind::Rest => format!("<{arg}...>"),
                    Kind::RestOptional => format!("[{arg}...]"),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let aliases = &self.alias;
        let help = doc_text(&self.attrs);
        let admin = self.admin;
        let usage = quote! {
            ::insim_extra::chat::CommandUsage::__new(#name, &[#(#aliases),*], #usage_args, #help, #admin)
        };

        Ok((arm, usage))
    }
}
