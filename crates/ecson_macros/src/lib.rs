use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

/// Implements `bevy_ecs::component::Component` via `#[ecson::component]`
/// or `#[component]` (after `use ecson::prelude::*`).
///
/// # Example
/// ```rust, ignore
/// use ecson::prelude::*;
///
/// #[component]
/// struct Position { x: f32, y: f32 }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let mut impl_generics = input.generics.clone();
    impl_generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (ig, tg, wc) = impl_generics.split_for_impl();

    quote! {
        #input
        impl #ig ::ecson::bevy_ecs::component::Component for #name #tg #wc {
            const STORAGE_TYPE: ::ecson::bevy_ecs::component::StorageType =
                ::ecson::bevy_ecs::component::StorageType::Table;
            type Mutability = ::ecson::bevy_ecs::component::Mutable;
        }
    }
    .into()
}

/// Implements `bevy_ecs::system::Resource` via `#[ecson::resource]`
/// or `#[resource]` (after `use ecson::prelude::*`).
///
/// # Example
/// ```ignore
/// use ecson::prelude::*;
///
/// #[resource]
/// struct PlayerCount(u32);
/// ```
#[proc_macro_attribute]
pub fn resource(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let mut impl_generics = input.generics.clone();
    impl_generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (ig, tg, wc) = impl_generics.split_for_impl();

    quote! {
        #input
        impl #ig ::ecson::bevy_ecs::resource::Resource for #name #tg #wc {}
    }
    .into()
}

/// Implements `bevy_ecs::message::Message` via `#[ecson::message]`
/// or `#[message]` (after `use ecson::prelude::*`).
///
/// # Example
/// ```ignore
/// use ecson::prelude::*;
///
/// #[message]
/// struct ChatMessage { content: String }
/// ```
#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let mut impl_generics = input.generics.clone();
    impl_generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (ig, tg, wc) = impl_generics.split_for_impl();

    quote! {
        #input
        impl #ig ::ecson::bevy_ecs::message::Message for #name #tg #wc {}
    }
    .into()
}
