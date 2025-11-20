//! Macros for the kitcar crate
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, FnArg, ItemFn, parse_macro_input};

fn is_cx(arg: &FnArg) -> bool {
    if let FnArg::Typed(pat_type) = arg {
        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
            return pat_ident.ident == "cx";
        }
    }
    false
}

/// Attribute macro that converts a function into a kitcar::ui::Component implementation
/// The parameter cx: &mut Scope is automatically injected.
/// It is expected the users will Camelcase their function name.
///
/// # Example
/// ```ignore
/// #[component]
/// fn MyComponent(name: String, count: u32) -> Option<Element> {
///     // component logic
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &item_fn.sig.ident;
    let fn_vis = &item_fn.vis;
    let fn_body = &item_fn.block;

    // Extract all parameters as props fields
    let mut fn_args = Vec::new();
    for arg in &item_fn.sig.inputs {
        if is_cx(arg) {
            return TokenStream::from(
                Error::new_spanned(
                    arg,
                    "Component functions cannot have cx parameter. This is automatically provided.",
                )
                .to_compile_error(),
            );
        }

        if let FnArg::Typed(pat_type) = arg {
            let pat_name = &pat_type.pat;
            let ty = &pat_type.ty;
            fn_args.push((pat_name.clone(), ty.clone()));
        }
    }

    if fn_args.is_empty() {
        return TokenStream::from(
            Error::new_spanned(&item_fn, "Component functions must have at least one prop")
                .to_compile_error(),
        );
    }

    // Generate Props struct
    let props_struct_name = quote::format_ident!("{}Props", fn_name);
    let props_struct_fields = fn_args.iter().map(|(name, ty)| {
        quote! { pub #name: #ty }
    });
    let props_names = fn_args.iter().map(|(name, _)| name);

    // Generate the component struct and impl
    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub struct #props_struct_name {
            #(#props_struct_fields),*
        }

        #[derive(Debug, Clone)]
        #fn_vis struct #fn_name;

        impl kitcar::ui::Component for #fn_name {
            type Props = #props_struct_name;

            fn render(props: Self::Props, cx: &mut Scope) -> Option<Element> {
                let #props_struct_name { #(#props_names),* } = props;
                #fn_body
            }
        }
    };

    TokenStream::from(expanded)
}

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
    };

    TokenStream::from(expanded)
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn extract_option_inner(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return inner;
                    }
                }
            }
        }
    }
    ty
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value().trim().to_string());
                        }
                    }
                }
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
        if attr.path().is_ident("chat") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(nested_metas) = meta_list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                ) {
                    for nested in nested_metas {
                        if let syn::Meta::NameValue(nv) = nested {
                            if nv.path.is_ident("prefix") {
                                if let syn::Expr::Lit(expr_lit) = &nv.value {
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
                }
            }
        }
    }
    None
}
