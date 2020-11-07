use syn::spanned::Spanned as _;

struct AssertExpr {
    type_trait_object: syn::TypeTraitObject,
    expr: syn::Expr,
}
impl syn::parse::Parse for AssertExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_trait_object = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let expr = input.parse()?;
        input.parse::<Option<syn::Token![,]>>()?;
        Ok(AssertExpr {
            type_trait_object,
            expr,
        })
    }
}

#[proc_macro]
pub fn assert_trait(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as AssertExpr);

    let mut lifetimes = std::collections::HashSet::new();
    for bound in &input.type_trait_object.bounds {
        collect_lifetimes_in_type_param_bound(&mut lifetimes, bound);
    }

    let type_trait_object = input.type_trait_object;
    let mut params = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    for lifetime in lifetimes {
        params.push(syn::GenericParam::Lifetime(syn::LifetimeDef {
            attrs: Vec::new(),
            lifetime,
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
        }));
    }
    params.push(syn::GenericParam::Type(syn::TypeParam {
        attrs: Vec::new(),
        ident: syn::Ident::new("T", type_trait_object.span()),
        colon_token: None,
        bounds: syn::punctuated::Punctuated::new(),
        eq_token: None,
        default: None,
    }));
    let expr = input.expr;
    let expanded = quote::quote! {{
        fn assert<#params>(x: T) -> T where T: #type_trait_object { x }
        assert(#expr)
    }};
    proc_macro::TokenStream::from(expanded)
}

fn collect_lifetimes_in_type_param_bound(
    lifetimes: &mut std::collections::HashSet<syn::Lifetime>,
    bound: &syn::TypeParamBound,
) {
    match bound {
        syn::TypeParamBound::Lifetime(lifetime) => {
            lifetimes.insert(lifetime.clone());
        }
        syn::TypeParamBound::Trait(trait_bound) => {
            collect_lifetimes_in_path(lifetimes, &trait_bound.path);
        }
    }
}

fn collect_lifetimes_in_path(
    lifetimes: &mut std::collections::HashSet<syn::Lifetime>,
    path: &syn::Path,
) {
    for segment in &path.segments {
        match &segment.arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                for arg in &angle_bracketed_generic_arguments.args {
                    match arg {
                        syn::GenericArgument::Lifetime(lifetime) => {
                            lifetimes.insert(lifetime.clone());
                        }
                        syn::GenericArgument::Type(ty) => {
                            collect_lifetimes_in_type(lifetimes, &ty);
                        }
                        syn::GenericArgument::Binding(binding) => {
                            collect_lifetimes_in_type(lifetimes, &binding.ty);
                        }
                        syn::GenericArgument::Constraint(_) => {
                            todo!();
                        }
                        syn::GenericArgument::Const(_) => {
                            todo!();
                        }
                    }
                }
            }
            syn::PathArguments::Parenthesized(_) => {
                todo!();
            }
        }
    }
}

fn collect_lifetimes_in_type(
    lifetimes: &mut std::collections::HashSet<syn::Lifetime>,
    ty: &syn::Type,
) {
    match ty {
        syn::Type::Path(type_path) => {
            collect_lifetimes_in_path(lifetimes, &type_path.path);
        }
        syn::Type::Reference(type_reference) => {
            if let Some(ref lifetime) = type_reference.lifetime {
                lifetimes.insert(lifetime.clone());
            }
        }
        _ => {
            todo!();
        }
    }
}
