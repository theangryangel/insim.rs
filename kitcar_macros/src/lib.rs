//! Macros for the kitcar crate
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Convert an enum to something that impl a parse and parse_with_prefix function to handle chat
/// messages
#[proc_macro_derive(ParseChat, attributes(chat))]
pub fn derive_command_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        panic!("CommandParser can only be derived for enums");
    };

    let prefix = extract_prefix(&input.attrs);

    let mut match_arms = vec![];
    let mut help_entries = vec![];

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let cmd_name = variant_name.to_string().to_lowercase();

        let doc_string = extract_doc_comments(&variant.attrs);

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().expect("Missing field name"))
                    .collect();

                let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

                let required_fields: Vec<_> = field_names
                    .iter()
                    .zip(field_types.iter())
                    .filter(|(_, ty)| !is_option_type(ty))
                    .collect();

                let optional_fields: Vec<_> = field_names
                    .iter()
                    .zip(field_types.iter())
                    .enumerate()
                    .filter(|(_, (_, ty))| is_option_type(ty))
                    .collect();

                let parse_required: Vec<_> = required_fields.iter().enumerate()
                    .map(|(i, (name, ty))| {
                        let name_str = name.to_string();
                        quote! {
                            let #name = args.get(#i)
                                .ok_or_else(|| kitcar::chat::ParseError::MissingRequiredArg(#name_str.to_string()))?;
                            let #name = <#ty as kitcar::chat::FromArg>::from_arg(#name)
                                .map_err(|e| kitcar::chat::ParseError::InvalidArg(#name_str.to_string(), e))?;
                        }
                    })
                    .collect();

                let parse_optional: Vec<_> = optional_fields.iter()
                    .map(|(i, (name, ty))| {
                        // Extract T from Option<T>
                        let inner_ty = extract_option_inner(ty);
                        quote! {
                            let #name = args.get(#i)
                                .map(|s| <#inner_ty as kitcar::chat::FromArg>::from_arg(s))
                                .transpose()
                                .map_err(|e| kitcar::chat::ParseError::InvalidArg(stringify!(#name).to_string(), e))?;
                        }
                    })
                    .collect();

                // Generate help text
                let required_args: Vec<_> = required_fields
                    .iter()
                    .map(|(name, _)| format!("<{}>", name))
                    .collect();
                let optional_args: Vec<_> = optional_fields
                    .iter()
                    .map(|(_, (name, _))| format!("[{}]", name))
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

                help_entries.push(quote! {
                    #help_str
                });

                match_arms.push(quote! {
                    #cmd_name => {
                        #(#parse_required)*
                        #(#parse_optional)*
                        Ok(#enum_name::#variant_name { #(#field_names),* })
                    }
                });
            },
            Fields::Unnamed(_) => {
                panic!("CommandParser does not support tuple variants");
            },
            Fields::Unit => {
                let help_str = if let Some(doc) = doc_string {
                    format!(" - {}{} - {}", prefix.unwrap_or(' '), cmd_name, doc)
                } else {
                    format!(" - {}{}", prefix.unwrap_or(' '), cmd_name)
                };

                help_entries.push(quote! {
                    #help_str
                });

                match_arms.push(quote! {
                    #cmd_name => Ok(#enum_name::#variant_name)
                });
            },
        }
    }

    let strip = if let Some(prefix_char) = prefix.as_ref() {
        quote! {
            if let Some(stripped) = input.strip_prefix(#prefix_char) {
                stripped
            } else {
                return Err(kitcar::chat::ParseError::MissingPrefix(#prefix_char));
            }
        }
    } else {
        quote! {
            input
        }
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

                // Strip prefix if required
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

            fn try_from(value: &Mso) -> Result<Self, Self::Error> {
                Self::parse(value.msg_from_textstart())
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

fn extract_option_inner(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return inner;
    }
    ty
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
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

fn extract_prefix(attrs: &[syn::Attribute]) -> Option<char> {
    for attr in attrs {
        if attr.path().is_ident("chat")
            && let Ok(meta_list) = attr.meta.require_list()
            && let Ok(nested_metas) = meta_list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
        {
            for nested in nested_metas {
                if let syn::Meta::NameValue(nv) = nested
                    && nv.path.is_ident("prefix")
                    && let syn::Expr::Lit(expr_lit) = &nv.value
                {
                    if let syn::Lit::Char(lit_char) = &expr_lit.lit {
                        return Some(lit_char.value());
                    }
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let s = lit_str.value();
                        if s.len() == 1 {
                            return s.chars().next();
                        }
                    }
                }
            }
        }
    }
    None
}
