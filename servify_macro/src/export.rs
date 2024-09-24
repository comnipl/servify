use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser}, spanned::Spanned, Error, ImplItem, ImplItemFn, ItemImpl, Result, TypePath
};

pub(crate) fn impl_export(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse.parse2(item)
        .unwrap_or_else(Error::into_compile_error)
}

struct ExportParent {
    mod_path: TypePath,
}

fn parse(input: ParseStream) -> Result<TokenStream> {
    let top: ItemImpl = input.parse()?;
    let mod_path = match *top.self_ty {
        syn::Type::Path(path) => {
            path   
        }
        _ => Err(Error::new(
            top.self_ty.span(),
            "servify_macro::export can only be used on impl blocks with a TypePath.",
        ))?,
    };

    let parent = ExportParent {
        mod_path
    };
    top.items.iter()
        .map(|item| match item {
            ImplItem::Fn(item) => parse_method(item, &parent),
            item => Err(Error::new(
                item.span(),
                format!("servify_macro::export cannot handle implementations other than functions."),
            ))
        })
        .collect()
}

fn parse_method(input: &ImplItemFn, parent: &ExportParent) -> Result<TokenStream> {
    let mod_path = parent.mod_path.clone();
    let name = input.sig.ident.clone();
    Ok(quote! { #mod_path - #name })
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
                A - a A - b
            }.to_string()
        };
    }
}
