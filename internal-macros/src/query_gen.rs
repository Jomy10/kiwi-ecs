use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) fn query_impl(max_query_comps: usize) -> TokenStream2 {
    (1..max_query_comps).map(|i| {
        let (
            func_name_query,
            func_name_query_mut_ptr,
            func_name_query_ids,
            func_name_query_mut_ids,
        ) = query_names(i);
        
        // Vec<{A}, {B}, ...>
        let generic_names: Vec<syn::Ident> = (0..i).map(|i| syn::Ident::new(&crate::itos(i).to_string(), proc_macro2::Span::call_site())).collect();
        
        // Vec<{A: Component + 'static}, {B: Component + 'static}, ...>
        let generics: Vec<_> = generics(&generic_names, i);
        
        let (
            // impl ::std::iter::Iterator<Item= (...)>
            query_return_type,
        ) = return_types(&generic_names, i);
        
        // Vec<{let archetypes_a = A::get_archetypes()}, ...>
        let archetypes_vars: Vec<_> = archetypes_vars(&generic_names);
        
        let filter_iterator = filter_iterator(&generic_names, i);
                
        quote! {
            pub fn #func_name_query<'a, #(#generics,)*>(&'a self) -> #query_return_type {
                #(#archetypes_vars)*
                
                #filter_iterator
                    .map(|arch_id| {
                        let archetype = self.arch_store.get_archetype(*arch_id);
                        let entities: Vec<crate::arch::ArchRowId> = archetype.get_arch_rows(&self.entity_store).collect();
                    
                        (
                            #(unsafe { archetype.get_all_components::<#generic_names>(&entities) },)*
                        )
                    })
            }
        }
    }).collect()
}

fn query_names(i: usize) -> (syn::Ident, syn::Ident, syn::Ident, syn::Ident) {
    (
        syn::Ident::new(&format!("query{i}"), proc_macro2::Span::call_site()),
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

fn return_types_query(generic_names: &Vec<syn::Ident>, i: usize) -> Vec<TokenStream2> {
    (0..i).map(|i| {
        let generic_name = &generic_names[i];
        quote! {
            impl ::std::iter::Iterator<Item = &'a #generic_name>
        }
    }).collect()
}

fn return_types(generic_names: &Vec<syn::Ident>, i: usize) -> (TokenStream2, ) {
    let (
        // Vec<{impl ::std::iter::Iterator<Item = &A>}, ...>
        return_types,
        // TODO: return_types_mut_ptr, return_types_ids, return_types_mut_ids
    ): (Vec<_>,) = (
        return_types_query(generic_names, i),
    );
    
    (
        // impl ::std::iter::Iterator<Item = (
        //    impl ::std::iter::Iterator<Item = &A>,
        //    ...
        // )>
        if return_types.len() == 1 {
            quote! { impl ::std::iter::Iterator<Item = #(#return_types)*> + 'a }
        } else {
            quote! { impl ::std::iter::Iterator<Item = ( #(#return_types,)* )> + 'a }
        },
        // TODO: ...
    )
}

fn archetypes_vars(generic_names: &Vec<syn::Ident>) -> Vec<TokenStream2> {
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
            archetypes_a.iter()
        }
    } else {
        let filters: Vec<TokenStream2> = (2..i)
            .map(|i| {
                let name = syn::Ident::new(&format!("archetypes_{}", &generic_names[i].to_string().to_lowercase()), proc_macro2::Span::call_site());
                
                quote! {
                    .filter(move |elem| #name.contains(elem))
                }
            }).collect();
        quote! {
            archetypes_a.iter()
            #(#filters)*
        }
    }
}
