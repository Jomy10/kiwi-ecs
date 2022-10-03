use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

const MAX_ENTITY_COMPONENTS: usize = 10;

#[proc_macro]
pub fn gen_spawn_entity(_: TokenStream) -> TokenStream {
    let mut fns = Vec::new();
    (0..MAX_ENTITY_COMPONENTS).for_each(|i| {
        let name = syn::Ident::new(&format!("spawn_entity{i}"), proc_macro2::Span::call_site());
        let chars: Vec<syn::Ident> = (0..i).map(|i| syn::Ident::new(&itoc(i).to_string(), proc_macro2::Span::call_site())).collect();
        let mut generics: Vec<TokenStream2> = chars.iter().map(|c| {
            quote! {
                #c: Component + 'static,
            }
        }).collect();
        let param_names: Vec<syn::Ident> = (0..i).map(|i| syn::Ident::new(&format!("comp{i}"), proc_macro2::Span::call_site())).collect();
        let params: Vec<TokenStream2> = chars.iter().enumerate().map(|(i, c)| {
            let name = &param_names[i];
            quote! {
                #name: #c
            }
        }).collect();
        if generics.len() != 0 {
            generics.insert(0, quote! {<});
            generics.push(quote! {>});
        }
        fns.push(quote! {
            pub fn #name #(#generics)* (&mut self, #(#params , )*) -> EntityId {
                let ent_id = self.entity_store.new_id();
                let components = vec![#(<#chars>::id(), )*];
                let arch_id = match self.arch_store.get_new_entity_archetype(components) {
                    NewEntityResult::NewArchetype(id) => {
                        #(
                            <#chars>::add_archetype(id);
                        )*
                        id
                    }
                    NewEntityResult::OldArchetype(id) => id
                };
                let archetype = &mut self.arch_store.archetypes[arch_id as usize];
                let arch_row = archetype.new_archrow_id(ent_id);
                #(
                    archetype.set_component(arch_row, #param_names);
                )*
                self.entity_store.spawn_with_id(ent_id, arch_id, arch_row);
                return ent_id;
            }
        });
    });
    TokenStream::from(quote! {
        #(#fns)*
    })
}

/// integer to character
fn itoc(int: usize) -> char {
    ((int + 65) as u8) as char
}

const MAX_QUERY_COMPONENTS: usize = 10;

