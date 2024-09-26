use std::vec;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::bracketed;
use syn::parenthesized;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::token::Struct;
use syn::Error;
use syn::Ident;
use syn::ItemStruct;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::TypePath;

use crate::util::type_path_ext::TypePathExt;


pub(crate) fn impl_service(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse2::<ServiceParentAttrs>(attrs)
        .and_then(|attrs| attrs.parse_item(item))
        .unwrap_or_else(Error::into_compile_error)
}

pub struct ServiceParentAttrs {
    impls: Vec<TypePath>,
}

impl Parse for ServiceParentAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut impls = vec![];

        while !input.is_empty() {
            let property_name: Ident = input.parse()?;
            match property_name.to_string().as_str() {
                "impls" => {
                    let _eq: Token![=] = input.parse()?;
                    let group;
                    let _paren = bracketed!(group in input);
                    let paths = Punctuated::<TypePath, Comma>::parse_terminated(&group)?;
                    impls.extend(paths);

                }
                _ => {
                    return Err(Error::new(
                        property_name.span(),
                        "Unknown property. expected `impls`",
                    ))
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { impls })
    }
}

struct ImplTokens {
    internal_function: TokenStream
}

impl ServiceParentAttrs {
    fn parse_item(self, item: TokenStream) -> Result<TokenStream> {
        let server: ItemStruct = parse2(item)?;
        
        let mod_name = server.ident.clone();
        let server_items = server.fields;

        let tokens: Vec<ImplTokens> = self.impls.clone().into_iter()
            .map(|path| {
                let fn_name = path.path.segments.last().unwrap().ident.clone();
                let internal_fn_name = Ident::new(&format!("__internal_{}", fn_name.to_string()), fn_name.span());
                
                let enum_name = Ident::new(&fn_name.to_string().to_camel(), fn_name.span());

                let request_path = path.clone().to_super().with_trail_ident("Request");
                let response_path = path.clone().to_super().with_trail_ident("Response");

                let internal_function = quote! {
                    #[doc(hidden)]
                    pub async fn #internal_fn_name(
                        client: &Client,
                        req: #request_path,
                    ) -> #response_path {
                        let (tx, rx) = ::tokio::sync::oneshot::channel();
                        client.tx.send(Message::#enum_name(req, tx)).await.unwrap();
                        rx.await.unwrap()
                    }
                };

                ImplTokens { internal_function }
            })
            .collect();

        let internal_functions: TokenStream = tokens.iter().map(|t| t.internal_function.clone()).collect();

        Ok(quote! {
            #[allow(non_snake_case)]
            mod #mod_name {
                pub struct Server #server_items

                #[derive(Clone)]
                pub struct Client {
                    tx: ::tokio::sync::mpsc::Sender<Message>,
                }

                #internal_functions
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::impl_service;
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn single() {
        assert_eq! {
            impl_service(quote!{
                impls = [increment],
            }, quote!{
                struct SomeStruct {
                    pub count: u32,
                }
            }).to_string(),
            quote!{
                #[allow(non_snake_case)]
                mod SomeStruct {
                    pub struct Server {
                        pub count: u32,
                    }

                    #[derive(Clone)]
                    pub struct Client {
                        tx: ::tokio::sync::mpsc::Sender<Message>,
                    }

                    #[doc(hidden)]
                    pub async fn __internal_increment(
                        client: &Client,
                        req: super::increment::Request,
                    ) -> super::increment::Response {
                        let (tx, rx) = ::tokio::sync::oneshot::channel();
                        client.tx.send(Message::Increment(req, tx)).await.unwrap();
                        rx.await.unwrap()
                    }
                }
            }.to_string(),
        };
    }
}
