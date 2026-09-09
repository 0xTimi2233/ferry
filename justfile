default:
    @just --list

[group('all')]
ci: check lint test audit

[group('all')]
check: check-server

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
lint-server:
    cargo clippy --all-targets --all-features -- -D warnings

[group('server')]
test-server:
    cargo test --all-features

[group('server')]
audit-server:
    cargo deny check

[group('github')]
sync-labels repo:
    #!/usr/bin/env bash
    set -euo pipefail
    jq -c '.[]' "{{justfile_directory()}}/.github/labels.json" | while read -r item; do
        name=$(echo "$item" | jq -r .name)
        color=$(echo "$item" | jq -r .color)
        desc=$(echo "$item" | jq -r .description)
        gh label create "$name" --color "$color" --description "$desc" --repo "{{repo}}" --force
    done
