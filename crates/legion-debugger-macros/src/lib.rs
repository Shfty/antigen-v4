/// Legion Debugger Macros
///
/// Provides procedural macros to simplify registration of debuggable legion components and resources

mod register_component;
mod register_resource;

#[proc_macro]
pub fn register_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    register_component::impl_register_component(input)
}

#[proc_macro]
pub fn register_resource(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    register_resource::impl_register_resource(input)
}
