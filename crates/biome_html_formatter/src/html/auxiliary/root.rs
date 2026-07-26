use crate::prelude::*;
use crate::verbatim::format_html_trailing_comments;
use biome_formatter::write;
use biome_html_syntax::{HtmlRoot, HtmlRootFields};
use biome_rowan::AstNode;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatHtmlRoot;
impl FormatNodeRule<HtmlRoot> for FormatHtmlRoot {
    fn fmt_fields(&self, node: &HtmlRoot, f: &mut HtmlFormatter) -> FormatResult<()> {
        let HtmlRootFields {
            html,
            bom_token,
            directive,
            frontmatter,
            eof_token,
        } = node.as_fields();

        if let Some(bom) = bom_token {
            bom.format().fmt(f)?;
        }

        if let Some(frontmatter) = frontmatter {
            write!(f, [frontmatter.format(), empty_line()])?;
        }

        if let Some(directive) = directive {
            directive.format().fmt(f)?;
        }

        html.format().fmt(f)?;

        write!(f, [format_removed(&eof_token?)])?;

        Ok(())
    }

    fn fmt_trailing_comments(&self, node: &HtmlRoot, f: &mut HtmlFormatter) -> FormatResult<()> {
        // A comment that trails the last element of the document is attached to
        // the root, so the closing line break has to be written after it rather
        // than at the end of the fields.
        format_html_trailing_comments(node.syntax()).fmt(f)?;

        if f.options().trailing_newline().value() {
            write!(f, [hard_line_break()])?;
        }

        Ok(())
    }
}
