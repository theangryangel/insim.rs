use convert_case::Casing;
use darling::{ast, FromDeriveInput, FromField};
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(indexed_slab), supports(struct_any))]
struct StructData {
    pub name: Option<String>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,

    pub data: ast::Data<(), FieldData>,
}

impl StructData {
    fn named(&self) -> syn::Ident {
        if let Some(name) = &self.name {
            format_ident!("{}", name)
        } else {
            format_ident!("IndexedSlab{}", self.ident)
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(indexed_slab), and_then = "FieldData::validate")]
struct FieldData {
    pub rename: Option<String>,

    #[darling(default)]
    pub unique: bool,

    #[darling(default)]
    pub hashed: bool,

    // custom index type
    pub index_type: Option<String>,

    // custom iter type for given index
    pub iter_type: Option<String>,

    #[darling(default)]
    pub ordered: bool,

    // escape hatch incase we need to skip
    #[darling(default)]
    pub skip: bool,

    pub ident: Option<syn::Ident>,

    pub ty: syn::Type,
}

impl FieldData {
    fn validate(self) -> darling::Result<Self> {
        let mut e = darling::Error::accumulator();

        if self.hashed && self.ordered {
            e.push(darling::Error::custom("Cannot be both hashed and ordered"))
        }

        e.finish_with(self)
    }

    fn can_be_indexed(&self) -> bool {
        !self.skip && (self.unique || self.hashed || self.ordered)
    }

    fn is_indexable(&self) -> Option<&Self> {
        if self.can_be_indexed() {
            Some(self)
        } else {
            None
        }
    }

    fn named(&self) -> syn::Ident {
        if let Some(name) = &self.rename {
            format_ident!("{}", name)
        } else {
            self.ident.as_ref().unwrap().clone()
        }
    }

    fn index_ident(&self) -> syn::Ident {
        format_ident!("_{}_index", self.named())
    }

    fn index_field_definition(&self) -> proc_macro2::TokenStream {
        let field_name_string = self.named().to_string();
        let index_name = self.index_ident();
        let ty = &self.ty;

        let field_type = if let Some(t) = &self.index_type {
            syn::parse_str(t).unwrap()
        } else if self.hashed {
            quote! { ::std::collections::HashMap }
        } else if self.ordered {
            quote! { ::std::collections::BTreeMap }
        } else {
            panic!("Unknown index type on field '{}'", field_name_string)
        };

        if self.unique {
            quote! {
                #index_name: #field_type<#ty, usize>,
            }
        } else {
            quote! {
                #index_name: #field_type<#ty, Vec<usize>>,
            }
        }
    }

    fn clear_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();

        quote! {
            self.#index_name.clear();
        }
    }

    fn insert_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_ident = self.ident.as_ref().unwrap();
        let field_named = self.named().to_string();

