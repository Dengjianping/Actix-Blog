// about how to use proc-macro see this ref. and please update your rustc to version 1.30 at least.
// because the stable rust supports proc-macro begin 1.30.
// https://doc.rust-lang.org/reference/procedural-macros.html

extern crate proc_macro;
// extern crate proc_macro2;
// extern crate syn;
// #[macro_use] extern crate quote;


// https://doc.rust-lang.org/1.30.0/proc_macro/
use proc_macro::TokenStream;
use quote::quote;
// use proc_macro2::TokenStream;

// remember to add 'full' feature for sys in toml file, 
// or you'll get error like "unresolved imports `syn::ItemFn`, `syn::FnDecl`, `syn::FnArg`""
use syn::{ /*DeriveInput,*/ ItemFn, FnDecl, FnArg, Ident }; 
// MacroInput is obsoloted, use DeriveInput
// https://github.com/mystor/book/commit/94946f20ce3eda2a374e48f8e8fa27c4deda21b6


// remenber to add 
// [lib]
// proc-macro = true
// in your toml file
#[proc_macro_attribute]
pub fn builtin_decorator(attr: TokenStream, func: TokenStream) -> TokenStream { 
    // cannot use proc_macro2::TokenStream now
    // let func = func.into();
    let attr = parse_attr(attr);

    // https://docs.rs/syn/0.15.21/syn/struct.ItemFn.html
    // let item_fn: ItemFn = syn::parse(func).expect("Input is not a function");
    let item_fn: ItemFn = syn::parse(func).expect("Input is not a function");
    let vis = &item_fn.vis; // like pub
    let ident = &item_fn.ident; // variable or function name ,like x, add_method
    let block = &item_fn.block; // { some statement or expression here }

    // https://docs.rs/syn/0.15.21/syn/struct.FnDecl.html
    // Header of a function declaration, without including the body.
    let decl: FnDecl = *item_fn.decl; 
    let inputs = &decl.inputs;
    let output = &decl.output;

    let input_values: Vec<_> = inputs
        .iter()
        .map(|arg| match arg {
            // https://docs.rs/syn/0.15.21/syn/enum.FnArg.html#variant.Captured
            &FnArg::Captured(ref val) => &val.pat,
            _ => unreachable!(""),
        })
        .collect();

    let caller = quote!{
        #vis fn #ident(#inputs) #output {
            let f = #attr(deco_internal);
            return f(#(#input_values,) *);

            fn deco_internal(#inputs) #output #block
        }
    };
    // build a TokenStream
    // https://docs.rs/quote/0.6.10/quote/macro.quote.html
    caller.into() 
}

fn parse_attr(attr: TokenStream) -> Ident {
    // let pat: &[_] = &['"', '(', ')', ' ']; // the same effect as the comming line
    let pat: &[_] = &['=', ' ', '"'][..];
    let s = attr.to_string().trim_matches(pat).to_string();
    // use proc_macro2 Idet, quote doesn't implement for proc_macro Ident
    proc_macro2::Ident::new(&s, proc_macro2::Span::call_site()) 
}