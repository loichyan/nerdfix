# Updates cached icons.
update-cache:
	$(eval commit := $(shell cat src/cache-version))
	$(eval tmpfile := $(shell mktemp /tmp/nerdfix.XXXX))
	curl -fL https://raw.githubusercontent.com/ryanoasis/nerd-fonts/$(commit)/_posts/2017-01-04-icon-cheat-sheet.md -o $(tmpfile)
	cargo run --release -- -i $(tmpfile) cache -o src/cached.json
	@rm $(tmpfile)
