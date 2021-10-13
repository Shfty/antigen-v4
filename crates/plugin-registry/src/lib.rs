/// Plugin registry
///
/// This acts as a thin wrapper around the linkme and inventory crates, similar to how the
/// profiling crate wraps tracing, superluminal-perf and other runtime profiling crates

mod init;
mod register;
mod iter;

#[proc_macro]
pub fn init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    init::impl_init(input)
}

#[proc_macro]
pub fn register(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    register::impl_register(input)
}

#[proc_macro]
pub fn iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    iter::impl_iter(input)
}

