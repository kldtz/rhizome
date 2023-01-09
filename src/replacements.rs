use lazy_static::lazy_static;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use pulldown_cmark::{html, Options, Parser};
use regex::{CaptureMatches, Captures, Regex};

use crate::error::KBResult;

lazy_static! {
    static ref LINK: Regex = Regex::new(r"\[(.*?)\]\(([bu]):(.+?)\)").unwrap();
}

pub const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

fn replace_all<'a>(
    captures: CaptureMatches<'_, 'a>,
    raw_content: &'a str,
    compute_replacement: impl Fn(Captures<'a>) -> KBResult<String>,
) -> KBResult<String> {
    let mut content = String::new();
    let mut last_offset = 0;
    for cap in captures {
        let full_match = cap.get(0).unwrap();
        content.push_str(&raw_content[last_offset..full_match.start()]);
        let replacement = compute_replacement(cap)?;
        content.push_str(&replacement);
        last_offset = full_match.end();
    }
    content.push_str(&raw_content[last_offset..]);
    Ok(content)
}

fn encode_internal_link_with_spaces(caps: Captures, prefix: &str) -> KBResult<String> {
    let text = caps.get(1).unwrap().as_str();
    let url = caps.get(3).unwrap().as_str();
    let text = if text.is_empty() { url } else { text };
    let url: String = utf8_percent_encode(url, FRAGMENT).collect();
    Ok(format!("[{}]({}{})", text, prefix, url))
}

pub fn preprocess_internal_links(text: &str, prefix: &str) -> KBResult<String> {
    let captures = LINK.captures_iter(text);
    replace_all(captures, text,
                |c| encode_internal_link_with_spaces(c, prefix))
}

pub fn markdown2html(markdown: &str, prefix: &str) -> KBResult<String> {
    let markdown = preprocess_internal_links(markdown, prefix)?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&markdown, options);
    let mut content = String::new();
    html::push_html(&mut content, parser);
    Ok(content)
}