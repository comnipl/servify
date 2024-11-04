use proc_macro::TokenStream;

use export_macro::ServifyExport;
use export_macro::ServifyExportArgs;

pub(crate) mod export_macro;
pub(crate) mod util;

#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as ServifyExportArgs);
    syn::parse_macro_input!(item as ServifyExport)
        .to_tokens(attr)
        .into()
}
