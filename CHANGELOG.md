# Changelog

## v0.4.0 (2023-10-20)

### Overview

This releases primarily introduces the predefined substitutions suggested in
[#9](https://github.com/loichyan/nerdfix/issues/9) (thanks [@Finii](https://github.com/Finii)!) and
also brings a few refactors on the CLI. Here are some guides for migrating from `v0.3`:

1. Use `dump` instead of `cache` to show all icons and substitutions in the runtime database.
2. Previous release of `nerdfix` supports the `--replace FROM:TO` argument to perform a prefix
   substitution, now it defines a new argument `--sub TYPE:FROM/TO` which supports both `exact` and
   `prefix` substitutions. This means that you should use `--sub prefix:FROM/TO` in place of the old
   one.
3. You can pipe the input/output with `nerdfix` using the special `-` path.

### Feat

- **database**: make `prefix:mdi-/md-` a default substitution
- support read from and write to stdio in more options
- **runtime**: generate icon substitutions list for `nf-mdi-*`
- **autofix**: load predefined substitutions list
- **parser**: support new cheat-sheet format
- **parser**: strip `nf-` prefixes
- **cli**: use `-v/-q` to control log level

### Fix

- **runtime**: clear buffer before check sources
- **log**: exit non-zero if any error occurs
- **cli**: warn deprecated arguments
- **justfile**: remove redundant `nf-` prefixes

### Refactor

- **cli**: rename term input to database
- implement a syntax to define substitutions
- **cli**: use `--input` to load substitutions
- clean up typo and unused codes
- **colored**: replace `colored` with `nu_ansi_term`
- **autofix**: move `--replace` to global options
- **cli**: rename subcommand `cache` to `index`
- **cache**: save in json
- **macros**: match block instead of statements
- **error**: show path in logs
- replace inline closures with try-block macro

## v0.3.1 (2023-05-15)

### Fix

- **cli**: pad output codepoints

## v0.3.0 (2023-05-12)

### Feat

- **cli**: support optional paths to write content to
- **cli**: support recursive directories traversal (#6)
- **cli**: improve prompts auto confirmation (#4)

### Fix

- **parser**: icons with `removed` label should be considered as obsolete

### Refactor

- **util**: replace some macros with ext traits

## v0.2.3 (2023-05-03)

### Fix

- **stdio**: logs and diagnostics should be written to different streams

### Refactor

- `error::Io` can be directly constructed with `&Path`

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
