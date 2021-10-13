use quote::quote;
use syn::Type;

pub fn impl_iter(input: Type) -> proc_macro::TokenStream {
    let ty = if let Type::Path(ty) = input {
        ty
    } else {
        panic!("Supplied type must be in Path format")
    };

    let path = ty.path;
    let segment = path.segments.last().expect("Empty path");
    let ident = &segment.ident;

    #[cfg(feature = "registry-linkme")]
    let ident_registry = syn::Ident::new(
        &("REGISTRY_".to_string() + &ident.to_string().to_uppercase()),
        proc_macro2::Span::call_site(),
    );

    #[cfg(feature = "registry-linkme")]
    let tokens = quote! {
        #ident_registry
    };

    #[cfg(feature = "registry-inventory")]
    let tokens = quote! {
        inventory::iter::<#ident>
    };

    tokens.into()
}
