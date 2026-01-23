//! XML utilities.

/// Escapes special characters for XML content.
///
/// Escapes the five predefined XML entities:
/// - `&` -> `&amp;`
/// - `<` -> `&lt;`
/// - `>` -> `&gt;`
/// - `"` -> `&quot;`
/// - `'` -> `&apos;`
///
/// # Examples
///
/// ```
/// use agent_skills_cli::xml::escape_xml;
///
/// assert_eq!(escape_xml("foo & bar"), "foo &amp; bar");
/// assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
/// ```
#[must_use]
pub fn escape_xml(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_ampersand() {
        assert_eq!(escape_xml("foo & bar"), "foo &amp; bar");
    }

    #[test]
    fn escapes_less_than() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
    }

    #[test]
    fn escapes_greater_than() {
        assert_eq!(escape_xml("a > b"), "a &gt; b");
    }

    #[test]
    fn escapes_double_quote() {
        assert_eq!(escape_xml(r#"say "hello""#), "say &quot;hello&quot;");
    }

    #[test]
    fn escapes_single_quote() {
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn escapes_multiple_special_chars() {
        assert_eq!(
            escape_xml("<tag attr=\"val\">text & more</tag>"),
            "&lt;tag attr=&quot;val&quot;&gt;text &amp; more&lt;/tag&gt;"
        );
    }

    #[test]
    fn preserves_normal_text() {
        assert_eq!(escape_xml("hello world 123"), "hello world 123");
    }

    #[test]
    fn handles_empty_string() {
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn handles_unicode() {
        assert_eq!(escape_xml("hello & cafe"), "hello &amp; cafe");
    }
}
