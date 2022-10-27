use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn gen_flags_tokens(item: TokenStream) -> TokenStream {
    let enum_ast: syn::ItemEnum = syn::parse(item)
        .expect("The `flags` macro attribute can only be applied to enums.");
    let name = &enum_ast.ident;
    
    // ATTENTION: if type of FlagId changes, also change `#[repr(...)]`
    TokenStream::from(quote! {
        #[repr(u32)]
        #enum_ast
        
        impl kiwi_ecs::Flag for #name {} 
        impl ::std::convert::Into<kiwi_ecs::FlagId> for #name {
            #[inline]
            fn into(self) -> kiwi_ecs::FlagId {
                self as kiwi_ecs::FlagId
            }
        }
    })
}
