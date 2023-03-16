#[macro_use]
mod util;
mod autocomplete;
mod cli;
mod error;
mod icon;
mod parser;

use autocomplete::Autocompleter;
use clap::Parser;
use cli::{Command, UserInput};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{self, termcolor::StandardStream},
};
use colored::Colorize;
use icon::{CachedIcon, Icon};
use inquire::InquireError;
use ngrammatic::{Corpus, CorpusBuilder};
use once_cell::unsync::OnceCell;
use std::{collections::HashMap, path::Path, rc::Rc};
use thisctx::{IntoError, WithContext};

static CACHED: &str = include_str!("./cached.txt");
const SIMILARITY: f32 = 0.75;
const MAX_CHOICES: usize = 7;

macro_rules! cprintln {
    ($fmt:literal $(,$args:expr)* $(,)?) => {
        cprintln!($fmt.red $(,$args)*);
    };
    ($fmt:literal.$color:ident $(,$args:expr)* $(,)?) => {
        println!("{}", format!($fmt $(,$args)*).$color());
    };
}

type FstSet = fst::Set<Vec<u8>>;

#[derive(Default)]
pub struct Runtime {
    icons: Vec<Icon>,
    good_icons: OnceCell<Rc<Vec<Icon>>>,
    index: OnceCell<HashMap<char, usize>>,
    name_index: OnceCell<Rc<HashMap<String, usize>>>,
    corpus: OnceCell<Rc<Corpus>>,
    fst_set: OnceCell<Rc<FstSet>>,
}

impl Runtime {
    pub fn load_cache(&mut self, path: &Path) -> error::Result<()> {
        let content = std::fs::read_to_string(path).context(error::Io(path))?;
        for (i, line) in content.lines().enumerate() {
            let icon = line
                .parse::<CachedIcon>()
                .map_err(|e| error::CorruptedCache(e, path, i).build())?;
            self.icons.push(icon.0);
        }
        Ok(())
    }

    pub fn load_cheat_sheet(&mut self, path: &Path) -> error::Result<()> {
        let content = std::fs::read_to_string(path).context(error::Io(path))?;
        // Skips yaml metadata.
        let Some(start) = content.find('<') else { return Ok(()) };
        self.icons.extend(parser::parse(&content[start..])?);
        Ok(())
    }

    pub fn load_inline_cache(&mut self, cached: &str) {
        for line in cached.lines() {
            let icon = line.parse::<CachedIcon>().unwrap();
            self.icons.push(icon.0);
        }
    }

    pub fn save_cache(&self, path: &Path) -> error::Result<()> {
        let mut content = String::default();
        for icon in self.icons.iter() {
            let icon = CachedIcon(icon);
            content.push_str(&format!("{icon}\n"));
        }
        std::fs::write(path, content).context(error::Io(path))?;
        Ok(())
    }

