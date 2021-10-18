use syn::{Ident, Type};

pub fn impl_register_component(input: Type) -> proc_macro::TokenStream {
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
        fn #func_name(registry: &mut legion_debugger::legion::serialize::Registry<String>) {
            registry.register::<#ident>(stringify!(#ident).to_string());
        }

        use legion_debugger::plugin_registry::*;
        register!(#ident, legion_debugger::ComponentRegistrar, legion_debugger::ComponentRegistrar(#func_name));
    };

    tokens.into()
}
