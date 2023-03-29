# 🔣 nerdfix

![GitHub release](https://img.shields.io/github/v/release/loichyan/nerdfix)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/loichyan/nerdfix/release.yaml)

`nerdfix` helps you to find/fix obsolete
[Nerd Font](https://github.com/ryanoasis/nerd-fonts) icons in your project.

## 💭 Why

Nerd Fonts is used in a lot of projects for a beautiful ui. It provides more
than 10,000 icons, but some codepoints conflict with other fonts (especially CJK
fonts). To ensure icons are in the private use area, Nerd Fonts has changed the
codepoints of some icons in recent releases, for example, `nf-mdi-*` icons
(including over 2,000 icons) are deprecated since
[v2.3.3](https://github.com/ryanoasis/nerd-fonts/releases/tag/v2.3.3) and will
be removed in v3.

These icons are marked as obsolete in
[the official cheat sheet](https://www.nerdfonts.com/cheat-sheet) and it's
recommended to replace them with the new ones. However, you may find it boring
to check all used icons one by one, so `nerdfix` is wrote for indexing the cheat
sheet and finding obsolete icons in your project.

## ⚙️ Installation

You can download the pre-built binaries from
[the release page](https://github.com/loichyan/nerdfix/releases/latest) or
manually build this project from the source.

In addition, the binaries come with a recently updated cheat sheet and you can
override it with the latest one via `nerdfix -i /path/to/your/file` (follow
[this link](https://github.com/ryanoasis/nerd-fonts/blob/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md)
to get the latest file).

## 📋 Notice

Please make sure you're using Nerd Fonts after v2.3.3, otherwise the replaced
new icons may not be displayed correctly. If you are a plugin author, it's also
recommended to notify this in updates.

## 🔍 Usage

The `check` command checks input files and reports obsolete icons with some
suggestions that you could replace it with.

```sh
nerdfix check test/test-data.txt
```

You get the output as follows:

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

### Interactive patching

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

### Fuzzy autocompletion/search

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

### Autofix

`nerdfix` provides some features to automatically patch obsolete icons:

- The last input is picked if an icon appears twice.
- Use `fix --replace FROM,TO` to replace the prefix of an icon name with
  another, e.g. `nf-mdi-tab` is replaced by `nf-md-tab` when
  `--replace nf-mdi-,nf-md-` is specified.

### Structured output

You can use `check --format json` to get structured output for further use.
`nerdfix` prints diagnostics with the following fields line by line:

| Field       | Description                                             |
| ----------- | ------------------------------------------------------- |
| `severity`  | Severity of a diagnostic                                |
| `path`      | Source file of a diagnostic                             |
| `type`      | Diagnostic type, currently only `obsolete` is supported |
| `span`      | Byte index span of an obsolete icon                     |
| `name`      | Icon name                                               |
| `codepoint` | Icon codepoint                                          |

## ⚖️ License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
