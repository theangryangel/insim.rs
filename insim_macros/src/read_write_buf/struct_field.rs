use quote::quote;

#[derive(Debug, darling::FromMeta, Clone)]
pub(super) struct AsciiArgs {
    length: usize,
    #[darling(default)]
    trailing_nul: bool,
}

#[derive(Debug, darling::FromMeta, Clone)]
pub(super) struct CodepageArgs {
    length: usize,
    align_to: Option<usize>,
    #[darling(default)]
    trailing_nul: bool,
}

#[derive(Debug, darling::FromMeta, Clone)]
pub(super) enum DurationUoM {
    Milliseconds(syn::Path),
    Centiseconds(syn::Path),
    Deciseconds(syn::Path),
    Seconds(syn::Path),
}

impl DurationUoM {
    fn scale(&self) -> u64 {
        match self {
            DurationUoM::Milliseconds(_) => 1,
            DurationUoM::Centiseconds(_) => 10,
            DurationUoM::Deciseconds(_) => 100,
            DurationUoM::Seconds(_) => 1000,
        }
    }
    fn ty(&self) -> syn::Path {
        match self {
            DurationUoM::Milliseconds(v) => v,
            DurationUoM::Centiseconds(v) => v,
            DurationUoM::Deciseconds(v) => v,
            DurationUoM::Seconds(v) => v,
        }
        .clone()
    }
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(read_write_buf))]
pub(super) struct Field {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub pad_before: Option<usize>,
    pub pad_after: Option<usize>,
    pub skip: Option<bool>,
    pub codepage: Option<CodepageArgs>,
    pub ascii: Option<AsciiArgs>,
    pub duration: Option<DurationUoM>,
}

impl Field {
    pub(super) fn skip(&self) -> bool {
        self.skip.unwrap_or(false)
    }

    pub(super) fn impl_read_buf(&self) -> proc_macro2::TokenStream {
        let f = self;
        let field_name = f.ident.as_ref().unwrap();
        let pad_after = f.pad_after.unwrap_or(0);
        let pad_before = f.pad_before.unwrap_or(0);
        let field_type = f.ty.clone();

        let mut tokens = quote! {};

        if pad_before > 0 {
            tokens = quote! {
                #tokens
                <::bytes::Bytes as ::bytes::buf::Buf>::advance(buf, #pad_before);
            }
        }

        if let Some(CodepageArgs { length, .. }) = f.codepage.as_ref() {
            tokens = quote! {
                #tokens
                let #field_name = <#field_type as ::insim_core::Codepage>::from_codepage_bytes(
                    buf, #length
                )?;
            }
        } else if let Some(AsciiArgs { length, .. }) = f.ascii.as_ref() {
            tokens = quote! {
                #tokens
                let #field_name = <#field_type as ::insim_core::Ascii>::from_ascii_bytes(
                    buf, #length
                )?;
            }
        } else if let Some(duration_args) = f.duration.as_ref() {
            let duration_repr = duration_args.ty();
            let scale = duration_args.scale();

            tokens = quote! {
                #tokens
                let #field_name = match TryInto::<u64>::try_into(#duration_repr::decode(buf)?) {
                    Ok(v) => std::time::Duration::from_millis(v * #scale),
                    Err(_) => return Err(::insim_core::Error::TooLarge),
                };
            };
        } else {
            // converts a Vec<u8> into Vec::<u8> for usage in the decoding calls
            // if it doesnt match that format then just output the original type
            // wrapped in angled brackets
            let mut typ = quote! { #field_type };

            if let syn::Type::Path(syn::TypePath {
                qself: None,
                ref path,
                ..
            }) = field_type
            {
                if path.leading_colon.is_none() && path.segments.len() == 1 {
                    if let syn::PathArguments::AngleBracketed(ang) = &path.segments[0].arguments {
                        let ident = &path.segments[0].ident;
                        typ = quote! { #ident::#ang };
                    }
                }
            }

            tokens = quote! {
                #tokens
                let #field_name = <#typ>::decode(buf)?;
            };
        }

        if pad_after > 0 {
            tokens = quote! {
                #tokens
                <::bytes::Bytes as ::bytes::buf::Buf>::advance(buf, #pad_after);
            }
        }
        tokens
    }

    pub(super) fn impl_write_buf(&self) -> proc_macro2::TokenStream {
        let f = self;
        let field_name = f.ident.as_ref().unwrap();
        let pad_after = f.pad_after.unwrap_or(0);
        let pad_before = f.pad_before.unwrap_or(0);
        let field_type = f.ty.clone();
        let mut tokens = quote! {};

        if pad_before > 0 {
            tokens = quote! {
                #tokens
                <::bytes::BytesMut as ::bytes::buf::BufMut>::put_bytes(buf, 0, #pad_before);
            }
        }

        if let Some(codepage_args) = f.codepage.as_ref() {
            match codepage_args {
                CodepageArgs {
                    length,
                    align_to: None,
                    trailing_nul,
                } => {
                    tokens = quote! {
                        #tokens
                        <#field_type as ::insim_core::Codepage>::to_codepage_bytes(
                            &self.#field_name, buf, #length, #trailing_nul
                        )?;
                    }
                },
                CodepageArgs {
                    length,
                    align_to,
                    trailing_nul,
                } => {
                    tokens = quote! {
                        #tokens
                        <#field_type as ::insim_core::Codepage>::to_codepage_bytes_aligned(
                            &self.#field_name, buf, #length, #align_to, #trailing_nul
                        )?;
                    }
                },
            }
        } else if let Some(AsciiArgs {
            length,
            trailing_nul,
        }) = f.ascii.as_ref()
        {
            tokens = quote! {
                #tokens
                <#field_type as ::insim_core::Ascii>::to_ascii_bytes(
                    &self.#field_name, buf, #length, #trailing_nul
                )?;
            };
        } else if let Some(duration_args) = f.duration.as_ref() {
            let duration_repr = duration_args.ty().clone();
            let scale = duration_args.scale();

            tokens = quote! {
                #tokens
                match #duration_repr::try_from(self.#field_name.as_millis() / (#scale as u128)) {
                    Ok(v) => v.encode(buf)?,
                    Err(_) => return Err(::insim_core::Error::TooLarge)
                };
            };
        } else {
            tokens = quote! {
                #tokens
                self.#field_name.encode(buf)?;
            };
        }

        if pad_after > 0 {
            tokens = quote! {
                #tokens
                <::bytes::BytesMut as ::bytes::buf::BufMut>::put_bytes(buf, 0, #pad_after);
            }
        }

        tokens
    }
}
