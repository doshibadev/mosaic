use anyhow::Result;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

pub fn inject_module_script(poly_xml: &str, name: &str, source: &str) -> Result<String> {
    let exists = poly_xml.contains(&format!("<string name=\"Name\">{}</string>", name));
    if exists {
        return update_module_script(poly_xml, name, source);
    }

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
                        if attr.value.as_ref() as &[u8] == b"ScriptService" {
                            in_script_service = true;
                        }
                    }
                }
                writer.write_event(Event::Start(e))?;
            }
            Event::End(e) => {
                depth -= 1;
                if in_script_service && e.local_name().as_ref() == b"Item" && depth == 1 {
                    // Inject the new module before closing ScriptService
                    
                    // Indentation for the Item
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                    
                    let mut script_item = BytesStart::new("Item");
                    script_item.push_attribute(("class", "ModuleScript"));
                    writer.write_event(Event::Start(script_item))?;

                    // Properties
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                    let props_start = BytesStart::new("Properties");
                    writer.write_event(Event::Start(props_start))?;

                    // Source
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n        ")))?;
                    let mut source_start = BytesStart::new("string");
                    source_start.push_attribute(("name", "Source"));
                    writer.write_event(Event::Start(source_start))?;
                    // quick-xml automatically escapes special characters in Text events
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(source)))?;
                    writer.write_event(Event::End(BytesEnd::new("string")))?;

                    // Name
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n        ")))?;
                    let mut name_start = BytesStart::new("string");
                    name_start.push_attribute(("name", "Name"));
                    writer.write_event(Event::Start(name_start))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(name)))?;
                    writer.write_event(Event::End(BytesEnd::new("string")))?;

                    // Close Properties
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                    writer.write_event(Event::End(BytesEnd::new("Properties")))?;

                    // Close Item
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                    writer.write_event(Event::End(BytesEnd::new("Item")))?;

                    // Indentation for closing ScriptService
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n  ")))?;

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

pub fn update_module_script(poly_xml: &str, name: &str, source: &str) -> Result<String> {
    let mut reader = Reader::from_str(poly_xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut in_script_service = false;
    let mut depth = 0;

    let mut capturing_module = false;
    let mut module_buffer: Vec<Event<'static>> = Vec::new();
    let mut is_target_module = false;
    let mut capturing_name = false;

    loop {
        let event = reader.read_event_into(&mut buf)?;
        match &event {
            Event::Start(e) => {
                depth += 1;
                if e.local_name().as_ref() == b"Item" {
                    if let Some(attr) = e.try_get_attribute("class")? {
                        let class_val = attr.value.as_ref() as &[u8];
                        if class_val == b"ScriptService" {
                            in_script_service = true;
                        } else if in_script_service && class_val == b"ModuleScript" && depth == 3 {
                            capturing_module = true;
                        }
                    }
                } else if capturing_module && e.local_name().as_ref() == b"string" {
                    if let Some(attr) = e.try_get_attribute("name")? {
                        if attr.value.as_ref() as &[u8] == b"Name" {
                            capturing_name = true;
                        }
                    }
                }
            }
            Event::End(e) => {
                depth -= 1;
                if e.local_name().as_ref() == b"Item" && in_script_service && depth == 1 {
                    in_script_service = false;
                }
            }
            Event::Text(t) => {
                if capturing_name {
                    let decoded = reader.decoder().decode(t.as_ref())?;
                    if decoded.trim() == name {
                        is_target_module = true;
                    }
                    capturing_name = false;
                }
            }
            Event::Eof => break,
            _ => {}
        }

        if capturing_module {
            module_buffer.push(event.into_owned());
            if let Event::End(e) = module_buffer.last().unwrap() {
                if e.local_name().as_ref() == b"Item" && depth == 2 {
                    if is_target_module {
                        // Write replacement module
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                        let mut script_item = BytesStart::new("Item");
                        script_item.push_attribute(("class", "ModuleScript"));
                        writer.write_event(Event::Start(script_item))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                        let props_start = BytesStart::new("Properties");
                        writer.write_event(Event::Start(props_start))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n        ")))?;
                        let mut source_start = BytesStart::new("string");
                        source_start.push_attribute(("name", "Source"));
                        writer.write_event(Event::Start(source_start))?;
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(source)))?;
                        writer.write_event(Event::End(BytesEnd::new("string")))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n        ")))?;
                        let mut name_start = BytesStart::new("string");
                        name_start.push_attribute(("name", "Name"));
                        writer.write_event(Event::Start(name_start))?;
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(name)))?;
                        writer.write_event(Event::End(BytesEnd::new("string")))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                        writer.write_event(Event::End(BytesEnd::new("Properties")))?;
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                        writer.write_event(Event::End(BytesEnd::new("Item")))?;
                    } else {
                        // Not the target, write original events
                        for ev in module_buffer.drain(..) {
                            writer.write_event(ev)?;
                        }
                    }
                    capturing_module = false;
                    is_target_module = false;
                    module_buffer.clear();
                }
            }
        } else {
            writer.write_event(event)?;
        }

        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

pub fn remove_module_script(poly_xml: &str, name: &str) -> Result<String> {
    let mut reader = Reader::from_str(poly_xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut in_script_service = false;
    let mut depth = 0;
    let mut capturing_item = false;
    let mut item_buffer: Vec<quick_xml::events::Event> = Vec::new();
    let mut current_item_name = String::new();
    let mut capturing_name_text = false;

    loop {
        let event = reader.read_event_into(&mut buf)?;
        match &event {
            Event::Start(e) => {
                depth += 1;
                if e.local_name().as_ref() == b"Item" {
                    if let Some(attr) = e.try_get_attribute("class")? {
                        let class_val = attr.value.as_ref() as &[u8];
                        if class_val == b"ScriptService" {
                            in_script_service = true;
                        } else if in_script_service && class_val == b"ModuleScript" && depth == 3 {
                            capturing_item = true;
                        }
                    }
                } else if capturing_item && e.local_name().as_ref() == b"string" {
                    if let Some(attr) = e.try_get_attribute("name")? {
                        if attr.value.as_ref() as &[u8] == b"Name" {
                            capturing_name_text = true;
                        }
                    }
                }
            }
            Event::End(e) => {
                depth -= 1;
                if e.local_name().as_ref() == b"Item" && in_script_service && depth == 1 {
                    in_script_service = false;
                }
            }
            Event::Text(t) => {
                if capturing_name_text {
                    let text = reader.decoder().decode(t.as_ref())?;
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        current_item_name = trimmed.to_string();
                        capturing_name_text = false;
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }

        if capturing_item {
            item_buffer.push(event.into_owned());
            let last_event = item_buffer.last().unwrap();
            if let Event::End(e) = last_event {
                if e.local_name().as_ref() == b"Item" && depth == 2 {
                    if current_item_name != name {
                        for ev in item_buffer.drain(..) {
                            writer.write_event(ev)?;
                        }
                    } else {
                        item_buffer.clear();
                    }
                    capturing_item = false;
                    current_item_name.clear();
                }
            }
        } else {
            writer.write_event(event)?;
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}
