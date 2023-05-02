# Changelog

## v0.2.2 (2023-05-02)

### Fix

- **fix**: patched content should be created in autofix
- **prompt**: keyboard hints should follow the same style
- **error**: `Error` should display brief message

### Refactor

- **cli**: show more metadata
- **error**: make path optional
- **error**: derive `From` for source errors

## v0.2.1 (2023-04-11)

### Feat

- **search**: print the icon name

### Fix

- **prompt**: display formatted user input
- **search**: use the first string of user input
- **runtime**: sort candidates by their names for stable results

### Refactor

- **search**: use n-grams to search subset matches
- **runtime**: replace ngrammatic with noodler

## v0.2.0 (2023-03-30)

### Feat

- **cli**: support output structured data
- **cli**: use `--verbose` to increase log level
- **cli**: use `fix --replace` to auto fix icons
- **cli**: support send yes to all prompts

### Fix

- **runtime**: add path in structured output
- **runtime**: log errors and continue running
- **runtime**: dont load inlined icons if user input is present

### Refactor

- **iter**: use itertools to simplify code
- **cli**: dont allow short argument for `fix --replace`
- **report**: change the severity of obsolete fonts to NOTE
- **runtime**: lazily construct patched content

### Perf

- **runtime**: lazily initialize candidates

## v0.1.0 (2023-03-25)

### Feat

- **parser**: parse both cheat sheet and cached content input
- **cli**: add `search` to fuzzy search an icon
- **autocomplete**: search icons whose name contain the input string
- **fix**: show icons in suggesions
- **check**: auto patch using last user input
- **cli**: add `cache`, `check` and `fix` commands

### Fix

- **cli**: dont prompt if no icon is patched
- **cli**: ignore errors in prompt
- **clippy**: fix clippy warnings

### Refactor

- **runtime**: use `IndexMap` to store icons
- **runtime**: move `Runtime` to a standalone module `crate::runtime`
- move some type definitions and implementations to standalone modules
- **icon**: rename `crate::db` to `crate::icon`
- **db**: change type of `Icon::codepoint` to `char`

### Perf

- **runtime**: dont clone icons
