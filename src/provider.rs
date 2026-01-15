use std::io::Read;

use crate::{event::*, keyword::*, task::*, template::*};
use uuid::Uuid;
use xml::{reader::XmlEvent, EventReader};

use super::{find_attribute, Error, ErrorKind};

/// A instrumentation provider parsed from a instrumentation manifest
#[derive(Debug)]
#[non_exhaustive]
pub struct Provider {
    /// The name of the provider
    pub name: String,
    /// The GUID of the provider
    pub guid: Uuid,
    // pub resource_file_name: String,
    // pub message_file_name: String,
    /// The symbol(-name) of the provider
    pub symbol: String,
    // pub source: String,
    /// Keywords defined for this provider
    pub keywords: Vec<Keyword>,
    /// Tasks defined for this provider
    pub tasks: Vec<Task>,
    // maps: Vec<Map>,
    /// Events
    pub events: Vec<Event>,
    /// Templates for event payloads
    pub templates: Vec<Template>,
}
impl Provider {
    pub(super) fn parse<R: Read>(r: &mut EventReader<R>) -> Result<Provider, Error> {
        let (attributes, _) = super::xml_match_start(r, "provider")?;

        let name = find_attribute(&attributes, "name")?;
        let guid_str = find_attribute(&attributes, "guid")?;
        let symbol = find_attribute(&attributes, "symbol")?;

        let guid = Uuid::parse_str(&guid_str).map_err(|_| {
            Error::new(
                ErrorKind::TypeParseError,
                format!("failed to parse GUID from `{guid_str}`"),
            )
        })?;

        let mut keywords = Vec::new();
        let mut tasks = Vec::new();
        let mut events = Vec::new();
        let mut templates = Vec::new();
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes: _,
                    namespace: _,
                } => match name.local_name.as_str() {
                    "keywords" => Keyword::parse_keywords(r, &mut keywords)?,
                    "tasks" => Task::parse_tasks(r, &mut tasks)?,
                    "events" => Event::parse_events(r, &mut events)?,
                    "templates" => Template::parse_templates(r, &mut templates)?,
                    "maps" => r.skip()?,
                    "channels" => r.skip()?,
                    "levels" => r.skip()?,
                    "opcodes" => r.skip()?,
                    "filters" => r.skip()?,
                    _ => return Err(Error::new_unexpected_tag(name.local_name)),
                },
                XmlEvent::EndElement { name } => {
                    if name.local_name == "provider" {
                        break;
                    } else {
                        return Err(Error::new(
                            ErrorKind::UnexpectedStructure,
                            format!("Unexpeced end tag in <provider> ({})", name.local_name),
                        ));
                    }
                }
                _ => return Err(Error::new_unexpected(None)),
            }
        }

        Ok(Provider {
            name,
            guid,
            symbol,
            keywords,
            tasks,
            events,
            templates,
        })
    }
}
