use anyhow::Result;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

pub fn inject_module_script(poly_xml: &str, name: &str, source: &str) -> Result<String> {
    let mut reader = Reader::from_str(poly_xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut in_script_service = false;
    let mut depth = 0;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                depth += 1;
                if e.local_name().as_ref() == b"Item" {
                    if let Some(attr) = e.try_get_attribute("class")? {
                        if attr.value.as_ref() == b"ScriptService" {
                            in_script_service = true;
                        }
                    }
                }
                writer.write_event(Event::Start(e))?;
            }
            Event::End(e) => {
                depth -= 1;
                if in_script_service && e.local_name().as_ref() == b"Item" && depth == 1 {
                    // Inject our module script
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n  ")))?;
                    let mut script_item = BytesStart::new("Item");
                    script_item.push_attribute(("class", "ModuleScript"));
                    writer.write_event(Event::Start(script_item))?;

                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                    let props_start = BytesStart::new("Properties");
                    writer.write_event(Event::Start(props_start))?;

                    writer
                        .write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                    let mut source_start = BytesStart::new("string");
                    source_start.push_attribute(("name", "Source"));
                    writer.write_event(Event::Start(source_start))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(source)))?;
                    writer.write_event(Event::End(BytesEnd::new("string")))?;

                    writer
                        .write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                    let mut name_start = BytesStart::new("string");
                    name_start.push_attribute(("name", "Name"));
                    writer.write_event(Event::Start(name_start))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(name)))?;
                    writer.write_event(Event::End(BytesEnd::new("string")))?;

                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                    writer.write_event(Event::End(BytesEnd::new("Properties")))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n  ")))?;
                    writer.write_event(Event::End(BytesEnd::new("Item")))?;

                    in_script_service = false;
                }
                writer.write_event(Event::End(e))?;
            }
            Event::Eof => break,
            e => {
                writer.write_event(e)?;
            }
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}
