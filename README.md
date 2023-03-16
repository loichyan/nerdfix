# 🔣 nerdfix

`nerdfix` helps you to find/fix obsolete
[nerd font](https://github.com/ryanoasis/nerd-fonts) icons in your project.

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
