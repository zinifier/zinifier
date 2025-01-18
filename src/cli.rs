use camino::{Utf8PathBuf, Utf8Path};
use clap::{Parser, ValueEnum};

use crate::{compile, watch};

#[derive(Debug, Parser)]
struct Cli {
    action: Action,
    file: Utf8PathBuf,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Action {
    Compile,
    #[cfg(feature="watch")]
    Watch,
}

impl Action {
    pub fn run(&self, root: &Utf8Path, file: &Utf8Path) {
        let s = SourceType::from_ext(file);
        
        match self {
            Self::Compile => s.compile(root, file),
            #[cfg(feature="watch")]
            Self::Watch => s.watch(root, file),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SourceType {
    Markdown,
    Typst,
}

impl SourceType {
    pub fn from_ext(path: &Utf8Path) -> Self {
        match path.extension().unwrap() {
            "md" => Self::Markdown,
            "typ" => Self::Typst,
            _ => {
                panic!("Use me with .typ/.md file!");
            }
        }
    }

    pub fn compile(&self, root: &Utf8Path, file: &Utf8Path) {
        match self {
            Self::Markdown => {
                compile::compile_md(root, file);
            }
            Self::Typst => {
                match compile::compile_typ(root, file) {
                    Ok(_) => {
                        info!("OK");
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
    }

    #[cfg(feature="watch")]
    pub fn watch(&self, root: &Utf8Path, file: &Utf8Path) {
        watch::watch(self, root, file);
    }
}
