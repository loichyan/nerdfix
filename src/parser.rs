//! Parses font infomation from nerd font official cheat sheet.

use crate::{
    error,
    icon::{Icon, Indices},
};
use once_cell::sync::Lazy;
use regex::Regex;
use thisctx::WithContext;

pub fn parse(s: &str) -> error::Result<Vec<Icon>> {
    let s = s.trim_start();
    if s.starts_with('{') {
        Ok(serde_json::from_str::<Indices>(s)?.icons)
    } else {
        parse_cheat_sheet(s)
    }
}

fn parse_cheat_sheet(s: &str) -> error::Result<Vec<Icon>> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^ *"nf(old)?-(.+)": "(.+)",$"#).unwrap());

    enum State {
        Init,
        Matching,
    }

    let mut state = State::Init;
    let mut icons = None::<Vec<Icon>>;
    let mut n = 0;
    for line in s.lines() {
        n += 1;
        match state {
            State::Init => {
                if line == "const glyphs = {" {
                    state = State::Matching;
                    icons = Some(<_>::default());
                }
            }
            State::Matching => {
                if line == "}" {
                    break;
                }
                let caps = RE.captures(line).context(error::InvalidCheatSheet(n))?;
                icons.as_mut().unwrap().push(Icon {
                    name: caps.get(2).unwrap().as_str().to_owned(),
                    obsolete: caps.get(1).is_some(),
                    codepoint: crate::icon::parse_codepoint(caps.get(3).unwrap().as_str())?,
                });
            }
        }
    }

    icons.context(error::InvalidCheatSheet(0))
}

#[cfg(test)]
mod tests {
    const INDEX: &str = r#"{
    "METADATA": "v1",
    "cod-account": { "codepoint": "eb99" },
    "cod-activate_breakpoints": { "codepoint": "ea97" },
    "mdi-access_point": { "codepoint": "f501", "obsolete": true },
    "mdi-access_point_network": { "codepoint": "f502", "obsolete": true }
}"#;

    // Author: Ryan L McIntyre
    // License: MIT
    // Upstream: https://github.com/ryanoasis/nerd-fonts/blob/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md
    const CHEAT_SHEET: &str = r#"
const glyphs = {
    "nf-cod-account": "eb99",
    "nf-cod-activate_breakpoints": "ea97",
    "nfold-mdi-access_point": "f501",
    "nfold-mdi-access_point_network": "f502",
}
"#;

    #[test]
    fn parse_index() {
        let icons = super::parse(INDEX).unwrap();
        let expected = vec![
            icon!("cod-account", 0xeb99),
            icon!("cod-activate_breakpoints", 0xea97),
            icon!("mdi-access_point", 0xf501, true),
            icon!("mdi-access_point_network", 0xf502, true),
        ];
        assert_eq!(icons, expected);
    }

    #[test]
    fn parse_cheat_sheet() {
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
