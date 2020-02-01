// about how to use proc-macro see this ref. and please update your rustc to version 1.30 at least.
// because the stable rust supports proc-macro begin 1.30.
// https://doc.rust-lang.org/reference/procedural-macros.html

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{ quote, ToTokens };

// remember to add 'full' feature for sys in toml file, 
use syn::{ parse_macro_input, NestedMeta, Meta, ItemFn, FnArg, AttributeArgs };


// remenber to add 
// [lib]
// proc-macro = true
// in your toml file
#[proc_macro_attribute]
pub fn login_required(_: TokenStream, func: TokenStream) -> TokenStream {
    // _ means there's no attribute here to pass and handle.
    let func = parse_macro_input!(func as ItemFn);
    let func_block = func.block;
    let func_vis = func.vis;
    
    let func_sig = func.sig;
    let func_name = func_sig.ident;
    let asyncness = func_sig.asyncness;
    let func_inputs = func_sig.inputs;
    let func_output = func_sig.output;
    let func_generics = func_sig.generics;
    
    let params: Vec<_> = func_inputs.iter().filter_map(|i| {
        match i {
            // https://docs.rs/syn/1.0.1/syn/struct.PatType.html
            FnArg::Typed(ref pat_type) => {
                let identity_type = pat_type.ty.clone().into_token_stream();
                let identity_type_name = identity_type.to_string();
                if identity_type_name.eq("Identity") {
                    Some((&pat_type.pat, identity_type))
                } else {
                    None
                }
            }
            _ => unreachable!("it's not gonna happen."),
        }
    }).collect();
    let (identity_param, identity_type) = params.get(0).unwrap();
    
    let caller = quote!{
        // rebuild the function, add a func named is_expired to check user login session expire or not.
        #func_vis #asyncness fn #func_name #func_generics(#func_inputs) #func_output {
            fn is_expired(#identity_param: &#identity_type) -> bool {
                if let Some(_) = #identity_param.identity() {
                    false
                } else {
                    true
                }
            }
            
            if is_expired(&#identity_param) {
                Err(ErrorKind::IdentityExpiredError)
            } else {
                #func_block
            }
        }
    };
    
    // build a TokenStream
    // https://docs.rs/quote/1.0.0/quote/macro.quote.html
    caller.into() 
}

// this proc-macro is not ergonomic to use, I have to define a function to receive
// a closure as parameter, but this closure has uncertain count of parameters.
// how to use, see branch 0.7, and I have made some improvements on this proc-macro.
#[proc_macro_attribute]
pub fn builtin_decorator(attr: TokenStream, func: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    // only on attribute here
    let attr_ident = match attr.get(0).as_ref().unwrap() {
        NestedMeta::Meta(Meta::Path(ref attr_ident)) => attr_ident.clone(),
        _ => unreachable!("it not gonna happen."),
    };
    
    let func = parse_macro_input!(func as ItemFn);
    let func_block = func.block;
    let func_vis = func.vis;
    
    let func_sig = func.sig;
    let func_name = func_sig.ident;
    let func_inputs = func_sig.inputs;
    let func_output = func_sig.output;
    let func_generics = func_sig.generics;

    let params: Vec<_> = func_inputs.iter().map(|i| {
        match i {
            // https://docs.rs/syn/1.0.1/syn/struct.PatType.html
            FnArg::Typed(ref pat_type) => &pat_type.pat, // cannot move val out of pat，use ref or val.pat.clone()
            _ => unreachable!("it's not gonna happen."),
        }
    }).collect();
    
    let caller = quote!{
        #func_vis fn #func_name #func_generics(#func_inputs) #func_output {
            fn rebuild_func #func_generics(#func_inputs) #func_output #func_block
            
            let f = #attr_ident(rebuild_func);

            // may have couple of parameters, #(#params,) *
            f(#(#params,) *)
        }
    };
    
    caller.into() 
}