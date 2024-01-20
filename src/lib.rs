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
    let guid_tup = providers
        .iter()
        .map(|x| (x.guid_constant_name(), x.guid.to_string()));
    let (guid_idents, guid_literals): (Vec<_>, Vec<_>) = guid_tup.unzip();

    let names = providers.iter().map(|x| x.name.as_str());

    quote! {
        #(const #guid_idents: Uuid = uuid!(#guid_literals);)*

        impl ModernEvent {
            pub fn get_provider_name(&self) -> Option<&str> {
                match self.header.provider_id {
                    #(#guid_idents => Some(#names),)*
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
