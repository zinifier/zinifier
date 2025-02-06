use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, ValueEnum};

use crate::{error::*, path::RootPath, typ::CompileMode, watch, zine::ZineFile};

#[derive(Debug, Parser)]
struct Cli {
    action: Action,
    #[clap(short, long, default_value = "pdf")]
    mode: CompileMode,
    file: Utf8PathBuf,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Action {
    Compile,
    #[cfg(feature = "watch")]
    Watch,
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

    // We take a RootPath and not a simple path because we need the BaseDir context
    // to resolve themes etc...
    pub fn compile(&self, path: &RootPath, mode: CompileMode) -> Result<(), Error> {
        trace!("SourceType::compile({path:?}, {mode:?})");

        if path.path.is_dir() {
            panic!("Can only compile a .md or .typ file, not folder!");
        }

        let zine = ZineFile::new(path);

        let compiled_zine = match self {
            Self::Markdown => zine.compile_md()?,
            Self::Typst => zine.compile()?,
        };

        match mode {
            CompileMode::Png => {
                compiled_zine.to_png()?;
            }
            CompileMode::Pdf => {
                compiled_zine.to_pdf()?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "watch")]
    pub fn watch(&self, path: &RootPath) -> Result<(), Error> {
        watch::watch(self, path);

        Ok(())
    }
}
