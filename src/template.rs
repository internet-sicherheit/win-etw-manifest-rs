use std::{error::Error as StdError, fmt::Display, io::Read, str::FromStr};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use super::{find_attribute, Error};

/// A event payload template
///
/// Information about payload items, their data types and order.
///
/// [XML Schema Documentation](https://learn.microsoft.com/en-us/windows/win32/wes/eventmanifestschema-templateitemtype-complextype)
#[derive(Debug)]
#[non_exhaustive]
pub struct Template {
    /// Unique identifier of the template
    pub tid: String,
    /// Optional name of the Template
    pub name: Option<String>,
    /// Layout of the data this template describes
    pub data: Vec<DataType>,
}
impl Template {
    pub(crate) fn parse<R: Read>(
        r: &mut EventReader<R>,
        tid: String,
        t_name: Option<String>,
    ) -> Result<Template, Error> {
        let mut data = Vec::new();
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    if name.local_name != "data" {
                        return Err(Error::new_unexpected_tag(format!(
                            "Expected <data>, found {}",
                            name.local_name
                        )));
                    }
                    data.push(DataType::from_attributes(&attributes)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "template" {
                        return Ok(Template {
                            tid,
                            name: t_name,
                            data,
                        });
                    } else {
                        continue;
                    }
                }
                _ => return Err(Error::new_unexpected(None)),
            }
        }
    }
}

/// Description of data items
#[derive(Debug)]
#[non_exhaustive]
pub struct DataType {
    /// Name of the data item
    pub name: String,
    /// The `in_type` of the item
    pub in_type: WinInType,
    /// Optional name of the data item, which stores the lengh of this item
    ///
    /// Used for items with [WinInType::Binary].
    /// Items with [WinInType::Binary], which have a known size (an IPv6 address for example), might still be missing a length field.
    pub length: Option<String>,
}
impl DataType {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<DataType, Error> {
        let name = find_attribute(attr, "name")?;
        let in_type = find_attribute(attr, "inType")?.parse().map_err(|e| {
            Error::new(
                super::ErrorKind::TypeParseError,
                format!("Encountered unknown in-type: {:?}", e),
            )
        })?;
        let length = find_attribute(attr, "length").ok();
        if in_type == WinInType::Binary && length.is_none() {
            // eprintln!("proc-etw-manifest [Warning]: Found template with Binary data type but no length field, attributes {:?}", attr);
            return Err(Error::new(
                super::ErrorKind::MissingAttribute,
                "Template with Binary in-type but no length field".to_string(),
            ));
        }
        Ok(DataType {
            name,
            in_type,
            length,
        })
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
                        return Err(Error::new_unexpected_tag(format!(
                            "Expected <template>, found {}",
                            name.local_name
                        )));
                    }
                    let tid = find_attribute(&attributes, "tid")?;
                    let t_name = find_attribute(&attributes, "name").ok();
                    vec.push(Template::parse(r, tid, t_name)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "templates" {
                        return Ok(());
                    } else {
                        continue;
                    }
                }
                _ => return Err(Error::new_unexpected(None)),
            }
        }
    }
}

/// Windows InTypes
///
/// [XML Schema Documentation](https://learn.microsoft.com/en-us/windows/win32/wes/eventmanifestschema-inputtype-complextype)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WinInType {
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
pub struct ParseInTypeError(String);
impl StdError for ParseInTypeError {}

impl Display for ParseInTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "can't parse `{}` to a WinInType", self.0)
    }
}
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
            "win:AnsiChar" => unimplemented!(),
            "win:UnicodeChar" => unimplemented!(),
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

#[cfg(test)]
mod tests {

    use super::Template;
    use crate::xml_match_start;
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
}
