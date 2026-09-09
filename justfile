default:
    @just --list

# 提交前全量校验
[group('全仓')]
ci: check lint test audit

# 全仓格式检查
[group('全仓')]
check: check-server

# 全仓静态检查
[group('全仓')]
lint: lint-server

# 全仓测试
[group('全仓')]
test: test-server

# 全仓依赖审计
[group('全仓')]
audit: audit-server

# 代码格式规范检查
[group('服务端')]
check-server:
    cargo fmt --check

# 代码规范静态检查
[group('服务端')]
lint-server:
    cargo clippy --all-targets --all-features -- -D warnings

# 全量测试
[group('服务端')]
test-server:
    cargo test --all-features

# 依赖漏洞审计
[group('服务端')]
audit-server:
    cargo deny check

# 批量同步标准标签到指定的 GitHub 仓库，用法如 just sync-labels 0xTimi2233/target-repo
[group('协作')]
sync-labels repo:
    #!/usr/bin/env bash
    set -euo pipefail
    jq -c '.[]' "{{justfile_directory()}}/.github/labels.json" | while read -r item; do
        name=$(echo "$item" | jq -r .name)
        color=$(echo "$item" | jq -r .color)
        desc=$(echo "$item" | jq -r .description)
        gh label create "$name" --color "$color" --description "$desc" --repo "{{repo}}" --force
    done
