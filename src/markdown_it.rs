// use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, TextMergeStream};
use markdown_it::parser::{block::*, inline::*};
use markdown_it::plugins::cmark::{
    block::{
        heading::ATXHeading,
        list::{BulletList, ListItem, OrderedList},
        paragraph::Paragraph,
    },
    inline::{
        emphasis::Strong,
        image::Image,
        newline::{Hardbreak, Softbreak},
    },
};
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};
use markdown_it_footnote::{
    definitions::FootnoteDefinition, inline::InlineFootnote, references::FootnoteReference,
};
use markdownmacros::{
    block::{parse_block, parse_block_end, BlockMacro},
    // value::Value,
};

use crate::typ::typst_escape;

enum ListType {
    Numbered,
    NotNumbered,
}

pub fn markdown_to_typst_content(markdown: &str) -> String {
    let md = &mut MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it_footnote::add(md);
    md.block.add_rule::<BlockMacroScanner>();

    // Context for the current list items
    let mut list_type = ListType::NotNumbered;

    let mut root = md.parse(markdown);

    let mut out = String::new();

    // We walk mutably so we can consume inner children
    // However, we always need a second pass because footnotes are linear in MarkdownIt AST (first the ref, then the rest of document,
    // then the actual footnote content).
    let mut footnote_counter = 0;
    // The first optional string is the label (like NdT for translation notes)
    // If only a number is in the label, it is ignored.
    let mut footnotes: Vec<(Option<String>, String)> = vec![];
    // let mut footnotes: Vec<String> = vec!();

    root.walk_mut(|node, _| {
        if node.is::<ATXHeading>() {
            let node: &ATXHeading = node.node_value.downcast_ref().unwrap();
            out.push_str(&"=".repeat(node.level as usize));
            out.push_str(" ");
        } else if node.is::<Strong>() {
            out.push_str("#strong([");
            // TODO: maybe parse those as markdown too?
            out.push_str(&node.collect_text());
            // IS THIS CORRECT? Replace collected children with emptiness
            node.children = vec![];
            out.push_str("])")
        } else if node.is::<Text>() {
            let node: &Text = node.node_value.downcast_ref().unwrap();
            out.push_str(&typst_escape(&node.content));
            out.push_str("\n");
        } else if node.is::<TextSpecial>() {
            let node: &TextSpecial = node.node_value.downcast_ref().unwrap();
            out.push_str(&node.markup);
        } else if node.is::<Paragraph>() {
            out.push_str("\n");
            out.push_str("\n");
        } else if node.is::<Hardbreak>() {
            out.push_str("\\\n");
        } else if node.is::<Softbreak>() {
            out.push_str("\n");
        } else if node.is::<TypstMacroNode>() {
            let node: &TypstMacroNode = node.node_value.downcast_ref().unwrap();
            // Recurse parsing markdown inside the macro
            out.push_str(&markdown_to_typst_content(&node.0));
        } else if node.is::<InlineFootnote>() {
            footnote_counter += 1;
            out.push_str(&format!("[^{}]", footnote_counter));
        } else if node.is::<FootnoteReference>() {
            footnote_counter += 1;
            out.push_str(&format!("[^{}]", footnote_counter));
        } else if node.is::<FootnoteDefinition>() {
            let typed_node: &FootnoteDefinition = node.node_value.downcast_ref().unwrap();
            let label = typed_node.label.to_owned();

            footnotes.push((label, format!("#footnote[{}]", &node.collect_text())));
            node.children = vec![];
        } else if node.is::<BulletList>() {
            list_type = ListType::NotNumbered;
        } else if node.is::<OrderedList>() {
            list_type = ListType::Numbered;
        } else if node.is::<ListItem>() {
            out.push_str("\n");

            match list_type {
                ListType::NotNumbered => {
                    out.push_str("- ");
                }
                ListType::Numbered => {
                    out.push_str("+ ");
                }
            }

            out.push_str(&node.collect_text());
        } else if node.is::<Image>() {
            // TODO: support caption
            // TODO: support specifying image size
            //  for exmaple with https://stackoverflow.com/questions/14675913/changing-image-size-in-markdown/21242579#21242579
            //  or with title https://docs.rs/markdown-it/latest/markdown_it/generics/inline/full_link/index.html
            let typed_node: &Image = node.node_value.downcast_ref().unwrap();
            out.push_str(&format!("\n#image(height: 100%, \"{}\")\n", typed_node.url));

            // Remove the image caption for the moment
            node.children = vec!();
        } else {
            debug!("Unknown node type: {}", node.node_type.name);
        }
    });

    if footnote_counter != footnotes.len() {
        panic!(
            "Counted {} footnotes but found {} actual content for footnotes",
            footnote_counter,
            footnotes.len()
        );
    }

    while footnote_counter > 0 {
        let (label, content) = &footnotes[footnote_counter - 1];
        if let Some(label) = label {
            if let Ok(_n) = label.parse::<u8>() {
                // Only a number, don't care about nothing
                out = out.replace(&format!("[^{}]", footnote_counter), &content);
            } else {
                // Label like NdT1 extract "NdT"
                let new_label = sanitize_label(&label);
                out = out.replace(
                    &format!("[^{}]", footnote_counter),
                    &content.replace("#footnote[", &format!("#footnote[#emph[{}:] ", new_label)), // &format!("{}: {}", new_label, &content),
                );
            }
        } else {
            // No label, is that even possible?
            // out = out.replace(&format!("[^{}]", footnote_counter), &content);
            unreachable!();
        }
        footnote_counter -= 1;
    }

    out
}

