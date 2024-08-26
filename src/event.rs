use std::io::Read;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use super::{find_attribute, Error, ErrorKind};

/// A event defined by a provider
#[derive(Debug)]
pub struct Event {
    /// Numerical identifier of the event
    pub value: u16,
    pub symbol: String,
    /// Version of the event
    pub version: u8,
    /// Task name corresponding to this event
    pub task: String,
    /// Optional opcode
    pub opcode: Option<String>,
    /// Trace level of the event (e.g. `win:Informational`)
    pub level: String,
    /// List of keywords (usually one)
    pub keywords: Option<String>,
    /// Template name
    pub template: Option<String>,
}

impl Event {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<Event, Error> {
        let value = find_attribute(attr, "value")?;
        let value: u16 = value.parse().map_err(|_| {
            Error::new(
                ErrorKind::TypeParseError,
                format!("found `{value}` expeced u16"),
            )
        })?;

        let symbol = find_attribute(attr, "symbol")?;
        let version = find_attribute(attr, "version")?;
        let version: u8 = version.parse().map_err(|_| {
            Error::new(
                ErrorKind::TypeParseError,
                format!("found `{value}` expected u8"),
            )
        })?;
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
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::xml_match_start;
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
