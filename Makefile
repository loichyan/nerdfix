# Updates cached icons.
update-cache:
	wget https://raw.githubusercontent.com/ryanoasis/nerd-fonts/gh-pages/_posts/2017-01-04-icon-cheat-sheet.md -O /tmp/nerdfix-cache.md
	cargo run -- -C /tmp/nerdfix-cache.md cache -o src/cached.txt
	rm /tmp/nerdfix-cache.md
