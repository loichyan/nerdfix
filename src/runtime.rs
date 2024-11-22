use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::rc::Rc;

use indexmap::IndexMap;
use inquire::InquireError;
use itertools::Itertools;
use miette::{Diagnostic, ReportHandler};
use once_cell::unsync::{Lazy, OnceCell};
use serde::Serialize;
use thisctx::IntoError;
use tracing::{info, warn};

use crate::autocomplete::Autocompleter;
use crate::cli::{IoPath, OutputFormat, UserInput};
use crate::error;
use crate::icon::{parse_codepoint, Database, Icon, Substitution, SubstitutionType, Substitutions};
use crate::utils::NGramSearcherExt as _;

const ARITY: usize = 3;
const PAD_LEN: usize = 2;
const WARP: f32 = 3.0;
const THRESHOLD: f32 = 0.7;
const MAX_CHOICES: usize = 4;

pub type NGram = noodler::NGram<(String, usize)>;

#[derive(Default)]
pub struct Runtime {
    icons: Rc<IndexMap<String, Icon>>,
    index: OnceCell<HashMap<char, usize>>,
    corpus: OnceCell<Rc<NGram>>,
    exact_sub: HashMap<String, String>,
    prefix_sub: Vec<Substitution>,
    codepoint_sub: HashMap<char, char>,
}

#[derive(Default)]
pub struct RuntimeBuilder {
    icons: IndexMap<String, Icon>,
    exact_sub: HashMap<String, String>,
    prefix_sub: Vec<Substitution>,
    codepoint_sub: HashMap<char, char>,
}

impl RuntimeBuilder {
    pub fn load_db_from(&mut self, input: &IoPath) -> error::Result<()> {
        info!("Load input from '{}'", input);
        let content = input.read_to_string()?;
        self.load_db(&content)?;
        Ok(())
    }

    pub fn load_db(&mut self, content: &str) -> error::Result<()> {
        let input = crate::parser::parse(content)?;
        for icon in input.icons {
            self.icons.insert(icon.name.clone(), icon);
        }
        self.with_substitutions(input.substitutions)?;
        Ok(())
    }

    pub fn with_substitutions(&mut self, substitutions: Substitutions) -> error::Result<()> {
        for sub in substitutions {
            match sub.ty {
                SubstitutionType::Exact => {
                    self.exact_sub.insert(sub.from, sub.to);
                }
                SubstitutionType::Prefix => self.prefix_sub.push(sub),
                SubstitutionType::Codepoint => {
                    let from = parse_codepoint(&sub.from)?;
                    let to = parse_codepoint(&sub.to)?;
                    self.codepoint_sub.insert(from, to);
                }
            }
        }
        Ok(())
    }

    pub fn build(self) -> Runtime {
        let Self {
            icons,
            exact_sub,
            prefix_sub,
            codepoint_sub,
        } = self;
        Runtime {
            icons: Rc::new(icons),
            prefix_sub,
            exact_sub,
            codepoint_sub,
            ..Default::default()
        }
    }
}

