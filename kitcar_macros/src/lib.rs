//! Macros for the kitcar crate
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, FnArg, ItemFn, parse_macro_input};

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
        #[derive(Clone)]
        pub struct #props_struct_name {
            #(#props_struct_fields),*
        }

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

/// Converts a function into a Service implementation
///
/// # Example
/// ```ignore
/// #[service]
/// async fn MyService() {
///     // Service logic here - insim SpawnedHandle argument is auto-injected
/// }
/// ```
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let struct_name = &input_fn.sig.ident;

    let fn_body = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_asyncness = &input_fn.sig.asyncness;
    let stmts = &fn_body.stmts;

    let expanded = quote! {
        #fn_vis struct #struct_name;

        impl kitcar::Service for #struct_name {
            #fn_asyncness fn spawn(insim: insim::builder::SpawnedHandle) {
                #(#stmts)*
            }
        }
    };

    TokenStream::from(expanded)
}
