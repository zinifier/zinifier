use camino::Utf8PathBuf;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "No basedir (containing content/themes folders) found in parent tree of {path}"
    ))]
    NoBaseDir { path: Utf8PathBuf },
    #[snafu(display("Typst compilation for {path} failed. See errors/warnings above."))]
    Typst { path: Utf8PathBuf },
    #[snafu(display("Failed to write PDF file to {path} due to error:\n{source}"))]
    PDFWrite {
        path: Utf8PathBuf,
        source: std::io::Error,
    },
    #[snafu(display(
        "Failed to save Typst export from Markdown file to {path} due to error:\n{source}"
    ))]
    MDSave {
        path: Utf8PathBuf,
        source: std::io::Error,
    },
}
