use anyhow::Result;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

/// Injects a package as a ModuleScript into the .poly XML file.
///
/// If the module already exists (by name), updates it instead.
/// Otherwise, finds the ScriptService and adds the new ModuleScript as a child.
pub fn inject_module_script(poly_xml: &str, name: &str, source: &str) -> Result<String> {
    // Quick check: does this module already exist?
    // If so, just update it instead of trying to inject a duplicate.
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
                // Look for the ScriptService item—that's where we'll inject the module.
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
                // When we close the ScriptService Item, that's our cue to inject the module.
                if in_script_service && e.local_name().as_ref() == b"Item" && depth == 1 {
                    // Insert the new ModuleScript before closing ScriptService

                    // Indentation (matches the style of other Items in ScriptService)
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;

                    // Create the ModuleScript Item
                    let mut script_item = BytesStart::new("Item");
                    script_item.push_attribute(("class", "ModuleScript"));
                    writer.write_event(Event::Start(script_item))?;

                    // Properties container
                    writer
                        .write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                    let props_start = BytesStart::new("Properties");
                    writer.write_event(Event::Start(props_start))?;

                    // Source property (the actual Lua code)
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                        "\n        ",
                    )))?;
                    let mut source_start = BytesStart::new("string");
                    source_start.push_attribute(("name", "Source"));
                    writer.write_event(Event::Start(source_start))?;
                    // quick-xml auto-escapes XML special chars here, so we don't have to worry about that
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(source)))?;
                    writer.write_event(Event::End(BytesEnd::new("string")))?;

                    // Name property (what users see in the project)
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                        "\n        ",
                    )))?;
                    let mut name_start = BytesStart::new("string");
                    name_start.push_attribute(("name", "Name"));
                    writer.write_event(Event::Start(name_start))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(name)))?;
                    writer.write_event(Event::End(BytesEnd::new("string")))?;

                    // Close Properties
                    writer
                        .write_event(Event::Text(quick_xml::events::BytesText::new("\n      ")))?;
                    writer.write_event(Event::End(BytesEnd::new("Properties")))?;

                    // Close Item
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new("\n    ")))?;
                    writer.write_event(Event::End(BytesEnd::new("Item")))?;

                    // Indentation for closing ScriptService tag
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

/// Replaces an existing ModuleScript with new source code.
///
/// This is more complex than injection because we have to:
/// 1. Find the right ModuleScript (by Name property)
/// 2. Buffer up all its XML events
/// 3. Decide whether to keep it or replace it
/// 4. Write out the result
///
/// It's a bit stateful and gross, but XML is like that sometimes.
pub fn update_module_script(poly_xml: &str, name: &str, source: &str) -> Result<String> {
    let mut reader = Reader::from_str(poly_xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut in_script_service = false;
    let mut depth = 0;

    // State for capturing an entire ModuleScript Item
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
                        // Found a ModuleScript at the right depth—start capturing it
                        } else if in_script_service && class_val == b"ModuleScript" && depth == 3 {
                            capturing_module = true;
                        }
                    }
                // While capturing, look for the Name property to identify which module this is
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
            // When we're capturing the Name text, check if it matches our target
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
            // When we reach the closing Item tag for this module, decide what to do
            if let Event::End(e) = module_buffer.last().unwrap() {
                if e.local_name().as_ref() == b"Item" && depth == 2 {
                    if is_target_module {
                        // This is the one we're updating—write a fresh replacement
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                            "\n    ",
                        )))?;
                        let mut script_item = BytesStart::new("Item");
                        script_item.push_attribute(("class", "ModuleScript"));
                        writer.write_event(Event::Start(script_item))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                            "\n      ",
                        )))?;
                        let props_start = BytesStart::new("Properties");
                        writer.write_event(Event::Start(props_start))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                            "\n        ",
                        )))?;
                        let mut source_start = BytesStart::new("string");
                        source_start.push_attribute(("name", "Source"));
                        writer.write_event(Event::Start(source_start))?;
                        writer
                            .write_event(Event::Text(quick_xml::events::BytesText::new(source)))?;
                        writer.write_event(Event::End(BytesEnd::new("string")))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                            "\n        ",
                        )))?;
                        let mut name_start = BytesStart::new("string");
                        name_start.push_attribute(("name", "Name"));
                        writer.write_event(Event::Start(name_start))?;
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(name)))?;
                        writer.write_event(Event::End(BytesEnd::new("string")))?;

                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                            "\n      ",
                        )))?;
                        writer.write_event(Event::End(BytesEnd::new("Properties")))?;
                        writer.write_event(Event::Text(quick_xml::events::BytesText::new(
                            "\n    ",
                        )))?;
                        writer.write_event(Event::End(BytesEnd::new("Item")))?;
                    } else {
                        // Not our target—preserve the original module as-is
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
            // Not in a module we're capturing—just pass through
            writer.write_event(event)?;
        }

        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

/// Removes a ModuleScript from the .poly file by name.
///
/// Similar dance to update: walk the tree, find the matching module, skip it.
/// Everything else gets written through unchanged.
pub fn remove_module_script(poly_xml: &str, name: &str) -> Result<String> {
    let mut reader = Reader::from_str(poly_xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut in_script_service = false;
    let mut depth = 0;

    // State for capturing a ModuleScript Item to decide whether to skip it
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
                // Extract the module's name to check if it matches our target
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
            // When we reach the closing Item, decide: keep it or skip it?
            let last_event = item_buffer.last().unwrap();
            if let Event::End(e) = last_event {
                if e.local_name().as_ref() == b"Item" && depth == 2 {
                    if current_item_name != name {
                        // Not our target—write it back out unchanged
                        for ev in item_buffer.drain(..) {
                            writer.write_event(ev)?;
                        }
                    } else {
                        // This is the one we're removing—just skip the buffer
                        item_buffer.clear();
                    }
                    capturing_item = false;
                    current_item_name.clear();
                }
            }
        } else {
            // Not capturing—pass through everything
            writer.write_event(event)?;
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}
