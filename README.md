# 🔣 nerdfix

![GitHub Release](https://img.shields.io/github/v/release/loichyan/nerdfix)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/loichyan/nerdfix/cicd.yml)

`nerdfix` helps you to find/fix obsolete [Nerd Font](https://github.com/ryanoasis/nerd-fonts) icons
in your project.

## 💭 Why

Nerd Fonts is used in many projects for a beautiful UI. It provides more than 10,000 icons, but some
codepoints conflict with other fonts (especially CJK fonts). To ensure that the icons remain in the
private use area, Nerd Fonts has changed the codepoints of some icons in recent releases, for
example, `mdi-*` icons (including over 2,000 icons) are deprecated since
[v2.3.3](https://github.com/ryanoasis/nerd-fonts/releases/tag/v2.3.3) and will be removed in v3.

These icons are marked as obsolete in
[the official cheat sheet](https://www.nerdfonts.com/cheat-sheet) and it's recommended to replace
them with the new ones. However, you may find it boring to check all the used icons one by one, so
`nerdfix` was written to index the cheat sheet and find obsolete icons in your project.

## ⚙️ Installation

You can download the pre-built binaries from
[the release page](https://github.com/loichyan/nerdfix/releases/latest) or manually build this
project manually from source.

In addition, the binaries come with a recently updated cheat sheet and you can overwrite it with the
latest one using `nerdfix -i /path/to/your/file` (follow
[this link](https://github.com/ryanoasis/nerd-fonts/blob/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md)
to get the latest file).

### Install from source

You can build and install from the source code with `cargo`:

```sh
cargo install --git https://github.com/loichyan/nerdfix.git
```

Or with `nix`:

```sh
nix run github:loichyan/nerdfix
```

## 📋 Note

Please make sure you're using Nerd Fonts after v2.3.3, otherwise the replaced new icons may not be
displayed correctly. If you are a plugin author, it's also recommended to notify this in updates.

## 🔍 Usage

The `check` command checks input files and reports obsolete icons with some suggestions (sorted by
similarity) that you could replace them with.

```sh
nerdfix check test/test-data.txt
```

You get the output as follows:

```text
warning: Found obsolete icon U+F752
  ┌─ tests/test-data.txt:1:27
  │
1 │ mdi-folder_multiple = ""
  │                           ^ Icon 'mdi-folder_multiple' is marked as obsolete
  │
  = You could replace it with:
        1. 󰉓 U+F0253 md-folder_multiple
        2. 󱏓 U+F13D3 md-folder_star_multiple
        ...
```

### Interactive patching

The `fix` command reports the same information as `check` and displays a prompt asking the user to
input a new icon to replace the obsolete one.

```text
warning: Found obsolete icon U+F719
  ┌─ tests/test-data.txt:4:29
  │
4 │ mdi-file_document_box = ""
  │                             ^ Icon 'mdi-file_document_box' is marked as obsolete
  │
  = You could replace it with:
        1. 󰈙 U+F0219 md-file_document
        2. 󰷈 U+F0DC8 md-file_document_edit
        ...
> Input an icon: 1
# Your input: 󰈙
```

The prompt accepts several types of input:

| Type              | Example            |
| ----------------- | ------------------ |
| Suggestion number | `1`                |
| Codepoint         | `U+F0219`          |
| Icon name         | `md-file_document` |
| Icon character    | `󰈙`                |

### Fuzzy autocompletion/search

The prompt also provides fuzzy matching suggestions when you type the icon name:

```text
> Input an icon: documentmultiple
  󱔗 md-file_document_multiple
  󱔘 md-file_document_multiple_outline
  󰡟 md-comment_multiple
  ...
```

You can also use the `search` command to call the prompt directly for a fuzzy search.

### Autofix

`nerdfix` provides some features to automatically patch obsolete icons:

- The last user input is picked if an icon appears twice.
- Use `--sub FROM/TO` (or `--sub exact:FROM/TO` for full syntax) to replace one icon with another,
  e.g. `mdi-tab` is replaced with `md-tab` when `--sub mdi-tab/md-tab` is specified.
- Use `--sub prefix:FROM/TO` to replace the prefix of an icon name with another, e.g. `mdi-tab` is
  replaced with `md-tab` when `--sub prefix:mdi-/md-` is specified.

### Database

`nerdfix` accepts icons and substutitions in the following JSON format:

```json
{
  "icons": [
    { "name": "md-file_document", "codepoint": "f0219" },
    { "name": "mdi-file_document_box", "codepoint": "0f719", "obsolete": true }
  ],
  "substitutions": ["exact:mdi-file_document_box/md-file_document"]
}
```

As mentioned above, the precompiled `nerdfix` binary comes bundled with an indexed database that
includes all available icons and some common substitutions.

### Structured output

You can use `check --format json` to get structured output for further use. `nerdfix` prints
diagnostics with the following fields line by line:

| Field       | Description                                             |
| ----------- | ------------------------------------------------------- |
| `severity`  | Severity of a diagnostic                                |
| `path`      | Source file of a diagnostic                             |
| `type`      | Diagnostic type, currently only `obsolete` is supported |
| `span`      | Byte index span of an obsolete icon                     |
| `name`      | Icon name                                               |
| `codepoint` | Icon codepoint                                          |

## 💬 FAQ

### How can I find all files that contain obsolete icons?

```sh
nerdfix check --format=json -r /path/to/root 2>/dev/null |
  jq -s -r '[.[].path] | sort | unique | .[]'
```

### How can I save patched content to a file other than the input? ([#7](https://github.com/loichyan/nerdfix/pull/7))

```sh
nerdfix fix /path/to/input:/path/to/output
```

### How can I recursively traverse all directories? ([#5](https://github.com/loichyan/nerdfix/issues/5))

```sh
nerdfix fix --recursive /path/to/root
# Or use fd/find
nerdfix fix $(fd -t f . /path/to/root)
```

### How can I skip interactive prompts? ([#3](https://github.com/loichyan/nerdfix/issues/3))

```sh
nerdfix fix --write --select-first /path/to/file
```

## ⚖️ License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
