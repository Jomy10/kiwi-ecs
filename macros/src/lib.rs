use proc_macro::TokenStream;

mod system;
mod query;
mod component;
mod entity;

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
/// - world: &mut World
/// - components...: Type: Component...
pub fn query(item: TokenStream) -> TokenStream {
    crate::query::gen_query_tokens(item, "query")
}

#[proc_macro]
pub fn query_mut(item: TokenStream) -> TokenStream {
    crate::query::gen_query_tokens(item, "query_mut")
}

#[proc_macro]
pub fn query_mut_ptr(_item: TokenStream) -> TokenStream {
    unimplemented!("Mut qqueries with pointers is currently not supported. These will be reintroduced in version 1.0.4")
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
    let name = &ast.ident;
    
    TokenStream::from(crate::component::derive_component_impl(name))
}
