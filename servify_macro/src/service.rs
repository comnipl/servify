use std::vec;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::quote;
use syn::bracketed;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Error;
use syn::Ident;
use syn::ItemStruct;
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
    internal_function: TokenStream,
    enum_element: TokenStream,
    server_arm: TokenStream,
}

impl ServiceParentAttrs {
    fn parse_item(self, item: TokenStream) -> Result<TokenStream> {
        let server: ItemStruct = parse2(item)?;

        let mod_name = server.ident.clone();
        let server_items = server.fields;

        let tokens: Vec<ImplTokens> = self
            .impls
            .clone()
            .into_iter()
            .filter_map(|path| {
                let fn_name = path.path.segments.last().unwrap().ident.clone();
                let fn_name = Ident::new(
                    &fn_name.to_string()
                    .strip_prefix(&mod_name.to_string())
                    .map(|p| p.trim_start_matches('_').to_string())?,
                    fn_name.span(),
                );
                let internal_fn_name = Ident::new(
                    &format!("__internal_{}", fn_name),
                    fn_name.span(),
                );

                let enum_name = Ident::new(&fn_name.to_string().to_camel(), fn_name.span());

                let super_path = path.clone().to_super();

                let internal_function = quote! {
                    #[doc(hidden)]
                    pub async fn #internal_fn_name(
                        client: &Client,
                        req: <#super_path as ::servify::ServifyExport>::Request,
                    ) -> <#super_path as ::servify::ServifyExport>::Response {
                        let (tx, rx) = ::tokio::sync::oneshot::channel();
                        client.tx.send(Message::#enum_name(req, tx)).await.unwrap();
                        rx.await.unwrap()
                    }
                };

                let enum_element = quote! {
                    #enum_name(
                        <#super_path as ::servify::ServifyExport>::Request,
                        ::tokio::sync::oneshot::Sender<<#super_path as ::servify::ServifyExport>::Response>,
                    ),
                };

                let server_arm = quote! {
                    Message::#enum_name(req, tx) => {
                        let res = self.#fn_name(req).await;
                        tx.send(res).unwrap();
                    },
                };


                Some(ImplTokens {
                    internal_function,
                    enum_element,
                    server_arm,
                })
            })
            .collect();

        let internal_functions: TokenStream =
            tokens.iter().map(|t| t.internal_function.clone()).collect();
        let enum_elements: TokenStream = tokens.iter().map(|t| t.enum_element.clone()).collect();
        let server_arms: TokenStream = tokens.iter().map(|t| t.server_arm.clone()).collect();

        Ok(quote! {
            #[allow(non_snake_case)]
            mod #mod_name {

                pub enum Message {
                    #enum_elements
                }

                pub struct Server #server_items

                #[derive(Clone)]
                pub struct Client {
                    tx: ::tokio::sync::mpsc::Sender<Message>,
                }

                impl Server {
                    pub async fn listen(&mut self, mut rx: ::tokio::sync::mpsc::Receiver<Message>) {
                        while let Some(msg) = rx.recv().await {
                            match msg {
                                #server_arms
                            }
                        }
                    }
                }

                #internal_functions

                pub fn initiate_message_passing(buffer: usize) -> (::tokio::sync::mpsc::Receiver<Message>, Client) {
                    let (tx, rx) = ::tokio::sync::mpsc::channel(buffer);
                    let client = Client { tx };
                    (rx, client)
                }
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
                impls = [SomeStruct_increment],
            }, quote!{
                struct SomeStruct {
                    pub count: u32,
                }
            }).to_string(),
            quote!{
                #[allow(non_snake_case)]
                mod SomeStruct {
                    pub enum Message {
                        Increment(
                            <super::SomeStruct_increment as ::servify::ServifyExport>::Request,
                            ::tokio::sync::oneshot::Sender<<super::SomeStruct_increment as ::servify::ServifyExport>::Response>,
                        ),
                    }

                    pub struct Server {
                        pub count: u32,
                    }

                    #[derive(Clone)]
                    pub struct Client {
                        tx: ::tokio::sync::mpsc::Sender<Message>,
                    }

                    impl Server {
                        pub async fn listen(&mut self, mut rx: ::tokio::sync::mpsc::Receiver<Message>) {
                            while let Some(msg) = rx.recv().await {
                                match msg {
                                    Message::Increment(req, tx) => {
                                        let res = self.increment(req).await;
                                        tx.send(res).unwrap();
                                    },
                                }
                            }
                        }
                    }

                    #[doc(hidden)]
                    pub async fn __internal_increment(
                        client: &Client,
                        req: <super::SomeStruct_increment as ::servify::ServifyExport>::Request,
                    ) ->  <super::SomeStruct_increment as ::servify::ServifyExport>::Response {
                        let (tx, rx) = ::tokio::sync::oneshot::channel();
                        client.tx.send(Message::Increment(req, tx)).await.unwrap();
                        rx.await.unwrap()
                    }

                    pub fn initiate_message_passing(buffer: usize) -> (::tokio::sync::mpsc::Receiver<Message>, Client) {
                        let (tx, rx) = ::tokio::sync::mpsc::channel(buffer);
                        let client = Client { tx };
                        (rx, client)
                    }
                }
            }.to_string(),
        };
    }
}
