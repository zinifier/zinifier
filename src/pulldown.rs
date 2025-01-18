use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, TextMergeStream};

use crate::typ::typst_escape;

pub fn markdown_to_typst_content(markdown: &str) -> String {
    let options = markdown_options();
    let iterator = TextMergeStream::new(Parser::new_ext(markdown, options));

    let mut out = String::new();
    
    for event in iterator {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        out.push_str(&"=".repeat(level as usize));
                        out.push_str(" ");
                    }
                    _ => continue,
                }
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading(_) => out.push_str("\n\n"),
                    _ => continue,
                }
            }
            Event::Text(text) => out.push_str(&typst_escape(&text)),
            _ => {}
        }
    }

    out
}

pub fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    // options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    options
}
