use crate::parser::template::*;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(crate) fn generate_templates(provider: Ident, templates: &[Template]) -> TokenStream {
    let mut template_functions = Vec::new();
    for t in templates {
        let fn_name = t.function_name();
        let (parse_fns, names): (Vec<_>, Vec<_>) = t
            .data
            .iter()
            .map(|d| (d.quote_parse_fn(), d.name_literal()))
            .unzip();
        let ts = quote! {
            fn #fn_name(&mut self) -> ::std::io::Result<()> {
                use core::ops::DerefMut;
                let mut map: ::std::collections::HashMap<&str, WinOutType> = ::std::collections::HashMap::new();

                #(map.insert(#names, self.modern_event.#parse_fns?);)*
                self.payload = Some(map);
                Ok(())
            }
        };
        template_functions.push(ts);
    }
    quote! {
        impl #provider {
            #(#template_functions)*
        }
    }
}
