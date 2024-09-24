use super::type_path_ext::TypePathExt;
use syn::TypePath;

pub(crate) fn to_super(original: TypePath) -> TypePath {
    if original.path.leading_colon.is_some() {
        return original;
    }

    if original
        .path
        .segments
        .first()
        .map(|e| e.ident == "crate")
        .unwrap_or(false)
    {
        return original;
    }

    original.with_inserted_ident("super", 0)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use quote::{quote, ToTokens};
    use syn::parse2;

    use super::*;

    #[test]
    fn relative_path() {
        assert_eq! {
            to_super(
                parse2(quote! {
                    tests::expanded_1::SomeStruct
                })
                .unwrap()
            ).to_token_stream().to_string(),
            quote! {
                super::tests::expanded_1::SomeStruct
            }.to_string()
        }
    }

    #[test]
    fn absolute_path() {
        assert_eq! {
            to_super(
                parse2(quote! {
                    ::tests::expanded_1::SomeStruct
                })
                .unwrap()
            ).to_token_stream().to_string(),
            quote! {
                ::tests::expanded_1::SomeStruct
            }.to_string()
        }
    }

    #[test]
    fn crate_absolute_path() {
        assert_eq! {
            to_super(
                parse2(quote! {
                    crate::tests::expanded_1::SomeStruct
                })
                .unwrap()
            ).to_token_stream().to_string(),
            quote! {
                crate::tests::expanded_1::SomeStruct
            }.to_string()
        }
    }

    #[test]
    fn ident() {
        assert_eq! {
            to_super(
                parse2(quote! {
                    SomeStruct
                })
                .unwrap()
            ).to_token_stream().to_string(),
            quote! {
                super::SomeStruct
            }.to_string()
        }
    }
}
