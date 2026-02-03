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

#[derive(Debug, darling::FromField)]
#[darling(attributes(insim))]
pub(super) struct Field {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub pad_before: Option<usize>,
    pub pad_after: Option<usize>,
    pub skip: Option<bool>,
    pub codepage: Option<CodepageArgs>,
    pub ascii: Option<AsciiArgs>,
    pub duration: Option<syn::TypePath>,
}

impl Field {
    pub(super) fn skip(&self) -> bool {
        self.skip.unwrap_or(false)
    }

    pub(super) fn decode(
        &self,
        parent: &syn::Ident,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let f = self;
        let field_name = f
            .ident
            .as_ref()
            .ok_or_else(|| darling::Error::custom("missing field name").with_span(&f.ty))?;
        let pad_after = f.pad_after.unwrap_or(0);
        let pad_before = f.pad_before.unwrap_or(0);
        let field_type = f.ty.clone();
        let context = format!("{}::{}", parent, field_name);

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
                let #field_name = <#field_type as ::insim_core::DecodeString>::decode_codepage(
                    buf, #length
                ).map_err(|e|
                    e.nested().context(#context)
                )?;
            }
        } else if let Some(AsciiArgs { length, .. }) = f.ascii.as_ref() {
            tokens = quote! {
                #tokens
                let #field_name = <#field_type as ::insim_core::DecodeString>::decode_ascii(
                    buf, #length
                ).map_err(|e|
                    e.nested().context(#context)
                )?;
            }
        } else if let Some(duration_repr) = f.duration.as_ref() {
            tokens = quote! {
                #tokens
                let __raw_field_name = #duration_repr::decode(buf).map_err(|e|
                    e.nested().context(#context)
                )?;
                let #field_name = match TryInto::<u64>::try_into(__raw_field_name) {
                    Ok(v) => std::time::Duration::from_millis(v),
                    Err(_) => return Err(::insim_core::DecodeErrorKind::OutOfRange { min: 0, max: u64::MAX as usize, found: __raw_field_name as usize }.context(#context)),
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
                && path.leading_colon.is_none()
                && path.segments.len() == 1
                && let syn::PathArguments::AngleBracketed(ang) = &path.segments[0].arguments
            {
                let ident = &path.segments[0].ident;
                typ = quote! { #ident::#ang };
            }

            tokens = quote! {
                #tokens
                let #field_name = <#typ>::decode(buf).map_err(|e|
                    e.nested().context(#context)
                )?;
            };
        }

        if pad_after > 0 {
            tokens = quote! {
                #tokens
                <::bytes::Bytes as ::bytes::buf::Buf>::advance(buf, #pad_after);
            }
        }
        Ok(tokens)
    }

    pub(super) fn encode(
        &self,
        parent: &syn::Ident,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let f = self;
        let field_name = f
            .ident
            .as_ref()
            .ok_or_else(|| darling::Error::custom("missing field name").with_span(&f.ty))?;
        let pad_after = f.pad_after.unwrap_or(0);
        let pad_before = f.pad_before.unwrap_or(0);
        let field_type = f.ty.clone();
        let context = format!("{}::{}", parent, field_name);
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
                        <#field_type as ::insim_core::EncodeString>::encode_codepage(
                            &self.#field_name, buf, #length, #trailing_nul
                        ).map_err(|e|
                            e.nested().context(#context)
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
                        <#field_type as ::insim_core::EncodeString>::encode_codepage_with_alignment(
                            &self.#field_name, buf, #length, #align_to, #trailing_nul
                        ).map_err(|e|
                            e.nested().context(#context)
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
                <#field_type as ::insim_core::EncodeString>::encode_ascii(
                    &self.#field_name, buf, #length, #trailing_nul
                ).map_err(|e|
                    e.nested().context(#context)
                )?;
            };
        } else if let Some(duration_repr) = f.duration.as_ref() {
            tokens = quote! {
                #tokens
                match #duration_repr::try_from(self.#field_name.as_millis()) {
                    Ok(v) => v.encode(buf).map_err(|e|
                        e.nested().context(#context)
                    )?,
                    Err(_) => return Err(::insim_core::EncodeErrorKind::OutOfRange { min: 0, max: #duration_repr::MAX as usize, found: self.#field_name.as_millis() as usize}.context(#context))
                };
            };
        } else {
            tokens = quote! {
                #tokens
                self.#field_name.encode(buf).map_err(|e| e.nested().context(#context))?;
            };
        }

        if pad_after > 0 {
            tokens = quote! {
                #tokens
                <::bytes::BytesMut as ::bytes::buf::BufMut>::put_bytes(buf, 0, #pad_after);
            }
        }

        Ok(tokens)
    }
}
