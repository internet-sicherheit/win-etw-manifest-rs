use std::io::Read;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use super::{find_attribute, Error, ErrorKind};

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
}

#[derive(Debug)]
pub(crate) struct DataType {
    pub(crate) name: String,
    pub(crate) in_type: String,
}
impl DataType {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<DataType, Error> {
        let name = find_attribute(attr, "name")?;
        let in_type = find_attribute(attr, "inType")?;
        Ok(DataType { name, in_type })
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

#[cfg(test)]
mod tests {
    use super::Template;
    use crate::parser::xml_match_start;
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