        if self.unique {
            quote! {
                let orig_elem_idx = self.#index_name.insert(elem.#field_ident.clone(), idx);
                if orig_elem_idx.is_some() {
                    panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_named);
                }
            }
        } else {
            quote! {
                self.#index_name.entry(elem.#field_ident.clone()).or_insert(Vec::with_capacity(1)).push(idx);
            }
        }
    }

    fn removes_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_ident = self.ident.as_ref().unwrap();
        let field_named = self.named().to_string();

        let error_msg = format!("Internal invariants broken, unable to find element in index '{field_named}' despite being present in another");

        match self.unique {
            true => quote! {
                // For unique indexes we know that removing an element will not affect any other elements
                let removed_elem = self.#index_name.remove(&elem_orig.#field_ident);
            },
            false => quote! {
                // For non-unique indexes we must verify that we have not affected any other elements
                if let Some(mut elems) = self.#index_name.remove(&elem_orig.#field_ident) {
                    // If any other elements share the same non-unique index, we must reinsert them into this index
                    if elems.len() > 1 {
                        let pos = elems.iter().position(|e| *e == idx).expect(#error_msg);
                        elems.remove(pos);
                        self.#index_name.insert(elem_orig.#field_ident.clone(), elems);
                    }
                }
            },
        }
    }

    fn modifies_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_ident = self.ident.as_ref().unwrap();
        let field_named = self.named().to_string();

        let error_msg = format!("Internal invariants broken, unable to find element in index '{field_named}' despite being present in another");

        match self.unique {
            true => quote! {
                let idx = self.#index_name.remove(&elem_orig.#field_ident).expect(#error_msg);
                let orig_elem_idx = self.#index_name.insert(elem.#field_ident.clone(), idx);
                if orig_elem_idx.is_some() {
                    panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_named);
                }
            },
            false => quote! {
                let idxs = self.#index_name.get_mut(&elem_orig.#field_ident).expect(#error_msg);
                let pos = idxs.iter().position(|x| *x == idx).expect(#error_msg);
                idxs.remove(pos);
                self.#index_name.entry(elem.#field_ident.clone()).or_insert(Vec::with_capacity(1)).push(idx);
            },
        }
    }

    fn accessors_definition(
        &self,
        map_name: &syn::Ident,
        item: &syn::Ident,
        vis: &syn::Visibility,
        removes: &Vec<proc_macro2::TokenStream>,
        modifies: &Vec<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        let field_name = self.named();

        let index_name = format_ident!("_{}_index", field_name);
        let getter_name = format_ident!("get_by_{}", field_name);
        let remover_name = format_ident!("remove_by_{}", field_name);
        let modifier_name = format_ident!("modify_by_{}", field_name);
        let iter_name = format_ident!(
            "{}{}Iter",
            map_name,
            field_name
                .to_string()
                .to_case(convert_case::Case::UpperCamel)
        );
        let iter_getter_name = format_ident!("iter_by_{}", field_name);
        let ty = &self.ty;

        // TokenStream representing the get_by_ accessor for this field.
        // For non-unique indexes we must go through all matching elements and find their positions,
        // in order to return a Vec of references to the backing storage.
        let getter = match self.unique {
            true => quote! {
                #vis fn #getter_name(&self, key: &#ty) -> Option<&#item> {
                    Some(&self._store[*self.#index_name.get(key)?])
                }
            },
            false => quote! {
                #vis fn #getter_name(&self, key: &#ty) -> Vec<&#item> {
                    if let Some(idxs) = self.#index_name.get(key) {
                        let mut elem_refs = Vec::with_capacity(idxs.len());
                        for idx in idxs {
                            elem_refs.push(&self._store[*idx])
                        }
                        elem_refs
                    } else {
                        Vec::new()
                    }
                }
            },
        };

        // TokenStream representing the remove_by_ accessor for this field.
        // For non-unique indexes we must go through all matching elements and find their positions,
        // in order to return a Vec elements from the backing storage.
        let remover = if self.unique {
            quote! {
                #vis fn #remover_name(&mut self, key: &#ty) -> Option<#item> {
                    let idx = self.#index_name.remove(key)?;
                    let elem_orig = self._store.remove(idx);
                    #(#removes)*
                    Some(elem_orig)
                }
            }
        } else {
            quote! {
                #vis fn #remover_name(&mut self, key: &#ty) -> Vec<#item> {
                    if let Some(idxs) = self.#index_name.remove(key) {
                        let mut elems = Vec::with_capacity(idxs.len());
                        for idx in idxs {
                            let elem_orig = self._store.remove(idx);
                            #(#removes)*
                            elems.push(elem_orig)
                        }
                        elems
                    } else {
                        Vec::new()
                    }
                }
            }
        };

        // TokenStream representing the modify_by_ accessor for this field.
        // Unavailable for NonUnique fields for now, because the modification logic gets quite complicated.
        let modifier = if self.unique {
            quote! {
                #vis fn #modifier_name(&mut self, key: &#ty, f: impl FnOnce(&mut #item)) -> Option<&#item> {
                    let idx = *self.#index_name.get(key)?;
                    let elem = &mut self._store[idx];
                    let elem_orig = elem.clone();
                    f(elem);

                    #(#modifies)*

                    Some(elem)
                }
            }
        } else {
            quote! {}
        };

        // Put all these TokenStreams together, and put a TokenStream representing the iter_by_ accessor on the end.
        quote! {
            #getter

            #remover

            #modifier

            #vis fn #iter_getter_name(&self) -> #iter_name {
                #iter_name {
                    _store_ref: &self._store,
                    _iter: self.#index_name.iter(),
                    _inner_iter: None,
                }
            }
        }
    }

    fn iter_definition(
        &self,
        map_name: &syn::Ident,
        item: &syn::Ident,
        vis: &syn::Visibility,
    ) -> proc_macro2::TokenStream {
        let field_name = self.named();
        let field_name_string = field_name.to_string();
        let error_msg = format!("Internal invariants broken, found empty slice in non_unique index '{field_name_string}'");

        let iter_name = format_ident!(
            "{}{}Iter",
            map_name,
            field_name
                .to_string()
                .to_case(convert_case::Case::UpperCamel)
        );
        let ty = &self.ty;

        // TokenStream representing the actual type of the iterator
        //
        let iter_field_type = if let Some(t) = &self.iter_type {
            syn::parse_str(t).unwrap()
        } else if self.hashed {
            quote! { ::std::collections::hash_map::Iter }
        } else if self.ordered {
            quote! { ::std::collections::btree_map::Iter }
        } else {
            panic!("Unknown index type on field '{}'", field_name_string)
        };

        let iter_type = if self.unique {
            quote! {
                #iter_field_type<'a, #ty, usize>
            }
        } else {
            quote! {
                #iter_field_type<'a, #ty, Vec<usize>>
            }
        };

        // TokenStream representing the logic for performing iteration.
        let iter_action = if self.unique {
            quote! { Some(&self._store_ref[*self._iter.next()?.1]) }
        } else {
            quote! {
                // If we have an inner_iter already, then get the next (optional) value from it.
                let inner_next = if let Some(inner_iter) = &mut self._inner_iter {
                    inner_iter.next()
                } else {
                    None
                };

                // If we have the next value, find it in the backing store.
                if let Some(next_index) = inner_next {
                    Some(&self._store_ref[*next_index])
                } else {
                    let hashmap_next = self._iter.next()?;
                    self._inner_iter = Some(hashmap_next.1.iter());
                    Some(&self._store_ref[*self._inner_iter.as_mut().unwrap().next().expect(#error_msg)])
                }
            }
        };

        // We have a different iterator type for each indexed field.
        // We maintain an optional inner_iter, only used for non-unique indexes.
        // This is used to iterate through the Vec of matching elements for a given index value.
        quote! {
            #vis struct #iter_name<'a> {
                _store_ref: &'a ::indexed_slab::slab::Slab<#item>,
                _iter: #iter_type,
                _inner_iter: Option<core::slice::Iter<'a, usize>>,
            }

            impl<'a> Iterator for #iter_name<'a> {
                type Item = &'a #item;

                fn next(&mut self) -> Option<Self::Item> {
                    #iter_action
                }
            }
        }
    }
}

impl ToTokens for StructData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields_to_index = self.data.as_ref().take_struct().unwrap();

        // For each indexed field generate a TokenStream representing the lookup table for that field
        // Each lookup table maps it's index to a position in the backing storage,
        // or multiple positions in the backing storage in the non-unique indexes.
        let lookup_table_fields = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.index_field_definition());

        // For each indexed field generate a TokenStream representing inserting the position in the backing storage to that field's lookup table
        // Unique indexed fields just require a simple insert to the map, whereas non-unique fields require appending to the Vec of positions,
        // creating a new Vec if necessary.
        let inserts: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.insert_definition())
            .collect();

        // For each indexed field generate a TokenStream representing the remove from that field's lookup table.
        let removes: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.removes_definition())
            .collect();

        // For each indexed field generate a TokenStream representing the combined remove and insert from that field's lookup table.
        let modifies: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.modifies_definition())
            .collect();

        let clears: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.clear_definition())
            .collect();

        let item = &self.ident;

        // Generate the name of the IndexedSlab
        let map_name = self.named();

        // For each indexed field generate a TokenStream representing all the accessors for the underlying storage via that field's lookup table.
        let accessors = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.accessors_definition(&map_name, item, &self.vis, &removes, &modifies));

        // For each indexed field generate a TokenStream representing the Iterator over the backing storage via that field,
        // such that the elements are accessed in an order defined by the index rather than the backing storage.
        let iterators = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.iter_definition(&map_name, item, &self.vis));

        let vis = &self.vis;

        // Build the final output using quasi-quoting
        tokens.extend(quote! {

            #[derive(Default, Clone)]
            #vis struct #map_name {
                _store: ::indexed_slab::slab::Slab<#item>,
                #(#lookup_table_fields)*
            }

            impl #map_name {
                #vis fn len(&self) -> usize {
                    self._store.len()
                }

                #vis fn is_empty(&self) -> bool {
                    self._store.is_empty()
                }

                #vis fn insert(&mut self, elem: #item) {
                    let idx = self._store.insert(elem);
                    let elem = &self._store[idx];

                    #(#inserts)*
                }

                #vis fn clear(&mut self) {
                    self._store.clear();
                    #(#clears)*
                }

                #vis fn iter(&self) -> ::indexed_slab::slab::Iter<#item> {
                    self._store.iter()
                }

                #(#accessors)*
            }

            #(#iterators)*
        });
    }
}

#[proc_macro_derive(IndexedSlab, attributes(indexed_slab))]
#[proc_macro_error]
pub fn indexed_slab(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Darling ensures that we only support named structs, and extracts the relevant fields
    let receiver = match StructData::from_derive_input(&input) {
        Ok(receiver) => receiver,
        Err(err) => return err.write_errors().into(),
    };

    quote!(#receiver).into()
}
