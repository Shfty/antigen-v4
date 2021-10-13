use syn::{Ident, Type};

pub fn impl_register_resource(input: Type) -> proc_macro::TokenStream {
    let ty = if let Type::Path(ty) = input {
        ty
    } else {
        panic!("Supplied type must be in Path format")
    };

    let path = ty.path;
    let segment = path.segments.last().expect("Empty path");
    let ident = &segment.ident;

    let func_name = Ident::new(
        &("legion_debugger_register_".to_string() + &ident.to_string()),
        proc_macro2::Span::call_site(),
    );

    let tokens = quote::quote! {
        fn #func_name(resources: &legion::Resources) -> Result<reflection::data::Data, reflection::serializer::Error> {
            let resource = resources.get::<#ident>().unwrap_or_else(|| panic!("No resource of type {}", stringify!(#ident)));
            let resource = &*resource;
            reflection::to_data(resource, true)
        }

        plugin_registry::register!(#ident, legion_debugger::ResourceRegistrar, legion_debugger::ResourceRegistrar(#func_name));
    };

    tokens.into()
}
