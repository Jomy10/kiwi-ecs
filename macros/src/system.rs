use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;


pub fn system_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
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
