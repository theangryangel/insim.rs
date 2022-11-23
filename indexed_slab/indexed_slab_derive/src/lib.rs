use convert_case::Casing;
use darling::{ast, FromDeriveInput, FromField};
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};
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
#[darling(attributes(indexed_slab))]
struct FieldData {
    pub name: Option<String>,

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

    #[darling(default)]
    pub ignore_none: bool,

    pub ident: Option<syn::Ident>,

    pub ty: syn::Type,
}

impl FieldData {
    fn can_be_indexed(&self) -> bool {
        self.unique || self.hashed || self.ordered
    }

    fn is_indexable(&self) -> Option<&Self> {
        if self.can_be_indexed() {
            Some(self)
        } else {
            None
        }
    }

    fn named(&self) -> syn::Ident {
        if let Some(name) = &self.name {
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
            quote! { ::indexed_slab::rustc_hash::FxHashMap }
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
        let field_name = self.named();
        let field_name_string = field_name.to_string();

        if self.unique {
            if self.ignore_none {
                quote! {
                    if elem.#field_name.is_some() {
                        let orig_elem_idx = self.#index_name.insert(elem.#field_name.clone(), idx);
                        panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_name_string);
                    }
                }
            } else {
                quote! {
                    let orig_elem_idx = self.#index_name.insert(elem.#field_name.clone(), idx);
                    if orig_elem_idx.is_some() {
                        panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_name_string);
                    }
                }
            }
        } else {
            quote! {
                self.#index_name.entry(elem.#field_name.clone()).or_insert(Vec::with_capacity(1)).push(idx);
            }
        }
    }

    fn removes_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_name = self.named();
        let field_name_string = field_name.to_string();

        let error_msg = format!("Internal invariants broken, unable to find element in index '{field_name_string}' despite being present in another");

        match (self.unique, self.ignore_none) {
            (true, false) => quote! {
                // For unique indexes we know that removing an element will not affect any other elements
                let removed_elem = self.#index_name.remove(&elem_orig.#field_name);
            },
            (true, true) => quote! {
                if elem_orig.#field_name.is_some() {
                    // For unique indexes we know that removing an element will not affect any other elements
                    let removed_elem = self.#index_name.remove(&elem_orig.#field_name);
                }
            },
            (false, _) => quote! {
                // For non-unique indexes we must verify that we have not affected any other elements
                if let Some(mut elems) = self.#index_name.remove(&elem_orig.#field_name) {
                    // If any other elements share the same non-unique index, we must reinsert them into this index
                    if elems.len() > 1 {
                        let pos = elems.iter().position(|e| *e == idx).expect(#error_msg);
                        elems.remove(pos);
                        self.#index_name.insert(elem_orig.#field_name.clone(), elems);
                    }
                }
            },
        }
    }

    fn modifies_definition(&self) -> proc_macro2::TokenStream {
        let index_name = self.index_ident();
        let field_name = self.named();
        let field_name_string = field_name.to_string();

        let error_msg = format!("Internal invariants broken, unable to find element in index '{field_name_string}' despite being present in another");

        match (self.unique, self.ignore_none) {
            (true, false) => quote! {
                let idx = self.#index_name.remove(&elem_orig.#field_name).expect(#error_msg);
                let orig_elem_idx = self.#index_name.insert(elem.#field_name.clone(), idx);
                if orig_elem_idx.is_some() {
                    panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_name_string);
                }
            },
            (true, true) => quote! {

                // FIXME this is broken

                if elem_orig.#field_name.is_some() {
                    let idx = self.#index_name.remove(&elem_orig.#field_name).expect(#error_msg);
                    let orig_elem_idx = self.#index_name.insert(elem.#field_name.clone(), idx);
                    if orig_elem_idx.is_some() {
                        panic!("Unable to insert element, uniqueness constraint violated on field '{}'", #field_name_string);
                    }
                } else {
                    self.#index_name.insert(elem.#field_name.clone(), idx);
                }
            },
            (false, _) => quote! {
                let idxs = self.#index_name.get_mut(&elem_orig.#field_name).expect(#error_msg);
                let pos = idxs.iter().position(|x| *x == idx).expect(#error_msg);
                idxs.remove(pos);
                self.#index_name.entry(elem.#field_name.clone()).or_insert(Vec::with_capacity(1)).push(idx);
            },
        }
    }

    fn accessors_definition(
        &self,
        map_name: &syn::Ident,
        element_name: &syn::Ident,
        vis: &syn::Visibility,
        removes: &Vec<proc_macro2::TokenStream>,
        modifies: &Vec<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        let field_name = self.named();

        let index_name = format_ident!("_{}_index", field_name);
        let getter_name = format_ident!("get_by_{}", field_name);
        let mut_getter_name = format_ident!("get_mut_by_{}", field_name);
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
        let getter = match (self.unique, self.ignore_none) {
            (true, false) => quote! {
                #vis fn #getter_name(&self, key: &#ty) -> Option<&#element_name> {
                    Some(&self._store[*self.#index_name.get(key)?])
                }
            },
            (true, true) => quote! {
                #vis fn #getter_name(&self, key: &#ty) -> Option<&#element_name> {
                    if key.is_some() {
                       Some(&self._store[*self.#index_name.get(key)?])
                    } else {
                        None
                    }
                }
            },
            (false, _) => quote! {
                #vis fn #getter_name(&self, key: &#ty) -> Vec<&#element_name> {
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

        // TokenStream representing the get_mut_by_ accessor for this field.
        // Unavailable for NonUnique fields for now, because this would require returning multiple mutable references to the same backing storage.
        // This is not impossible to do safely, just requires some unsafe code and a thought out approach similar to split_at_mut.
        let mut_getter = if self.unique {
            quote! {
                // SAFETY:
                // It is safe to mutate the non-indexed fields, however mutating any of the indexed fields will break the internal invariants.
                // If the indexed fields need to be changed, the modify() method must be used.
                #vis unsafe fn #mut_getter_name(&mut self, key: &#ty) -> Option<&mut #element_name> {
                    Some(&mut self._store[*self.#index_name.get(key)?])
                }
            }
        } else {
            quote! {}
        };

        // TokenStream representing the remove_by_ accessor for this field.
        // For non-unique indexes we must go through all matching elements and find their positions,
        // in order to return a Vec elements from the backing storage.
        let remover = if self.unique {
            quote! {
                #vis fn #remover_name(&mut self, key: &#ty) -> Option<#element_name> {
                    let idx = self.#index_name.remove(key)?;
                    let elem_orig = self._store.remove(idx);
                    #(#removes)*
                    Some(elem_orig)
                }
            }
        } else {
            quote! {
                #vis fn #remover_name(&mut self, key: &#ty) -> Vec<#element_name> {
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
                #vis fn #modifier_name(&mut self, key: &#ty, f: impl FnOnce(&mut #element_name)) -> Option<&#element_name> {
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

            #mut_getter

            #remover

            #modifier

            #vis fn #iter_getter_name(&mut self) -> #iter_name {
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
        element_name: &syn::Ident,
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

        // TokenStream representing the iterator over each indexed field.
        // We have a different iterator type for each indexed field. Each one wraps the standard Iterator for that lookup table, but adds in a couple of things:
        // First we maintain a reference to the backing store, so we can return references to the elements we are interested in.
        // Second we maintain an optional inner_iter, only used for non-unique indexes. This is used to iterate through the Vec of matching elements for a given index value.
        quote! {
            #vis struct #iter_name<'a> {
                _store_ref: &'a ::indexed_slab::slab::Slab<#element_name>,
                _iter: #iter_type,
                _inner_iter: Option<core::slice::Iter<'a, usize>>,
            }

            impl<'a> Iterator for #iter_name<'a> {
                type Item = &'a #element_name;

                fn next(&mut self) -> Option<Self::Item> {
                    #iter_action
                }
            }
        }
    }
}

#[proc_macro_derive(IndexedSlab, attributes(indexed_slab))]
#[proc_macro_error]
pub fn indexed_slab(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Darling ensures that we only support named structs, and extracts the relevant fields
    let receiver = match StructData::from_derive_input(&input) {
        Ok(receiver) => receiver,
        Err(err) => return err.write_errors().into(),
    };

    // Store the visibility of the struct as we'll use this as a basis for the derived type
    let vis = &receiver.vis;

    let fields_to_index = receiver.data.as_ref().take_struct().unwrap();

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

    let element_name = &receiver.ident;

    // Generate the name of the IndexedSlab
    let map_name = receiver.named();

    // For each indexed field generate a TokenStream representing all the accessors for the underlying storage via that field's lookup table.
    let accessors = fields_to_index
        .iter()
        .filter_map(|f| f.is_indexable())
        .map(|f| {
            f.accessors_definition(&map_name, element_name, &receiver.vis, &removes, &modifies)
        });

    // For each indexed field generate a TokenStream representing the Iterator over the backing storage via that field,
    // such that the elements are accessed in an order defined by the index rather than the backing storage.
    let iterators = fields_to_index
        .iter()
        .filter_map(|f| f.is_indexable())
        .map(|f| f.iter_definition(&map_name, element_name, &receiver.vis));

    // Build the final output using quasi-quoting
    let expanded = quote! {

        #[derive(Default, Clone)]
        #vis struct #map_name {
            _store: ::indexed_slab::slab::Slab<#element_name>,
            #(#lookup_table_fields)*
        }

        impl #map_name {
            #vis fn len(&self) -> usize {
                self._store.len()
            }

            #vis fn is_empty(&self) -> bool {
                self._store.is_empty()
            }

            #vis fn insert(&mut self, elem: #element_name) {
                let idx = self._store.insert(elem);
                let elem = &self._store[idx];

                #(#inserts)*
            }

            #vis fn clear(&mut self) {
                self._store.clear();
                #(#clears)*
            }

            // Allow iteration directly over the backing storage
            #vis fn iter(&self) -> ::indexed_slab::slab::Iter<#element_name> {
                self._store.iter()
            }

            #(#accessors)*
        }

        #(#iterators)*
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
