use crate::{
    autocomplete::Autocompleter,
    cli::{Replace, UserInput},
    error,
    icon::{CachedIcon, Icon},
};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{self, termcolor::StandardStream},
};
use colored::Colorize;
use indexmap::IndexMap;
use inquire::InquireError;
use ngrammatic::{Corpus, CorpusBuilder};
use once_cell::unsync::OnceCell;
use std::{collections::HashMap, path::Path, rc::Rc};
use thisctx::{IntoError, WithContext};
use tracing::warn;

const SIMILARITY: f32 = 0.75;
const MAX_CHOICES: usize = 4;

pub type FstSet = fst::Set<Vec<u8>>;

#[derive(Default)]
pub struct Runtime {
    icons: Rc<IndexMap<String, Icon>>,
    index: OnceCell<HashMap<char, usize>>,
    corpus: OnceCell<Rc<Corpus>>,
    fst_set: OnceCell<Rc<FstSet>>,
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
                    let candidates = self.candidates(icon)?;
                    let diag = Diagnostic::warning()
                        .with_message(format!("Found obsolete icon U+{:X}", icon.codepoint as u32))
                        .with_labels(vec![Label::primary(file_id, start..end)
                            .with_message(format!("Icon '{}' is marked as obsolete", icon.name))])
                        .with_notes(self.diagnostic_notes(&candidates)?);
                    term::emit(&mut context.writer, &context.config, &context.files, &diag)
                        .context(error::Reporter)?;
                    // Autofix use history.
                    if let Some(&last) = context.history.get(&icon.codepoint) {
                        cprintln!("# Auto fix it using last input '{}'".green, last);
                        ch = last;
                    // Autofix use replacing.
                    } else if let Some(new) = self.try_replace(context, icon) {
                        cprintln!("# Auto replace it with '{}'".green, new);
                        ch = new;
                    } else {
                        // Input a new icon
                        if does_fix {
                            // Push all non-patched content.
                            let res = result.get_or_insert_with(|| content[..start].to_owned());
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
            .search(&icon.name, SIMILARITY)
            .into_iter()
            .enumerate()
            .filter_map(|(i, candi)| {
                if i >= MAX_CHOICES {
                    None
                } else {
                    Some(self.icons.get(&candi.text).unwrap())
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
                .with_help_message("<Tab> to autocomplete, <Esc> to cancel, <Ctrl-C> to abort")
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
            cprintln!("# Your input: {}".green, icon.codepoint);
            break Some(icon.codepoint);
        })
    }

    fn autocompleter(&self, candidates: usize) -> Autocompleter {
        Autocompleter {
            icons: self.icons.clone(),
            corpus: self.corpus().clone(),
            fst: self.fst_set().clone(),
            candidates,
            last: None,
        }
    }

    fn fst_set(&self) -> &Rc<FstSet> {
        self.fst_set.get_or_init(|| {
            Rc::new(FstSet::from_iter(self.good_icons().map(|(_, icon)| &icon.name)).unwrap())
        })
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

    fn corpus(&self) -> &Rc<Corpus> {
        self.corpus.get_or_init(|| {
            Rc::new(
                CorpusBuilder::default()
                    .fill(self.good_icons().map(|(_, icon)| icon.name.clone()))
                    .finish(),
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
        let icons = crate::parser::parse(&content).map_err(|e| e.with_path(path))?;
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
}

impl Default for CheckerContext {
    fn default() -> Self {
        Self {
            files: SimpleFiles::new(),
            writer: StandardStream::stderr(term::termcolor::ColorChoice::Always),
            config: term::Config::default(),
            history: HashMap::default(),
            replace: Vec::default(),
        }
    }
}
