default:
    @just --list

# 代码格式规范检查
check:
    cargo fmt --check

# 代码规范静态检查
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# 全量测试
test:
    cargo test --all-features

# 依赖漏洞审计
audit:
    cargo deny check

# 提交前全量校验
ci: check lint test audit

# 批量同步标准标签到指定的 GitHub 仓库，用法如 just sync-labels 0xTimi2233/target-repo
sync-labels repo:
    #!/usr/bin/env bash
    set -euo pipefail
    jq -c '.[]' "{{justfile_directory()}}/.github/labels.json" | while read -r item; do
        name=$(echo "$item" | jq -r .name)
        color=$(echo "$item" | jq -r .color)
        desc=$(echo "$item" | jq -r .description)
        gh label create "$name" --color "$color" --description "$desc" --repo "{{repo}}" --force
    done
