use std::io::Read;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{find_attribute, ErrorKind};

use super::Error;

/// A task defined by a provider
#[derive(Debug)]
pub struct Task {
    /// Name of the Task
    pub name: String,
    /// Numerical identifier of the task
    pub value: u64,
}

impl Task {
    fn from_attributes(attr: &[OwnedAttribute]) -> Result<Task, Error> {
        let name = find_attribute(attr, "name")?;
        let value_str = find_attribute(attr, "value")?;
        let value: u64 = value_str
            .parse()
            .map_err(|_| Error::new_with_kind(ErrorKind::TypeParseError))?;
        Ok(Task { name, value })
    }

    pub(super) fn parse_tasks<R: Read>(
        r: &mut EventReader<R>,
        vec: &mut Vec<Task>,
    ) -> Result<(), Error> {
        loop {
            match r.next()? {
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    if name.local_name == "opcodes" {
                        r.skip()?;
                        continue;
                    }
                    if name.local_name != "task" {
                        return Err(Error::new_unexpected_tag(format!(
                            "Expected <task>, found {}",
                            name.local_name
                        )));
                    }
                    vec.push(Task::from_attributes(&attributes)?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "tasks" {
                        return Ok(());
                    } else {
                        continue;
                    }
                }
                _ => {
                    return Err(Error::new_unexpected(Some(
                        "Unexpeced element in <tasks>".to_string(),
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Task;
    use crate::xml_match_start;
    use xml::{EventReader, ParserConfig};

    #[test]
    fn test_parse_tasks() {
        const XML: &str = r#"
        <tasks>
          <task name="task_0" message="$(string.task_task_0)" value="0" />
          <task name="ProcessStart" message="$(string.task_ProcessStart)" value="1" />
          <task name="ProcessStop" message="$(string.task_ProcessStop)" value="2" />
          <task name="ThreadStart" message="$(string.task_ThreadStart)" value="3" />
          <task name="ThreadStop" message="$(string.task_ThreadStop)" value="4" />
          <task name="ImageLoad" message="$(string.task_ImageLoad)" value="5" />
          <task name="ImageUnload" message="$(string.task_ImageUnload)" value="6" />
          <task name="CpuBasePriorityChange" message="$(string.task_CpuBasePriorityChange)" value="7" />
          <task name="CpuPriorityChange" message="$(string.task_CpuPriorityChange)" value="8" />
          <task name="PagePriorityChange" message="$(string.task_PagePriorityChange)" value="9" />
          <task name="IoPriorityChange" message="$(string.task_IoPriorityChange)" value="10" />
          <task name="ProcessFreeze" message="$(string.task_ProcessFreeze)" value="11" />
          <task name="JobStart" message="$(string.task_JobStart)" value="13" />
          <task name="JobTerminate" message="$(string.task_JobTerminate)" value="14" />
          <task name="ProcessRundown" message="$(string.task_ProcessRundown)" value="15" />
          <task name="PsDiskIoAttribution" message="$(string.task_PsDiskIoAttribution)" value="16" />
          <task name="PsIoRateControl" message="$(string.task_PsIoRateControl)" value="17" />
          <task name="ThreadWorkOnBehalfUpdate" message="$(string.task_ThreadWorkOnBehalfUpdate)" value="18" />
          <task name="JobServerSiloStart" message="$(string.task_JobServerSiloStart)" value="19" />
        </tasks>
        "#;
        let conf = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true);
        let mut reader = EventReader::new_with_config(XML.as_bytes(), conf);
        reader.next().unwrap();
        xml_match_start(&mut reader, "tasks").unwrap();
        let mut tasks = Vec::new();
        Task::parse_tasks(&mut reader, &mut tasks).unwrap();
        assert_eq!(tasks.len(), 19, "Check if all tasks were parsed");
        let image_unload_task = tasks.iter().find(|&t| t.name == "ImageUnload").unwrap();
        assert_eq!(image_unload_task.value, 6);
    }
}
