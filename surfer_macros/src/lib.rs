extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn surfer_launch(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Get the components of the input function
    let name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let body = &input.block;

    // Generate the new function
    let expanded = quote! {
        #[async_std::main]
        async fn #name(#inputs) #output {
            #body
        }
    };
    expanded.into()
}