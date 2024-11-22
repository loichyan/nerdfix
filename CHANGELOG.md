# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Here's a template for each release section. This file should only include changes that are
noticeable to end-users since the last release. For developers, this project follows
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) to track changes.

## [1.0.0] - YYYY-MM-DD

### Added

- [**breaking**] Always place breaking changes at the top.
- Append other changes in chronological order under the relevant subsections.

### Changed

### Deprecated

### Removed

### Fixed

### Security

[1.0.0]: https://github.com/user/repo/compare/v0.0.0..v1.0.0
-->

## [Unreleased]

### Added

- Add a subcommand, `nerdfix completions <SHELL>`, to generate completions for your shell
  ([#30](https://github.com/loichyan/nerdfix/pull/30)).
- Support comments in JSON files ([#33](https://github.com/loichyan/nerdfix/pull/33)).
- Add a new `nerdfix query` subcommand, useful for querying icon infos from the database
  ([#33](https://github.com/loichyan/nerdfix/pull/33)).
- Add a new `codepoint:from/to` substitution type
  ([#33](https://github.com/loichyan/nerdfix/pull/33)).
- Support checking dropped icons of Nerd Fonts v3.3.0 through the newly added
  `nerdfix --nf-version=3.3.0` option ([#33](https://github.com/loichyan/nerdfix/pull/33), thanks
  [@Finii](https://github.com/Finii) and [@hasecilu](https://github.com/hasecilu)).

## [0.4.1] - 2024-07-14

This release mainly addresses the high memory usage issue reported in
[#18](https://github.com/loichyan/nerdfix/pull/18): fixed a potential memory leak in
[#21](https://github.com/loichyan/nerdfix/pull/21), and implemented stream processing in
[#22](https://github.com/loichyan/nerdfix/pull/22).

Also, some UI changes were introduced in [#21](https://github.com/loichyan/nerdfix/pull/21), as we
switched the diagnostic reporter from
[codespan_reporting](https://docs.rs/codespan-reporting/latest/codespan_reporting) to
[miette](https://docs.rs/miette/latest/miette).

### Added

- Support filter out binary files
  ([d1f29e4](https://github.com/loichyan/nerdfix/commit/d1f29e4bdd40b784090486fc7bf798ecd42997fb)).
- Support filter out files by size
  ([db421eb](https://github.com/loichyan/nerdfix/commit/db421ebfa941d7ea4e2ce386fef4d576922bbf4a)).

### Changed

- Report the source paths of diagnostics ([#23](https://github.com/loichyan/nerdfix/pull/23)).
- Process files in bytes stream ([#22](https://github.com/loichyan/nerdfix/pull/22)).
- Use `miette` in place of `codespan-reporting` as the diagnostic reporter
  ([#21](https://github.com/loichyan/nerdfix/pull/21)).

### Fixed

- Fix subtract with overflow
  ([aa29181](https://github.com/loichyan/nerdfix/commit/aa29181aa41c094e60e519b7c61b95adf331f866)).
- Fix potential memory leaks ([#21](https://github.com/loichyan/nerdfix/pull/21)).

## [0.4.0] - 2023-10-20

This release introduces the predefined substitutions support suggested in
[#9](https://github.com/loichyan/nerdfix/issues/9) (thanks [@Finii](https://github.com/Finii)), and
also brings some UX breaking changes. Here are some guides for migrating from `v0.3`:

1. Use `dump` instead of `cache` to show all icons and substitutions in the runtime database.
2. The Previous release supports use `--replace FROM:TO` to perform a prefix substitution, now it
   uses the newly add `--sub TYPE:FROM/TO` argument to support both `exact` and `prefix`
   substitutions. This means that you should use `--sub prefix:FROM/TO` in place of the old one.

### Added

- Add `-v/-q` to control log level
  ([1294e24](https://github.com/loichyan/nerdfix/commit/1294e24972baaf5e0b88a3021745f2ae6a261e20)).
- Support load predefined substitutions list ([#10](https://github.com/loichyan/nerdfix/pull/10)).
- Support parse the new cheat-sheet format ([#10](https://github.com/loichyan/nerdfix/pull/10)).
- Support read from and write to STDIO in more options
  ([#7](https://github.com/loichyan/nerdfix/pull/7)).

### Changed

- [**breaking**] Rename subcommand `cache` to `dump`
  ([f148638](http://github.com/loichyan/nerdfix/commit/2bbfc04ea356228a92f714a84a23246004320c3f)).
- [**breaking**] Deprecated `--replace FROM:TO`, use `--sub prefix:FROM/TO` instead
  ([557c6fd](http://github.com/loichyan/nerdfix/commit/557c6fd1a7173ad6e34e431406577a3adf66ed97)).
- Move `--sub` to global options
  ([6c8808e](https://github.com/loichyan/nerdfix/commit/6c8808e61dabaaf1bb91bd079c47862a62eed7ff)).
- Save database in JSON
  ([e8a0ccf](https://github.com/loichyan/nerdfix/commit/e8a0ccf2a944a2a25e49251ceaf0158cbd0791df)).
- Show source paths in logs
  ([2699dd4](https://github.com/loichyan/nerdfix/commit/2699dd4f4f7d1a1cf540f6afb7e4d38215648400)).
- Strip `nf-` prefixes in messages, and generated databases
  ([#12](https://github.com/loichyan/nerdfix/pull/12)).
- Warn usage of deprecated arguments
  ([a783b7e](https://github.com/loichyan/nerdfix/commit/a783b7e96b38edfb0e7dda0de1f56d6d9c64100a)).

### Fixed

- Exit non-zero if any error occurs ([#15](https://github.com/loichyan/nerdfix/pull/15)).

## [0.3.1] - 2023-05-14

### Fixed

- Fix testing regressions
  ([4070f9e](https://github.com/loichyan/nerdfix/commit/4070f9e894337ca7d3f7641258428ad6d7cd6332)).
- Pad output codepoints
  ([021d313](https://github.com/loichyan/nerdfix/commit/021d313ab3d1821e5bcf5d0d8d39a7d5fcdec776)).

## [0.3.0] - 2023-05-12

### Added

- Support recursive directories traversal ([#6](https://github.com/loichyan/nerdfix/pull/6)).
- Support specify optional paths where patched contents are written to
  ([ee9b398](https://github.com/loichyan/nerdfix/commit/ee9b398268b38ebbec59609f30d6f753ab6ef152)).

### Changed

- [**breaking**] Deprecated `--yes`, use `--write` and `--select-first` to force non-interactive
  fixes ([#4](https://github.com/loichyan/nerdfix/pull/4)).
- Streamline Nix package derivation
  ([a9a3630](https://github.com/loichyan/nerdfix/commit/a9a3630c6eafe6558fcca49aa964243d0f5b688f)).

### Fixed

- Do not consider icons with `removed` label as obsolete
  ([750ace5](https://github.com/loichyan/nerdfix/commit/750ace506f4c52fd0fa437411102b5be18a3a354)).

## [0.2.3] - 2023-05-03

### Added

- Add Nix flake package
  ([b33907b](https://github.com/loichyan/nerdfix/commit/b33907b0d5b605376377dabd950526eacb3cd5e4)).

### Fixed

- Write logs and diagnostics to different streams
  ([0e7b3f6](https://github.com/loichyan/nerdfix/commit/0e7b3f6389b0a783a2f2838f701f645f69548e2f)).

## [0.2.2] - 2023-05-02

### Changed

- Display brief error messages
  ([8319fcb](https://github.com/loichyan/nerdfix/commit/8319fcbfa4eccb5f7f87d5a4804e5cda51d9393b)).
- Use the same style of keyboard hints
  ([e5a8975](https://github.com/loichyan/nerdfix/commit/e5a8975cffbeac417c4b68e56a742941e33f85bd)).

### Fixed

- Sync non-patched contents before autofix
  ([44ad79d](https://github.com/loichyan/nerdfix/commit/44ad79dd357cd351685f8aea7aa54cab88f1ea32)).

## [0.2.1] - 2023-04-11

### Added

- Show the name of a selected icon
  ([48d7aff](https://github.com/loichyan/nerdfix/commit/48d7aff8b57fd04312f311d09bb9d2b718e8b461)).

### Fixed

- Only use the first string of user input
  ([f67f157](https://github.com/loichyan/nerdfix/commit/f67f157218e4d4c018fdc8aedb0c21542d9f7de7)).
- Sort candidates by names for stable results
  ([1529a4b](https://github.com/loichyan/nerdfix/commit/1529a4b1b186dd2369e5ccf712d4844fd84be5d2)).

## [0.2.0] - 2023-03-30

### Added

- Support fix sources non-interactively through `--yes`
  ([e2e4bc9](https://github.com/loichyan/nerdfix/commit/e2e4bc9c275294ff4f1d97650b26475b80e7af47)).
- Support structured format output through `--format=json`
  ([#2](https://github.com/loichyan/nerdfix/pull/2)).
- Add `--replace` to autofix icons by name ([#1](https://github.com/loichyan/nerdfix/pull/1)).
- Add `--verbose` to control log level
  ([4830e37](https://github.com/loichyan/nerdfix/commit/4830e3766cc4892b6eefad567da2cc5fb3a4a677)).

### Changed

- Change the severity of obsolete fonts to NOTE
  ([2debe0b](https://github.com/loichyan/nerdfix/commit/2debe0b337f5f4c101abd53701ab4fb59e740475)).

### Fixed

- Do not load builtin icons if users specify their own database
  ([428b468](https://github.com/loichyan/nerdfix/commit/428b468e92d575740bd283a37719e03924a89bcf)).

## [0.1.0] - 2023-03-25

🎉 Initial release. See [README](https://github.com/loichyan/nerdfix/blob/v0.1.0/README.md) for more
details.

[Unreleased]: https://github.com/loichyan/nerdfix/compare/v0.4.1..HEAD
[0.4.1]: https://github.com/loichyan/nerdfix/compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com/loichyan/nerdfix/compare/v0.3.1..v0.4.0
[0.3.1]: https://github.com/loichyan/nerdfix/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/loichyan/nerdfix/compare/v0.2.3..v0.3.0
[0.2.3]: https://github.com/loichyan/nerdfix/compare/v0.2.2..v0.2.3
[0.2.2]: https://github.com/loichyan/nerdfix/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/loichyan/nerdfix/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/loichyan/nerdfix/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/loichyan/nerdfix/releases/tag/v0.1.0
