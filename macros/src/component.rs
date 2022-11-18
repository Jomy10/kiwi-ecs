use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::sync::Mutex;

static mut COMP_ID_COUNTER: Mutex<u32> = Mutex::new(0);

/// `field_less`: whether the strut has no fields
pub fn derive_component_impl(name: &proc_macro2::Ident, generics_and_lifetimes: &syn::Generics) -> TokenStream2 {
    let comp_id = unsafe { &COMP_ID_COUNTER };
    let mut guard = comp_id.lock().unwrap();
    let this_id = *guard;
    *guard += 1;
    
    let name = quote! {
        #name
    };

    let generics_code = generics_and_lifetimes.params.iter()
        .map(|param| {
            match param {
                syn::GenericParam::Type(param) => {
                    let name = &param.ident;
                    quote! {
                        #name
                    }
                },
                syn::GenericParam::Lifetime(lifetime) => {
                    let name = &lifetime.lifetime.ident;
                    let lifetime = syn::Lifetime::new(&format!("'{}", name), proc_macro2::Span::call_site());
                    quote! {
                         #lifetime
                    }
                },
                syn::GenericParam::Const(_) => unimplemented!("Const generics are not yet implemented for the derive component macro. Feel free to open a PR, or an issue."),
            }
        })
        .enumerate()
        .map(|(i, v)| {
            if i != generics_and_lifetimes.params.len() - 1 {
                quote! { #v, }
            } else {
                quote! { #v }
            }
        }).collect::<Vec<TokenStream2>>();
    
    let generics_def = generics_and_lifetimes;
    
    quote! {
        impl #generics_def #name<#(#generics_code)*> {
            #[inline(always)]
            fn get_archetypes_rwlock() -> &'static std::sync::RwLock<Vec<kiwi_ecs::ArchetypeId>> {
                static ARCHETYPES: std::sync::RwLock<Vec<kiwi_ecs::ArchetypeId>> = std::sync::RwLock::new(Vec::new());
                &ARCHETYPES
            }
            
            #[inline(always)]
            fn get_archetypes_read() -> std::sync::RwLockReadGuard<'static, Vec<kiwi_ecs::ArchetypeId>> {
                let archetype = Self::get_archetypes_rwlock();
                let guard = archetype.read().unwrap();
                guard
            }
            
            #[inline(always)]
            fn get_archetypes_write() -> std::sync::RwLockWriteGuard<'static, Vec<kiwi_ecs::ArchetypeId>> {
                let archetype = Self::get_archetypes_rwlock();
                let guard = archetype.write().unwrap();
                guard
            }
        }
        impl #generics_def Component for #name<#(#generics_code)*> {
            #[inline]
            fn get_archetypes() -> ::std::sync::RwLockReadGuard<'static, Vec<kiwi_ecs::ArchetypeId>> {
                Self::get_archetypes_read()
            }
            #[inline]
            fn add_archetype(arch_id: kiwi_ecs::ArchetypeId) where Self: Sized {
                let mut arch_guard = Self::get_archetypes_write();
                (*arch_guard).push(arch_id);
            }
            #[inline(always)]
            fn id() -> kiwi_ecs::ComponentId where Self: Sized { #this_id }
        }
    }
}
