mod keyword;
mod node;

use node::Node;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro]
pub fn view(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Node);
    input.into_token_stream().into()
}
