mod parser;

use std::{
    fs::{read_dir, DirEntry, File},
    path::Path,
};

use proc_macro::TokenStream;

use parser::parse;

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
    let _files: Vec<_> = read_dir(path)
        .unwrap()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().is_ok_and(|x| x.is_file()))
        .map(|x| x.path())
        .collect();

    todo!()
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
