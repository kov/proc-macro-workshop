#![feature(proc_macro_diagnostic)]
extern crate proc_macro;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::visit_mut::{self, VisitMut};
use syn::{AttrStyle, ExprMatch, Item, ItemFn, Pat, Variant};
use syn::spanned::Spanned;

#[derive(Default)]
struct MatchFinder {
    error: Option<syn::Error>,
}

// BEGIN: shamelessly stolen from https://github.com/jonhoo/proc-macro-workshop/blob/master/sorted/src/lib.rs
fn path_as_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| format!("{}", quote! {#s}))
        .collect::<Vec<_>>()
        .join("::")
}

fn get_arm_path(arm: &syn::Pat) -> Option<syn::Path> {
    match *arm {
        syn::Pat::Ident(syn::PatIdent { ident: ref id, .. }) => Some(id.clone().into()),
        syn::Pat::Path(ref p) => Some(p.path.clone()),
        syn::Pat::Struct(ref s) => Some(s.path.clone()),
        syn::Pat::TupleStruct(ref s) => Some(s.path.clone()),
        _ => None,
    }
}
// END: shamelessly stolen from https://github.com/jonhoo/proc-macro-workshop/blob/master/sorted/src/lib.rs

impl VisitMut for MatchFinder {
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        if let Some(position) = node.attrs.iter()
            .position(|attr| attr.style == AttrStyle::Outer && attr.path.is_ident("sorted"))
        {
            let _ = node.attrs.remove(position);

            let mut previous_ident = String::new();
            let mut badly_sorted: Option<&Pat> = None;
            let mut badly_sorted_ident = String::new();

            // Identify what is badly sorted.
            for a in &node.arms {
                let path = get_arm_path(&a.pat).unwrap();
                let ident = path_as_string(&path);
                if previous_ident != "" && previous_ident > ident {
                    badly_sorted = Some(&a.pat);
                    badly_sorted_ident = ident;
                    break;
                }
                previous_ident = ident;
            }

            // Figure out where it should go and report the error.
            if let Some(badly_sorted) = badly_sorted {
                for a in &node.arms {
                    let path = get_arm_path(&a.pat).unwrap();
                    let ident = path_as_string(&path);
                    if ident > badly_sorted_ident {
                        let bad_path = get_arm_path(badly_sorted).unwrap();
                        self.error = Some(syn::Error::new(bad_path.span(), format!("{} should sort before {}", badly_sorted_ident, ident)));
                        break;
                    }
                }
            }
        }

        // Delegate to the default impl to visit nested nodeessions.
        visit_mut::visit_expr_match_mut(self, node);
    }
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemFn);

    let mut match_finder = MatchFinder::default();
    match_finder.visit_item_fn_mut(&mut item);

    let mut result = quote! {#item};
    if let Some(error) = match_finder.error {
        result.extend(error.to_compile_error());
    }
    result.into()
}

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
                    return syn::Error::new(badly_sorted.ident.span(), format!("{} should sort before {}", badly_sorted.ident.to_string(), ident)).to_compile_error().into();
                }
            }
        }
    } else {
        return syn::Error::new(Span::call_site(), "expected enum or match expression").to_compile_error().into();
    }
    result
}
