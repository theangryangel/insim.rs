use convert_case::Casing;
use darling::{ast, FromDeriveInput, FromField, FromMeta};
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(indexed_slab), supports(struct_any))]
struct StructData {
    pub rename: Option<String>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,

    pub data: ast::Data<(), FieldData>,
}

impl StructData {
    fn named(&self) -> syn::Ident {
        if let Some(name) = &self.rename {
            format_ident!("{}", name)
        } else {
            format_ident!("IndexedSlab{}", self.ident)
        }
    }
}

#[derive(Debug, Clone, Copy, FromMeta, Eq, PartialEq)]
#[darling(default)]
enum How {
    Hashed,
    Ordered,
    Custom,
}

impl Default for How {
    fn default() -> Self {
        How::Hashed
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(indexed_slab), and_then = "Self::validate")]
struct FieldData {
    pub ident: Option<syn::Ident>,

    pub ty: syn::Type,

    /// Rename the field to change function names
    pub rename: Option<String>,

    /// How are we indexed?
    pub how: Option<How>,

    /// Is this unique? Inserting a duplicate will result in a panic!
    #[darling(default)]
    pub unique: bool,

    // escape hatch incase we need to skip
    #[darling(default)]
    pub skip: bool,

    /// Customise the types used for this field index
    pub custom: Option<CustomData>,

    /// If a field is optional, ignore None
    #[darling(default)]
    pub ignore_none: bool,
}

#[derive(Debug, FromMeta)]
struct CustomData {
    /// The custom type, i.e. ::std::collections::HashMap
    pub ty: String,

    /// The iterator for the custom type, i.e. ::std::collections::hash_map::Iter
    pub iter: String,
}

impl CustomData {
    fn ty(&self) -> proc_macro2::TokenStream {
        syn::parse_str(&self.ty).unwrap()
    }

    fn iter(&self) -> proc_macro2::TokenStream {
        syn::parse_str(&self.iter).unwrap()
    }
}

impl FieldData {
    fn validate(self) -> darling::Result<Self> {
        let mut e = darling::Error::accumulator();

        if let (false, Some(How::Custom), None) = (self.skip, &self.how, &self.custom) {
            e.push(darling::Error::custom(
                "Cannot have mode of custom without custom attributes",
            ));
        }

        if !self.skip && self.how != Some(How::Custom) && self.custom.is_some() {
            e.push(darling::Error::custom(
                "Custom attributes present, but did not specific custom mode",
            ));
        }

        if let (false, Some(How::Custom), None) = (self.skip, &self.how, &self.custom) {
            e.push(darling::Error::custom(
                "Cannot have mode of custom without custom attributes",
            ));
        }

        // FIXME
        if !self.skip
            && self.ignore_none
            && !self.ty.to_token_stream().to_string().contains("Option <")
        {
            e.push(darling::Error::custom(
                "Cannot set ignore_none on a non-optional field",
            ));
        }

        e.finish_with(self)
    }

    fn is_indexable(&self) -> Option<&Self> {
        if !self.skip && (self.how.is_some()) {
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

        let field_type = match self.how {
            Some(How::Hashed) => {
                quote! { ::std::collections::HashMap }
            }
            Some(How::Ordered) => {
                quote! { ::std::collections::BTreeMap }
            }
            Some(How::Custom) => self.custom.as_ref().unwrap().ty(),
            None => panic!("Unknown index type on field '{}'", field_name_string),
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

        let mut action = if self.unique {
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
        };

        if self.ignore_none {
            action = quote! {
                if elem.#field_ident.is_some() {
                    #action
                }
            }
        }

        action
    }

    fn removes_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_ident = self.ident.as_ref().unwrap();
        let field_named = self.named().to_string();

        let error_msg = format!("Internal invariants broken, unable to find element in index '{field_named}' despite being present in another");

        let mut action = if self.unique {
            quote! {
                // For unique indexes we know that removing an element will not affect any other elements
                let removed_elem = self.#index_name.remove(&elem_orig.#field_ident);
            }
        } else {
            quote! {
                // For non-unique indexes we must verify that we have not affected any other elements
                if let Some(mut elems) = self.#index_name.remove(&elem_orig.#field_ident) {
                    // If any other elements share the same non-unique index, we must reinsert them into this index
                    if elems.len() > 1 {
                        let pos = elems.iter().position(|e| *e == idx).expect(#error_msg);
                        elems.remove(pos);
                        self.#index_name.insert(elem_orig.#field_ident.clone(), elems);
                    }
                }
            }
        };

        if self.ignore_none {
            action = quote! {
                if elem_orig.#field_ident.is_some() {
                    #action
                }
            }
        }

