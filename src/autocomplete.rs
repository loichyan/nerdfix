//! Autocompletion and fuzzy search for nerd fonts.

use crate::icon::Icon;
use inquire::Autocomplete;
use std::rc::Rc;

pub type SearchIndex = indicium::simple::SearchIndex<usize>;

#[derive(Clone)]
pub struct Autocompleter {
    pub(crate) icons: Rc<Vec<Icon>>,
    pub(crate) corpus: Rc<SearchIndex>,
    pub(crate) candidates: usize,
}

impl Autocomplete for Autocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        if input.is_empty() {
            Ok((0..self.candidates).map(|i| (i + 1).to_string()).collect())
        } else {
            Ok(self
                .corpus
                .search(input)
                .into_iter()
                .map(|&i| {
                    let icon = &self.icons[i];
                    format!("{} {}", icon.codepoint, icon.name)
                })
                .collect())
        }
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        if let Some(s) = highlighted_suggestion {
            Ok(Some(s))
        } else if input.is_empty() {
            Ok(Some(String::from("1")))
        } else {
            Ok(self
                .corpus
                .search_with(&indicium::simple::SearchType::Live, &1, input)
                .into_iter()
                .next()
                .map(|&i| self.icons[i].name.clone()))
        }
    }
}
