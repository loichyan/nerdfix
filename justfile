index_rev := `cat src/index-rev`
md_rev := "f1e17ff8aad81f4b58f25a2e1956807297aa926e"

_just := quote(just_executable()) + ' --justfile=' + quote(justfile())
_curl := "curl -fsSL"
_setup_bash := "set -euxo pipefail"

_default:
	@{{_just}} --list

# Updates icon indices.
update-index:
	#!/usr/bin/env bash
	{{_setup_bash}}
	{{_curl}} "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/{{index_rev}}/_posts/2017-01-04-icon-cheat-sheet.md" \
	| cargo run -- index -i - -o src/index.json

# https://github.com/loichyan/nerdfix/issues/9#issuecomment-1576944348
substitutions-md := '{
  "account-card-details": "card-account-details",
  "azure": "microsoft-azure",
  "bing": "microsoft-bing",
  "circle": "circle-medium",
  "circle-outline": "checkbox-blank-circle-outline",
  "do-not-disturb": "minus-circle",
  "do-not-disturb-off": "minus-circle-off",
  "edge": "microsoft-edge",
  "face-profile": "face-man-profile",
  "github-circle": "github",
  "gradient": "gradient-vertical",
  "hangouts": "google-hangouts",
  "internet-explorer": "microsoft-internet-explorer",
  "json": "code-json",
  "linkedin-box": "linkedin",
  "login-variant": "exit-to-app",
  "markdown": "language-markdown",
  "office": "microsoft-office",
  "onedrive": "microsoft-onedrive",
  "onenote": "microsoft-onenote",
  "playstation": "sony-playstation",
  "radiobox-blank": "checkbox-blank-circle-outline",
  "sort-alphabetical": "sort-alphabetical-variant",
  "sort-numeric": "sort-numeric-variant",
  "tablet-ipad": "tablet",
  "terrain": "image-filter-hdr",
  "textbox": "form-textbox",
  "textbox-password": "form-textbox-password",
  "towing": "tow-truck",
  "voice": "account-voice",
  "wii": "nintendo-wii",
  "wiiu": "nintendo-wiiu",
  "windows": "microsoft-windows",
  "xamarin-outline": "microsoft-xamarin",
  "xbox": "microsoft-xbox",
  "xbox-controller": "microsoft-xbox-gamepad",
  "xbox-controller-battery-alert": "microsoft-xbox-gamepad-battery-alert",
  "xbox-controller-battery-empty": "microsoft-xbox-controller-battery-empty",
  "xbox-controller-battery-full": "microsoft-xbox-controller-battery-full",
  "xbox-controller-battery-low": "microsoft-xbox-controller-battery-low",
  "xbox-controller-battery-medium": "microsoft-xbox-controller-battery-medium",
  "xbox-controller-battery-unknown": "microsoft-xbox-controller-battery-unknown",
  "xbox-controller-off": "microsoft-xbox-controller-off"
}'

# Update icon substitutions list.
update-substitution:
	#!/usr/bin/env bash
	{{_setup_bash}}
	{
		{{_curl}} "https://raw.githubusercontent.com/Templarian/MaterialDesign-Meta/{{md_rev}}/meta.json" | jq '.[] | {(.aliases[]): .name}'
		echo '{{substitutions-md}}'
	} | jq -cs '
		add | { substitutions: . | to_entries | map("exact:\(.key | sub("-";"_") | "mdi-\(.)")/\(.value | sub("-";"_") | "md-\(.)")") }
	' >src/substitution.json
