mod args;
mod errors;
mod tests;
mod to_token;

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Ident, ImplItemFn, Token, TypePath};

use crate::export_macro::errors::{
    ERR_MULTIPLE_FN_IN_EXPORT, ERR_NO_FN_IN_EXPORT, ERR_UNEXPECTED_ITEM_IN_EXPORT,
};

pub(crate) use self::args::ServifyExportArgs;

/// Represents the `#[servify::export]` content.
pub(crate) struct ServifyExport {
    service_module_path: TypePath,
    fn_item: ImplItemFn,
    fn_name: Ident,
}

impl Parse for ServifyExport {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let mut fn_item = None;

        // Parse head impl block
        let _: Token![impl] = input.parse()?;
        let module_path: TypePath = input.parse()?;
        braced!(content in input);

        // Parse all items in the export block
        while !content.is_empty() {
            if let Ok(item) = content.parse::<ImplItemFn>() {
                if fn_item.is_some() {
                    return Err(syn::Error::new_spanned(item, ERR_MULTIPLE_FN_IN_EXPORT));
                }

                fn_item.replace(item);
                continue;
            }

            // Error if there is any other item in the export block
            return Err(syn::Error::new_spanned(
                content.parse::<syn::Item>()?,
                ERR_UNEXPECTED_ITEM_IN_EXPORT,
            ));
        }

        // Ensure that there is a function in the export block
        let fn_item =
            fn_item.ok_or_else(|| syn::Error::new(Span::call_site(), ERR_NO_FN_IN_EXPORT))?;

        Ok(Self {
            service_module_path: module_path,
            fn_name: fn_item.sig.ident.clone(),
            fn_item,
        })
    }
}
