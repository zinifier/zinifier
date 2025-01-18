use camino::Utf8PathBuf;
use ecow::EcoString;
use snafu::prelude::*;

#[derive(Debug)]
pub struct DummyError(EcoString);

impl std::error::Error for DummyError {}

impl std::fmt::Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<EcoString> for DummyError {
    fn from(e: EcoString) -> DummyError {
        DummyError(e)
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Compilation for {document} failed due to error:\n{source}"))]
    Typst { document: Utf8PathBuf, source: DummyError },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum MarkdownError {
    #[snafu(display("Failed to compile Markdown zine {path} to PDF due to error:\n{source}"))]
    MDCompile { path: Utf8PathBuf, source: Error },
    #[snafu(display("Failed to save Typst export from Markdown file to {path} due to error:\n{source}"))]
    MDSave { path: Utf8PathBuf, source: std::io::Error },
}
