#![cfg(test)]
use insta::assert_snapshot;
use quote::quote;
use rust_format::{Formatter, RustFmt};

use crate::export_macro::{ServifyExport, ServifyExportArgs};

#[test]
fn test_snapshot_export_all_written() {
    let attr = quote! { super::counter };
    let input = quote! { get };

    let args = syn::parse2::<ServifyExportArgs>(attr).unwrap();
    let export = syn::parse2::<ServifyExport>(input).unwrap();

    let generated = export.to_tokens(args).to_string();

    let formatted = RustFmt::default()
        .format_str(generated)
        .expect("Failed to format");
    assert_snapshot!(formatted);
}
