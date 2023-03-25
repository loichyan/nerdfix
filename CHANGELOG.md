# Changelog

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
