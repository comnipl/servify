use proc_macro2::TokenStream;
pub(crate) fn impl_export(attrs: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use super::impl_export;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_export() {
        assert_eq!{
            impl_export(quote!{}, quote!{
                impl A {}
            }).to_string(),
            quote!{
                impl A {}
            }.to_string()
        };
    }
}