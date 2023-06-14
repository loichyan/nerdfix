//! Parses font infomation from nerd font official cheat sheet.

use crate::{
    error,
    icon::{Cache, Icon},
};
use select::document::Document;
use thisctx::IntoError;

pub fn parse(s: &str) -> error::Result<Vec<Icon>> {
    if s.trim_start().starts_with('{') {
        Ok(serde_json::from_str::<Cache>(s)?.icons)
    } else if let Some(start) = s.find('<') {
        // TODO: support new cheat-sheet format
        let s = &s[start..];
        let mut parser = Parser::new(s);
        parser.parse()?;
        Ok(parser.icons)
    } else {
        error::InvalidCheatSheet.fail()
    }
}

struct Parser<'a> {
    document: Document,
    icons: Vec<Icon>,
    _source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            document: Document::from(s),
            icons: Default::default(),
            _source: s,
        }
    }

    pub fn parse(&mut self) -> error::Result<()> {
        use select::predicate::*;
        for node in self
            .document
            .find(Attr("id", "glyphCheatSheet").child(Element))
        {
            tryb!({
                let name = node
                    .find(Class("class-name").child(Text))
                    .next()?
                    .as_text()?
                    .trim();
                let name = name.strip_prefix("nf-").unwrap_or(name);
                let codepoint = node
                    .find(Class("codepoint").child(Text))
                    .next()?
                    .as_text()?;
                let codepoint = u32::from_str_radix(codepoint, 16).ok()?;
                let codepoint = char::from_u32(codepoint)?;
                let obsolete = tryb!({
                    node.find(Class("corner-text").child(Text))
                        .next()?
                        .as_text()
                });
                let obsolete = matches!(obsolete, Some("obsolete" | "removed"));
                self.icons.push(Icon {
                    name: name.to_owned(),
                    codepoint,
                    obsolete,
                });
                Some(())
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Author: Ryan L McIntyre
    // License: MIT
    // Upstream: https://github.com/ryanoasis/nerd-fonts/blob/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md
    const CHEAT_SHEET: &str = r#"
<div id="glyphCheatSheet" class="nerd-font-cheat-sheet">
  <div class="column">
    <div class="nf nf-cod-account center"></div>
    <div class="class-name">nf-cod-account</div><div title="Copy Hex Code to Clipboard" class="codepoint">eb99</div>
  </div>
  <div class="column">
    <div class="nf nf-cod-activate_breakpoints center"></div>
    <div class="class-name">nf-cod-activate_breakpoints</div><div title="Copy Hex Code to Clipboard" class="codepoint">ea97</div>
  </div>
  <div class="column">
    <span class="corner-red"></span><span class="corner-text">obsolete</span>
    <div class="nf nf-mdi-access_point center"></div>
    <div class="class-name">nf-mdi-access_point</div><div title="Copy Hex Code to Clipboard" class="codepoint">f501</div>
  </div>
  <div class="column">
    <span class="corner-red"></span><span class="corner-text">removed</span>
    <div class="nf nf-mdi-access_point_network center"></div>
    <div class="class-name">nf-mdi-access_point_network</div><div title="Copy Hex Code to Clipboard" class="codepoint">f502</div>
  </div>
</div>"#;

    #[test]
    fn parser() {
        let icons = super::parse(CHEAT_SHEET).unwrap();
        let expected = vec![
            icon!("cod-account", 0xeb99),
            icon!("cod-activate_breakpoints", 0xea97),
            icon!("mdi-access_point", 0xf501, true),
            icon!("mdi-access_point_network", 0xf502, true),
        ];
        assert_eq!(icons, expected);
    }
}
