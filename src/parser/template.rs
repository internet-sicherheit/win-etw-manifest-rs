use std::{io::Read, str::FromStr};

use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use super::{find_attribute, Error};

#[derive(Debug)]
pub(crate) struct Template {
    pub(crate) tid: String,
    pub(crate) data: Vec<DataType>,
}
impl Template {
    pub(crate) fn parse<R: Read>(r: &mut EventReader<R>, tid: String) -> Result<Template, Error> {
        let mut data = Vec::new();
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    if name.local_name != "data" {
                        return Err(Error::new_unexpected());
                    }
                    data.push(DataType::from_attributes(&attributes)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "template" {
                        return Ok(Template { tid, data });
                    } else {
                        continue;
                    }
                }
                _ => return Err(Error::new_unexpected()),
            }
        }
    }

    pub(crate) fn function_name(&self) -> proc_macro2::Ident {
        let mut name = self.tid.clone();

        if !name.is_ascii() {
            panic!(
                "Non ASCII template id \"{}\" found, which is not supported!",
                name
            );
        }
        make_function_name(&mut name);

        format_ident!("parse_payload_{name}")
    }
}

pub(crate) fn make_function_name(value: &mut String) {
    while let Some(i) = value.find(char::is_uppercase) {
        let x = &mut value[i..=i];
        x.make_ascii_lowercase();
        if i == 0 {
            continue;
        }
        if value[i - 1..].starts_with('_') {
            continue;
        }
        value.insert(i, '_');
    }
    while let Some(i) = value.find(' ') {
        value.remove(i);
    }
    while let Some(i) = value.find('.') {
        value.remove(i);
    }
    while let Some(i) = value.find(':') {
        value.remove(i);
    }
    while let Some(i) = value.find('/') {
        value.remove(i);
    }
    while let Some(i) = value.find('(') {
        value.remove(i);
    }
    while let Some(i) = value.find(')') {
        value.remove(i);
    }
}

#[derive(Debug)]
pub(crate) struct DataType {
    pub(crate) name: String,
    pub(crate) in_type: WinInType,
}
impl DataType {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<DataType, Error> {
        let name = find_attribute(attr, "name")?;
        let in_type = find_attribute(attr, "inType")?.parse().map_err(|e| {
            Error::new(
                super::ErrorKind::TypeParseError,
                "Encountered unknown in-type".to_owned(),
            )
        })?;
        Ok(DataType { name, in_type })
    }
    pub(crate) fn quote_parse_fn(&self) -> proc_macro2::TokenStream {
        let intype: WinInType = self.in_type;
        let intype_variant_ident: Ident = intype.into();
        quote! {
            read_payload_item(WinInType::#intype_variant_ident)
        }
    }
    pub(crate) fn name_literal(&self) -> proc_macro2::Literal {
        proc_macro2::Literal::string(&self.name)
    }
}
impl Template {
    pub(super) fn parse_templates<R: Read>(
        r: &mut EventReader<R>,
        vec: &mut Vec<Template>,
    ) -> Result<(), Error> {
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    if name.local_name != "template" {
                        return Err(Error::new_unexpected());
                    }
                    let tid = find_attribute(&attributes, "tid")?;
                    vec.push(Template::parse(r, tid)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "templates" {
                        return Ok(());
                    } else {
                        continue;
                    }
                }
                _ => return Err(Error::new_unexpected()),
            }
        }
    }
}

