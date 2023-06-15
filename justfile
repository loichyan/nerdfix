_just := quote(just_executable()) + ' --justfile=' + quote(justfile())

_default:
	@{{_just}} --list

# Updates cached icons.
update-cache cache_version=`cat src/cache-version`:
	#!/usr/bin/env bash
	set -euxo pipefail
	tmp="$(mktemp)"
	curl -fsSL "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/{{cache_version}}/_posts/2017-01-04-icon-cheat-sheet.md" -o"${tmp}"
	cargo run -- -i "${tmp}" cache -o src/cached.json
	rm "${tmp}"
