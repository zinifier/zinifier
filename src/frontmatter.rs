use camino::Utf8Path;
use serde::{Deserialize, Serialize};
// use typst::foundations::{Repr, Value as TypstValue};

use std::collections::HashMap;

use crate::{theme::Theme, typ::typst_escape, zine::ZineFile};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub themes: HashMap<String, HashMap<String, String>>,
    // themes: HashMap<String, HashMap<String, TypstValue>>,
}

impl FrontMatter {
    pub fn to_typst(&self, zine: &ZineFile, theme: &Theme) -> String {
        let mut out = String::new();

        out.push_str("title: \"");
        out.push_str(&self.title);
        out.push_str("\",\n");

        if let Some(subtitle) = &self.subtitle {
            out.push_str("subtitle: \"");
            out.push_str(&subtitle);
            out.push_str("\",\n");
        }

        // out.push_str("border: \"");
        // out.push_str(&self.border);
        // out.push_str("\",\n");

        if let Some(author) = &self.author {
            out.push_str("author: \"");
            out.push_str(&typst_escape(&author));
            out.push_str("\",\n");
        }

        if let Some(description) = &self.description {
            out.push_str("description: [ ");
            out.push_str(&typst_escape(&description));
            out.push_str(" ],\n");
        }

        if let Some(summary) = &self.summary {
            out.push_str("summary: [ ");
            out.push_str(&typst_escape(&summary));
            out.push_str(" ],\n");
        }

        // if let Some(theme_settings) = &self.themes.get(&theme.name) {
        //     for (k, v) in theme_settings.iter() {
        //         // out.push_str(&format!("{k}: \"{v}\",\n"));
        //         // out.push_str(&format!("{k}: \"{}\",\n", v.clone().display().plain_text()));
        //         // out.push_str(&format!("{k}: {},\n", v.clone().display().plain_text()));
        //         info!("{k} has type {}", v.ty());
        //         let stringy_value = match &v {
        //             TypstValue::Color(_) => format!("rgb({})", v.clone().repr()),
        //             _ => format!("{}", v.clone().repr()),
        //         };
        //         out.push_str(&format!("{k}: {stringy_value},\n"));
        //     }
        // }

        if let Some(theme_settings) = &self.themes.get(&theme.name) {
            for (k, v) in theme_settings.iter() {
                let typst_value = if k.ends_with("_color") {
                    format!("rgb(\"{v}\")")
                } else if k.ends_with("_size")
                    || k.ends_with("_spacing")
                    || k.ends_with("_bool")
                    || k == "debug"
                {
                    format!("{v}")
                } else if k.ends_with("_res") {
                    format!(
                        "\"{}\"",
                        theme.zine_resource_relative_from_theme(v.as_str(), zine)
                    )
                } else {
                    format!("\"{v}\"")
                };

                out.push_str(&format!("{k}: {typst_value},\n"));
            }
        }

        // if let Some(background) = &self.background {
        //     // let background = zine.resource_relative_from_zine(background.as_str());
        //     let background = theme.zine_resource_relative_from_theme(background.as_str(), zine);
        //     debug!("Using background: {background}");
        //     out.push_str("background: \"");
        //     out.push_str(background.as_str());
        //     out.push_str("\",\n");
        // }

        out
    }

    pub fn with_typst_header(&self, zine: &ZineFile, theme: &Theme) -> String {
        let relative_theme_path = theme.relative_to_zine(zine);
        debug!("Using import theme: {relative_theme_path}");

        let mut out = String::new();
        out.push_str("#import \"");
        // out.push_str(theme.theme_relative().as_str());
        out.push_str(relative_theme_path.as_str());
        out.push_str("\": *");
        out.push_str("\n#show: zine.with(");

        for line in self.to_typst(zine, theme).lines() {
            out.push_str(&format!("  {line}\n"));
        }

        out.push_str(")\n\n");

        out
    }
}

pub fn split_frontmatter(file: &Utf8Path) -> (FrontMatter, String) {
    let content = std::fs::read_to_string(file).unwrap();

    let (toml_content, markdown_content) = content
        .trim_start_matches("+++")
        .split_once("\n+++")
        .unwrap();
    let frontmatter = toml::from_str(&toml_content).unwrap();

    (frontmatter, markdown_content.to_string())
}
