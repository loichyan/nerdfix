# 🔣 nerdfix

![GitHub release](https://img.shields.io/github/v/release/loichyan/nerdfix)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/loichyan/nerdfix/release.yaml)

`nerdfix` helps you to find/fix obsolete
[nerd font](https://github.com/ryanoasis/nerd-fonts) icons in your project.

## 💭 Why

Nerd Fonts is used in a lot of projects for a beautiful ui. It provides more
than 10,000 icons, but some of the codepoints confict with those of other fonts
(especially CJK fonts). In order to ensure icons are in the private use area,
Nerd Fonts has changed the codepoints of some icons in the recent release, for
example, `nf-mdi-*` icons (which provides about 2,000 icons) are deprecated
since [v2.3.3](https://github.com/ryanoasis/nerd-fonts/releases/tag/v2.3.3) and
will be removed in v3.

These icons are marked as obsolete in
[the offical cheat sheet](https://www.nerdfonts.com/cheat-sheet) and it's
recommended to replace them with the new ones. However, you may find it boring
to check all used icons one by one, so I wrote `nerdfix` for indexing the cheat
sheet and finding obsolete icons in your project.

## ⚙️ Installation

You can download the pre-built binaries from
[the release page](https://github.com/loichyan/nerdfix/releases/latest) or
manully build this project from the source.

In addition, the binaries come with a recently updated cheat sheet and you can
override it with the latest one via `nerdfix -i /path/to/your/file` (follow
[this link](https://github.com/ryanoasis/nerd-fonts/blob/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md)
to get the latest file).

## 🔍 Usage

The `check` command checks input files and reports obsolete icons with some
suggestions that you could replace it with.

```sh
nerdfix check test/test-data.txt
```

You will get the output as follows:

```text
warning: Found obsolete icon U+F752
  ┌─ tests/test-data.txt:1:27
  │
1 │ nf-mdi-folder_multiple = ""
  │                           ^ Icon 'nf-mdi-folder_multiple' is marked as obsolete
  │
  = You could replace it with:
        1. 󰉓 U+F0253 nf-md-folder_multiple
        2. 󱏓 U+F13D3 nf-md-folder_star_multiple
        ...
```

The output of `fix` command is similar to `check` and shows a prompt asking the
user to input a new icon to replace the obsolete one.

```text
warning: Found obsolete icon U+F719
  ┌─ tests/test-data.txt:4:29
  │
4 │ nf-mdi-file_document_box = ""
  │                             ^ Icon 'nf-mdi-file_document_box' is marked as obsolete
  │
  = You could replace it with:
        1. 󰈙 U+F0219 nf-md-file_document
        2. 󰷈 U+F0DC8 nf-md-file_document_edit
        ...
> Input an icon: 1
# Your input: 󰈙
```

The prompt accepts several types of input:

| Type              | Example               |
| ----------------- | --------------------- |
| Suggestion number | `1`                   |
| Codepoint         | `U+F0219`             |
| Icon name         | `nf-md-file_document` |
| Icon character    | `󰈙`                   |

The prompt also provides fuzzy matched suggestions when typing the icon name:

```text
> Input an icon: documentmultiple
  󱔗 nf-md-file_document_multiple
  󱔘 nf-md-file_document_multiple_outline
  󰡟 nf-md-comment_multiple
  ...
```

You can also use the `search` command to call the prompt directly for fuzzy
search.

## ⚖️ License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
