use std::io::Read;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use super::{find_attribute, template::make_function_name, Error, ErrorKind};

#[derive(Debug)]
pub(crate) struct Event {
    pub(crate) value: u16,
    pub(crate) symbol: String,
    pub(crate) version: u8,
    pub(crate) task: String,
    pub(crate) opcode: Option<String>,
    pub(crate) level: String,
    pub(crate) keywords: Option<String>,
    pub(crate) template: Option<String>,
}

impl Event {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<Event, Error> {
        let value = find_attribute(attr, "value")?;
        let value: u16 = value
            .parse()
            .map_err(|_| Error::new_with_kind(ErrorKind::TypeParseError))?;

        let symbol = find_attribute(attr, "symbol")?;
        let version = find_attribute(attr, "version")?;
        let version: u8 = version
            .parse()
            .map_err(|_| Error::new_with_kind(ErrorKind::TypeParseError))?;
        let task = find_attribute(attr, "task")?;
        let opcode = find_attribute(attr, "opcode").ok();
        let level = find_attribute(attr, "level")?;
        let keywords = find_attribute(attr, "keywords").ok();
        let template = find_attribute(attr, "template").ok();

        Ok(Event {
            value,
            symbol,
            version,
            task,
            opcode,
            level,
            keywords,
            template,
        })
    }

    pub(super) fn parse_events<R: Read>(
        r: &mut EventReader<R>,
        vec: &mut Vec<Event>,
    ) -> Result<(), Error> {
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    if name.local_name != "event" {
                        return Err(Error::new_unexpected_tag(format!(
                            "Expected <event>, found {}",
                            name.local_name
                        )));
                    }
                    vec.push(Event::from_attributes(&attributes)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "events" {
                        return Ok(());
                    } else {
                        continue;
                    }
                }
                _ => {
                    return Err(Error::new_unexpected(Some(
                        "Unexpected element in <event>".to_string(),
                    )))
                }
            }
        }
    }

    pub(crate) fn identifier_tuple(&self) -> proc_macro2::TokenStream {
        use quote::quote;
        let id = self.value;
        let version = self.version;
        quote! {
            (#id, #version)
        }
    }
    pub(crate) fn template_function_ident(&self) -> Option<proc_macro2::Ident> {
        let mut name = self.template.clone()?;

        if !name.is_ascii() {
            panic!(
                "Non ASCII template id \"{}\" found, which is not supported!",
                name
            );
        }
        make_function_name(&mut name);

        Some(quote::format_ident!("parse_payload_{name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::parser::xml_match_start;
    use xml::{EventReader, ParserConfig};

    #[test]
    fn test_parse_events() {
        const XML: &str = r#"
        <events>
          <event value="1" symbol="ProcessStart" version="0" task="ProcessStart" opcode="win:Start" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStartArgs" />
          <event value="1" symbol="ProcessStart_V1" version="1" task="ProcessStart" opcode="win:Start" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStartArgs_V1" />
          <event value="1" symbol="ProcessStart_V2" version="2" task="ProcessStart" opcode="win:Start" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStartArgs_V2" />
          <event value="1" symbol="ProcessStart_V3" version="3" task="ProcessStart" opcode="win:Start" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessRundownArgs_V1" />
          <event value="2" symbol="ProcessStop" version="0" task="ProcessStop" opcode="win:Stop" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStopArgs" />
          <event value="2" symbol="ProcessStop_V1" version="1" task="ProcessStop" opcode="win:Stop" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStopArgs_V1" />
          <event value="2" symbol="ProcessStop_V2" version="2" task="ProcessStop" opcode="win:Stop" level="win:Informational" keywords="WINEVENT_KEYWORD_PROCESS" template="ProcessStopArgs_V2" />
          <event value="3" symbol="ThreadStart" version="0" task="ThreadStart" opcode="win:Start" level="win:Informational" keywords="WINEVENT_KEYWORD_THREAD" template="ThreadStartArgs" />
          <event value="3" symbol="ThreadStart_V1" version="1" task="ThreadStart" opcode="win:Start" level="win:Informational" keywords="WINEVENT_KEYWORD_THREAD" template="ThreadStartArgs_V1" />
          <event value="4" symbol="ThreadStop" version="0" task="ThreadStop" opcode="win:Stop" level="win:Informational" keywords="WINEVENT_KEYWORD_THREAD" template="ThreadStartArgs" />
        </events>
        "#;
        let conf = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true);
        let mut reader = EventReader::new_with_config(XML.as_bytes(), conf);
        reader.next().unwrap();
        xml_match_start(&mut reader, "events").unwrap();
        let mut tasks = Vec::new();
        Event::parse_events(&mut reader, &mut tasks).unwrap();
        assert_eq!(tasks.len(), 10, "Check if all events were parsed");
    }
}
