index_rev := `cat src/index-rev`
md_rev := 'f1e17ff8aad81f4b58f25a2e1956807297aa926e'
icons := 'src/icons.json'
substitutions := 'src/substitutions.json'

_just := quote(just_executable()) + ' --justfile=' + quote(justfile())
_curl := 'curl -fsSL'
_setup_bash := 'set -euxo pipefail'

_default:
	@{{_just}} --list

# Updates builtin database
update-db: update-icons update-substitutions

# Updates icon indices.
update-icons:
	#!/usr/bin/env bash
	{{_setup_bash}}
	{{_curl}} "https://raw.githubusercontent.com/ryanoasis/nerd-fonts/{{index_rev}}/_posts/2017-01-04-icon-cheat-sheet.md" \
	| cargo run -- dump -i - -o "{{icons}}"

# https://github.com/loichyan/nerdfix/issues/9#issuecomment-1576944348
substitutions-extra := '
	"exact:account-card-details/card-account-details"
	"exact:azure/microsoft-azure"
	"exact:bing/microsoft-bing"
	"exact:circle/circle-medium"
	"exact:circle-outline/checkbox-blank-circle-outline"
	"exact:do-not-disturb/minus-circle"
	"exact:do-not-disturb-off/minus-circle-off"
	"exact:edge/microsoft-edge"
	"exact:face-profile/face-man-profile"
	"exact:github-circle/github"
	"exact:gradient/gradient-vertical"
	"exact:hangouts/google-hangouts"
	"exact:internet-explorer/microsoft-internet-explorer"
	"exact:json/code-json"
	"exact:linkedin-box/linkedin"
	"exact:login-variant/exit-to-app"
	"exact:markdown/language-markdown"
	"exact:office/microsoft-office"
	"exact:onedrive/microsoft-onedrive"
	"exact:onenote/microsoft-onenote"
	"exact:playstation/sony-playstation"
	"exact:radiobox-blank/checkbox-blank-circle-outline"
	"exact:sort-alphabetical/sort-alphabetical-variant"
	"exact:sort-numeric/sort-numeric-variant"
	"exact:tablet-ipad/tablet"
	"exact:terrain/image-filter-hdr"
	"exact:textbox/form-textbox"
	"exact:textbox-password/form-textbox-password"
	"exact:towing/tow-truck"
	"exact:voice/account-voice"
	"exact:wii/nintendo-wii"
	"exact:wiiu/nintendo-wiiu"
	"exact:windows/microsoft-windows"
	"exact:xamarin-outline/microsoft-xamarin"
	"exact:xbox/microsoft-xbox"
	"exact:xbox-controller/microsoft-xbox-gamepad"
	"exact:xbox-controller-battery-alert/microsoft-xbox-gamepad-battery-alert"
	"exact:xbox-controller-battery-empty/microsoft-xbox-controller-battery-empty"
	"exact:xbox-controller-battery-full/microsoft-xbox-controller-battery-full"
	"exact:xbox-controller-battery-low/microsoft-xbox-controller-battery-low"
	"exact:xbox-controller-battery-medium/microsoft-xbox-controller-battery-medium"
	"exact:xbox-controller-battery-unknown/microsoft-xbox-controller-battery-unknown"
	"exact:xbox-controller-off/microsoft-xbox-controller-off"
	"prefix:mdi-/md-"
'

# Update icon substitutions list.
update-substitutions:
	#!/usr/bin/env bash
	{{_setup_bash}}
	{
		{{_curl}} "https://raw.githubusercontent.com/Templarian/MaterialDesign-Meta/{{md_rev}}/meta.json" \
		| jq '.[] | {from: .aliases[], to: .name} | "exact:\(.from | sub("-";"_") | "mdi-\(.)")/\(.to | sub("-";"_") | "md-\(.)")"'
		echo '{{substitutions-extra}}'
	} | jq -cs '{ substitutions: . }' >"{{substitutions}}"
