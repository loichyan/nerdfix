use crate::{
    autocomplete::Autocompleter,
    cli::{OutputFormat, Replace, UserInput},
    error::{self, ResultTExt},
    icon::{CachedIcon, Icon},
    util::{NGramSearcherExt, TryLazy},
};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{self, termcolor::StandardStream},
};
use colored::Colorize;
use indexmap::IndexMap;
use inquire::InquireError;
use itertools::Itertools;
use once_cell::unsync::OnceCell;
use serde::Serialize;
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
};
use thisctx::{IntoError, WithContext};
use tracing::warn;

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
}

impl Runtime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::default()
    }

    pub fn save_cache(&self, path: &Path) -> error::Result<()> {
        let mut content = String::from("nerdfix v1\n");
        for icon in self.icons.values() {
            let icon = CachedIcon(icon);
            content.push_str(&format!("{icon}\n"));
        }
        std::fs::write(path, content).context(error::Io(path))?;
        Ok(())
    }

    pub fn check(
        &self,
        context: &mut CheckerContext,
        path: &Path,
        does_fix: bool,
    ) -> error::Result<Option<String>> {
        let mut result = None::<String>;
        let content = std::fs::read_to_string(path).context(error::Io(path))?;
        let file_id = context.files.add(path.display().to_string(), content);
        let content = context.files.get(file_id).unwrap().source();
        for (start, mut ch) in content.char_indices() {
            if let Some(&icon) = self.index().get(&ch) {
                let icon = &self.icons[icon];
                if icon.obsolete {
                    let mut end = start + 1;
                    while !content.is_char_boundary(end) {
                        end += 1;
                    }
                    let candidates = TryLazy::new(|| self.candidates(icon));
                    match context.format {
                        OutputFormat::Console => {
                            let diag = Diagnostic::new(Severity::Info.into())
                                .with_message(format!(
                                    "Found obsolete icon U+{:X}",
                                    icon.codepoint as u32
                                ))
                                .with_labels(vec![Label::primary(file_id, start..end)
                                    .with_message(format!(
                                        "Icon '{}' is marked as obsolete",
                                        icon.name
                                    ))])
                                .with_notes(self.diagnostic_notes(candidates.get()?)?);
                            term::emit(
                                &mut context.writer,
                                &context.config,
                                &context.files,
                                &diag,
                            )?;
                        }
                        OutputFormat::Json => {
                            let diag = DiagOutput {
                                severity: Severity::Info,
                                path: path.to_owned(),
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
                            )
                            .context(error::Io(error::Stdio))?;
                        }
                    }
                    if does_fix {
                        // Push all non-patched content.
                        let res = result.get_or_insert_with(|| content[..start].to_owned());
                        // Autofix use history.
                        if let Some(&last) = context.history.get(&icon.codepoint) {
                            cprintln!("# Auto fix it using last input '{}'".green, last);
                            ch = last;
                        // Autofix use replacing.
                        } else if let Some(new) = self.try_replace(context, icon) {
                            cprintln!("# Auto replace it with '{}'".green, new);
                            ch = new;
                        // Input a new icon
                        } else {
                            match self.prompt_input_icon(Some(candidates.get()?)) {
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
            }
            // Save the new character.
            if let Some(res) = result.as_mut() {
                res.push(ch);
            }
        }

        Ok(result)
    }

    fn try_replace(&self, ctx: &CheckerContext, icon: &Icon) -> Option<char> {
        for rep in ctx.replace.iter() {
            let Some(name) = icon.name.strip_prefix(&rep.from) else { continue };
            let Some(new_icon) = self.icons.get(&format!("{}{name}", rep.to)) else {
                warn!("{} cannot be replaced with '{}*'", icon.name, rep.to);
                continue;
            };
            return Some(new_icon.codepoint);
        }
        None
    }

    fn candidates(&self, icon: &Icon) -> error::Result<Vec<&Icon>> {
        Ok(self
            .corpus()
            .searcher(&icon.name)
            .exec_sorted_stable()
            .map(|((_, id), _)| &self.icons[*id])
            .take(MAX_CHOICES)
            .collect_vec())
    }

    fn diagnostic_notes(&self, candidates: &[&Icon]) -> error::Result<Vec<String>> {
        let mut notes = Vec::default();

        if !candidates.is_empty() {
            let mut s = String::from("You could replace it with:\n");
            for (i, &candi) in candidates.iter().enumerate() {
                s.push_str(&format!(
                    "    {}. {} U+{:X} {}\n",
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
                .with_help_message("(Tab) to autocomplete, (Esc) to cancel, (Ctrl-C) to abort")
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
                    cprintln!("# Invalid input!");
                    continue;
                }
                Err(e) => return Err(e),
            };
            let icon = match input {
                UserInput::Name(name) => {
                    if let Some(icon) = self.icons.get(&name) {
                        icon
                    } else {
                        cprintln!("# '{}' is not a valid icon name!", name);
                        continue;
                    }
                }
                UserInput::Char(ch) => match self.index().get(&ch) {
                    Some(&icon) if !self.icons[icon].obsolete => &self.icons[icon],
                    _ => {
                        cprintln!("# '{}' is not a valid icon!", ch);
                        continue;
                    }
                },
                UserInput::Candidate(i) => match candidates.get(i - 1) {
                    Some(&icon) => icon,
                    None => {
                        cprintln!("# '{}' is not a valid candidate!", i);
                        continue;
                    }
                },
                UserInput::Codepoint(hex) => {
                    match char::from_u32(hex).and_then(|ch| self.index().get(&ch)) {
                        Some(&icon) if !self.icons[icon].obsolete => &self.icons[icon],
                        _ => {
                            cprintln!("# 'U+{:X}' is not a valid icon codepoint!", hex);
                            continue;
                        }
                    }
                }
            };
            cprintln!("# Your input: {} {}".green, icon.codepoint, icon.name);
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

#[derive(Default)]
pub struct RuntimeBuilder {
    icons: IndexMap<String, Icon>,
}

impl RuntimeBuilder {
    pub fn load_input(&mut self, path: &Path) -> error::Result<()> {
        let content = std::fs::read_to_string(path).context(error::Io(path))?;
        let icons = crate::parser::parse(&content).with_path(path)?;
        for icon in icons {
            self.add_icon(icon);
        }
        Ok(())
    }

    pub fn load_cache(&mut self, cached: &str) {
        for icon in crate::parser::parse(cached).unwrap() {
            self.add_icon(icon);
        }
    }

    pub fn build(self) -> Runtime {
        Runtime {
            icons: Rc::new(self.icons),
            ..Default::default()
        }
    }

    fn add_icon(&mut self, icon: Icon) {
        if !self.icons.contains_key(&icon.name) {
            self.icons.insert(icon.name.clone(), icon);
        }
    }
}

pub struct CheckerContext {
    pub files: SimpleFiles<String, String>,
    pub writer: StandardStream,
    pub config: term::Config,
    pub history: HashMap<char, char>,
    pub replace: Vec<Replace>,
    pub yes: bool,
    pub format: OutputFormat,
}

impl Default for CheckerContext {
    fn default() -> Self {
        Self {
            files: SimpleFiles::new(),
            writer: StandardStream::stdout(term::termcolor::ColorChoice::Always),
            config: term::Config::default(),
            history: HashMap::default(),
            replace: Vec::default(),
            yes: false,
            format: OutputFormat::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DiagOutput {
    pub severity: Severity,
    pub path: PathBuf,
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
