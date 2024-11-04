use proc_macro2::Span;
use syn::Ident;

pub(crate) trait IdentExt {
    fn new_with_call_site(name: &str) -> Ident;
}

impl IdentExt for Ident {
    fn new_with_call_site(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }
}
