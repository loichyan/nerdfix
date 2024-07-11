//! Autocompletion and fuzzy search for nerd fonts.

use std::rc::Rc;

use indexmap::IndexMap;
use inquire::Autocomplete;
use itertools::Itertools;

use crate::icon::Icon;
use crate::runtime::NGram;

const THRESHOLD: f32 = 0.6;
const MAX_SUGGESTIONS: usize = 100;

#[derive(Clone)]
pub struct Autocompleter {
    pub(crate) icons: Rc<IndexMap<String, Icon>>,
    pub(crate) corpus: Rc<NGram>,
    pub(crate) candidates: usize,
    pub(crate) last: Option<String>,
}

impl Autocomplete for Autocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let suggestions = if input.is_empty() {
            (0..self.candidates)
                .map(|i| (i + 1).to_string())
                .collect_vec()
        } else {
            let max_diff_ngrams = self.corpus.arity() * 2 - 1;
            self.corpus
                .items_sharing_ngrams(input)
                .filter_map(|t| {
                    let sim = self.corpus.similarity(
                        t.shared_ngrams,
                        t.item_ngrams + t.query_ngrams,
                        None,
                    );
                    // Query string meets the threshold or is almost a subset of item.
                    if sim > THRESHOLD || t.query_ngrams - t.shared_ngrams < max_diff_ngrams {
                        Some((t.item, sim))
                    } else {
                        None
                    }
                })
                .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(b.0)))
                .map(|((_, id), _)| {
                    let icon = &self.icons[*id];
                    format!("{} {}", icon.codepoint, icon.name)
                })
                .take(MAX_SUGGESTIONS)
                .collect_vec()
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
            .map(ToOwned::to_owned))
    }
}
