use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) fn query_impl(max_query_comps: usize) -> TokenStream2 {
    (1..max_query_comps).map(|i| {
        
        // General //
        let (
            func_name_query,
            func_name_query_mut,
            _func_name_query_mut_ptr,
            _func_name_query_ids,
            _func_name_query_mut_ids,
        ) = query_names(i);
        
        // Vec<{A}, {B}, ...>
        let generic_names: Vec<syn::Ident> = (0..i).map(|i| syn::Ident::new(&crate::itos(i).to_string(), proc_macro2::Span::call_site())).collect();
        
        // Vec<{A: Component + 'static}, {B: Component + 'static}, ...>
        let generics: Vec<_> = generics(&generic_names, i);
        
        let (
            // impl ::std::iter::Iterator<Item= (&'a A, &'a B, ...)> + 'a
            query_return_type,
            // impl ::std::iter::Iterator<Item= (&'a mut A, &'a mut B, ...)> + 'a
            query_return_type_mut
        ) = return_types(&generic_names);
        
        // Implementation //
        // Vec<{let archetypes_a = A::get_archetypes();}, ...>
        let archetypes_defs: Vec<_> = archetypes_defs(&generic_names);
        
        let filter_iterator = filter_iterator(&generic_names, i);
        
        let zip_reg = zip(&generic_names, GetComponentsType::Regular);
        let zip_mut = zip(&generic_names, GetComponentsType::Mut);
        
        let end_map = end_map(i);
        
        quote! {
            pub fn #func_name_query<'a, #(#generics,)*>(&'a self) -> #query_return_type {
                #(#archetypes_defs)*
                
                #filter_iterator
                    .flat_map(|arch_id| {
                        let archetype = self.arch_store.get_archetype(arch_id);
                        let entities: Vec<crate::arch::ArchRowId> = archetype.get_arch_rows(&self.entity_store).collect();
                        
                        #zip_reg
                    })
                    #end_map
            }
            
            pub fn #func_name_query_mut<'a, #(#generics,)*>(&'a mut self) -> #query_return_type_mut {
                #(#archetypes_defs)*
                
                #filter_iterator
                    .flat_map(|arch_id| {
                        let archetype: *mut crate::arch::Archetype = self.arch_store.get_archetype_mut(arch_id);
                        let entities: Vec<crate::arch::ArchRowId> = unsafe { (*archetype).get_arch_rows(&self.entity_store).collect() };
                        
                        #zip_mut
                    })
                    #end_map
            }
        }
    }).collect()
}

fn query_names(i: usize) -> (syn::Ident, syn::Ident, syn::Ident, syn::Ident, syn::Ident) {
    (
        syn::Ident::new(&format!("query{i}"), proc_macro2::Span::call_site()),
        syn::Ident::new(&format!("query_mut{i}"), proc_macro2::Span::call_site()),
        syn::Ident::new(&format!("query_mut_ptr{i}"), proc_macro2::Span::call_site()),
        syn::Ident::new(&format!("query_ids{i}"), proc_macro2::Span::call_site()),
        syn::Ident::new(&format!("query_mut_ptr_ids{i}"), proc_macro2::Span::call_site()),
    )
}

fn generics(generic_names: &Vec<syn::Ident>, i: usize) -> Vec<TokenStream2> {
    (0..i).map(|i| {
        let generic_name = &generic_names[i];
        // A: Component + 'static
        quote! {
            #generic_name: Component + 'static
        }
    }).collect()
}

