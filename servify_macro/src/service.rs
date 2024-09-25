use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::Error;
use syn::Ident;
use syn::Result;

pub(crate) fn impl_service(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse2::<ServiceParentAttrs>(attrs)
        .and_then(|attrs| attrs.parse_item(item))
        .unwrap_or_else(Error::into_compile_error)
}

pub struct ServiceParentAttrs {}

impl Parse for ServiceParentAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        while !input.is_empty() {
            let property_name: Ident = input.parse()?;
            match property_name.to_string().as_str() {
                "impls" => {
                    
                } 
                _ => return Err(Error::new(property_name.span(), "Unknown property. expected `impls`")),
            }
        }
        Ok(Self {})
    }
}

impl ServiceParentAttrs {
    fn parse_item(self, item: TokenStream) -> Result<TokenStream> {
        Ok(item)
    }
}