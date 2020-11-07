use syn::spanned::Spanned as _;
struct AssertExpr {
    generics: Option<syn::Generics>,
    type_trait_object: syn::TypeTraitObject,
    expr: syn::Expr,
}
impl syn::parse::Parse for AssertExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let generics = if input.peek(syn::Token![<]) {
            let g = Some(input.parse()?);
            input.parse::<syn::Token![,]>()?;
            g
        } else {
            None
        };
        let type_trait_object = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let expr = input.parse()?;
        input.parse::<Option<syn::Token![,]>>()?;
        Ok(AssertExpr {
            generics,
            type_trait_object,
            expr,
        })
    }
}

fn build_assert_expr(assert_expr: AssertExpr) -> syn::export::TokenStream2 {
    let type_trait_object = assert_expr.type_trait_object;
    let expr = assert_expr.expr;
    if let Some(mut generics) = assert_expr.generics {
        generics
            .params
            .push(syn::GenericParam::Type(syn::TypeParam {
                attrs: Vec::new(),
                ident: syn::Ident::new("T", type_trait_object.span()),
                colon_token: None,
                bounds: syn::punctuated::Punctuated::new(),
                eq_token: None,
                default: None,
            }));
        quote::quote! {
            ({
                fn assert#generics(x: T) -> T where T: #type_trait_object { x }
                assert
            })(#expr)
        }
    } else {
        quote::quote! {
            ({
                fn assert<T>(x: T) -> T where T: #type_trait_object { x }
                assert
            })(#expr)
        }
    }
}

#[proc_macro]
pub fn assert_trait(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as AssertExpr);
    let expanded = build_assert_expr(input);
    proc_macro::TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn assert_trait() {
        let tokens = quote::quote!(Iterator<Item = char>, "abc".chars());
        let input = syn::parse2(tokens).unwrap();
        let actual = super::build_assert_expr(input);
        let expected = quote::quote! {
            ({
                fn assert<T>(x: T) -> T where T: Iterator<Item = char> { x }
                assert
            })("abc".chars())
        };
        assert_eq!(format!("{}", expected), format!("{}", actual));
    }

    #[test]
    fn assert_trait_with_lifetime() {
        let tokens = quote::quote!(<'a>, Iterator<Item=&'a str>, "a\nb\nc".lines());
        let input = syn::parse2(tokens).unwrap();
        let actual = super::build_assert_expr(input);
        let expected = quote::quote! {
            ({
                fn assert<'a, T>(x: T) -> T where T: Iterator<Item = &'a str> { x }
                assert
            })("a\nb\nc".lines())
        };
        assert_eq!(format!("{}", expected), format!("{}", actual));
    }
}
