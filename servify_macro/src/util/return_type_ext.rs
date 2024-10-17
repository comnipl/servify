use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{ReturnType, Type, TypeTuple};

#[allow(dead_code)]
pub(crate) trait ReturnTypeExt {
    fn to_type(self) -> Type;
}

impl ReturnTypeExt for ReturnType {
    fn to_type(self) -> Type {
        match self {
            ReturnType::Default => Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
                elems: Punctuated::new(),
            }),
            ReturnType::Type(_, t) => *t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::{parse2, ItemFn};

    #[test]
    fn test_default_type() {
        let func: ItemFn = parse2(quote! {
            fn some_function() {}
        })
        .unwrap();

        let return_type = func.sig.output.to_type();
        assert_eq!(quote!(#return_type).to_string(), quote!(()).to_string());
    }

    #[test]
    fn test_ident_type() {
        let func: ItemFn = parse2(quote! {
            fn some_function() -> i32 {}
        })
        .unwrap();

        let return_type = func.sig.output.to_type();
        assert_eq!(quote!(#return_type).to_string(), quote!(i32).to_string());
    }

    #[test]
    fn test_path_type() {
        let func: ItemFn = parse2(quote! {
            fn some_function() -> std::string::String {}
        })
        .unwrap();

        let return_type = func.sig.output.to_type();
        assert_eq!(
            quote!(#return_type).to_string(),
            quote!(std::string::String).to_string()
        );
    }
}
