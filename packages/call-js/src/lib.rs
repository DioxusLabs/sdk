use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, ExprCall, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct CallJsInput {
    asset_path: LitStr,
    function_call: ExprCall,
}

impl Parse for CallJsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let asset_path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let function_call: ExprCall = input.parse()?;

        Ok(CallJsInput {
            asset_path,
            function_call,
        })
    }
}

fn extract_function_name(call: &ExprCall) -> Result<String> {
    match &*call.func {
        Expr::Path(path) => {
            if let Some(ident) = path.path.get_ident() {
                Ok(ident.to_string())
            } else {
                Err(syn::Error::new_spanned(
                    &path.path,
                    "Function call must be a simple identifier",
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            &call.func,
            "Function call must be a simple identifier",
        )),
    }
}

#[proc_macro]
pub fn call_js(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CallJsInput);

    let asset_path = &input.asset_path;
    let function_call = &input.function_call;

    let function_name = match extract_function_name(function_call) {
        Ok(name) => name,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    let args: Vec<&Expr> = function_call.args.iter().collect();
    let arg_count = args.len();

    let send_calls: Vec<TokenStream2> = args
        .iter()
        .map(|arg| quote! { eval.send(#arg)?; })
        .collect();

    let mut js_format = format!(r#"const {{{{ {function_name} }}}} = await import("{{}}");"#,);
    for i in 0..arg_count {
        js_format.push_str(&format!("\nlet arg{} = await dioxus.recv();", i));
    }
    js_format.push_str(&format!("\nreturn {}(", function_name));
    for i in 0..arg_count {
        if i > 0 {
            js_format.push_str(", ");
        }
        js_format.push_str(&format!("arg{}", i));
    }
    js_format.push_str(");");

    let expanded = quote! {
        async move {
            const MODULE: Asset = asset!(#asset_path);
            let js = format!(#js_format, MODULE);
            let eval = document::eval(js.as_str());
            #(#send_calls)*
            eval.await
        }.await
    };

    TokenStream::from(expanded)
}
