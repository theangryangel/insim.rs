//! Macros for the kitcar crate
use darling::{FromDeriveInput, FromField, FromVariant, ast};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, parse_macro_input};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(chat), supports(enum_any))]
struct ChatInput {
    ident: syn::Ident,
    data: ast::Data<ChatVariant, ()>,
    #[darling(default)]
    prefix: Option<char>,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(doc))]
struct ChatVariant {
    ident: syn::Ident,
    fields: ast::Fields<ChatField>,
    attrs: Vec<Attribute>,
}

impl ChatVariant {
    fn doc_string(&self) -> Option<String> {
        let docs: Vec<String> = self
            .attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc")
                    && let syn::Meta::NameValue(meta) = &attr.meta
                    && let syn::Expr::Lit(expr_lit) = &meta.value
                    && let syn::Lit::Str(lit_str) = &expr_lit.lit
                {
                    return Some(lit_str.value().trim().to_string());
                }
                None
            })
            .collect();

        if docs.is_empty() {
            None
        } else {
            Some(docs.join(" "))
        }
    }
}

#[derive(Debug, FromField)]
struct ChatField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

impl ChatField {
    fn is_option(&self) -> bool {
        if let syn::Type::Path(type_path) = &self.ty
            && let Some(segment) = type_path.path.segments.last()
        {
            return segment.ident == "Option";
        }
        false
    }

    fn option_inner(&self) -> &syn::Type {
        if let syn::Type::Path(type_path) = &self.ty
            && let Some(segment) = type_path.path.segments.last()
            && segment.ident == "Option"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
        {
            return inner;
        }
        &self.ty
    }
}

/// Convert an enum to something that impl a parse and parse_with_prefix function to handle chat
/// messages
#[proc_macro_derive(ParseChat, attributes(chat))]
pub fn derive_command_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let chat_input = match ChatInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let enum_name = &chat_input.ident;
    let prefix = chat_input.prefix;

    let variants = chat_input
        .data
        .take_enum()
        .expect("ChatInput only supports enums");

    let mut match_arms = vec![];
    let mut help_entries = vec![];

    for variant in &variants {
        let variant_name = &variant.ident;
        let cmd_name = variant_name.to_string().to_lowercase();
        let doc_string = variant.doc_string();

        match &variant.fields.style {
            ast::Style::Struct => {
                let fields: Vec<_> = variant.fields.iter().collect();

                let field_names: Vec<_> = fields
                    .iter()
                    .map(|f| f.ident.as_ref().expect("Missing field name"))
                    .collect();

                let required_fields: Vec<_> = fields.iter().filter(|f| !f.is_option()).collect();

                let optional_fields: Vec<_> = fields
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.is_option())
                    .collect();

                let parse_required: Vec<_> = required_fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let name = f.ident.as_ref().unwrap();
                        let name_str = name.to_string();
                        let ty = &f.ty;
                        quote! {
                            let #name = args.get(#i)
                                .ok_or_else(|| kitcar::chat::ParseError::MissingRequiredArg(#name_str.to_string()))?;
                            let #name = <#ty as kitcar::chat::FromArg>::from_arg(#name)
                                .map_err(|e| kitcar::chat::ParseError::InvalidArg(#name_str.to_string(), e))?;
                        }
                    })
                    .collect();

                let parse_optional: Vec<_> = optional_fields
                    .iter()
                    .map(|(i, f)| {
                        let name = f.ident.as_ref().unwrap();
                        let inner_ty = f.option_inner();
                        quote! {
                            let #name = args.get(#i)
                                .map(|s| <#inner_ty as kitcar::chat::FromArg>::from_arg(s))
                                .transpose()
                                .map_err(|e| kitcar::chat::ParseError::InvalidArg(stringify!(#name).to_string(), e))?;
                        }
                    })
                    .collect();

                let required_args: Vec<_> = required_fields
                    .iter()
                    .map(|f| format!("<{}>", f.ident.as_ref().unwrap()))
                    .collect();
                let optional_args: Vec<_> = optional_fields
                    .iter()
                    .map(|(_, f)| format!("[{}]", f.ident.as_ref().unwrap()))
                    .collect();
                let all_args = [required_args, optional_args].concat();
                let args_str = all_args.join(" ");

                let help_str = if let Some(doc) = doc_string {
                    format!(
                        " - {}{} {} - {}",
                        prefix.unwrap_or(' '),
                        cmd_name,
                        args_str,
                        doc
                    )
                } else {
                    format!(" - {}{} {}", prefix.unwrap_or(' '), cmd_name, args_str)
                };

                help_entries.push(quote! { #help_str });

                match_arms.push(quote! {
                    #cmd_name => {
                        #(#parse_required)*
                        #(#parse_optional)*
                        Ok(#enum_name::#variant_name { #(#field_names),* })
                    }
                });
            },
            ast::Style::Tuple => {
                return syn::Error::new_spanned(
                    variant_name,
                    "ParseChat does not support tuple variants",
                )
                .to_compile_error()
                .into();
            },
            ast::Style::Unit => {
                let help_str = if let Some(doc) = doc_string {
                    format!(" - {}{} - {}", prefix.unwrap_or(' '), cmd_name, doc)
                } else {
                    format!(" - {}{}", prefix.unwrap_or(' '), cmd_name)
                };

                help_entries.push(quote! { #help_str });

                match_arms.push(quote! {
                    #cmd_name => Ok(#enum_name::#variant_name)
                });
            },
        }
    }

    let strip = if let Some(prefix_char) = prefix {
        quote! {
            if let Some(stripped) = input.strip_prefix(#prefix_char) {
                stripped
            } else {
                return Err(kitcar::chat::ParseError::MissingPrefix(#prefix_char));
            }
        }
    } else {
        quote! { input }
    };

    let prefix_return = match prefix {
        Some(c) => quote! { Some(#c) },
        None => quote! { None },
    };

    let expanded = quote! {
        #[allow(missing_docs)]
        impl kitcar::chat::Parse for #enum_name {
            fn parse(input: &str) -> Result<Self, kitcar::chat::ParseError> {
                let input = input.trim();

                if input.is_empty() {
                    return Err(kitcar::chat::ParseError::EmptyInput);
                }

                let input = #strip;

                let parts: Vec<&str> = input.split_whitespace().collect();

                if parts.is_empty() {
                    return Err(kitcar::chat::ParseError::EmptyInput);
                }

                let cmd_name = parts[0];
                let args = &parts[1..];

                match cmd_name {
                    #(#match_arms,)*
                    _ => Err(kitcar::chat::ParseError::UnknownCommand(cmd_name.to_string()))
                }
            }

            fn help() -> Vec<&'static str> {
                vec![
                    #(#help_entries),*
                ]
            }

            fn prefix() -> Option<char> {
                #prefix_return
            }
        }

        impl TryFrom<&insim::insim::Mso> for #enum_name {
            type Error = kitcar::chat::ParseError;

            fn try_from(value: &insim::insim::Mso) -> Result<Self, Self::Error> {
                Self::parse(value.msg_from_textstart())
            }
        }
    };

    TokenStream::from(expanded)
}