/// Windows InTypes
#[derive(Debug, Clone, Copy)]
pub(crate) enum WinInType {
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float,
    Double,
    Boolean,
    AnsiString,
    UnicodeString,
    Binary,
    Pointer,
    SizeT,
    Guid,
    Sid,
    Filetime,
    Systemtime,
}
#[derive(Debug, Clone)]
pub(crate) struct ParseInTypeError(String);
impl FromStr for WinInType {
    type Err = ParseInTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WinInType::*;
        match s {
            "win:Int8" => Ok(Int8),
            "win:UInt8" => Ok(UInt8),
            "win:Int16" => Ok(Int16),
            "win:UInt16" => Ok(UInt16),
            "win:Int32" => Ok(Int32),
            "win:UInt32" => Ok(UInt32),
            "win:HexInt32" => Ok(UInt32),
            "win:Int64" => Ok(Int64),
            "win:UInt64" => Ok(UInt64),
            "win:HexInt64" => Ok(UInt64),
            "win:Float" => Ok(Float),
            "win:Double" => Ok(Double),
            "win:Boolean" => Ok(Boolean),
            "win:AnsiChar" => todo!(),
            "win:UnicodeChar" => todo!(),
            "win:AnsiString" => Ok(AnsiString),
            "win:UnicodeString" => Ok(UnicodeString),
            "win:Binary" => Ok(Binary),
            "win:HexDump" => Ok(Binary),
            "win:Pointer" => Ok(Pointer),
            "win:SizeT" => Ok(SizeT),
            "win:GUID" => Ok(Guid),
            "win:SID" => Ok(Sid),
            "win:FILETIME" => Ok(Filetime),
            "win:SYSTEMTIME" => Ok(Systemtime),
            _ => Err(ParseInTypeError(s.to_owned())),
        }
    }
}
impl From<WinInType> for proc_macro2::Ident {
    fn from(value: WinInType) -> Self {
        match value {
            WinInType::Int8 => Ident::new("Int8", Span::call_site()),
            WinInType::UInt8 => Ident::new("UInt8", Span::call_site()),
            WinInType::Int16 => Ident::new("Int16", Span::call_site()),
            WinInType::UInt16 => Ident::new("UInt16", Span::call_site()),
            WinInType::Int32 => Ident::new("Int32", Span::call_site()),
            WinInType::UInt32 => Ident::new("UInt32", Span::call_site()),
            WinInType::Int64 => Ident::new("Int64", Span::call_site()),
            WinInType::UInt64 => Ident::new("UInt64", Span::call_site()),
            WinInType::Float => Ident::new("Float", Span::call_site()),
            WinInType::Double => Ident::new("Double", Span::call_site()),
            WinInType::Boolean => Ident::new("Boolean", Span::call_site()),
            WinInType::AnsiString => Ident::new("AnsiString", Span::call_site()),
            WinInType::UnicodeString => Ident::new("UnicodeString", Span::call_site()),
            WinInType::Binary => Ident::new("Binary", Span::call_site()),
            WinInType::Pointer => Ident::new("Pointer", Span::call_site()),
            WinInType::SizeT => Ident::new("SizeT", Span::call_site()),
            WinInType::Guid => Ident::new("Guid", Span::call_site()),
            WinInType::Sid => Ident::new("Sid", Span::call_site()),
            WinInType::Filetime => Ident::new("Filetime", Span::call_site()),
            WinInType::Systemtime => Ident::new("Systemtime", Span::call_site()),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::Template;
    use crate::parser::xml_match_start;
    use quote::quote;
    use xml::{EventReader, ParserConfig};

    #[test]
    fn test_parse_templates() {
        const XML: &str = r#"
        <templates>
          <template tid="ProcessStartArgs">
            <data name="ProcessID" inType="win:UInt32" />
            <data name="CreateTime" inType="win:FILETIME" />
            <data name="ParentProcessID" inType="win:UInt32" />
            <data name="SessionID" inType="win:UInt32" />
            <data name="ImageName" inType="win:UnicodeString" />
          </template>
          <template tid="ProcessStopArgs">
            <data name="ProcessID" inType="win:UInt32" />
            <data name="CreateTime" inType="win:FILETIME" />
            <data name="ExitTime" inType="win:FILETIME" />
            <data name="ExitCode" inType="win:UInt32" />
            <data name="TokenElevationType" inType="win:UInt32" />
            <data name="HandleCount" inType="win:UInt32" />
            <data name="CommitCharge" inType="win:UInt64" />
            <data name="CommitPeak" inType="win:UInt64" />
            <data name="ImageName" inType="win:AnsiString" />
          </template>
          <template tid="ThreadStartArgs">
            <data name="ProcessID" inType="win:UInt32" />
            <data name="ThreadID" inType="win:UInt32" />
            <data name="StackBase" inType="win:Pointer" />
            <data name="StackLimit" inType="win:Pointer" />
            <data name="UserStackBase" inType="win:Pointer" />
            <data name="UserStackLimit" inType="win:Pointer" />
            <data name="StartAddr" inType="win:Pointer" />
            <data name="Win32StartAddr" inType="win:Pointer" />
            <data name="TebBase" inType="win:Pointer" />
          </template>
        </templates>
        "#;
        let conf = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true);
        let mut reader = EventReader::new_with_config(XML.as_bytes(), conf);
        reader.next().unwrap();

        xml_match_start(&mut reader, "templates").unwrap();
        let mut templates = Vec::new();
        Template::parse_templates(&mut reader, &mut templates).unwrap();
        assert_eq!(templates.len(), 3, "Check if all templates were parsed");
    }
    #[test]
    fn test_make_function_name() {
        let mut name = "somethingThatsNotConvention".to_string();
        super::make_function_name(&mut name);
        assert_eq!(name, "something_thats_not_convention");

        let mut name = "StartsWithCapital".to_string();
        super::make_function_name(&mut name);
        assert_eq!(name, "starts_with_capital");

        let mut name = "already_has_Underscores".to_string();
        super::make_function_name(&mut name);
        assert_eq!(name, "already_has_underscores");

        let mut name = "has.Dot".to_string();
        super::make_function_name(&mut name);
        assert_eq!(name, "has_dot");

        let mut name = "has Space".to_string();
        super::make_function_name(&mut name);
        assert_eq!(name, "has_space");
    }
    #[test]
    fn test_quote_parse_fn() {
        let expected = quote! {
            read_payload_item(WinInType::Int8)
        };
        let data = super::DataType {
            name: "test".to_owned(),
            in_type: "win:Int8".parse().unwrap(),
        };

        let got = data.quote_parse_fn();
        println!("{got}");
        assert_eq!(format!("{got}"), format!("{expected}"));
    }
}
