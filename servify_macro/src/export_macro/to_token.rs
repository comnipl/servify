use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::util::ident_ext::IdentExt as _;

use super::{ServifyExport, ServifyExportArgs};

impl ServifyExport {
    pub(crate) fn to_tokens(self, args: ServifyExportArgs) -> TokenStream {
        let ServifyExport {
            service_module_path,
            fn_item,
            fn_name,
        } = self;

        let service_name = &service_module_path.path.segments.last().unwrap().ident;
        let mod_name = Ident::new_with_call_site(&format!("{service_name}_{fn_name}"));

        quote! {
            mod #mod_name {  }
        }
    }
}
