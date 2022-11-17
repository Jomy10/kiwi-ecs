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
    let sys_attr = &ast.attrs;
    let sys_vis = &ast.vis;
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
    
    // component names + entity id name
    let mut param_vars = Vec::new();
    // component types
    let mut param_types = Vec::new();
    let mut entity_id: bool = false;
    
    for attr in attrs {
        match attr {
            ParamType::Param(param) => {
                let name = syn::Ident::new(&param.var_name.to_string(), proc_macro2::Span::call_site());
                let ty = syn::Ident::new(&param.var_type.to_string(), proc_macro2::Span::call_site());
                
                param_vars.push(name);
                param_types.push(ty.clone());
            },
            ParamType::EntityId(ident) => {
                let name = syn::Ident::new(&ident.var_name.to_string(), proc_macro2::Span::call_site());
                let mut new = vec![name];
                new.append(&mut param_vars);
                param_vars = new; // id is always first variable in query result
                entity_id = true;
            },
        }
    }
    
    let count = param_types.len();
     
    let query_func = match is_world_mutable {
        true => {
            if entity_id == true {
                syn::Ident::new(&format!("query_mut_ids{count}"), proc_macro2::Span::call_site())
            } else {
                syn::Ident::new(&format!("query_mut{count}"), proc_macro2::Span::call_site())
            }
        }
        false => {
            if entity_id == true {
                syn::Ident::new(&format!("query_ids{count}"), proc_macro2::Span::call_site())
            } else {
                syn::Ident::new(&format!("query{count}"), proc_macro2::Span::call_site())
            }
        }
    };
    
    let for_each_parameter = if param_vars.len() == 1 {
        let param_var = &param_vars[0];
        quote! {
            #param_var
        }
    } else {
        quote! {
            (#(#param_vars,)*)
        }
    };
    
    let ts = TokenStream::from(quote! {
        #(#sys_attr)*
        #sys_vis #sys_sig {
            let __query = #world_name_ident.#query_func ::<#(#param_types,)*>();
            
            __query.for_each(|#for_each_parameter| {
                #func_body
            });
            
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
