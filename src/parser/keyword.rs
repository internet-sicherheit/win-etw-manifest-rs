use std::io::Read;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use super::{find_attribute, Error, ErrorKind};

#[derive(Debug)]
pub(crate) struct Keyword {
    pub(crate) name: String,
    pub(crate) mask: u64,
}

impl Keyword {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<Keyword, Error> {
        let name = find_attribute(attr, "name")?;
        let mask_str = find_attribute(attr, "mask")?;
        let hex_str = mask_str
            .strip_prefix("0x")
            .ok_or(Error::new_with_kind(ErrorKind::TypeParseError))?;
        let mask: u64 = u64::from_str_radix(hex_str, 16)
            .map_err(|_| Error::new_with_kind(ErrorKind::TypeParseError))?;
        Ok(Keyword { name, mask })
    }

    pub(super) fn parse_keywords<R: Read>(
        r: &mut EventReader<R>,
        vec: &mut Vec<Keyword>,
    ) -> Result<(), Error> {
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    if name.local_name != "keyword" {
                        return Err(Error::new_unexpected_tag(format!(
                            "Expected <keyword>, found {}",
                            name.local_name
                        )));
                    }
                    vec.push(Keyword::from_attributes(&attributes)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "keywords" {
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

#[cfg(test)]
mod tests {
    use crate::parser::{keyword::Keyword, xml_match_start};
    use xml::{EventReader, ParserConfig};

    #[test]
    fn test_parse_keywords() {
        const XML: &str = r#"
        <keywords>
          <keyword name="WINEVENT_KEYWORD_PROCESS" message="$(string.keyword_WINEVENT_KEYWORD_PROCESS)" mask="0x10" />
          <keyword name="WINEVENT_KEYWORD_THREAD" message="$(string.keyword_WINEVENT_KEYWORD_THREAD)" mask="0x20" />
          <keyword name="WINEVENT_KEYWORD_IMAGE" message="$(string.keyword_WINEVENT_KEYWORD_IMAGE)" mask="0x40" />
          <keyword name="WINEVENT_KEYWORD_CPU_PRIORITY" message="$(string.keyword_WINEVENT_KEYWORD_CPU_PRIORITY)" mask="0x80" />
          <keyword name="WINEVENT_KEYWORD_OTHER_PRIORITY" message="$(string.keyword_WINEVENT_KEYWORD_OTHER_PRIORITY)" mask="0x100" />
          <keyword name="WINEVENT_KEYWORD_PROCESS_FREEZE" message="$(string.keyword_WINEVENT_KEYWORD_PROCESS_FREEZE)" mask="0x200" />
          <keyword name="WINEVENT_KEYWORD_JOB" message="$(string.keyword_WINEVENT_KEYWORD_JOB)" mask="0x400" />
          <keyword name="WINEVENT_KEYWORD_ENABLE_PROCESS_TRACING_CALLBACKS" message="$(string.keyword_WINEVENT_KEYWORD_ENABLE_PROCESS_TRACING_CALLBACKS)" mask="0x800" />
          <keyword name="WINEVENT_KEYWORD_JOB_IO" message="$(string.keyword_WINEVENT_KEYWORD_JOB_IO)" mask="0x1000" />
          <keyword name="WINEVENT_KEYWORD_WORK_ON_BEHALF" message="$(string.keyword_WINEVENT_KEYWORD_WORK_ON_BEHALF)" mask="0x2000" />
          <keyword name="WINEVENT_KEYWORD_JOB_SILO" message="$(string.keyword_WINEVENT_KEYWORD_JOB_SILO)" mask="0x4000" />
        </keywords>
        "#;
        let conf = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true);
        let mut reader = EventReader::new_with_config(XML.as_bytes(), conf);
        reader.next().unwrap();
        xml_match_start(&mut reader, "keywords").unwrap();
        let mut keywords = Vec::new();
        Keyword::parse_keywords(&mut reader, &mut keywords).unwrap();
        assert_eq!(keywords.len(), 11);
    }
}