impl Runtime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::default()
    }

    pub fn dump_db(&self, output: &IoPath) -> error::Result<()> {
        info!("Save indices to '{}'", output);
        let indices = Database {
            icons: self.icons.values().cloned().collect(),
            substitutions: self
                .exact_sub
                .iter()
                .map(|(k, v)| Substitution {
                    ty: SubstitutionType::Exact,
                    from: k.clone(),
                    to: v.clone(),
                })
                .chain(self.prefix_sub.iter().cloned())
                .collect(),
        };
        let content = serde_json::to_string(&indices).unwrap();
        output.write_str(&content)?;
        Ok(())
    }

    pub fn check(
        &self,
        context: &mut CheckerContext,
        path: &IoPath,
        mut output: Option<&mut String>,
    ) -> error::Result<bool> {
        info!("Check input from '{}'", path);

        if context.size_limit != 0 && path.file_size()?.unwrap_or(0) >= context.size_limit {
            warn!("Skip large file '{}'", path);
            return Ok(false);
        }

        let mut reader = path.open()?;
        let Some(mut line) = reader.next_line()? else {
            return Ok(false);
        };

        if !context.include_binary && line.content_type() == content_inspector::ContentType::BINARY
        {
            warn!("Skip binary file '{}'", path);
            return Ok(false);
        }

        let mut updated = false;
        loop {
            let text = std::str::from_utf8(line.contents())?;
            for (start, mut ch) in text.char_indices() {
                let start = line.offset_of(start);

                if let Some(icon) = self
                    .index()
                    .get(&ch)
                    .map(|&i| &self.icons[i])
                    .filter(|icon| icon.obsolete)
                {
                    let end = start + ch.len_utf8();
                    let candidates = Lazy::new(|| self.get_candidates(icon));
                    match context.format {
                        OutputFormat::Console => {
                            let diag = error::ObsoleteIcon {
                                input: &line,
                                path,
                                icon,
                                span: (start, end),
                                candidates: &candidates,
                            };
                            writeln!(
                                &mut context.writer,
                                "{:?}",
                                DiagReporter {
                                    handler: &*context.handler,
                                    diag: &diag,
                                }
                            )?;
                        }
                        OutputFormat::Json => {
                            let diag = DiagOutput {
                                severity: Severity::Info,
                                path: path.to_string(),
                                ty: DiagType::Obsolete {
                                    span: (start, end),
                                    name: icon.name.clone(),
                                    codepoint: icon.codepoint.into(),
                                },
                            };
                            writeln!(
                                &mut context.writer,
                                "{}",
                                serde_json::to_string(&diag).unwrap()
                            )?;
                        }
                    }
                    if let Some(output) = output.as_mut() {
                        if let Some(&last) = context.history.get(&icon.codepoint) {
                            msginfo!("# Autofix with the last input '{}'", last);
                            ch = last;
                        } else if let Some(new) = self.get_substitutions(icon).next() {
                            msginfo!("# Autofix with substitution '{}'", new.codepoint);
                            ch = new.codepoint;
                        } else if context.select_first {
                            if let Some(&first) = candidates.first() {
                                msginfo!(
                                    "# Autofix with the first suggestion '{}'",
                                    first.codepoint
                                );
                                ch = first.codepoint;
                            } else {
                                error!(
                                    "Cannot find a similar icon for '{} {}'",
                                    icon.codepoint, icon.name
                                );
                            }
                        } else {
                            // Input a new icon
                            match self.prompt_input_icon(Some(&candidates)) {
                                Ok(Some(new)) => {
                                    context.history.insert(icon.codepoint, new);
                                    ch = new;
                                }
                                Ok(None) => (),
                                Err(error::Error::Interrupted) => {
                                    output.push_str(&text[start..]);
                                    return Ok(updated);
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                    updated = true;
                }
                // Save the character.
                if let Some(output) = output.as_mut() {
                    output.push(ch);
                }
            }

            if let Some(l) = reader.next_line()? {
                line = l;
            } else {
                break;
            }
        }

        Ok(updated)
    }

    pub fn get_icon_by_codepoint(&self, c: char) -> Option<&Icon> {
        self.index().get(&c).map(|&i| &self.icons[i])
    }

    pub fn get_icon_by_name(&self, name: &str) -> Option<&Icon> {
        self.icons.get(name)
    }

    fn get_candidates<'a>(&'a self, icon: &'a Icon) -> Vec<&'a Icon> {
        self.corpus()
            .searcher(&icon.name)
            .exec_sorted_stable()
            .map(|((_, id), _)| &self.icons[*id])
            .take(MAX_CHOICES)
            .collect()
    }

    fn get_substitutions<'a>(&'a self, icon: &'a Icon) -> impl 'a + Iterator<Item = &'a Icon> {
        std::iter::empty()
            .chain(
                self.codepoint_sub
                    .get(&icon.codepoint)
                    .and_then(|c| self.index().get(c))
                    .map(|&i| &self.icons[i]),
            )
            .chain(
                self.exact_sub
                    .get(&icon.name)
                    .and_then(|name| self.icons.get(name)),
            )
            .chain(self.prefix_sub.iter().filter_map(|rep| {
                let name = icon.name.strip_prefix(&rep.from)?;
                self.icons.get(&format!("{}{name}", rep.to))
            }))
    }

    pub fn prompt_input_icon(&self, candidates: Option<&[&Icon]>) -> error::Result<Option<char>> {
        fn fmt_input(s: &str) -> &str {
            s.split_ascii_whitespace().next().unwrap_or("")
        }

        let candidates = candidates.unwrap_or(&[]);
        Ok(loop {
            let prompt = inquire::Text::new("Input an icon:")
                .with_formatter(&|s| fmt_input(s).to_owned())
                .with_help_message("(Tab) to autocomplete, (Esc) to cancel, (Ctrl_C) to abort")
                .with_autocomplete(self.autocompleter(candidates.len()));
            let input = match prompt.prompt() {
                Ok(t) => t,
                Err(InquireError::OperationCanceled) => break None,
                Err(InquireError::OperationInterrupted) => return Err(error::Interrupted.build()),
                Err(e) => return Err(e.into()),
            };
            let input = fmt_input(&input);
            let input = match input.parse::<UserInput>() {
                Ok(t) => t,
                Err(error::Error::InvalidInput) => {
                    msgerror!("# Invalid input!");
                    continue;
                }
                Err(e) => return Err(e),
            };
            let icon = match input {
                UserInput::Name(name) => {
                    if let Some(icon) = self.icons.get(&name) {
                        icon
                    } else {
                        msgerror!("# '{}' is not a valid icon name!", name);
                        continue;
                    }
                }
                UserInput::Char(ch) => match self.index().get(&ch) {
                    Some(&icon) if !self.icons[icon].obsolete => &self.icons[icon],
                    _ => {
                        msgerror!("# '{}' is not a valid icon!", ch);
                        continue;
                    }
                },
                UserInput::Candidate(i) => match candidates.get(i - 1) {
                    Some(&icon) => icon,
                    None => {
                        msgerror!("# '{}' is not a valid candidate!", i);
                        continue;
                    }
                },
                UserInput::Codepoint(hex) => {
                    match char::from_u32(hex).and_then(|ch| self.index().get(&ch)) {
                        Some(&icon) if !self.icons[icon].obsolete => &self.icons[icon],
                        _ => {
                            msgerror!("# 'U+{:X}' is not a valid icon codepoint!", hex);
                            continue;
                        }
                    }
                }
            };
            msginfo!("# Your input: {} {}", icon.codepoint, icon.name);
            break Some(icon.codepoint);
        })
    }

    fn autocompleter(&self, candidates: usize) -> Autocompleter {
        Autocompleter {
            icons: self.icons.clone(),
            corpus: self.corpus().clone(),
            candidates,
            last: None,
        }
    }

    fn good_icons(&self) -> impl Iterator<Item = (usize, &Icon)> {
        self.icons
            .values()
            .enumerate()
            .filter(|(_, icon)| !icon.obsolete)
    }

    fn index(&self) -> &HashMap<char, usize> {
        self.index.get_or_init(|| {
            self.icons
                .values()
                .enumerate()
                .map(|(i, icon)| (icon.codepoint, i))
                .collect()
        })
    }

    fn corpus(&self) -> &Rc<NGram> {
        self.corpus.get_or_init(|| {
            Rc::new(
                NGram::builder()
                    .arity(ARITY)
                    .pad_len(PAD_LEN)
                    .warp(WARP)
                    .threshold(THRESHOLD)
                    .build()
                    .fill(
                        self.good_icons()
                            .map(|(i, icon)| (icon.name.to_owned(), i))
                            .sorted_by(|(a, _), (b, _)| a.cmp(b)),
                    ),
            )
        })
    }
}

pub struct CheckerContext {
    pub writer: Box<dyn std::io::Write>,
    pub handler: Box<dyn ReportHandler>,
    pub history: HashMap<char, char>,
    pub format: OutputFormat,
    pub write: bool,
    pub select_first: bool,
    pub include_binary: bool,
    pub size_limit: u64,
}

impl Default for CheckerContext {
    fn default() -> Self {
        Self {
            writer: Box::new(std::io::stderr()),
            handler: Box::new(miette::MietteHandler::new()),
            history: HashMap::default(),
            format: OutputFormat::default(),
            write: false,
            select_first: false,
            include_binary: false,
            size_limit: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DiagOutput {
    pub severity: Severity,
    pub path: String,
    #[serde(flatten)]
    pub ty: DiagType,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DiagType {
    Obsolete {
        span: (usize, usize),
        name: String,
        codepoint: u32,
    },
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl From<Severity> for miette::Severity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Info => Self::Advice,
        }
    }
}

struct DiagReporter<'a> {
    handler: &'a dyn ReportHandler,
    diag: &'a dyn Diagnostic,
}

impl fmt::Debug for DiagReporter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handler.debug(self.diag, f)
    }
}
