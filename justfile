default:
    @just --list

# 提交前全量校验
[group('all')]
ci: check lint test audit

# 全仓格式检查
[group('all')]
check: check-server

# 全仓静态检查
[group('all')]
lint: lint-server

# 全仓测试
[group('all')]
test: test-server

# 全仓依赖审计
[group('all')]
audit: audit-server

# 代码格式规范检查
[group('server')]
check-server:
    cargo fmt --check

# 代码规范静态检查
[group('server')]
lint-server:
    cargo clippy --all-targets --all-features -- -D warnings

# 全量测试
[group('server')]
test-server:
    cargo test --all-features

# 依赖漏洞审计
[group('server')]
audit-server:
    cargo deny check

# 批量同步标准标签到指定的 GitHub 仓库，用法如 just sync-labels 0xTimi2233/target-repo
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
