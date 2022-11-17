use proc_macro::TokenStream;

mod system;
mod query;
mod component;
mod entity;
mod flags;

//======================
// System
//======================

#[proc_macro_attribute]
pub fn system(attr: TokenStream, item: TokenStream) -> TokenStream {
    crate::system::system_macro(attr, item)
}

//==============
// Query
//==============

#[proc_macro]
/// Takes the following parameters:
/// - world: &World
/// - ...components: ...Component
pub fn query(item: TokenStream) -> TokenStream {
    crate::query::gen_query_tokens(item, "query")
}

#[proc_macro]
/// Takes the following parameters:
/// - world: &mut World
/// - ...components: ...Component
pub fn query_mut(item: TokenStream) -> TokenStream {
    crate::query::gen_query_tokens(item, "query_mut")
}

#[proc_macro]
/// Takes the following parameters:
/// - world: &mut World
/// - ...components: ...Component
pub fn query_mut_ptr(_item: TokenStream) -> TokenStream {
    unimplemented!("Mut queries with pointers is currently not supported. These will be reintroduced in a future version.")
}

//==============
// Spawn entity
//==============

#[proc_macro]
pub fn spawn_entity(item: TokenStream) -> TokenStream {
    crate::entity::spawn_entity_macro(item)
}

//======================
// Component
//======================

#[proc_macro_derive(Component)]
pub fn derive_component(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let field_less = match &ast.data {
        syn::Data::Struct(data) => {
            data.fields == syn::Fields::Unit
        },
        syn::Data::Enum(_) |
        syn::Data::Union(_) => false,
    };
    
    if field_less {
        panic!("`#[derive(Component)]` cannot be applied to unit structs. Unit struct can not be used as component, but should be used as flags, use `#[derive(Flag)]` instead.")
    }
    
    let name = &ast.ident;
    let generics_and_lifetimes = &ast.generics;
    
    let ts = TokenStream::from(crate::component::derive_component_impl(name, generics_and_lifetimes));
    
    ts
}

#[proc_macro_attribute]
pub fn flags(_attr: TokenStream, item: TokenStream) -> TokenStream {
    crate::flags::gen_flags_tokens(item)
}