#[proc_macro]
pub fn gen_query(_: TokenStream) -> TokenStream {
    let mut fns = Vec::new();
    (1..MAX_QUERY_COMPONENTS).for_each(|i| {
        let func_name_query = syn::Ident::new(&format!("query{i}"), proc_macro2::Span::call_site());
        let func_name_query_mut_ptr = syn::Ident::new(&format!("query_mut_ptr{i}"), proc_macro2::Span::call_site());
        let func_name_query_ids = syn::Ident::new(&format!("query_ids{i}"), proc_macro2::Span::call_site());
        let func_name_query_mut_ids = syn::Ident::new(&format!("query_mut_ptr_ids{i}"), proc_macro2::Span::call_site());
        let generic_names: Vec<syn::Ident> = (0..i).map(|i| syn::Ident::new(&itoc(i).to_string(), proc_macro2::Span::call_site())).collect();
        let generics: Vec<TokenStream2> = (0..i).map(|i| {
            let generic_name = &generic_names[i];
            quote! {
                #generic_name: Component + 'static
            }
        }).collect();
        
        // Return types //
        let return_types: Vec<TokenStream2> = (0..i).map(|i| {
            let generic_name = syn::Ident::new(&itoc(i).to_string(), proc_macro2::Span::call_site());
            quote! {
                Vec<&#generic_name>
            }
        }).collect();
        let return_types_mut_ptr: Vec<TokenStream2> = (0..i).map(|i| {
            let generic_name = syn::Ident::new(&itoc(i).to_string(), proc_macro2::Span::call_site());
            quote! {
                Vec<*mut #generic_name>
            }
        }).collect();
        let mut return_types_ids = return_types.clone();
        return_types_ids.insert(0, quote! { Vec<EntityId> });
        let mut return_types_mut_ids = return_types_mut_ptr.clone();
        return_types_mut_ids.insert(0, quote! { Vec<EntityId> });

        let return_type: TokenStream2 = if return_types.len() == 1 {
            quote! { #(#return_types)* }
        } else {
            quote! { (#(#return_types, )*) }
        };
        let return_type_mut_ptr: TokenStream2 = if return_types.len() == 1 {
            quote! { #(#return_types_mut_ptr)* }
        } else {
            quote! { (#(#return_types_mut_ptr, )*) }
        };
        let return_type_ids: TokenStream2 = quote! {
            (#(#return_types_ids, )*)
        };
        let return_type_mut_ids: TokenStream2 = quote! {
            (#(#return_types_mut_ids, )*)
        };
        
        ///////////
        let archetype_variable_names: Vec<syn::Ident> = (0..i).map(|i| {
            syn::Ident::new(&format!("archetypes_{}", itoc(i).to_lowercase()), proc_macro2::Span::call_site())
        }).collect();
        
        let fn_generics = if generics.len() != 0 {
            quote! {
                <#(#generics,)*>
            }
        } else {
            quote! {}
        };
        
        let mut filter_expression = Vec::new();
        if archetype_variable_names.len() == 1 {
            filter_expression.push(quote! { archetypes_a });
        } else {
            archetype_variable_names.iter()
                .enumerate()
                .for_each(|(i, name)| {
                    if i == 0 {
                        filter_expression.push(quote! { #name.iter() });
                    } else {
                        filter_expression.push(quote! { .filter(|elem| #name.contains(elem)) });
                    }
                });
            filter_expression.push(quote! { .collect::<Vec<&ArchetypeId>>() });
        };
        
        let component_names: Vec<TokenStream2> = (0..i).map(|i| {
            let name = syn::Ident::new(&format!("c{}", itoc(i).to_lowercase()), proc_macro2::Span::call_site());
            quote! { #name }
        }).collect();
        
        let vec_new_exprs: Vec<TokenStream2> = (0..i).map(|_| {
            quote! { Vec::new() }
        }).collect();
        let comps_names: Vec<syn::Ident> = (0..i).map(|i| {
            syn::Ident::new(&format!("comp{}", itoc(i).to_lowercase()), proc_macro2::Span::call_site())
        }).collect();
        let return_val: TokenStream2 = if return_types.len() == 1 {
            quote! { #(#component_names)* }
        } else {
            quote! { (#(#component_names,)*) }
        };
        let archetypes_def: TokenStream2 = if archetype_variable_names.len() == 1 {
            quote! { let archetypes: &[ArchetypeId] = #(#filter_expression)*.as_slice(); }
        } else {
            quote! { let archetypes: Vec<&ArchetypeId> = #(#filter_expression)*; }
        };
        
        fns.push(quote! {
            /// Query the entities that have the #(#generic_names, )* component(s)
            pub fn #func_name_query #fn_generics (&self) -> #return_type {
                #(
                    let #archetype_variable_names = <#generic_names>::get_archetypes();
                )*
                #archetypes_def
                let (#(mut #component_names,)*) = (#(#vec_new_exprs,)*);
                archetypes.into_iter().for_each(|arch_id| {
                    let archetype = &self.arch_store.archetypes[*arch_id as usize];
                    let entities = archetype.get_arch_rows(&self.entity_store);
                    #(
                        let mut #comps_names: Vec<&#generic_names> = archetype.get_all_components(&entities);
                        #component_names.append(&mut #comps_names);
                    )*
                });
                #return_val
            }
            
            pub fn #func_name_query_ids #fn_generics (&self) -> #return_type_ids {
                #(
                    let #archetype_variable_names = <#generic_names>::get_archetypes();
                )*
                #archetypes_def
                let (mut ids, #(mut #component_names,)*) = (Vec::new(), #(#vec_new_exprs,)*);
                archetypes.into_iter().for_each(|arch_id| {
                    let archetype = &self.arch_store.archetypes[*arch_id as usize];
                    let entities = archetype.get_rows_and_ids(&self.entity_store);
                    let rows = entities.iter().map(|(row, _)| *row).collect();
                    let mut ent_ids = entities.iter().map(|(_, id)| *id).collect();
                    #(
                        let mut #comps_names: Vec<&#generic_names> = archetype.get_all_components(&rows);
                        #component_names.append(&mut #comps_names);
                    )*
                    ids.append(&mut ent_ids);
                });
                (ids, #(#component_names,)*)
            }
            
            /// Query the entities that have the #(#generic_names, )* component(s)
            /// 
            /// # Safety
            /// Might causes undefined behaviour if one or more of the component
            /// types have the same type
            pub unsafe fn #func_name_query_mut_ptr #fn_generics (&mut self) -> #return_type_mut_ptr {
                #(
                    let #archetype_variable_names = <#generic_names>::get_archetypes();
                )*
                #archetypes_def
                
                let (#(mut #component_names,)*) = (#(#vec_new_exprs, )*);
                archetypes.into_iter().for_each(|arch_id| {
                    let archetype = &mut self.arch_store.archetypes[*arch_id as usize];
                    let entities = archetype.get_arch_rows(&self.entity_store);
                    #(
                        let mut #comps_names: Vec<*mut #generic_names> = archetype.get_all_components_mut(&entities);
                        #component_names.append(&mut #comps_names);
                    )*
                });
                #return_val
            }

            pub unsafe fn #func_name_query_mut_ids #fn_generics (&mut self) -> #return_type_mut_ids {
                #(
                    let #archetype_variable_names = <#generic_names>::get_archetypes();
                )*
                #archetypes_def
                
                let (mut ids, #(mut #component_names,)*) = (Vec::new(), #(#vec_new_exprs, )*);
                archetypes.into_iter().for_each(|arch_id| {
                    let archetype = &mut self.arch_store.archetypes[*arch_id as usize];
                    let entities = archetype.get_rows_and_ids(&self.entity_store);
                    let rows = entities.iter().map(|(row, _)| *row).collect();
                    let mut ent_ids = entities.iter().map(|(_, id)| *id).collect();
                    #(
                        let mut #comps_names: Vec<*mut #generic_names> = archetype.get_all_components_mut(&rows);
                        #component_names.append(&mut #comps_names);
                    )*
                    ids.append(&mut ent_ids);
                });
                (ids, #(#component_names,)*)
            }
        });
    });
    
    let tokens = TokenStream::from(quote! {
        #(#fns)*
    });
    
    // println!("{}", &tokens.to_string());
    
    tokens
}