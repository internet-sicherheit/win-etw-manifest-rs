use crate::parser::template::*;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_templates(templates: &[Template]) -> TokenStream {
    let mut template_functions = Vec::new();
    for t in templates {
        let fn_name = t.function_name();
        let (parse_fns, names): (Vec<_>, Vec<_>) = t
            .data
            .iter()
            .map(|d| (d.quote_parse_fn(), d.name_literal()))
            .unzip();
        let ts = quote! {
            fn #fn_name(&self) {
                todo!()
            }
        };
        template_functions.push(ts);
    }
    todo!()
}
