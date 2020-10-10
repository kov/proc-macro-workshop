#![feature(proc_macro_diagnostic)]
extern crate proc_macro;

#[macro_use]
extern crate syn;

use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::{Error, Item};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let result = input.clone();
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Enum(_) => (),
        _ => return Error::new(Span::call_site(), "expected enum or match expression").to_compile_error().into(),
    }
    result
}
