use quote::quote;
use syn::{parse_quote, ItemEnum};

pub fn generate_event_enum() -> proc_macro2::TokenStream {
    let event_enum: ItemEnum = parse_quote! {
        enum Event {
            Tick,
        }
    };

    quote! {
        #event_enum
    }
}
