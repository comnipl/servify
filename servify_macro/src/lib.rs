mod export;
use export::impl_export;

#[proc_macro_attribute]
pub fn export(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_export(attrs.into(), item.into()).into()
}
