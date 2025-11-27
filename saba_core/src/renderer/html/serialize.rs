use crate::renderer::dom::node::{ElementKind, Node, NodeData};
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};

pub fn serialize(node: &Node) -> String {
    let mut res = String::new();

    for child in node.children() {
        let child = child.borrow();
        match child.data() {
            NodeData::Element(element) => {
                let tag_name = element.kind.to_string();
                res += "<";
                res += &tag_name;

                for attr in element.attributes.iter() {
                    res += " ";
                    res += &serialize_attribute_name(&attr.name);
                    res += "=\"";
                    res += &escape_attribute_value(&attr.value);
                    res += "\"";
                }

                res += ">";

                res += &serialize(&child);

                res += "</";
                res += &tag_name;
                res += ">";
            }
            NodeData::Document => {}
            NodeData::Text(text) => {
                let parent = child.parent.upgrade();
                let parent_kind = parent.and_then(|pn| {
                    if let NodeData::Element(ref pe) = pn.borrow().data() {
                        Some(pe.kind.clone())
                    } else {
                        None
                    }
                });

                match parent_kind {
                    Some(ElementKind::Style | ElementKind::Script) => {
                        // Append the text literally.
                        res += &text;
                    }
                    _ => {
                        // Escape the text.
                        res += &escape_html(&text)
                    }
                }
            }
        }
    }

    res
}

fn serialize_attribute_name(input: &str) -> String {
    // TODO: How is the namespace of attributes determined?
    input.to_owned()
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('\u{00a0}', "&nbsp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attribute_value(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('\u{00a0}', "&nbsp;")
        // Comment out the below two replaces to disable the mXSS mitigation
        // introduced on 2025-05 to escape <> in attribute values
        // HTML Standard: <https://github.com/whatwg/html/commit/e21bd3b4a94bfdbc23d863128e0b207be9821a0f>
        // Chrome: <https://developer.chrome.com/blog/escape-attributes>
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        // end of comment out
        .replace('"', "&quot;")
}
