use std::sync::RwLock;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

const MAX_ENTITY_COMPONENTS_ENV: Option<&'static str> = std::option_env!("MAX_ENT_COMPS");
static mut MAX_ENTITY_COMPONENTS: RwLock<Option<usize>> = RwLock::new(None);

fn max_ent_comps() -> usize {
    unsafe {
        let mut max_ent_comps_opt = MAX_ENTITY_COMPONENTS.write().unwrap();
        max_ent_comps_opt.unwrap_or_else(|| {
            let max = MAX_ENTITY_COMPONENTS_ENV.map(|v| {
                v.parse::<usize>()
                    .expect("MAX_ENT_COMPS should be a number")
            }).unwrap_or(50);
            *max_ent_comps_opt = Some(max);
            max
        })
    }
}

#[proc_macro]
pub fn gen_spawn_entity(_: TokenStream) -> TokenStream {
    let max_ent_comps = max_ent_comps();
    let mut fns = Vec::new();
    (0..max_ent_comps).for_each(|i| {
        let name = syn::Ident::new(&format!("spawn_entity{i}"), proc_macro2::Span::call_site());
        let chars: Vec<syn::Ident> = (0..i).map(|i| syn::Ident::new(&itos(i), proc_macro2::Span::call_site())).collect();
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
        let new_archetype_sizes_vector_elements = chars.iter().flat_map(|generic| {
            quote! {
                (<#generic>::id(), ::std::mem::size_of::<#generic>()),
            }
        }).collect::<TokenStream2>();
        
        let init_archetype_size = if chars.len() == 0 {
            quote!{}
        } else {
            quote! {
                let sizes = {
                    let mut v = vec![
                        #new_archetype_sizes_vector_elements
                    ];
                    
                    v.sort_by(|a, b| {
                        a.0.partial_cmp(&b.0).unwrap()
                    });
                    
                    v.iter().map(|(_, size)| *size).collect::<Vec<usize>>()
                };
                self.arch_store.get_archetype_mut(id).init(&components, &sizes);
            }
        };
        
        fns.push(quote! {
            #[doc(hidden)]
            pub fn #name #(#generics)* (&mut self, #(#params , )*) -> EntityId {
                let ent_id = self.entity_store.new_id();
                let mut components = vec![#(<#chars>::id(), )*];
                components.sort();
                let arch_id = match self.arch_store.get_new_entity_archetype(&components) {
                    NewEntityResult::NewArchetype(id) => {
                        #(
                            <#chars>::add_archetype(id);
                        )*
                        #init_archetype_size
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

/// integer to string
fn itos(int: usize) -> String {
    let mut s = String::new();
    if int + 65 >= 90 {
        s.push(89 as char);
        s.push_str(&itos(int + 65 - 90));
    } else {
        let c: char = ((int + 65) as u8) as char;
        s.push(c);
    }
    return s;
}

const MAX_QUERY_COMPONENTS_ENV: Option<&'static str> = std::option_env!("MAX_QUERY_COMPS");
static mut MAX_QUERY_COMPONENTS: RwLock<Option<usize>> = RwLock::new(None);

fn max_query_comps() -> usize {
    unsafe {
        let mut max_ent_comps_opt = MAX_QUERY_COMPONENTS.write().unwrap();
        max_ent_comps_opt.unwrap_or_else(|| {
            let max = MAX_QUERY_COMPONENTS_ENV.map(|v| {
                v.parse::<usize>()
                    .expect("MAX_QUERY_COMPS environment variable should be a number")
            }).unwrap_or(15);
            *max_ent_comps_opt = Some(max);
            max
        })
    }
}

mod query_gen;

#[proc_macro]
pub fn gen_query(_: TokenStream) -> TokenStream {
    let max_query_comps = max_query_comps();
    // let mut fns = Vec::new();
    
    let code = TokenStream::from(query_gen::query_impl(max_query_comps));
    
    // println!("{}", code.to_string());
    
    code
}
