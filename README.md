<div align="center">

# codemod-pilot 🛩️

**Transform your codebase by example. No AST knowledge required.**

[![CI](https://github.com/codemod-pilot/codemod-pilot/actions/workflows/ci.yml/badge.svg)](https://github.com/codemod-pilot/codemod-pilot/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/codemod-pilot.svg)](https://crates.io/crates/codemod-pilot)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**[🇨🇳 简体中文](#简体中文) · [🇺🇸 English](#english)**

</div>

---

# 🇺🇸 English

[Features](#-features) · [Quick Start](#-quick-start) · [How It Works](#-how-it-works) · [Examples](#-examples) · [Rule Format](#-rule-format) · [Roadmap](#-roadmap) · [Contributing](#-contributing)

---

## The Problem

Large-scale code refactoring is painful. Renaming an API across 300 files, migrating from one library to another, updating deprecated patterns — these tasks are:

- **Manual**: Find-and-replace is dangerous for code; regex can't understand AST structure
- **Complex**: Writing AST-based codemods with tools like jscodeshift requires deep knowledge of AST manipulation
- **Risky**: Without proper preview and rollback, batch changes can introduce subtle bugs

## The Solution

**codemod-pilot** lets you describe code transformations **by example**. Show it a "before" and "after" snippet, and it will:

1. 🧠 **Infer** the structural transformation pattern from your examples
2. 🔍 **Scan** your entire codebase for matching patterns
3. 👀 **Preview** all proposed changes as a unified diff
4. ✅ **Apply** changes safely with automatic rollback support

```bash
# Show it what you want to change
codemod-pilot learn \
  --before 'fetchUserInfo({ userId: id })' \
  --after  'getUserProfile({ profileId: id })'

# Preview all matches across your codebase
codemod-pilot scan --target ./src/

# Apply with confidence
codemod-pilot apply --execute
```

## ✨ Features

- **Example-Driven** — No AST knowledge needed. Just show before → after.
- **Multi-Example Inference** — Provide multiple examples for complex patterns; the engine finds the common transformation rule.
- **Safe by Default** — Preview all changes before applying. Automatic rollback patch generated.
- **Interactive Conflicts** — Ambiguous cases enter interactive mode for human decision.
- **Reusable Rules** — Export inferred patterns to `.codemod.yaml` files. Share with your team via version control.
- **Built-in Templates** — Common refactoring patterns ready to use out of the box.
- **CI/CD Ready** — `--ci` mode outputs machine-readable JSON. Use `--fail-on-match` for automated checks.
- **Blazing Fast** — Built in Rust with parallel file scanning. Handles 100k+ file codebases in seconds.
- **Multi-Language** — Powered by tree-sitter. Currently supports TypeScript/JavaScript, with Python and Go coming soon.

## 🚀 Quick Start

### Installation

```bash
# Via cargo
cargo install codemod-pilot

# Via curl (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/wellkilo/codemod-pilot/main/scripts/install.sh | sh

# Via Homebrew (coming soon)
brew install codemod-pilot
```

### Your First Codemod (30 seconds)

```bash
# 1. Teach it a transformation
codemod-pilot learn \
  --before 'console.log(msg)' \
  --after  'logger.info(msg)'

# 2. See what it found
codemod-pilot scan --target ./src/

# 3. Preview the diff
codemod-pilot apply --preview

# 4. Apply changes
codemod-pilot apply --execute

# 5. Made a mistake? Roll back instantly
codemod-pilot apply --rollback
```

## 🧠 How It Works

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   Example    │────▶│   Pattern    │────▶│  Codebase   │────▶│    Apply     │
│ before/after │     │  Inference   │     │   Scanner   │     │  & Rollback  │
└─────────────┘     └──────────────┘     └─────────────┘     └──────────────┘
                          │                     │
                    ┌─────▼─────┐         ┌─────▼─────┐
                    │ AST Diff  │         │ Parallel  │
                    │ Analysis  │         │ Matching  │
                    └───────────┘         └───────────┘
```

1. **Parse**: Both "before" and "after" code snippets are parsed into ASTs using tree-sitter
2. **Diff**: The engine computes a structural diff between the two ASTs, identifying which nodes changed and how
3. **Generalize**: Variable parts (identifiers, literals) are detected and converted into pattern variables
4. **Match**: The generalized pattern is used to scan target files, finding all structurally similar code
5. **Transform**: Matched code is rewritten according to the inferred transformation rule
6. **Validate**: Results are presented for review; a rollback patch is always generated before any writes

## 💡 Examples

### Rename a Function and Its Parameters

```bash
codemod-pilot learn \
  --before 'fetchUserInfo({ userId: id })' \
  --after  'getUserProfile({ profileId: id })'

codemod-pilot apply --target ./src/ --execute
```

### Migrate from Moment.js to Day.js

```bash
# Create examples file
cat > examples.yaml << 'EOF'
examples:
  - before: "moment(date).format('YYYY-MM-DD')"
    after: "dayjs(date).format('YYYY-MM-DD')"
  - before: "moment(date).add(1, 'days')"
    after: "dayjs(date).add(1, 'day')"
  - before: "moment(date).diff(other, 'hours')"
    after: "dayjs(date).diff(other, 'hour')"
EOF

codemod-pilot learn --examples examples.yaml
codemod-pilot apply --target ./src/ --preview
```

### Use a Shared Rule File

```bash
# Apply a team-shared codemod rule
codemod-pilot apply --rule ./codemods/replace-deprecated-api.yaml --target ./src/

# Export your codemod for others
codemod-pilot export --output ./codemods/my-migration.yaml
```

### CI/CD Integration

```yaml
# .github/workflows/codemod-check.yml
name: Codemod Check
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: codemod-pilot/action@v1
        with:
          rule: ./codemods/deprecated-apis.yaml
          fail-on-match: true
```

## 📐 Rule Format

Codemod rules are stored as `.codemod.yaml` files:

```yaml
name: replace-fetch-user-info
description: Migrate fetchUserInfo to getUserProfile
language: typescript
version: "1.0"

pattern:
  before: |
    fetchUserInfo({ userId: $id })
  after: |
    getUserProfile({ profileId: $id })

# Optional: restrict to specific file patterns
include:
  - "src/**/*.ts"
  - "src/**/*.tsx"
exclude:
  - "**/*.test.ts"
  - "**/*.spec.ts"
```

Pattern variables (prefixed with `$`) match any expression and are preserved across the transformation.

For the full rule specification, see [docs/rule-format.md](docs/rule-format.md).

## 🗣️ Supported Languages

| Language | Status | Grammar |
|:---|:---:|:---|
| TypeScript | ✅ Stable | tree-sitter-typescript |
| JavaScript | ✅ Stable | tree-sitter-javascript |
| Python | 🚧 Coming in v0.2 | tree-sitter-python |
| Go | 🚧 Coming in v0.3 | tree-sitter-go |
| Rust | 📋 Planned | tree-sitter-rust |
| Java | 📋 Planned | tree-sitter-java |

Want to add a new language? See our [Adding a Language Guide](docs/adding-a-language.md).

## 🗺️ Roadmap

### v0.1 — Core Engine (Current)
- [x] Example-based pattern inference (single example)
- [x] Codebase scanning with parallel file processing
- [x] Diff preview and safe apply with rollback
- [x] TypeScript/JavaScript support
- [x] Basic CLI (`learn`, `scan`, `apply`)

### v0.2 — Team Ready
- [ ] Multi-example inference
- [ ] Rule export/import (`.codemod.yaml`)
- [ ] Built-in rule templates
- [ ] Interactive conflict resolution
- [ ] Python language support

### v0.3 — CI Ready
- [ ] `--ci` mode with JSON output
- [ ] GitHub Action
- [ ] Go language support
- [ ] Incremental scanning (git-diff based)

### v1.0 — Community Edition
- [ ] Plugin system for new languages
- [ ] Community rule marketplace
- [ ] VS Code extension
- [ ] Web playground

## 🏗️ Architecture

See [docs/architecture.md](docs/architecture.md) for a detailed design overview.

```
codemod-pilot/
├── crates/
│   ├── codemod-core/       # Pattern inference, matching, transformation
│   ├── codemod-cli/        # CLI commands and user interaction
│   └── codemod-languages/  # Tree-sitter language adapters
├── rules/                  # Built-in codemod rules
├── tests/                  # Integration tests and fixtures
└── docs/                   # Documentation
```

## 🤝 Contributing

We welcome contributions of all kinds! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

**Quick ways to contribute:**
- 🐛 [Report a bug](https://github.com/codemod-pilot/codemod-pilot/issues/new?template=bug_report.md)
- 💡 [Request a feature](https://github.com/codemod-pilot/codemod-pilot/issues/new?template=feature_request.md)
- 🌍 [Add a new language](https://github.com/codemod-pilot/codemod-pilot/issues/new?template=new_language.md)
- 📝 Submit a built-in codemod rule
- 📖 Improve documentation

## 📄 License

Licensed under [Apache License, Version 2.0](LICENSE).

---

# 🇨🇳 简体中文

[功能特点](#-功能特点) · [快速开始](#-快速开始) · [工作原理](#-工作原理) · [示例](#-示例) · [规则格式](#-规则格式) · [路线图](#-路线图) · [贡献指南](#-贡献指南)

---

## 问题背景

大规模代码重构非常痛苦。跨 300 个文件重命名 API、从一个库迁移到另一个库、更新废弃的模式——这些任务：

- **手动**：查找替换对代码很危险；正则表达式无法理解 AST 结构
- **复杂**：使用 jscodeshift 等工具编写基于 AST 的 codemod 需要深入的 AST 操作知识
- **风险高**：没有适当的预览和回滚，批量更改可能引入细微 bug

## 解决方案

**codemod-pilot** 让你通过**示例**描述代码转换。只需展示"之前"和"之后"的代码片段，它就会：

1. 🧠 **推断**从示例中推断结构性转换模式
2. 🔍 **扫描**在整个代码库中查找匹配的模式
3. 👀 **预览**以统一 diff 格式预览所有建议的更改
4. ✅ **应用**安全地应用更改，支持自动回滚

```bash
# 展示你想要更改的内容
codemod-pilot learn \
  --before 'fetchUserInfo({ userId: id })' \
  --after  'getUserProfile({ profileId: id })'

# 预览代码库中所有匹配项
codemod-pilot scan --target ./src/

# 放心应用更改
codemod-pilot apply --execute
```

## ✨ 功能特点

- **示例驱动** — 无需 AST 知识，只需展示之前 → 之后
- **多示例推断** — 为复杂模式提供多个示例；引擎会找到通用转换规则
- **默认安全** — 应用前预览所有更改，自动生成回滚补丁
- **交互式冲突处理** — 模糊情况进入交互模式由人工决策
- **可复用规则** — 将推断的模式导出为 `.codemod.yaml` 文件，通过版本控制与团队共享
- **内置模板** — 开箱即用的常见重构模式
- **CI/CD 就绪** — `--ci` 模式输出机器可读的 JSON，使用 `--fail-on-match` 进行自动检查
- **极速** — 使用 Rust 构建，支持并行文件扫描，可在数秒内处理 10 万+文件的代码库
- **多语言支持** — 由 tree-sitter 提供支持，目前支持 TypeScript/JavaScript，Python 和 Go 即将推出

## 🚀 快速开始

### 安装

```bash
# 通过 cargo
cargo install codemod-pilot

# 通过 curl (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/wellkilo/codemod-pilot/main/scripts/install.sh | sh

# 通过 Homebrew（即将推出）
brew install codemod-pilot
```

### 你的第一个 Codemod（30 秒）

```bash
# 1. 教它一个转换
codemod-pilot learn \
  --before 'console.log(msg)' \
  --after  'logger.info(msg)'

# 2. 查看它找到了什么
codemod-pilot scan --target ./src/

# 3. 预览 diff
codemod-pilot apply --preview

# 4. 应用更改
codemod-pilot apply --execute

# 5. 犯了错误？立即回滚
codemod-pilot apply --rollback
```

## 🧠 工作原理

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   示例       │────▶│   模式        │────▶│  代码库      │────▶│    应用      │
│ 之前/之后   │     │   推断        │     │   扫描器     │     │  & 回滚      │
└─────────────┘     └──────────────┘     └─────────────┘     └──────────────┘
                          │                     │
                    ┌─────▼─────┐         ┌─────▼─────┐
                    │  AST Diff │         │  并行     │
                    │   分析     │         │  匹配     │
                    └───────────┘         └───────────┘
```

1. **解析**：使用 tree-sitter 将"之前"和"之后"的代码片段解析为 AST
2. **Diff**：引擎计算两个 AST 之间的结构性 diff，识别哪些节点发生了变化以及如何变化
3. **泛化**：检测变量部分（标识符、字面量）并将其转换为模式变量
4. **匹配**：使用泛化模式扫描目标文件，查找所有结构相似的代码
5. **转换**：根据推断的转换规则重写匹配的代码
6. **验证**：展示结果供审查；在任何写入之前始终生成回滚补丁

## 💡 示例

### 重命名函数及其参数

```bash
codemod-pilot learn \
  --before 'fetchUserInfo({ userId: id })' \
  --after  'getUserProfile({ profileId: id })'

codemod-pilot apply --target ./src/ --execute
```

### 从 Moment.js 迁移到 Day.js

```bash
# 创建示例文件
cat > examples.yaml << 'EOF'
examples:
  - before: "moment(date).format('YYYY-MM-DD')"
    after: "dayjs(date).format('YYYY-MM-DD')"
  - before: "moment(date).add(1, 'days')"
    after: "dayjs(date).add(1, 'day')"
  - before: "moment(date).diff(other, 'hours')"
    after: "dayjs(date).diff(other, 'hour')"
EOF

codemod-pilot learn --examples examples.yaml
codemod-pilot apply --target ./src/ --preview
```

### 使用共享规则文件

```bash
# 应用团队共享的 codemod 规则
codemod-pilot apply --rule ./codemods/replace-deprecated-api.yaml --target ./src/

# 导出你的 codemod 供他人使用
codemod-pilot export --output ./codemods/my-migration.yaml
```

### CI/CD 集成

```yaml
# .github/workflows/codemod-check.yml
name: Codemod Check
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: codemod-pilot/action@v1
        with:
          rule: ./codemods/deprecated-apis.yaml
          fail-on-match: true
```

## 📐 规则格式

Codemod 规则存储为 `.codemod.yaml` 文件：

```yaml
name: replace-fetch-user-info
description: Migrate fetchUserInfo to getUserProfile
language: typescript
version: "1.0"

pattern:
  before: |
    fetchUserInfo({ userId: $id })
  after: |
    getUserProfile({ profileId: $id })

# 可选：限制为特定文件模式
include:
  - "src/**/*.ts"
  - "src/**/*.tsx"
exclude:
  - "**/*.test.ts"
  - "**/*.spec.ts"
```

模式变量（以 `$` 为前缀）匹配任何表达式，并在转换过程中保留。

完整的规则规范请参见 [docs/rule-format.md](docs/rule-format.md)。

## 🗣️ 支持的语言

| 语言 | 状态 | 语法分析器 |
|:---|:---:|:---|
| TypeScript | ✅ 稳定 | tree-sitter-typescript |
| JavaScript | ✅ 稳定 | tree-sitter-javascript |
| Python | 🚧 v0.2 推出 | tree-sitter-python |
| Go | 🚧 v0.3 推出 | tree-sitter-go |
| Rust | 📋 计划中 | tree-sitter-rust |
| Java | 📋 计划中 | tree-sitter-java |

想要添加新语言？请参阅我们的[添加语言指南](docs/adding-a-language.md)。

## 🗺️ 路线图

### v0.1 — 核心引擎（当前版本）
- [x] 基于示例的模式推断（单个示例）
- [x] 并行文件处理的代码库扫描
- [x] Diff 预览和安全应用与回滚
- [x] TypeScript/JavaScript 支持
- [x] 基本 CLI（`learn`、`scan`、`apply`）

### v0.2 — 团队就绪
- [ ] 多示例推断
- [ ] 规则导出/导入（`.codemod.yaml`）
- [ ] 内置规则模板
- [ ] 交互式冲突解决
- [ ] Python 语言支持

### v0.3 — CI 就绪
- [ ] 带 JSON 输出的 `--ci` 模式
- [ ] GitHub Action
- [ ] Go 语言支持
- [ ] 增量扫描（基于 git-diff）

### v1.0 — 社区版
- [ ] 新语言插件系统
- [ ] 社区规则市场
- [ ] VS Code 扩展
- [ ] Web playground

## 🏗️ 架构

有关详细设计概述，请参阅 [docs/architecture.md](docs/architecture.md)。

```
codemod-pilot/
├── crates/
│   ├── codemod-core/       # 模式推断、匹配、转换
│   ├── codemod-cli/        # CLI 命令和用户交互
│   └── codemod-languages/  # Tree-sitter 语言适配器
├── rules/                  # 内置 codemod 规则
├── tests/                  # 集成测试和测试用例
└── docs/                   # 文档
```

## 🤝 贡献指南

欢迎各种形式的贡献！详情请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。

**快速贡献方式：**
- 🐛 报告 bug
- 💡 请求新功能
- 🌍 添加新语言
- 📝 提交内置 codemod 规则
- 📖 改进文档

## 📄 许可证

根据 [Apache License, Version 2.0](LICENSE) 获得许可。

---
