use crate::{
    autocomplete::Autocompleter,
    cli::{IoPath, OutputFormat, UserInput},
    error,
    icon::{Database, Icon, Substitution, SubstitutionType, Substitutions},
    util::NGramSearcherExt,
};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term,
    term::termcolor::{ColorChoice, StandardStream},
};
use indexmap::IndexMap;
use inquire::InquireError;
use itertools::Itertools;
use once_cell::unsync::{Lazy, OnceCell};
use serde::Serialize;
use std::{collections::HashMap, io::Write, rc::Rc};
use thisctx::IntoError;
use tracing::info;

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
}

#[derive(Default)]
pub struct RuntimeBuilder {
    icons: IndexMap<String, Icon>,
    exact_sub: HashMap<String, String>,
    prefix_sub: Vec<Substitution>,
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
            if !self.icons.contains_key(&icon.name) {
                self.icons.insert(icon.name.clone(), icon);
            }
        }
        self.with_substitutions(input.substitutions);
        Ok(())
    }

    pub fn with_substitutions(&mut self, substitutions: Substitutions) {
        for sub in substitutions {
            match sub.ty {
                SubstitutionType::Exact => {
                    self.exact_sub.insert(sub.from, sub.to);
                }
                SubstitutionType::Prefix => self.prefix_sub.push(sub),
            }
        }
    }

    pub fn build(self) -> Runtime {
        let Self {
            icons,
            exact_sub,
            prefix_sub,
        } = self;
        Runtime {
            icons: Rc::new(icons),
            prefix_sub,
            exact_sub,
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
        input: &IoPath,
        should_fix: bool,
    ) -> error::Result<Option<String>> {
        info!("Check input file from '{}'", input);
        let mut result = None::<String>;
        let content = input.read_to_string()?;
        let file_id = context.files.add(input.to_string(), content);
        let content = context.files.get(file_id).unwrap().source();
        for (start, mut ch) in content.char_indices() {
            if let Some(icon) = self
                .index()
                .get(&ch)
                .map(|&i| &self.icons[i])
                .filter(|icon| icon.obsolete)
            {
                let mut end = start + 1;
                while !content.is_char_boundary(end) {
                    end += 1;
                }
                let candidates = Lazy::new(|| self.get_candidates(icon));
                match context.format {
                    OutputFormat::Console => {
                        let diag = Diagnostic::new(Severity::Info.into())
                            .with_message(format!(
                                "Found obsolete icon U+{:X}",
                                icon.codepoint as u32
                            ))
                            .with_labels(vec![Label::primary(file_id, start..end).with_message(
                                format!("Icon '{}' is marked as obsolete", icon.name),
                            )])
                            .with_notes(self.diagnostic_notes(&candidates)?);
                        term::emit(&mut context.writer, &context.config, &context.files, &diag)?;
                    }
                    OutputFormat::Json => {
                        let diag = DiagOutput {
                            severity: Severity::Info,
                            path: input.to_string(),
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
                if should_fix {
                    // Push all non-patched content.
                    let res = result.get_or_insert_with(|| content[..start].to_owned());
                    if let Some(&last) = context.history.get(&icon.codepoint) {
                        msginfo!("# Autofix with the last input '{}'", last);
                        ch = last;
                    } else if let Some(new) = self.get_substitutions(icon).next() {
                        msginfo!("# Autofix with substitution '{}'", new.codepoint);
                        ch = new.codepoint;
                    } else if context.select_first {
                        if let Some(&first) = candidates.first() {
                            msginfo!("# Autofix with the first suggestion '{}'", first.codepoint);
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
                                res.push_str(&content[start..]);
                                return Ok(result);
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
            // Save the new character.
            if let Some(res) = result.as_mut() {
                res.push(ch);
            }
        }

        Ok(result)
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
        self.exact_sub
            .get(&icon.name)
            .into_iter()
            .filter_map(|name| self.icons.get(name))
            .chain(self.prefix_sub.iter().filter_map(|rep| {
                let name = icon.name.strip_prefix(&rep.from)?;
                self.icons.get(&format!("{}{name}", rep.to))
            }))
    }

    fn diagnostic_notes(&self, candidates: &[&Icon]) -> error::Result<Vec<String>> {
        let mut notes = Vec::default();

        if !candidates.is_empty() {
            let mut s = String::from("You could replace it with:\n");
            for (i, &candi) in candidates.iter().enumerate() {
                s.push_str(&format!(
                    "    {}. {} U+{:05X} {}\n",
                    i + 1,
                    candi.codepoint,
                    candi.codepoint as u32,
                    &candi.name
                ));
            }
            notes.push(s);
        }
        Ok(notes)
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
    pub files: SimpleFiles<String, String>,
    pub writer: StandardStream,
    pub config: term::Config,
    pub history: HashMap<char, char>,
    pub format: OutputFormat,
    pub write: bool,
    pub select_first: bool,
}

impl Default for CheckerContext {
    fn default() -> Self {
        Self {
            files: SimpleFiles::new(),
            writer: StandardStream::stderr(ColorChoice::Always),
            config: term::Config::default(),
            history: HashMap::default(),
            format: OutputFormat::default(),
            write: false,
            select_first: false,
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
    Hint,
}

impl From<Severity> for codespan_reporting::diagnostic::Severity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Info => Self::Note,
            Severity::Hint => Self::Help,
        }
    }
}
