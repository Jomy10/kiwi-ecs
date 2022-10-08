use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::sync::Mutex;

//======================
// System
//======================

#[proc_macro_attribute]
pub fn system(attr: TokenStream, item: TokenStream) -> TokenStream {
    if item.is_empty() {
        panic!("The system attribute macro can only be applied to functions");
    }
    
    let ast: syn::ItemFn = syn::parse(item.clone())
        .expect("The system attribute macro can only be applied to functions.");
    
    // Function signature
    let sys_sig = &ast.sig;
    
    let return_ok: TokenStream2 = if sys_sig.output != syn::ReturnType::Default {
        quote! { Ok(()) }
    } else {
        quote! {}
    };
    
    // get world param
    let inputs = &ast.sig.inputs;
    let (world_name_ident, is_world_mutable) = system_get_world_name(inputs);
    
    // system body
    let func_body = &ast.block;
    
    // attribute params
    let attrs = parse_system_attr(attr);
    
    let mut param_vars = Vec::new();
    // Vec<*mut T> | Vec<&T>
    let mut param_vec_types = Vec::new();
    // T
    let mut param_types = Vec::new();
    let mut entity_id: Option<Param> = None;
    
    for attr in attrs {
        match attr {
            ParamType::Param(param) => {
                let name = syn::Ident::new(&param.var_name.to_string(), proc_macro2::Span::call_site());
                let ty = syn::Ident::new(&param.var_type.to_string(), proc_macro2::Span::call_site());
                
                param_vars.push(name);
                param_types.push(ty.clone());
                
                match is_world_mutable {
                    true => {
                        param_vec_types.push(quote! {
                            Vec<*mut #ty>
                        });
                    }
                    false => {
                        param_vec_types.push(quote! {
                            Vec<&#ty>
                        });
                    }
                }
            },
            ParamType::EntityId(ident) => {
                entity_id = Some(ident);
            },
        }
    }
    
    let first_param_var = &param_vars[0];
    let mut param_vars_init = param_vars.clone();
    
    let id_idx: TokenStream2 = if let Some(entity_id) = &entity_id {
        param_vars_init.insert(0, proc_macro2::Ident::new(&entity_id.var_name.to_string(), proc_macro2::Span::call_site()));
        param_vec_types.insert(0, quote! { Vec<kiwi_ecs::EntityId> });
        let id_ident = syn::Ident::new(&entity_id.var_name.to_string(), proc_macro2::Span::call_site());
        quote! { let #id_ident = #id_ident[i]; }
    } else {
        quote!{}
    };
    let count = param_vars.len();
     
    let query_func = match is_world_mutable {
        true => {
            if entity_id.is_some() {
                syn::Ident::new(&format!("query_mut_ptr_ids{count}"), proc_macro2::Span::call_site())
            } else {
                syn::Ident::new(&format!("query_mut_ptr{count}"), proc_macro2::Span::call_site())
            }
        }
        false => {
            if entity_id.is_some() {
                syn::Ident::new(&format!("query_ids{count}"), proc_macro2::Span::call_site())
            } else {
                syn::Ident::new(&format!("query{count}"), proc_macro2::Span::call_site())
            }
        }
    };
    
    for i in 0..param_types.len() {
        for j in 0..param_types.len() {
            if i == j { continue }
            
            if param_types[i].to_string() == param_types[j].to_string() {
                panic!("Can't query on duplicate types {}", param_types[i]);
            }
        }
    }
    
    let param_vars_init: TokenStream2 = if param_vars_init.len() == 1 {
        quote! { let #(#param_vars_init)*: #(#param_vec_types)* = unsafe { #world_name_ident.#query_func::<#(#param_types,)*>() }; }
    } else {
        quote! { let (#(#param_vars_init,)*): (#(#param_vec_types,)*) = unsafe { #world_name_ident.#query_func::<#(#param_types,)*>() }; }
    };
    
    let query_body: TokenStream2 = if is_world_mutable {
        quote! {
            #(let #param_vars = #param_vars[i];)*
            #(let #param_vars = unsafe { &mut*#param_vars };)*
        }
    } else {
        quote! {
            #(let #param_vars = #param_vars[i];)*
        }
    };
    
    let ts = TokenStream::from(quote! {
        #sys_sig {
            #param_vars_init
            for i in 0..#first_param_var.len() {
                #id_idx
                #query_body
                
                #func_body
            }
            
            #return_ok
        }  
    });
    
    
    ts
}

//==============
// Query
//==============

#[proc_macro]
/// Takes the following parameters:
/// - world: &mut World
/// - components...: Type: Component...
pub fn query(item: TokenStream) -> TokenStream {
    gen_query_tokens(item, "query")
}

#[proc_macro]
pub fn query_mut(item: TokenStream) -> TokenStream {
    gen_query_tokens(item, "query_mut_ptr")
}

fn gen_query_tokens(item: TokenStream, func_name: &str) -> TokenStream {
    let item_iter = item.into_iter();
    let mut item_collect = ItemCollect { item: item_iter, collected: Vec::new() };
    let mut is_end = false;
    let world: String = match item_collect.collect_next() {
        ItemCollectResult::ContainsMore(s) => s,
        ItemCollectResult::End(s) => {
            is_end = true;
            s
        }
    };
    let world: &TokenStream2 = &world.parse().unwrap();
    
    if is_end {
        let func_name: TokenStream2 = format!("{func_name}0").parse().unwrap();
        return TokenStream::from(quote! {
            (#world).#func_name()
        });
    }
    
    let mut components: Vec<TokenStream2> = Vec::new();
    loop {
        match item_collect.collect_next() {
            ItemCollectResult::ContainsMore(s) => {
                components.push(s.parse().unwrap());
            }
            ItemCollectResult::End(s) => {
                components.push(s.parse().unwrap());
                break;
            }
        }
    }
    
    let func_name: TokenStream2 = format!("{func_name}{}", components.len()).parse().unwrap();
    
    TokenStream::from(quote! {
        (#world).#func_name::<#(#components,)*>()
    })
}

enum ItemCollectResult {
    ContainsMore(String),
    End(String)
}

struct ItemCollect {
    item: proc_macro::token_stream::IntoIter,
    collected: Vec<proc_macro::TokenTree>
}

impl ItemCollect {
    fn collect_next(&mut self) -> ItemCollectResult {
        loop {
            let next = self.item.next();
            match next {
                Some(tt) => {
                    match &tt {
                        proc_macro::TokenTree::Group(_) |
                        proc_macro::TokenTree::Ident(_) |
                        proc_macro::TokenTree::Literal(_) 
                            => self.collected.push(tt),
                        proc_macro::TokenTree::Punct(p) => {
                            if p.as_char() == ',' {
                                let collected = self.collected.clone();
                                self.collected.clear();
                                return ItemCollectResult::ContainsMore(
                                    collected.iter().map(|tt| {
                                        tt.to_string()
                                    }).collect::<String>()
                                );
                            } else {
                                self.collected.push(tt);
                            }
                        },
                    }
                }
                None => {
                    return ItemCollectResult::End(
                        self.collected.iter().map(|tt| {
                                tt.to_string()
                        }).collect::<String>()
                    );
                }
            }
        }
    }
}

//==============
// Spawn entity
//==============

#[proc_macro]
pub fn spawn_entity(item: TokenStream) -> TokenStream {
    let mut item_iter = item.into_iter();
    let world = syn::Ident::new(&item_iter.next().unwrap().to_string(), proc_macro2::Span::call_site());
    let mut items: Vec<TokenStream2> = Vec::new();
    let mut count = 0;
    let item_vec: Vec<proc_macro::TokenTree> = item_iter.collect();
    let item_split = item_vec.split(|v| { if let proc_macro::TokenTree::Punct(p) = v { p.as_char() == ',' } else { false } });
    for item in item_split {
        if item.len() == 0 { continue }
        items.push(
            item.iter()
                .map(|tokentree| {
                    TokenStream2::from(tokentree.to_string().parse::<TokenStream>().unwrap())
                }).collect()
        );
        count += 1;
    }
    
    let func_name = syn::Ident::new(&format!("spawn_entity{count}"), proc_macro2::Span::call_site());
    
    TokenStream::from(quote! {
        #world.#func_name(#(#items,)*)
    })
}

//======================
// System
//======================

#[derive(Debug)]
enum ParamType {
    Param(Param),
    /// contains the name of the EntityId parameter
    EntityId(Param)
}

#[derive(Debug)]
struct Param {
    var_name: proc_macro::Ident,
    var_type: proc_macro::Ident,
}

fn parse_system_attr(attrs: TokenStream) -> Vec<ParamType> {
    let mut cur_var_name: Option<proc_macro::Ident> = None;
    // let cur_var_type: Option<syn::Ident> = None;
    
    let mut param: Vec<ParamType> = Vec::new();
    
    for attr in attrs {
        if let proc_macro::TokenTree::Ident(ident) = attr {
            if (&cur_var_name).is_none() {
                cur_var_name = Some(ident);
            } else {
                if ident.to_string() == "EntityId" {
                    param.push(ParamType::EntityId(Param {
                        var_name: cur_var_name.as_ref().unwrap().clone(),
                        var_type: ident
                    }));
                    cur_var_name = None;
                } else {
                    param.push(ParamType::Param(Param {
                        var_name: cur_var_name.as_ref().unwrap().clone(),
                        var_type: ident
                    }));
                    cur_var_name = None;
                }
            }
        }
    }
    
    return param;
}

fn system_get_world_name(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> (syn::Ident, bool) {
    let mut world_name_ident: Option<syn::Ident> = None;
    let mut world_is_mutable = false;
    // for input in inputs.clone() {
    if let syn::FnArg::Typed(typed_param) = &inputs[0] {
        // check if param is of type `World`
        let ty = typed_param.ty.clone();
        let ty = *ty;
        if let syn::Type::Reference(ref_type) = ty {
            if let Some(_) = ref_type.mutability {
                world_is_mutable = true;
            } else {
                world_is_mutable = false;
            }
            let elem = ref_type.elem;
            let elem = *elem;
            if let syn::Type::Path(path) = elem {
                let path = path.path;
                let segments = path.segments;
                let ident = &segments.last().unwrap().ident;
                if ident.to_string() == "World" {
                    // Parameter is World
                    
                    let pat = typed_param.pat.clone();
                    let pat = *pat;
                    if let syn::Pat::Ident(pat_ident) = pat {
                        world_name_ident = Some(pat_ident.ident);
                    }
                }
            }
        }
    }
    // }
    let world_name_ident = world_name_ident.expect("System function does not have a `world: &World` parameter");
    
    return (world_name_ident, world_is_mutable);
}

//======================
// Component
//======================

static mut COMP_ID_COUNTER: Mutex<u32> = Mutex::new(0);

#[proc_macro_derive(Component)]
pub fn derive_component(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let name = &ast.ident;
    
    let comp_id = unsafe { &COMP_ID_COUNTER };
    let mut guard = comp_id.lock().unwrap();
    let this_id = *guard;
    *guard += 1;
    
    TokenStream::from(quote! {
        impl #name {
            #[inline(always)]
            fn get_archetypes_rwlock<'a>() -> &'a std::sync::RwLock<Vec<kiwi_ecs::ArchetypeId>> {
                static mut ARCHETYPES: std::sync::RwLock<Vec<kiwi_ecs::ArchetypeId>> = std::sync::RwLock::new(Vec::new());
                unsafe { &ARCHETYPES }
            }
            
            #[inline(always)]
            fn get_archetypes_read<'a>() -> std::sync::RwLockReadGuard<'a, Vec<kiwi_ecs::ArchetypeId>> {
                let archetype = Self::get_archetypes_rwlock();
                let guard = archetype.read().unwrap();
                guard
            }
            
            #[inline(always)]
            fn get_archetypes_write<'a>() -> std::sync::RwLockWriteGuard<'a, Vec<kiwi_ecs::ArchetypeId>> {
                let archetype = Self::get_archetypes_rwlock();
                let guard = archetype.write().unwrap();
                guard
            }
        }
        impl Component for #name {
            #[inline]
            fn get_archetypes() -> std::sync::RwLockReadGuard<'static, Vec<kiwi_ecs::ArchetypeId>> where Self: Sized {
                let arch_guard = Self::get_archetypes_read();
                return arch_guard
            }
            #[inline]
            fn add_archetype(arch_id: kiwi_ecs::ArchetypeId) where Self: Sized {
                let mut arch_guard = Self::get_archetypes_write();
                (*arch_guard).push(arch_id);
            }
            #[inline(always)]
            fn id() -> kiwi_ecs::ComponentId where Self: Sized { #this_id }
            #[inline(always)]
            fn as_any<'a>(&'a self) -> &'a dyn std::any::Any { self }
            #[inline(always)]
            fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn std::any::Any { self }
        }
    })
}
