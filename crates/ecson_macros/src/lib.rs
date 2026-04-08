use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        // bevy_ecs をスコープに引き込み、derive が生成するパスを解決させる
        use ::ecson::bevy_ecs as bevy_ecs;
        #[derive(bevy_ecs::prelude::Component)]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn resource(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        use ::ecson::bevy_ecs as bevy_ecs;
        #[derive(bevy_ecs::prelude::Resource)]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        use ::ecson::bevy_ecs as bevy_ecs;
        #[derive(bevy_ecs::prelude::Message)]
        #item
    }
    .into()
}
