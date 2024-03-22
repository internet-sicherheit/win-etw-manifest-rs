#[allow(unused)]
mod parser;
#[allow(unused)]
mod template;

use std::{
    fs::{read_dir, File},
    path::Path,
};

use proc_macro::TokenStream;
use quote::quote;

use parser::{parse, Provider};

#[proc_macro]
pub fn include_manifests(input: TokenStream) -> TokenStream {
    let input = match input.into_iter().next().unwrap() {
        proc_macro::TokenTree::Literal(x) => x.to_string(),
        _ => panic!("Input must be path literal"),
    };
    let str_path = input.trim_matches('"');
    let path = Path::new(&str_path);

    // read all files from path
    let files: Vec<_> = read_dir(path)
        .unwrap()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().is_ok_and(|x| x.is_file()))
        .map(|x| x.path())
        .collect();

    let xml_files: Vec<_> = files
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ex| ex == "xml"))
        .collect();

    let mut providers = Vec::new();

    for path in xml_files {
        if let Ok(mut f) = File::open(&path) {
            match parse(&mut f) {
                Ok(provider) => {
                    providers.push(provider);
                }
                Err(e) => {
                    eprintln!("Parsing of {path:?} failed: {e}");
                    // parsing failed
                    // TODO use experimental diagnostic api to issue a warning
                }
            }
        } else {
            eprintln!("Failed to open {path:?}");
            // opening of file failed
            // TODO use experimental diagnostic api to issue a warning
        }
    }
    create_quote(&providers).into()
}

fn create_quote(providers: &[Provider]) -> proc_macro2::TokenStream {
    let provider_structs = quote_provider_structs(providers);

    quote! {
        /// Event Providers
        ///
        /// Event providers which are generated from their instrumentation manifest.
        pub mod provider {
            use super::*;
            #provider_structs
        }
    }
}

fn quote_provider_structs(providers: &[Provider]) -> proc_macro2::TokenStream {
    let mut quotes: Vec<proc_macro2::TokenStream> = Vec::new();
    for p in providers {
        quotes.push(quote_provider_struct(p));
    }

    let (guid_idents, struct_idents): (Vec<_>, Vec<_>) = providers
        .iter()
        .map(|x| {
            (
                x.guid_constant_name(),
                proc_macro2::Ident::new(x.symbol.as_str(), proc_macro2::Span::call_site()),
            )
        })
        .unzip();

    quote! {
        impl ModernEvent {
            /// Try to wrap this opaque event in a concise event
            ///
            /// Returns None if no implementation for the provided event exists.
            pub fn into_contained_event(self) -> Option<Box<dyn Event>> {
                match self.header.provider_id {
                    #(#struct_idents::#guid_idents => Some(Box::new(#struct_idents::from(self))),)*
                    _ => None,
                }
            }
        }
        #(#quotes)*
    }
}
fn quote_provider_struct(p: &Provider) -> proc_macro2::TokenStream {
    let symbol = proc_macro2::Ident::new(p.symbol.as_str(), proc_macro2::Span::call_site());

    let guid = p.guid.to_string();
    let guid_const_name = p.guid_constant_name();

    let name = p.name.as_str();

    let (event_ids, event_tasks): (Vec<_>, Vec<_>) = p
        .events
        .iter()
        .map(|x| {
            (
                proc_macro2::Literal::u16_unsuffixed(x.value),
                x.task.as_str(),
            )
        })
        .unzip();

    let (unique_ids, event_symbol): (Vec<_>, Vec<_>) = p
        .events
        .iter()
        .map(|e| (e.identifier_tuple(), e.symbol.as_str()))
        .unzip();

    let (event_to_template, template_fn): (Vec<_>, Vec<_>) = p
        .events
        .iter()
        .filter(|e| e.template_function_ident().is_some())
        .map(|e| (e.identifier_tuple(), e.template_function_ident().unwrap()))
        .unzip();

    let templates = template::generate_templates(symbol.clone(), &p.templates);
    quote! {
        pub struct #symbol {
            modern_event: ModernEvent,
            payload: ::core::option::Option<::std::collections::HashMap<&'static str, crate::modern_event::types::WinInTypeItem>>,
        }
        impl #symbol {
            pub const #guid_const_name: Uuid = uuid!(#guid);
            /// Wraps a [ModernEvent] into this concise event
            ///
            /// _Warning:_ No checks are performed if the [ModernEvent] is of this event type
            fn from(value: ModernEvent) -> Self {
                Self { modern_event: value, payload: None }
            }
        }
        impl ::core::ops::Deref for #symbol {
            type Target = ModernEvent;
            fn deref(&self) -> &Self::Target {
                &self.modern_event
            }
        }
        impl ::core::ops::DerefMut for #symbol {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.modern_event
            }
        }
        impl ::core::convert::TryFrom<ModernEvent> for #symbol {
            type Error = &'static str;
            fn try_from(value: ModernEvent) -> ::core::result::Result<Self, Self::Error> {
                if matches!(value.header.provider_id, Self::#guid_const_name) {
                    Ok(Self { modern_event: value, payload: None })
                } else {
                    Err("GUID of event doesn't match")
                }

            }
        }
        impl Event for #symbol {
            fn get_provider_name(&self) -> &str {
                #name
            }
            fn get_event_task_name(&self) -> Option<&str> {
                match self.header.event_descriptor.id {
                    #(#event_ids => Some(#event_tasks),)*
                    _ => None,
                }
            }
            fn get_event_symbol(&self) -> Option<&str> {
                let ed = &self.header.event_descriptor;
                match (ed.id, ed.version) {
                    #(#unique_ids => Some(#event_symbol),)*
                    _ => None,
                }
            }
            fn get_payload_items(&mut self) -> Option<&HashMap<&'static str, crate::modern_event::types::WinInTypeItem>> {
                if self.payload.is_some() {
                    return self.payload.as_ref();
                }

                let ed = &self.header.event_descriptor;
                match (ed.id, ed.version) {
                    #(#event_to_template => {
                        let res = self.#template_fn();
                        if let Err(e) = res {
                            ::log::warn!("Parsing of payload items failed: {e}");
                        }
                    })*
                    _ => {}
                }
                self.payload.as_ref()
            }
        }
        #templates
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use crate::parser::parse;

    #[test]
    fn test_parser() {
        let mut f = File::open("test.xml").unwrap();

        let provider = parse(&mut f).unwrap();

        let mut f = File::create("test_result.txt").unwrap();
        write!(f, "{provider:#?}").unwrap();
    }
}
