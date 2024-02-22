use std::io::Read;

use crate::parser::{event::*, keyword::*, task::*, template::*};
use uuid::Uuid;
use xml::{reader::XmlEvent, EventReader};

use super::{find_attribute, Error, ErrorKind};

#[derive(Debug)]
pub(crate) struct Provider {
    pub(crate) name: String,
    pub(crate) guid: Uuid,
    // pub(crate) resource_file_name: String,
    // pub(crate) message_file_name: String,
    pub(crate) symbol: String,
    // pub(crate) source: String,
    pub(crate) keywords: Vec<Keyword>,
    pub(crate) tasks: Vec<Task>,
    // maps: Vec<Map>,
    pub(crate) events: Vec<Event>,
    pub(crate) templates: Vec<Template>,
}
impl Provider {
    pub(super) fn parse<R: Read>(r: &mut EventReader<R>) -> Result<Provider, Error> {
        let (attributes, _) = super::xml_match_start(r, "provider")?;

        let name = find_attribute(&attributes, "name")?;
        let guid_str = find_attribute(&attributes, "guid")?;
        let symbol = find_attribute(&attributes, "symbol")?;

        let guid = Uuid::parse_str(&guid_str)
            .map_err(|_| Error::new_with_kind(ErrorKind::TypeParseError))?;

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

    pub(crate) fn guid_constant_name(&self) -> proc_macro2::Ident {
        use quote::format_ident;
        let name = self.name.to_uppercase().replace('-', "_").replace(' ', "");
        format_ident!("{}_GUID", name)
    }
}
