use case::CaseExt;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    spanned::Spanned,
    Error, Ident, ImplItem, ImplItemFn, ItemImpl, Result, TypePath,
};

use crate::util::{return_type_ext::ReturnTypeExt, to_super::to_super, type_path_ext::TypePathExt};

pub(crate) fn impl_export(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse.parse2(item).unwrap_or_else(Error::into_compile_error)
}

struct ExportParent {
    mod_path: TypePath,
}

fn parse(input: ParseStream) -> Result<TokenStream> {
    let top: ItemImpl = input.parse()?;
    let mod_path = match *top.self_ty {
        syn::Type::Path(path) => path,
        _ => Err(Error::new(
            top.self_ty.span(),
            "servify_macro::export can only be used on impl blocks with a TypePath.",
        ))?,
    };

    let parent = ExportParent { mod_path };
    top.items
        .iter()
        .map(|item| match item {
            ImplItem::Fn(item) => parse_method(item, &parent),
            item => Err(Error::new(
                item.span(),
                "servify_macro::export cannot handle implementations other than functions.",
            )),
        })
        .collect()
}

fn parse_method(input: &ImplItemFn, parent: &ExportParent) -> Result<TokenStream> {
    let mod_path = parent.mod_path.clone();

    let struct_name = mod_path.path.segments.last().unwrap().ident.clone();

    let fn_name = input.sig.ident.clone();

    let mod_name = Ident::new(
        &format!("{}_{}", struct_name.to_string().to_snake(), fn_name),
        Span::call_site(),
    );

    let request_name = Ident::new(
        &format!("__{}_request", fn_name.to_string().to_snake()),
        Span::call_site(),
    );

    let response_name = Ident::new(
        &format!("__{}_response", fn_name.to_string().to_snake()),
        Span::call_site(),
    );

    let server_path = mod_path.clone().with_trail_ident("Server");
    let client_path = mod_path.clone().with_trail_ident("Client");

    let internal_fn_name = Ident::new(&format!("__internal_{}", fn_name), Span::call_site());

    let sig = input.sig.inputs.clone();
    let body = input.block.clone();
    let response = input.sig.output.clone().to_type();

    Ok(quote! {
        type #response_name = #response;
        impl #server_path {
            pub async fn #fn_name(&mut self, req: Request) -> #response_name {
            }
            async fn #internal_fn_name(#sig) -> #response_name #body

        }
        mod #mod_name {
            pub use super::{#request_name as Request, #response_name as Response};
        }
    })
}

#[cfg(test)]
mod tests {
    use super::impl_export;
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn fail_if_contains_const() {
        assert_eq! {
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
        assert_eq! {
            impl_export(quote!{}, quote!{
                fn a() {}
            }).to_string(),
            r#":: core :: compile_error ! { "expected `impl`" }"#
        };
    }

    #[test]
    fn test_export() {
        assert_eq! {
            impl_export(quote!{}, quote!{
                impl TestStruct {
                    fn add_hello(&mut self, n: usize) -> String {
                        self.a.push_str(&"Hello".repeat(n));
                        self.a.clone()
                    }
                }
            }).to_string(),
            quote!{
                type __add_hello_response = String;
                impl TestStruct::Server {
                    pub async fn add_hello(&mut self, req: Request) -> __add_hello_response {
                    }
                    async fn __internal_add_hello(&mut self, n: usize) -> __add_hello_response {
                        self.a.push_str(&"Hello".repeat(n));
                        self.a.clone()
                    }
                }
                mod test_struct_add_hello {
                    pub use super::{__add_hello_request as Request, __add_hello_response as Response};
                }
            }.to_string()
        };
    }
}
