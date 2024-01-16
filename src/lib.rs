mod parser;

use std::{fs::File, path::Path};

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
