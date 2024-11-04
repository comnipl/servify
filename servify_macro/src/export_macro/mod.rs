mod tests;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, TypePath};

use crate::util::ident_ext::IdentExt as _;

/// Represents the arguments of the `#[servify::export]` attribute.
pub(crate) struct ServifyExportArgs {
    /// The path to the service module.
    service_module: TypePath,
}

impl Parse for ServifyExportArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let service_module = input.parse()?;
        Ok(Self { service_module })
    }
}

/// Represents the `#[servify::export]` content.
pub(crate) struct ServifyExport {
    fn_name: Ident,
}

impl Parse for ServifyExport {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            fn_name: input.parse()?,
        })
    }
}

impl ServifyExport {
    pub(crate) fn to_tokens(self, args: ServifyExportArgs) -> TokenStream {
        let ServifyExportArgs { service_module } = args;
        let ServifyExport { fn_name } = self;

        let service_module_name = &service_module.path.segments.last().unwrap().ident;

        let mod_name = Ident::new_with_call_site(&format!("{service_module_name}_{fn_name}"));

        quote! {
            mod #mod_name {  }
        }
    }
}
