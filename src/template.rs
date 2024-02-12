use crate::parser::template::*;
use proc_macro2::TokenStream;

pub(crate) fn generate_templates(templates: &[Template]) -> TokenStream {
    for t in templates {
        let fn_name = t.function_name();
    }
    todo!()
}
