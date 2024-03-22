use crate::parser::template::*;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(crate) fn generate_templates(provider: Ident, templates: &[Template]) -> TokenStream {
    let mut template_functions = Vec::new();
    for t in templates {
        let fn_name = t.function_name();
        let (parse_arg1, names): (Vec<Ident>, Vec<_>) = t
            .data
            .iter()
            .map(|d| (d.in_type.into(), d.name_literal()))
            .unzip();
        let binary_sizes: Vec<_> = t
            .data
            .iter()
            .map(|d| match &d.length {
                Some(l) => quote! {
                    {
                        let x: &crate::modern_event::types::WinInTypeItem = map.get(#l).ok_or(::std::io::Error::other("length to parse binary data not found"))?;
                        match x {
                            crate::modern_event::types::WinInTypeItem::Int8(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::UInt8(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::Int16(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::UInt16(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::Int32(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::UInt32(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::Int64(x) => ::core::option::Option::Some(*x as u16),
                            crate::modern_event::types::WinInTypeItem::UInt64(x) => ::core::option::Option::Some(*x as u16),
                            _ => ::core::option::Option::None
                        }
                    }
                },
                None => quote!(None),
            })
            .collect();
        let ts = quote! {
            fn #fn_name(&mut self) -> ::std::io::Result<()> {
                use ::core::ops::DerefMut;
                let mut map: ::std::collections::HashMap<&str, crate::modern_event::types::WinInTypeItem> = ::std::collections::HashMap::new();

                #(
                    // let size: ::core::option::Option<u16> = #binary_sizes;
                    let size: ::core::option::Option<u16> = #binary_sizes;
                    map.insert(#names, self.modern_event.read_payload_item(crate::modern_event::WinInType::#parse_arg1, size)?);
                )*
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
