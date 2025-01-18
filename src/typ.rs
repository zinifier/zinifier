/// Sanitize markdown content for Tyspt consumption
/// 
/// - replace `@` with `\@`
/// - TODO
pub fn typst_escape(s: &str) -> String {
    s.replace("@", "\\@")
}

