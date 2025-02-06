#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum CompileMode {
    Png,
    Pdf,
}

/// Sanitize markdown content for Tyspt consumption
///
/// - replace `@` with `\@`
/// - TODO
pub fn typst_escape(s: &str) -> String {
    s.replace("@", "\\@")
}
