use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser}, spanned::Spanned, Error, ImplItem, ImplItemFn, ItemImpl, Result
};

pub(crate) fn impl_export(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse.parse2(item)
        .unwrap_or_else(Error::into_compile_error)
}

fn parse(input: ParseStream) -> Result<TokenStream> {
    let top: ItemImpl = input.parse()?;
    top.items.iter()
        .map(|item| match item {
            ImplItem::Fn(item) => parse_method(item),
            item => Err(Error::new(
                item.span(),
                format!("servify_macro::export cannot handle implementations other than functions."),
            ))
        })
        .collect()
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
    fn fail_if_contains_const() {
        assert_eq!{
            impl_export(quote!{}, quote!{
                impl A {
                    const A: usize = 0;
                    fn a(&self) {}
                }
            }).to_string(),
            r#":: core :: compile_error ! { "servify_macro::export cannot handle implementations other than functions." }"#,
        };
    }

    #[test]
    fn fail_if_impl_to_fn() {
        assert_eq!{
            impl_export(quote!{}, quote!{
                fn a() {}
            }).to_string(),
            r#":: core :: compile_error ! { "expected `impl`" }"#
        };
    }

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
