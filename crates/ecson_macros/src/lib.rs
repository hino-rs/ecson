use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        #[derive(::bevy_ecs::prelude::Component)]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn resource(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        #[derive(::bevy_ecs::prelude::Resource)]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        #[derive(::bevy_ecs::prelude::Message)]
        #item
    }
    .into()
}
