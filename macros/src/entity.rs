use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Token, punctuated::Punctuated};

pub fn spawn_entity_macro(item: TokenStream) -> TokenStream {
    // let ast: Punctuated<syn::Expr, syn::token::Comma> = Punctuated::parse_terminated(
    //     syn::parse(item).unwrap()
    // ).unwrap(); //syn::parse(item.clone()).unwrap();
    let ast: Punctuated<syn::Expr, syn::token::Comma> = syn::parse_macro_input!(item with Punctuated::<syn::Expr, Token![,]>::parse_terminated);
    let tokens: Vec<TokenStream2> = ast.into_iter().map(|v| quote! { #v }).collect();
    let args: &[TokenStream2] = &tokens[1..tokens.len()];
    let count = tokens.len() - 1; // excluding world
    let fn_name: TokenStream2 = format!("spawn_entity{count}").parse().unwrap();
    let world: &TokenStream2 = &tokens[0];
    
    TokenStream::from(quote! {
        #world.#fn_name(#(#args, )*)
    })
}
