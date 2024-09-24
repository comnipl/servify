use case::CaseExt;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Expr, ExprField, ExprPath, Field, FieldMutability, FieldsNamed, FnArg, Ident, ImplItem,
    ImplItemFn, ItemImpl, Member, Pat, PatType, Result, Token, TypePath, Visibility,
};

use crate::util::{return_type_ext::ReturnTypeExt, type_path_ext::TypePathExt};

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

    let request_sig = sig
        .clone()
        .into_iter()
        .filter_map(|i| match i {
            FnArg::Typed(PatType { pat, ty, .. }) => match *pat {
                Pat::Ident(ident) => {
                    Some((Ident::new(&ident.ident.to_string(), Span::call_site()), *ty))
                }
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    let struct_block = FieldsNamed {
        brace_token: Default::default(),
        named: Punctuated::from_iter(request_sig.clone().into_iter().map(|(ident, ty)| Field {
            attrs: Default::default(),
            vis: Visibility::Inherited,
            ident: Some(ident),
            colon_token: Default::default(),
            ty,
            mutability: FieldMutability::None,
        })),
    };

    let call_server_args: Punctuated<ExprField, Token![,]> =
        Punctuated::from_iter(request_sig.clone().into_iter().map(|(ident, _)| ExprField {
            attrs: Default::default(),
            member: Member::Named(Ident::new(&ident.to_string(), Span::call_site())),
            dot_token: Default::default(),
            base: Box::new(Expr::Path(ExprPath {
                attrs: Default::default(),
                qself: None,
                path: Ident::new("req", Span::call_site()).into(),
            })),
        }));

    Ok(quote! {
        #[allow(non_camel_case_types)]
        type #response_name = #response;

        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        pub struct #request_name #struct_block

        impl #server_path {
            pub async fn #fn_name(&mut self, req: Request) -> #response_name {
                self.#internal_fn_name(#call_server_args).await
            }
            async fn #internal_fn_name(#sig) -> #response_name #body

        }
        pub mod #mod_name {
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
                impl SomeStruct {
                    fn increment(&mut self, count: u32) -> u32 {
                        self.count += count;
                        self.count
                    }
                }
            }).to_string(),

            quote!{
                #[allow(non_camel_case_types)]
                type __increment_response = u32;

                #[allow(non_camel_case_types)]
                #[derive(Clone)]
                pub struct __increment_request {
                    count: u32
                }

                impl SomeStruct::Server {
                    pub async fn increment(&mut self, req: Request) -> __increment_response {
                        self.__internal_increment(req.count).await
                    }
                    async fn __internal_increment(&mut self, count: u32) -> __increment_response {
                        self.count += count;
                        self.count
                    }
                }
                pub mod some_struct_increment {
                    pub use super::{__increment_request as Request, __increment_response as Response};
                }
            }.to_string()
        };
    }
}
