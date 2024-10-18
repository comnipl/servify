mod util;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::Error;
use syn::Ident;
use syn::ItemStruct;
use syn::Result;

#[proc_macro_attribute]
pub fn sandbox1(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_sandbox1(attrs.into(), item.into()).into()
}

pub(crate) fn impl_sandbox1(attrs: TokenStream, item: TokenStream) -> TokenStream {
    parse2::<Sandbox1Attrs>(attrs)
        .and_then(|attrs| attrs.parse_item(item))
        .unwrap_or_else(Error::into_compile_error)
}

struct Sandbox1Attrs {
    kind: Ident,
}

impl Parse for Sandbox1Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind: Ident = input.parse()?;
        Ok(Self { kind })
    }
}

impl Sandbox1Attrs {
    fn parse_item(self, item: TokenStream) -> Result<TokenStream> {
        let _struct = parse2::<ItemStruct>(item)?;
        let kind_x = self.kind.clone();
        let kind_y = self.kind.clone();
        Ok(quote! {
            enum KindsX {
                AAa, AAb, AB, AC
            }
            enum KindsY {
                Ab, Ac
            }
            fn sandbox() {
                // let _ = KindsX::#kind_x;
                let _ = KindsY::#kind_y;
            }
        })
    }
}