        action
    }

    fn modifies_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_ident = self.ident.as_ref().unwrap();
        let field_named = self.named().to_string();

        let error_msg = format!("Internal invariants broken, unable to find element in index '{field_named}' despite being present in another");

        match (self.unique, self.ignore_none) {
            (true, true) => quote! {
                if elem_orig.#field_ident.is_some() {
                    self.#index_name.remove(&elem_orig.#field_ident).expect(#error_msg);
                }

                if elem.#field_ident.is_some() {
                    let orig_elem_idx = self.#index_name.insert(elem.#field_ident.clone(), idx);
                    if orig_elem_idx.is_some() {
                        panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_named);
                    }
                }
            },
            (true, false) => quote! {
                self.#index_name.remove(&elem_orig.#field_ident).expect(#error_msg);
                let orig_elem_idx = self.#index_name.insert(elem.#field_ident.clone(), idx);
                if orig_elem_idx.is_some() {
                    panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_named);
                }
            },
            (false, true) => quote! {
                if elem_orig.#field_ident.is_some() {
                    let idxs = self.#index_name.get_mut(&elem_orig.#field_ident).expect(#error_msg);
                    let pos = idxs.iter().position(|x| *x == idx).expect(#error_msg);
                    idxs.remove(pos);
                }

                if elem.#field_ident.is_some() {
                    self.#index_name.entry(elem.#field_ident.clone()).or_insert(Vec::with_capacity(1)).push(idx);
                }
            },
            (false, false) => quote! {
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
        let iter_field_type = match self.how {
            Some(How::Hashed) => {
                quote! { ::std::collections::hash_map::Iter }
            }
            Some(How::Ordered) => {
                quote! { ::std::collections::btree_map::Iter }
            }
            Some(How::Custom) => self.custom.as_ref().unwrap().iter(),
            None => panic!("Unknown index type on field '{}'", field_name_string),
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
        let item = &self.ident;
        let name = self.named();
        let vis = &self.vis;
        let fields_to_index = self.data.as_ref().take_struct().unwrap();

        // Build the indexes for each index field
        let indexes = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.index_field_definition());

        // Build the functionality to populate the indexes for each indexed field, when inserting a
        // new item.
        let inserts: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.insert_definition())
            .collect();

        // Build the functionality to remove an item from the indexes for each indexed field, when
        // removing an item.
        let removes: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.removes_definition())
            .collect();

        // Build the functionality that updates the indexes when an item is changed.
        let modifies: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.modifies_definition())
            .collect();

        // Build the functionality that clears all field indexes.
        let clears: Vec<proc_macro2::TokenStream> = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.clear_definition())
            .collect();

        // Build the accessors (`get_by_`, etc.) methods for each index field
        let accessors = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.accessors_definition(&name, item, &self.vis, &removes, &modifies));

        // Build the iterator methods for each indexed field
        let iterators = fields_to_index
            .iter()
            .filter_map(|f| f.is_indexable())
            .map(|f| f.iter_definition(&name, item, &self.vis));

        tokens.extend(quote! {

            #[derive(Default, Clone)]
            #vis struct #name {
                _store: ::indexed_slab::slab::Slab<#item>,
                #(#indexes)*
            }

            #[allow(dead_code)]
            impl #name {
                #vis fn len(&self) -> usize {
                    self._store.len()
                }

                #vis fn is_empty(&self) -> bool {
                    self._store.is_empty()
                }

                #vis fn insert(&mut self, elem: #item) -> usize {
                    let idx = self._store.insert(elem);
                    let elem = &self._store[idx];
                    #(#inserts)*
                    idx
                }

                #vis fn get(&self, idx: usize) -> Option<&#item> {
                    self._store.get(idx)
                }

                #vis fn remove(&mut self, idx: usize) -> #item {
                    let elem_orig = self._store.remove(idx);
                    #(#removes)*
                    elem_orig
                }

                #[allow(dead_code)]
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