fn return_types(generic_names: &Vec<syn::Ident>) -> (TokenStream2, TokenStream2) {
    if generic_names.len() == 1 {
        let generic_name = &generic_names[0];
        (
            quote! {
                impl ::std::iter::Iterator<Item = &'a #generic_name> + 'a
            },
            quote! {
                impl ::std::iter::Iterator<Item = &'a mut #generic_name> + 'a
            }
        )
    } else {
        (
            quote! {
                impl ::std::iter::Iterator<Item = (#(&'a #generic_names,)*)> + 'a
            },
            quote! {
                impl ::std::iter::Iterator<Item = (#(&'a mut #generic_names,)*)> + 'a
            }
        )
    }
}

fn archetypes_defs(generic_names: &Vec<syn::Ident>) -> Vec<TokenStream2> {
    generic_names.iter()
        .map(|generic_name| {
            let var_name = syn::Ident::new(&format!("archetypes_{}", &generic_name.to_string().to_lowercase()), proc_macro2::Span::call_site());
            quote! {
                let #var_name = <#generic_name>::get_archetypes();
            }
        }).collect()
}

fn filter_iterator(generic_names: &Vec<syn::Ident>, i: usize) -> TokenStream2 {
    if generic_names.len() == 1 {
        quote! {
            archetypes_a.clone().into_iter()
        }
    } else {
        let filters: Vec<TokenStream2> = (1..i)
            .map(|i| {
                let name = syn::Ident::new(&format!("archetypes_{}", &generic_names[i].to_string().to_lowercase()), proc_macro2::Span::call_site());
                
                quote! {
                    .filter(move |elem| #name.contains(elem))
                }
            }).collect();
        
        quote! {
            archetypes_a.clone().into_iter()
                #(#filters)*
        }
    }
}

fn zip(generic_names: &Vec<syn::Ident>, ty: GetComponentsType) -> TokenStream2 {
    if generic_names.len() == 1 {
        let generic_name = &generic_names[0];
        let archetype = match ty {
            GetComponentsType::Regular => quote! { archetype },
            GetComponentsType::Mut => quote! { (*archetype) },
        };
        let func_name = match ty {
            GetComponentsType::Regular => quote! { get_all_components },
            GetComponentsType::Mut => quote! { get_all_components_mut },
        };
        quote! {
            unsafe { #archetype.#func_name ::<#generic_name>(entities) }
        }
    } else {
        get_next_zip(generic_names, 0, ty).unwrap()
    }
}

#[derive(Copy, Clone)]
enum GetComponentsType {
    Regular,
    Mut,
}

// returns the next part of the zip, ends with None
fn get_next_zip(generic_names: &Vec<syn::Ident>, i: usize, ty: GetComponentsType) -> Option<TokenStream2> {
    if generic_names.len() == i {
        return None;
    }
    
    let generic_name = &generic_names[i];
    
    let next = get_next_zip(generic_names, i + 1, ty);
    
    let func_name = syn::Ident::new(match ty {
        GetComponentsType::Regular => "get_all_components",
        GetComponentsType::Mut => "get_all_components_mut",
    }, proc_macro2::Span::call_site());
    
    let archetype = match ty {
        GetComponentsType::Regular => quote! { archetype },
        GetComponentsType::Mut => quote! { (*archetype) },
    };

    return Some(match next {
        Some(next) => {
            quote! {
                ::std::iter::zip(
                    unsafe { #archetype.#func_name ::<#generic_name>(entities.clone()) },
                    #next
                )
            }
        },
        None => {
            quote! {
                unsafe { #archetype.#func_name ::<#generic_name>(entities) }
            }
        }
    });
}

fn end_map(i: usize) -> TokenStream2 {
    if i == 1 {
        quote! {}
    } else {
        let n = get_next_end_map(i, 0, vec![0]).unwrap();
        quote! {
            .map(|tuple| (#n))
        }
    }
}

fn get_next_end_map(max: usize, i: usize, tuple_access: Vec<u32>) -> Option<TokenStream2> {
    if i == max {
        return None;
    }
    
    let mut new_tuple_access = tuple_access.clone();
    if *new_tuple_access.last().unwrap() == 0 {
        *new_tuple_access.last_mut().unwrap() = 1;
        if i + 2 < max {
            new_tuple_access.push(0);
        }
    } else if *new_tuple_access.last().unwrap() == 1 && i + 2 < max {
        new_tuple_access.push(0);
    }
    
    let next = get_next_end_map(max, i + 1, new_tuple_access);
    
    let tuple_access = tuple_access.iter()
        .map(|i| i.to_string().parse().unwrap())
        .collect::<Vec<TokenStream2>>();

    Some(match next {
        Some(next) => {
            quote! {
                tuple #(.#tuple_access)*, #next
            }
        },
        None => {
            quote! {
                tuple #(.#tuple_access)*
            }
        }
    })
}
