use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser}, Error, ImplItem, ImplItemFn, ItemImpl, Result
};

pub(crate) fn impl_export(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse.parse2(item)
        .unwrap_or_else(Error::into_compile_error)
}

fn parse(input: ParseStream) -> Result<TokenStream> {
    let top: ItemImpl = input.parse()?;
    top.items.iter()
        .filter_map(|item| match item {
            ImplItem::Fn(method) => Some(method),
            _ => None
        })
        .map(|item| parse_method(item)).collect()
}

fn parse_method(input: &ImplItemFn) -> Result<TokenStream> {
    let name = input.sig.ident.clone();
    Ok(quote! { #name })
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
                impl A {
                    fn a(&self) {}
                    fn b(&self) {}
                }
            }).to_string(),
            quote!{
                a b
            }.to_string()
        };
    }
}
