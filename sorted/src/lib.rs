#![feature(proc_macro_diagnostic)]
extern crate proc_macro;

#[macro_use]
extern crate syn;

use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::{Error, Item, Variant};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let result = input.clone();
    let item = parse_macro_input!(input as Item);
    if let Item::Enum(item) = item {
        let mut previous_ident = String::new();
        let mut badly_sorted: Option<&Variant> = None;

        // Identify what is badly sorted.
        for v in &item.variants {
            let ident = v.ident.to_string();
            if previous_ident != "" && previous_ident > ident {
                badly_sorted = Some(v);
                break;
            }
            previous_ident = ident;
        }

        // Figure out where it should go and report the error.
        if let Some(badly_sorted) = badly_sorted {
            for v in &item.variants {
                let ident = v.ident.to_string();
                if ident > badly_sorted.ident.to_string() {
                    return Error::new(badly_sorted.ident.span(), format!("{} should sort before {}", badly_sorted.ident.to_string(), ident)).to_compile_error().into();
                }
            }
        }
    } else {
        return Error::new(Span::call_site(), "expected enum or match expression").to_compile_error().into();
    }
    result
}
