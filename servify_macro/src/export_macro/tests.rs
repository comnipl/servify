#![cfg(test)]
use insta::assert_snapshot;
use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, RustFmt};
use thiserror::Error;

use crate::export_macro::{ServifyExport, ServifyExportArgs};

#[derive(Debug, Error)]
enum Error {
    #[error("Failed to parse args: {0}")]
    ParseArgs(TokenStream),
    #[error("Failed to parse item: {0}")]
    ParseItem(TokenStream),
    #[error("Failed to format: {0}")]
    RustFmt(#[from] rust_format::Error),
}

fn test(args: TokenStream, item: TokenStream) -> Result<String, Error> {
    let args = syn::parse2::<ServifyExportArgs>(args)
        .map_err(|e| e.to_compile_error())
        .map_err(Error::ParseArgs)?;
    let item = syn::parse2::<ServifyExport>(item)
        .map_err(|e| e.to_compile_error())
        .map_err(Error::ParseItem)?;

    let generated = item.to_tokens(args).to_string();

    let formatted = RustFmt::default().format_str(generated)?;
    Ok(formatted)
}

#[test]
fn test_snapshot_export_all_written() {
    assert_snapshot!(test(
        quote! {},
        quote! {
            impl super::counter {
                pub fn increment(&mut self, amount: u32) -> u32 {
                    self.count += amount;
                    self.amount
                }
            }
        }
    )
    .unwrap());
}

#[test]
fn test_snapshot_export_unwanted_struct() {
    assert_snapshot!(test(
        quote! {},
        quote! {
            impl super::counter {
                pub fn increment(&mut self, amount: u32) -> u32 {
                    self.count += amount;
                    self.amount
                }
                struct ThisMustBeUnexpected {  };
            }
        }
    )
    .unwrap_err());
}
