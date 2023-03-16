//! Autocompletion and fuzzy search for nerd fonts.

use crate::{icon::Icon, FstSet};
use fst::{Automaton, IntoStreamer, Streamer};
use inquire::Autocomplete;
use itertools::Itertools;
use ngrammatic::Corpus;
use std::{collections::HashMap, rc::Rc};

const SIMILARITY: f32 = 0.4;
const MAX_CONTAINS: usize = 30;

#[derive(Clone)]
pub struct Autocompleter {
    pub(crate) icons: Rc<Vec<Icon>>,
    pub(crate) corpus: Rc<Corpus>,
    pub(crate) name_index: Rc<HashMap<String, usize>>,
    pub(crate) fst: Rc<FstSet>,
    pub(crate) candidates: usize,
    pub(crate) last: Option<String>,
}

impl Autocomplete for Autocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let suggestions = if input.is_empty() {
            (0..self.candidates)
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
        } else {
            let mut stream = self
                .fst
                .search(Contains(fst::automaton::Str::new(input)))
                .into_stream();
            self.corpus
                .search(input, SIMILARITY)
                .into_iter()
                .map(|candi| self.new_suggestion(&candi.text))
                .chain(
                    std::iter::from_fn(|| {
                        stream
                            .next()
                            .map(|s| self.new_suggestion(std::str::from_utf8(s).unwrap()))
                    })
                    .take(MAX_CONTAINS),
                )
                .unique()
                .collect::<Vec<_>>()
        };
        self.last = suggestions.first().cloned();
        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(highlighted_suggestion
            .as_ref()
            .or(self.last.as_ref())
            .map(|s| {
                s.split_once(' ')
                    .map(|(_, s)| s.to_owned())
                    .unwrap_or_else(|| s.clone())
            }))
    }
}

impl Autocompleter {
    fn new_suggestion(&self, name: &str) -> String {
        let &icon = self.name_index.get(name).unwrap();
        let icon = &self.icons[icon];
        format!("{} {}", icon.codepoint, icon.name)
    }
}

#[derive(Clone, Debug)]
struct Contains<A>(A);

enum ContainsState<A: Automaton> {
    Done,
    Running(A::State),
}

impl<A: Automaton> Automaton for Contains<A> {
    type State = ContainsState<A>;

    fn start(&self) -> Self::State {
        let inner = self.0.start();
        if self.0.is_match(&inner) {
            ContainsState::Done
        } else {
            ContainsState::Running(inner)
        }
    }

    fn is_match(&self, state: &Self::State) -> bool {
        match state {
            ContainsState::Done => true,
            ContainsState::Running(_) => false,
        }
    }

    fn can_match(&self, state: &Self::State) -> bool {
        match state {
            ContainsState::Done => true,
            ContainsState::Running(inner) => self.0.can_match(inner),
        }
    }

    fn will_always_match(&self, state: &Self::State) -> bool {
        match state {
            ContainsState::Done => true,
            ContainsState::Running(_) => false,
        }
    }

    fn accept(&self, state: &Self::State, byte: u8) -> Self::State {
        match state {
            ContainsState::Done => ContainsState::Done,
            ContainsState::Running(inner) => {
                let next_inner = self.0.accept(inner, byte);
                if self.0.is_match(&next_inner) {
                    ContainsState::Done
                } else if !self.0.can_match(&next_inner) {
                    ContainsState::Running(self.0.start())
                } else {
                    ContainsState::Running(next_inner)
                }
            }
        }
    }
}