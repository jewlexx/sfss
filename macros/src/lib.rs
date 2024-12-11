use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod hooks;

#[proc_macro_derive(Hooks, attributes(no_hook, hook_name, command_name))]
#[proc_macro_error]
pub fn derive_hook_enum(input: TokenStream) -> TokenStream {
    hooks::hook_enum(parse_macro_input!(input as DeriveInput)).into()
}