// So we compute the macro body manually because in nom it's complicated ?! I can't figure out how to delimit
// the macro body...
// TODO: recursive macros

#[derive(Clone, Debug)]
struct TypstMacroNode(String);

impl TypstMacroNode {
    pub fn from_raw_macro(m: &BlockMacro) -> Self {
        let mut out = String::from("\n\n#");
        out.push_str(&m.name);
        out.push_str("(\n");
        for (k, v) in &m.args {
            out.push_str("  ");
            out.push_str(&k);
            out.push_str(": ");
            out.push_str(&v.to_typst());
            out.push_str(",\n");
        }
        out.push_str("  [\n");
        out.push_str(&typst_escape(&m.body));
        out.push_str("  ]\n");
        out.push_str(")\n");

        Self(out)
    }

    // pub fn n_lines(&self) -> usize {
    //     self.0.lines().count()
    // }
}

impl NodeValue for TypstMacroNode {
    fn render(&self, _node: &Node, _fmt: &mut dyn Renderer) {
        unimplemented!("Please don't use HTML on me!")
    }
}

// This is an extension for the inline subparser.
struct BlockMacroScanner;

impl BlockRule for BlockMacroScanner {
    // If custom structure is found, it:
    //  - creates a new `Node` in AST
    //  - increments `state.line` to a position after this node
    //  - returns true
    //
    // In "silent mode" (when `silent=true`) you aren't allowed to
    // create any nodes, should only increment `state.line`.
    //
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let line = state.get_line(state.line).trim();
        if !line.contains("{%") {
            return None;
        }

        match parse_block(line) {
            Some(mut m) => {
                state.line += 1;

                while state.line < state.line_max {
                    // Add lines to the body until we read end of macro marker
                    let line = state.get_line(state.line).trim();
                    if !line.contains("{%") {
                        m.body.push_str(line);
                        m.body.push_str("\n");
                        state.line += 1;
                        continue;
                    }

                    // TODO: recursive macro
                    if let Some(_endmacro) = parse_block_end(line, &m.name) {
                        // This specific macro was ended
                        state.line += 1;
                        break;
                    }
                }

                // Now we have the body, convert it to typst macro
                let typst_macro = TypstMacroNode::from_raw_macro(&m);
                // let n_lines = typst_macro.n_lines();
                Some((
                    Node::new(typst_macro),
                    // n_lines,
                    1,
                ))
            }
            None => {
                // It looks like a macro but isn't a macro
                None
            }
        }
    }
}

/// Only keep non-digit characters
pub fn sanitize_label(label: &str) -> String {
    label.chars().filter(|c| !c.is_ascii_digit()).collect()
}
