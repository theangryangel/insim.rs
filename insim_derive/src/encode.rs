use std::collections::HashMap;

use darling::ast::Data;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{extract_repr_type, gen_field_name, Receiver};

impl Receiver {
    pub fn to_encode_tokens(&self) -> TokenStream {
        match &self.data {
            Data::Enum(_) => self.to_enum_encode_tokens(),
            Data::Struct(_) => self.to_struct_encode_tokens(),
        }
    }

    fn encode_magic_tokens(&self) -> TokenStream {
        if let Some(magic) = &self.magic {
            quote! {
                buf.put_slice(#magic);
            }
        } else {
            quote! {}
        }
    }

    fn to_enum_encode_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        let repr_ty = extract_repr_type(&self.attrs);

        let variants_to_index = self.data.as_ref().take_enum().unwrap();

        for (i, v) in variants_to_index.iter().enumerate() {
            let ident = v.ident.clone();

            let ident = gen_field_name(&Some(ident), i);

            // FIXME, remove the unwrap, give a decent message
            let discriminant = &v.discriminant.as_ref().unwrap();

            let mut variant_field_tokens = TokenStream::new();
            let mut variant_field_names = TokenStream::new();

            for (j, f) in v.fields.iter().enumerate() {
                let field_name = if v.fields.style == darling::ast::Style::Struct {
                    format_ident!("{}", f.ident.as_ref().unwrap())
                } else {
                    format_ident!("f{}", j)
                };

                variant_field_names.extend(quote! {
                    #field_name,
                });

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
                    variant_field_tokens.extend(quote! {
                        buf.put_bytes(0, #size);
                    });
                }

                variant_field_tokens.extend(quote! {
                    #field_name.encode(buf, None)?;
                });

                if let Some(size) = f.pad_bytes_after {
                    tokens.extend(quote! {
                        buf.put_bytes(0, #size);
                    });
                }
            }

            match v.fields.style {
                darling::ast::Style::Tuple => {
                    tokens.extend(quote! {
                        Self::#ident(#variant_field_names) => {
                            (#discriminant as #repr_ty).encode(buf, None)?;

                            #variant_field_tokens
                        },
                    });
                }
                darling::ast::Style::Struct => {
                    tokens.extend(quote! {
                        Self::#ident{#variant_field_names} => {
                            (#discriminant as #repr_ty).encode(buf, None)?;

                            #variant_field_tokens
                        },
                    });
                }
                darling::ast::Style::Unit => {
                    tokens.extend(quote! {
                        Self::#ident => {
                            (#discriminant as #repr_ty).encode(buf, None)?;
                        },
                    });
                }
            }
        }

        let magic = self.encode_magic_tokens();

        quote! {
            #magic

            match self {
                #tokens
            }
        }
    }

    fn to_struct_encode_tokens(&self) -> TokenStream {
        let fields_to_index = self.data.as_ref().take_struct().unwrap();

        let mut tokens = TokenStream::new();

        tokens.extend(self.encode_magic_tokens());

        let countable: HashMap<usize, proc_macro2::TokenStream> = fields_to_index
            .iter()
            .enumerate()
            .filter(|(_i, f)| f.count.is_some())
            .filter(|(_i, f)| !f.count.as_ref().unwrap().chars().all(char::is_numeric))
            .map(|(i, f)| {
                // f is countable

                // find the field index that we need to update
                let index_to_update = fields_to_index
                    .iter()
                    .enumerate()
                    .find_map(|(j, g)| -> Option<usize> {
                        match &f.count {
                            Some(name) => {
                                if name == &gen_field_name(&g.ident, j).to_string() {
                                    Some(j)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        }
                    })
                    .unwrap();

                let update_from_field = gen_field_name(&f.ident, i);

                (index_to_update, update_from_field)
            })
            .collect();

        for (i, f) in fields_to_index.iter().enumerate() {
            let ident = gen_field_name(&f.ident, i);
            let ty = &f.ty;

            if f.skip {
                continue;
            }

            let mut pad_bytes_before: usize = 0;

            tokens.extend(quote! {
                let initial_len: usize = buf.len();
            });

            if let Some(size) = f.pad_bytes_before {
                pad_bytes_before = size;
                tokens.extend(quote! {
                    buf.put_bytes(0, #size);
                });
            }

            if let Some(update_from) = countable.get(&i) {
                // this field was marked to be used as the count from another field, so we'll just
                // use that instead of the actual value

                let update_from = quote! { #update_from };
                tokens.extend(quote! {
                    (self.#update_from.len() as #ty).encode(buf, None)?;
                });
            } else {
                // this field was not a count-able field, is there a limit?

                let limit = if let Some(bytes) = f.bytes {
                    quote! { Some(::insim_core::ser::Limit::Bytes(#bytes)) }
                } else {
                    quote! { None }
                };

                tokens.extend(quote! {
                    self.#ident.encode(buf, #limit)?;
                });
            }

            if let Some(bytes) = f.bytes {
                let ident_as_string = ident.to_string();

                tokens.extend(quote! {
                    let written_len: usize = (buf.len() - #pad_bytes_before - initial_len);
                    if #bytes != written_len {
                        return Err(
                            ::insim_core::EncodableError::WrongSize(
                                format!(
                                    "Expected to write {} bytes, wrote {} for field {}",
                                    #bytes,
                                    written_len,
                                    #ident_as_string
                                )
                            )
                        )
                    }
                });
            }

            if let Some(size) = f.pad_bytes_after {
                tokens.extend(quote! {
                    buf.put_bytes(0, #size);
                });
            }
        }

        tokens
    }
}
