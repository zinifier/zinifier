// use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, TextMergeStream};
use markdown_it::{MarkdownIt, NodeValue, Node, Renderer};
use markdown_it::parser::{
    inline::*,
    block::*,
};
use markdown_it::plugins::cmark::{
    block::{
        heading::ATXHeading,
        paragraph::Paragraph,
    },
    inline::{
        newline::{Hardbreak, Softbreak},
    },
};
use markdownmacros::{
    block::{BlockMacro, parse_block, parse_block_end},
    // value::Value,
};

use crate::typ::typst_escape;

pub fn markdown_to_typst_content(markdown: &str) -> String {
    let md = &mut MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    md.block.add_rule::<BlockMacroScanner>();

    let root = md.parse(markdown);

    let mut out = String::new();

    root.walk(|node, _| {
        if node.is::<ATXHeading>() {
            let node: &ATXHeading = node.node_value.downcast_ref().unwrap();
            out.push_str(&"=".repeat(node.level as usize));
            out.push_str(" ");
            // out.push_str("\n");
        }
        else if node.is::<Text>() {
            let node: &Text = node.node_value.downcast_ref().unwrap();
            out.push_str(&typst_escape(&node.content));
            out.push_str("\n");
        }
        else if node.is::<TextSpecial>() {
            let node: &TextSpecial = node.node_value.downcast_ref().unwrap();
            out.push_str(&node.markup);
        }
        else if node.is::<Paragraph>() {
            out.push_str("\n");
            // let node: &Paragraph = node.node_value.downcast_ref().unwrap();
            out.push_str("\n");
            // out.push_str("\n");
        }
        else if node.is::<Hardbreak>() {
            out.push_str("\\\n");
        }
        else if node.is::<Softbreak>() {
            out.push_str("\n");
        }
        else if node.is::<TypstMacroNode>() {
            let node: &TypstMacroNode = node.node_value.downcast_ref().unwrap();
            out.push_str(&node.0);
        }
    });
    
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
        if !line.contains("{%") { return None; }

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
            },
            None => {
                // It looks like a macro but isn't a macro
                None
            }
        }
    }
}
