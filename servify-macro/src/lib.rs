use proc_macro2::TokenStream;

#[proc_macro_attribute]
pub fn export(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_export(attrs.into(), item.into()).into()
}

fn impl_export(attrs: TokenStream, item: TokenStream) -> TokenStream {
    item
}