    pub fn check(
        &self,
        context: &mut CheckerContext,
        mut patched: Option<&mut String>,
        path: &Path,
    ) -> error::Result<()> {
        macro_rules! report {
            ($diag:expr) => {
                term::emit(&mut context.writer, &context.config, &context.files, $diag)
                    .context(error::Reporter)?;
            };
        }

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
                    let candidates = self.candidates(icon)?;
                    let diag = Diagnostic::warning()
                        .with_message(format!("Found obsolete icon U+{:X}", icon.codepoint as u32))
                        .with_labels(vec![Label::primary(file_id, start..end)
                            .with_message(format!("Icon '{}' is marked as obsolete", icon.name))]);
                    if let Some(&last) = context.history.get(&icon.codepoint) {
                        report!(&diag);
                        cprintln!("# Auto patch using last input '{}'".green, last);
                        ch = last;
                    } else {
                        let diag = diag.with_notes(self.diagnostic_notes(&candidates)?);
                        report!(&diag);
                        if let Some(patched) = &mut patched {
                            match self.prompt_input_icon(Some(&candidates)) {
                                Ok(Some(c)) => {
                                    ch = c;
                                    context.history.insert(icon.codepoint, ch);
                                }
                                Ok(None) => (),
                                Err(error::Error::Interrupted) => {
                                    patched.push_str(&content[start..]);
                                    return Ok(());
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
            if let Some(patched) = &mut patched {
                patched.push(ch);
            }
        }
        Ok(())
    }

    fn candidates(&self, icon: &Icon) -> error::Result<Vec<&Icon>> {
        Ok(self
            .corpus()
            .search(&icon.name, SIMILARITY)
            .into_iter()
            .enumerate()
            .filter_map(|(i, candi)| {
                if i >= MAX_CHOICES {
                    None
                } else {
                    let &candi = self.name_index().get(&candi.text).unwrap();
                    Some(&self.good_icons()[candi])
                }
            })
            .collect::<Vec<_>>())
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
        let candidates = candidates.unwrap_or(&[]);
        Ok(loop {
            let prompt = inquire::Text::new("Input an icon:")
                .with_help_message("Press <Tab> to autocomplete, <Esc> to cancel, <Ctrl-C> to quit")
                .with_autocomplete(self.autocompleter(candidates.len()));
            let input = match prompt.prompt() {
                Ok(t) => t,
                Err(InquireError::OperationCanceled) => break None,
                Err(InquireError::OperationInterrupted) => return Err(error::Interrupted.build()),
                Err(e) => return Err(error::Prompt.into_error(e)),
            };
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
                    if let Some(&icon) = self.name_index().get(&name) {
                        &self.good_icons()[icon]
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
            cprintln!("# Your input: {}".green, icon.codepoint);
            break Some(icon.codepoint);
        })
    }

    fn autocompleter(&self, candidates: usize) -> Autocompleter {
        Autocompleter {
            icons: self.good_icons().clone(),
            corpus: self.corpus().clone(),
            name_index: self.name_index().clone(),
            fst: self.fst_set().clone(),
            candidates,
            last: None,
        }
    }

    fn fst_set(&self) -> &Rc<FstSet> {
        self.fst_set.get_or_init(|| {
            Rc::new(FstSet::from_iter(self.good_icons().iter().map(|icon| &icon.name)).unwrap())
        })
    }

    fn good_icons(&self) -> &Rc<Vec<Icon>> {
        self.good_icons
            .get_or_init(|| Rc::new(self.icons.iter().filter(|i| !i.obsolete).cloned().collect()))
    }

    fn index(&self) -> &HashMap<char, usize> {
        self.index.get_or_init(|| {
            self.icons
                .iter()
                .enumerate()
                .map(|(i, icon)| (icon.codepoint, i))
                .collect()
        })
    }

    fn name_index(&self) -> &Rc<HashMap<String, usize>> {
        self.name_index.get_or_init(|| {
            Rc::new(
                self.good_icons()
                    .iter()
                    .enumerate()
                    .map(|(i, icon)| (icon.name.clone(), i))
                    .collect(),
            )
        })
    }

    fn corpus(&self) -> &Rc<Corpus> {
        self.corpus.get_or_init(|| {
            Rc::new(
                CorpusBuilder::default()
                    .fill(self.good_icons().iter().map(|icon| icon.name.clone()))
                    .finish(),
            )
        })
    }
}

pub struct CheckerContext {
    files: SimpleFiles<String, String>,
    writer: StandardStream,
    config: term::Config,
    history: HashMap<char, char>,
}

impl Default for CheckerContext {
    fn default() -> Self {
        Self {
            files: SimpleFiles::new(),
            writer: StandardStream::stderr(term::termcolor::ColorChoice::Always),
            config: term::Config::default(),
            history: HashMap::default(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let mut rt = Runtime::default();
    rt.load_inline_cache(CACHED);
    for path in args.cache.iter() {
        rt.load_cache(path)?;
    }
    for path in args.cheat_sheet.iter() {
        rt.load_cheat_sheet(path)?;
    }
    match args.cmd {
        Command::Cache { output } => rt.save_cache(&output)?,
        Command::Check { source } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                rt.check(&mut context, None, path)?;
            }
        }
        Command::Fix { source } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                let mut patched = String::default();
                rt.check(&mut context, Some(&mut patched), path)?;
                if inquire::Confirm::new("Are your sure to write the patched content?")
                    .prompt()
                    .unwrap_or(false)
                {
                    std::fs::write(path, patched).context(error::Io(path))?;
                }
            }
        }
        Command::Search {} => {
            rt.prompt_input_icon(None)?;
        }
    }
    Ok(())
}
