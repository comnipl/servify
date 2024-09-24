use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::Error;
use syn::Result;

pub(crate) fn impl_service(attrs: TokenStream, item: TokenStream) -> TokenStream {
    syn::parse2::<ServiceParent>(item)
        .map(|i| i.0)
        .unwrap_or_else(Error::into_compile_error)
}

pub struct ServiceParentAttrs {}

impl Parse for ServiceParentAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!()
    }
}

pub struct ServiceParent(TokenStream);

impl Parse for ServiceParent {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!()
    }
}
