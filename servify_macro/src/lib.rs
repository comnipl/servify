mod export;
mod service;
mod util;
use export::impl_export;
use service::impl_service;

#[proc_macro_attribute]
pub fn export(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_export(attrs.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn service(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_service(attrs.into(), item.into()).into()
}
