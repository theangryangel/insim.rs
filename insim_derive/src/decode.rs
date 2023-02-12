use crate::{extract_repr_type, extract_type, gen_field_name, Receiver};
use darling::{ast::Data, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;

impl Receiver {
    pub fn to_decode_tokens(&self) -> TokenStream {
        match &self.data {
            Data::Enum(_) => self.to_enum_decode_tokens(),
            Data::Struct(_) => self.to_struct_decode_tokens(),
        }
    }

    fn decode_magic_tokens(&self) -> TokenStream {
        if let Some(magic) = &self.magic {
            quote! {
                let magic = #magic;
                for expected_magic_byte in magic {
                    let decoded_magic_byte = u8::decode(buf, None)?;
                    if *expected_magic_byte != decoded_magic_byte {
                        return Err(::insim_core::DecodableError::MissingMagic(format!("Expected {:?}", magic)));
                    }
                }
            }
        } else {
            quote! {}
        }
    }

    fn to_enum_decode_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        let ident = &self.ident;

        let repr_ty = extract_repr_type(&self.attrs);

        let variants_to_index = self.data.as_ref().take_enum().unwrap();

        for (i, v) in variants_to_index.iter().enumerate() {
            let variant_name = gen_field_name(&Some(v.ident.clone()), i);

            // FIXME, remove the unwrap, give a decent message
            let discriminant = &v.discriminant.as_ref().unwrap();

            let mut variant_field_tokens = TokenStream::new();
            let mut variant_field_names = TokenStream::new();

            for (j, f) in v.fields.iter().enumerate() {
                if f.skip {
                    continue;
                }

                if f.count.is_some() {
                    // FIXME, we should return an Err here really
                    panic!("count is unsupported on enum fields");
                }

                if f.bytes.is_some() {
                    // FIXME, we should return an Err here really
                    panic!("bytes are unsupported on enum fields");
                }

                if let Some(size) = f.pad_bytes_before {
                    tokens.extend(quote! {
                        buf.advance(#size);
                    });
                }

                let field_name = if v.fields.style == darling::ast::Style::Struct {
                    format_ident!("{}", f.ident.as_ref().unwrap())
                } else {
                    format_ident!("f{}", j)
                };

                let field_ty = &f.ty.to_token_stream();

                variant_field_names.extend(quote! {
                    #field_name,
                });

                variant_field_tokens.extend(quote! {
                    let #field_name = #field_ty::decode(buf, None)?;
                });

                if let Some(size) = f.pad_bytes_after {
                    tokens.extend(quote! {
                        buf.advance(#size);
                    });
                }
            }

            match v.fields.style {
                darling::ast::Style::Tuple => {
                    tokens.extend(quote! {
                        #discriminant => {
                            #variant_field_tokens

                            #ident::#variant_name(#variant_field_names)
                        },
                    });
                }
                darling::ast::Style::Struct => {
                    tokens.extend(quote! {
                        #discriminant => {
                            #variant_field_tokens

                            #ident::#variant_name{ #variant_field_names }
                        },
                    });
                }
                darling::ast::Style::Unit => {
                    tokens.extend(quote! {
                        #discriminant => {
                            #ident::#variant_name
                        },
                    });
                }
            }
        }

        let magic = self.decode_magic_tokens();

        quote! {
            #magic

            let res = match #repr_ty::decode(buf, None)? {
                #tokens

                unmatched => {
                    return Err(::insim_core::DecodableError::UnmatchedDiscrimnant(
                        format!("found {}", unmatched)
                    ))
                },
            };

            Ok(res)
        }
    }

    fn to_struct_decode_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        tokens.extend(self.decode_magic_tokens());

        tokens.extend(quote! {
            let mut data = Self::default();
        });

        let fields_to_index = self.data.as_ref().take_struct().unwrap();

        for (i, f) in fields_to_index.iter().enumerate() {
            let ident = gen_field_name(&f.ident, i);

            let ty = extract_type(&f.ty);

            if f.skip {
                continue;
            }

            let mut pad_bytes_before = 0;

            tokens.extend(quote! {
                let initial_remaining = buf.remaining();
            });

            if let Some(size) = f.pad_bytes_before {
                pad_bytes_before = size;
                tokens.extend(quote! {
                    buf.advance(#size);
                });
            }

            let size = if let Some(count) = &f.count {
                let count_ident = format_ident!("{}", count);
                quote! {
                    Some(::insim_core::ser::Limit::Count(data.#count_ident as usize))
                }
            } else if let Some(bytes) = &f.bytes {
                quote! {
                    Some(::insim_core::ser::Limit::Bytes(#bytes as usize))
                }
            } else {
                quote! {
                    None
                }
            };

            tokens.extend(quote! {
                data.#ident = #ty::decode(buf, #size)?;
            });

            if let Some(bytes) = f.bytes {
                let ident_as_string = ident.to_string();
                tokens.extend(quote! {
                    let read_len = (initial_remaining - #pad_bytes_before - buf.remaining());
                    if #bytes > read_len {

                        if (#bytes - read_len) >= buf.remaining() {
                            buf.advance(#bytes - read_len);
                        } else {
                            return Err(::insim_core::DecodableError::NotEnoughBytes(
                                format!(
                                    "Not enough data for {}, wanted {} bytes, remaining {} bytes",
                                    #ident_as_string,
                                    #bytes,
                                    buf.remaining()
                                )
                            ))
                        }
                    }

                    if #bytes < read_len {
                        return Err(::insim_core::DecodableError::NotEnoughBytes(
                            format!(
                                "Not enough data for {}, wanted {}, read {}",
                                #ident_as_string,
                                #bytes,
                                read_len
                            )
                        ))
                    }
                });
            }

            if let Some(size) = f.pad_bytes_after {
                tokens.extend(quote! {
                    buf.advance(#size);
                });
            }
        }

        tokens.extend(quote! {
            Ok(data)
        });

        tokens
    }
}
