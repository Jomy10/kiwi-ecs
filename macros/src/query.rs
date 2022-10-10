use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn gen_query_tokens(item: TokenStream, func_name: &str) -> TokenStream {
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
    
    // TODO: make a better entity id check
    let query_id = components[0].to_string() == "EntityId".to_string();
    
    let components = if query_id {
        &components[1..components.len()]
    } else {
        &components
    };
    
    let func_name: TokenStream2 = format!("{func_name}{}{}", if query_id { "_ids" } else { "" }, components.len()).parse().unwrap();
    
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
