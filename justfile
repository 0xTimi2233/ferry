default:
    @just --list

[group('all')]
ci: check lint test audit

[group('all')]
check: check-server check-web

[group('all')]
lint: lint-server

[group('all')]
test: test-server

[group('all')]
audit: audit-server

[group('server')]
check-server:
    cargo fmt --check

[group('server')]
fmt-server:
    cargo fmt --all

[group('server')]
lint-server:
    cargo clippy --all-targets --all-features -- -D warnings

[group('server')]
test-server:
    cargo test --all-features

[group('server')]
audit-server:
    cargo deny check advisories bans sources

[group('web')]
check-web:
    bun run check

[group('web')]
format-web:
    bun run format

[group('github')]
sync-labels repo:
    #!/usr/bin/env bash
    set -euo pipefail
    manifest="{{justfile_directory()}}/.github/labels.json"
    wanted=$(mktemp)
    jq -r '.[].name' "$manifest" | sort > "$wanted"
    gh label list --repo "{{repo}}" --json name --jq '.[].name' | sort | comm -13 "$wanted" - | while read -r name; do
        gh label delete "$name" --repo "{{repo}}" --yes
    done
    rm -f "$wanted"
    jq -c '.[]' "$manifest" | while read -r item; do
        name=$(echo "$item" | jq -r .name)
        color=$(echo "$item" | jq -r .color)
        desc=$(echo "$item" | jq -r .description)
        gh label create "$name" --color "$color" --description "$desc" --repo "{{repo}}" --force
    done
