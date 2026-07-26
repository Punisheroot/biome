use crate::prelude::*;
use biome_formatter::{FormatRuleWithOptions, format_args, write};
use biome_html_syntax::{HtmlString, HtmlStringFields};
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatHtmlString {
    compact: bool,
}
impl FormatNodeRule<HtmlString> for FormatHtmlString {
    fn fmt_fields(&self, node: &HtmlString, f: &mut HtmlFormatter) -> FormatResult<()> {
        let HtmlStringFields { value_token } = node.as_fields();

        if self.compact {
            let value_token = value_token.clone()?;
            return format_removed(&value_token).fmt(f);
        }

        if let Ok(value) = value_token.as_ref() {
            let value_text = value.text_trimmed();
            let quoted = requote(value_text);

            // Rewriting the token loses the source mapping of everything
            // inside it, so it is only worth doing when the quoting really
            // does change.
            if quoted != value_text {
                return write!(
                    f,
                    [format_replaced(
                        value,
                        &text(&quoted, Some(value.text_trimmed_range().start()))
                    )]
                );
            }

            // An unquoted value still has to gain its quotes, and that much can
            // be done without giving up the mapping.
            if !is_quoted(value_text) {
                return write!(
                    f,
                    [format_replaced(
                        value,
                        &group(&format_args![
                            token("\""),
                            located_token_text(value, value.text_trimmed_range()),
                            token("\""),
                        ])
                    )]
                );
            }
        }

        write!(f, [value_token.format()])
    }
}

/// Whether the attribute value carries its own quotes, as opposed to being
/// written bare such as `<div class=header>`.
fn is_quoted(value_text: &str) -> bool {
    let mut characters = value_text.chars();
    // A lone quote leaves `next_back` empty, so a one character value is
    // correctly not treated as quoted.
    matches!(
        (characters.next(), characters.next_back()),
        (Some('"'), Some('"')) | (Some('\''), Some('\''))
    )
}

/// Rewrites an attribute value with the quote character that costs the fewest
/// escapes, matching Prettier.
///
/// Double quotes are preferred, and give way to single quotes only when the
/// value holds more of them than it holds single quotes. Whichever character
/// ends up as the delimiter is the only one that stays escaped, so a value that
/// was written as `'&apos;&quot;'` comes back as `"'&quot;"`.
fn requote(value_text: &str) -> String {
    let inner = if is_quoted(value_text) {
        &value_text[1..value_text.len() - 1]
    } else {
        value_text
    };

    let unescaped = inner.replace("&apos;", "'").replace("&quot;", "\"");
    let double_quotes = unescaped.matches('"').count();
    let single_quotes = unescaped.matches('\'').count();

    if double_quotes > single_quotes {
        std::format!("'{}'", unescaped.replace('\'', "&apos;"))
    } else {
        std::format!("\"{}\"", unescaped.replace('"', "&quot;"))
    }
}

impl FormatRuleWithOptions<HtmlString> for FormatHtmlString {
    type Options = bool;
    fn with_options(mut self, options: Self::Options) -> Self {
        self.compact = options;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::requote;

    #[test]
    fn prefers_double_quotes() {
        assert_eq!(requote("\"abc\""), "\"abc\"");
        assert_eq!(requote("'abc'"), "\"abc\"");
        assert_eq!(requote("abc"), "\"abc\"");
    }

    #[test]
    fn gives_way_to_single_quotes_only_when_they_cost_less() {
        // One double quote inside, none single: single quotes cost less.
        assert_eq!(requote("'123 \" 456'"), "'123 \" 456'");
        // One single quote inside, none double: double quotes still win.
        assert_eq!(requote("\"123 ' 456\""), "\"123 ' 456\"");
        // A tie goes to the preferred quote.
        assert_eq!(requote("'&apos;&quot;'"), "\"'&quot;\"");
    }

    #[test]
    fn unescapes_the_quote_that_is_no_longer_the_delimiter() {
        assert_eq!(requote("\"&apos;\""), "\"'\"");
        assert_eq!(requote("'&quot;&quot;'"), "'\"\"'");
    }

    #[test]
    fn escapes_the_delimiter_that_was_written_literally() {
        assert_eq!(requote("'a \" b'"), "'a \" b'");
        assert_eq!(requote("'a \" b \" c \" d'"), "'a \" b \" c \" d'");
    }

    #[test]
    fn leaves_other_entities_alone() {
        assert_eq!(requote("\"&amp;&lt;&#39;\""), "\"&amp;&lt;&#39;\"");
    }
}
