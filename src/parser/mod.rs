pub(crate) mod event;
pub(crate) mod keyword;
pub(crate) mod provider;
pub(crate) mod task;
pub(crate) mod template;

pub(crate) use provider::Provider;

use std::{fmt::Display, io::Read};

use xml::{
    attribute::OwnedAttribute, namespace::Namespace, reader::XmlEvent, EventReader, ParserConfig,
};

#[derive(Debug)]
pub(crate) struct Error {
    kind: ErrorKind,
    source: Option<xml::reader::Error>,
    description: Option<String>,
}
#[derive(Clone, Copy, Debug)]
enum ErrorKind {
    /// A error occured parsing the document
    Xml,
    /// The manifest isn't structured as expected
    UnexpectedStructure,
    UnexpectedTag,
    MissingAttribute,
    TypeParseError,
}
impl Error {
    fn new_unexpected(description: Option<String>) -> Error {
        Error {
            kind: ErrorKind::UnexpectedStructure,
            source: None,
            description,
        }
    }
    fn new_with_kind(kind: ErrorKind) -> Error {
        Error {
            kind,
            source: None,
            description: None,
        }
    }
    fn new(kind: ErrorKind, description: String) -> Error {
        Error {
            kind,
            source: None,
            description: Some(description),
        }
    }
    fn new_unexpected_tag(description: String) -> Error {
        Error {
            kind: ErrorKind::UnexpectedTag,
            source: None,
            description: Some(description),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            Some(e) => Some(e),
            None => None,
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::UnexpectedTag => write!(f, "Unexpected tag: Found {:?}", self.description),
            ErrorKind::MissingAttribute => write!(f, "Missing attribute: {:?}", self.description),
            _ => {
                if let Some(text) = &self.description {
                    write!(f, "{:?}: {}", self.kind, text)
                } else {
                    write!(f, "{:?}", self.kind)
                }
            }
        }
    }
}

impl From<xml::reader::Error> for Error {
    fn from(value: xml::reader::Error) -> Self {
        Error {
            kind: ErrorKind::Xml,
            source: Some(value),
            description: None,
        }
    }
}

pub(crate) fn parse<R: Read>(f: &mut R) -> Result<Provider, Error> {
    let conf = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true);
    let mut reader = EventReader::new_with_config(f, conf);

    match reader.next()? {
        XmlEvent::StartDocument {
            version: _,
            encoding: _,
            standalone: _,
        } => {
            // Expect the document start
        }
        _ => {
            return Err(Error::new_unexpected(Some(
                "Expected StartDocument tag, found other".to_string(),
            )))
        }
    }
    xml_match_start(&mut reader, "instrumentationManifest")?;
    loop {
        match reader.next()? {
            XmlEvent::StartElement {
                name,
                attributes: _,
                namespace: _,
            } => {
                if name.local_name != "instrumentation" {
                    reader.skip()?;
                } else {
                    break;
                }
            }
            _ => {
                return Err(Error::new_unexpected(Some(
                    "Encountered wrong element when searching for <instrumentation> start"
                        .to_string(),
                )))
            }
        }
    }
    xml_match_start(&mut reader, "events")?;

    let provider = Provider::parse(&mut reader)?;

    xml_match_end(&mut reader, "events")?;
    reader.skip()?;

    Ok(provider)
}

fn xml_match_start<R: Read>(
    r: &mut EventReader<R>,
    tag: &str,
) -> Result<(Vec<OwnedAttribute>, Namespace), Error> {
    let event = r.next()?;
    match event {
        XmlEvent::StartElement {
            name,
            attributes,
            namespace,
        } => {
            if name.local_name != tag {
                Err(Error::new_unexpected_tag(format!(
                    "Expected {}, found {}",
                    tag, name.local_name
                )))
            } else {
                Ok((attributes, namespace))
            }
        }
        _ => Err(Error::new_unexpected(None)),
    }
}
fn xml_match_end<R: Read>(r: &mut EventReader<R>, tag: &str) -> Result<(), Error> {
    match r.next()? {
        XmlEvent::EndElement { name } => {
            if name.local_name != tag {
                Err(Error::new_unexpected_tag(name.local_name))
            } else {
                Ok(())
            }
        }
        _ => Err(Error::new_unexpected(None)),
    }
}

fn find_attribute(attributes: &[OwnedAttribute], name: &str) -> Result<String, Error> {
    let name = attributes
        .iter()
        .find_map(|a| {
            if a.name.local_name == name {
                Some(a.value.clone())
            } else {
                None
            }
        })
        .ok_or(Error::new(
            ErrorKind::MissingAttribute,
            format!("Missing attribute {}", name),
        ))?;
    Ok(name)
}
