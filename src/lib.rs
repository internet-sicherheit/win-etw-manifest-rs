mod parser;

use std::{
    fs::{read_dir, File},
    path::Path,
};

use proc_macro::TokenStream;
use quote::quote;

use parser::{parse, Provider};

#[proc_macro]
pub fn parse_manifest(input: TokenStream) -> TokenStream {
    let input = match input.into_iter().next().unwrap() {
        proc_macro::TokenTree::Literal(x) => x.to_string(),
        _ => panic!("Input must be path literal"),
    };
    let str_path = input.trim_matches('"');
    let path = Path::new(&str_path);
    println!("{path:?}");
    let mut f = File::open(path).unwrap();

    let _provider = parse(&mut f).unwrap();

    TokenStream::new()
}

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
        if let Ok(mut f) = File::open(path) {
            if let Ok(provider) = parse(&mut f) {
                providers.push(provider);
            } else {
                // parsing failed
                // TODO use experimental diagnostic api to issue a warning
            }
        } else {
            // opening of file failed
            // TODO use experimental diagnostic api to issue a warning
        }
    }

    create_quote(&providers).into()
}

fn create_quote(providers: &[Provider]) -> proc_macro2::TokenStream {
    let guid_consts = quote_guid_consts(providers);
    let impl_get_provider_name = quote_get_provider_name(providers);
    let provider_structs = quote_provider_structs(providers);

    quote! {
        #guid_consts
        #impl_get_provider_name
        #provider_structs
    }
}

fn quote_guid_consts(providers: &[Provider]) -> proc_macro2::TokenStream {
    let guid_tup = providers
        .iter()
        .map(|x| (x.guid_constant_name(), x.guid.to_string()));
    let (guid_idents, guid_literals): (Vec<_>, Vec<_>) = guid_tup.unzip();

    quote! {
        #(const #guid_idents: Uuid = uuid!(#guid_literals);)*
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
            pub fn get_event_task_name(&self) -> Option<&str> {
                match self.header.provider_id {
                    #(#guid_idents => #struct_idents::get_event_task_name(&self.header.event_descriptor),)*
                    _ => None,
                }
            }
            pub fn get_event_symbol(&self) -> Option<&str> {
                match self.header.provider_id {
                    #(#guid_idents => #struct_idents::get_event_symbol(&self.header.event_descriptor),)*
                    _ => None,
                }
            }
        }
        #(#quotes)*
    }
}
fn quote_provider_struct(p: &Provider) -> proc_macro2::TokenStream {
    let symbol = proc_macro2::Ident::new(p.symbol.as_str(), proc_macro2::Span::call_site());
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

    quote! {
        struct #symbol;
        impl #symbol {
            fn get_event_task_name(ed: &EventDescriptor) -> Option<&str> {
                match ed.id {
                    #(#event_ids => Some(#event_tasks),)*
                    _ => None,
                }
            }
            fn get_event_symbol(ed: &EventDescriptor) -> Option<&str> {
                match (ed.id, ed.version) {
                    #(#unique_ids => Some(#event_symbol),)*
                    _ => None,
                }
            }
        }
    }
}

fn quote_get_provider_name(providers: &[Provider]) -> proc_macro2::TokenStream {
    let guid_tup = providers
        .iter()
        .map(|x| (x.guid_constant_name(), x.name.as_str()));
    let (guid_idents, prov_names): (Vec<_>, Vec<_>) = guid_tup.unzip();

    quote! {
        impl ModernEvent {
            pub fn get_provider_name(&self) -> Option<&str> {
                match self.header.provider_id {
                    #(#guid_idents => Some(#prov_names),)*
                    _ => None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use crate::parser::parse;

    #[test]
    fn testrun() {
        let mut f = File::open("test.xml").unwrap();

        let provider = parse(&mut f).unwrap();

        let mut f = File::create("test_result.txt").unwrap();
        write!(f, "{provider:#?}").unwrap();
    }
}
