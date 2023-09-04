_just := quote(just_executable()) + ' --justfile=' + quote(justfile())

_default:
	@{{_just}} --list

# Updates icons indices.
update-index rev=`cat src/index-rev`:
	#!/usr/bin/env bash
	set -euxo pipefail
	tmp="$(mktemp)"
	curl -fsSL "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/{{rev}}/_posts/2017-01-04-icon-cheat-sheet.md" -o"${tmp}"
	cargo run -- -i "${tmp}" index -o src/index.json
	rm "${tmp}"
