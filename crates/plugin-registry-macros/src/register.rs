use quote::quote;
use syn::{Ident, Type};

#[allow(dead_code)]
pub struct Args {
    ident: Ident,
    _comma_first: syn::Token![,],
    ty: Type,
    _comma_second: syn::Token![,],
    value: syn::Expr,
}

impl syn::parse::Parse for Args {
    fn parse(stream: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let ident = stream.parse()?;
        let _comma_first = stream.parse()?;
        let ty = stream.parse()?;
        let _comma_second = stream.parse()?;
        let value = stream.parse()?;
        Ok(Args {
            ident,
            _comma_first,
            ty,
            _comma_second,
            value,
        })
    }
}

pub fn impl_register(input: Args) -> proc_macro::TokenStream {
    #[cfg(feature = "registry-linkme")]
    let tokens = {
        let Args {
            ident, ty, value, ..
        } = input;

        let ty = if let Type::Path(ty) = ty {
            ty
        } else {
            panic!("Supplied type must be in Path format")
        };

        let path = &ty.path;
        let segment = path.segments.last().expect("Empty path");
        let ty_ident = &segment.ident;

        let ident_upper = Ident::new(
            &ident.to_string().to_uppercase(),
            proc_macro2::Span::call_site(),
        );

        let ident_registry = Ident::new(
            &("REGISTRY_".to_string() + &ty_ident.to_string().to_uppercase()),
            proc_macro2::Span::call_site(),
        );

        let path_registry = {
            let mut path_registry = path.clone();
            path_registry.segments.last_mut().unwrap().ident = ident_registry;
            path_registry
        };

        quote! {
            #[linkme::distributed_slice(#path_registry)]
            static #ident_upper: #ty = #value;
        }
    };

    #[cfg(feature = "registry-inventory")]
    let tokens = {
        let value = input.value;

        quote! {
            inventory::submit! {
                #value
            }
        }
    };

    tokens.into()
}
