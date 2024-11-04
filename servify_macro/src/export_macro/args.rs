use syn::parse::{Parse, ParseStream};

/// Represents the arguments of the `#[servify::export]` attribute.
pub(crate) struct ServifyExportArgs {}

impl Parse for ServifyExportArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}
