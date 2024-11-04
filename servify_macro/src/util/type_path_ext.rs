use proc_macro2::Span;
use syn::{Ident, PathArguments, PathSegment, TypePath};

#[allow(dead_code)]
pub(crate) trait TypePathExt {
    fn with_inserted_ident(self, ident: &str, index: usize) -> TypePath;
    fn with_trail_ident(self, ident: &str) -> TypePath;
    fn to_super(self) -> TypePath;
}

impl TypePathExt for TypePath {
    fn with_inserted_ident(mut self, ident: &str, index: usize) -> TypePath {
        self.path.segments.insert(
            index,
            PathSegment {
                ident: Ident::new(ident, Span::call_site()),
                arguments: PathArguments::None,
            },
        );
        self
    }
    fn with_trail_ident(mut self, ident: &str) -> TypePath {
        self.path.segments.push(PathSegment {
            ident: Ident::new(ident, Span::call_site()),
            arguments: PathArguments::None,
        });
        self
    }

    fn to_super(self) -> TypePath {
        if self.path.leading_colon.is_some() {
            return self;
        }

        if self
            .path
            .segments
            .first()
            .map(|e| e.ident == "crate")
            .unwrap_or(false)
        {
            return self;
        }

        self.with_inserted_ident("super", 0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use quote::{quote, ToTokens};
    use syn::parse2;

    use super::*;

    #[test]
    fn with_trail_ident() {
        let path: TypePath = parse2(quote! {
            std::collections::HashMap
        })
        .unwrap();
        assert_eq! {
            path.clone().with_trail_ident("Entry").to_token_stream().to_string(),
            quote! {
                std::collections::HashMap::Entry
            }.to_token_stream().to_string()
        }
    }

    #[test]
    fn with_inserted_ident() {
        let path: TypePath = parse2(quote! {
            std::collections::HashMap
        })
        .unwrap();
        assert_eq! {
            path.clone().with_inserted_ident("Entry", 1).to_token_stream().to_string(),
            quote! {
                std::Entry::collections::HashMap
            }.to_token_stream().to_string()
        }
    }

    #[test]
    fn to_super_relative_path() {
        assert_eq! {
            parse2::<TypePath>(quote! {
                tests::expanded_1::SomeStruct
            })
            .unwrap()
            .to_super()
            .to_token_stream().to_string(),
            quote! {
                super::tests::expanded_1::SomeStruct
            }.to_string()
        }
    }

    #[test]
    fn to_super_absolute_path() {
        assert_eq! {
                parse2::<TypePath>(quote! {
                    ::tests::expanded_1::SomeStruct
                })
                .unwrap()
            .to_super().to_token_stream().to_string(),
            quote! {
                ::tests::expanded_1::SomeStruct
            }.to_string()
        }
    }

    #[test]
    fn to_super_crate_absolute_path() {
        assert_eq! {
                parse2::<TypePath>(quote! {
                    crate::tests::expanded_1::SomeStruct
                })
                .unwrap()
            .to_super().to_token_stream().to_string(),
            quote! {
                crate::tests::expanded_1::SomeStruct
            }.to_string()
        }
    }

    #[test]
    fn to_super_ident() {
        assert_eq! {
                parse2::<TypePath>(quote! {
                    SomeStruct
                })
                .unwrap()
                .to_super()
            .to_token_stream().to_string(),
            quote! {
                super::SomeStruct
            }.to_string()
        }
    }
}
