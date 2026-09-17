# 翻译状态 — 简体中文（zh）

**语言（Language）：** 简体中文（Simplified Chinese）· 代码 `zh`
**翻译所依据的英文修订（English revision）：** `c28fc0c`
  — `git rev-parse --short HEAD`，并且包含当时工作区（working tree）中**尚未提交**的英文改动（见下）。
**规范来源（Canonical source）：** [`book/src`](../../src)
**规范（Contract）：** [`book/i18n/README.md`](../README.md)
**状态（Status）：** 全部 33 个文件已完成初译。译文尚未由人工审校。

## 工作区说明（重要）

翻译进行时，英文源树正在被并发编辑。本译文对应的是翻译当时的**工作区内容**，
而不是 `c28fc0c` 这个提交的内容。相对 `HEAD` 存在未提交改动的英文文件：

| 英文文件 | 未提交改动 |
| -------- | ---------- |
| `book/src/index.md` | 新增 `{{#include diagrams/status.svg}}` 及其说明文字；`ROADMAP.md` 链接改为绝对 URL |
| `book/src/03-compiler/pipeline.md` | 新增 `{{#include ../diagrams/pipeline.svg}}` 及图注 |
| `book/src/04-type-system/generated-verified.md` | 新增 `{{#include ../diagrams/trust-flow.svg}}` 及图注 |
| `book/src/05-agents-and-ai/tasks-tools-models.md` | 新增 `{{#include ../diagrams/interfaces.svg}}` 及图注 |
| `book/src/06-security/capabilities-effects.md` | 新增 `{{#include ../diagrams/authority.svg}}` 及图注 |

`book/src/diagrams/`（含 `status.svg`）当时也尚未纳入版本控制。
`translation.json` 记录的是翻译完成时刻的英文内容哈希；
若英文随后变化，`check_translations.py` 会把这些章节报告为 **BEHIND**。

## 已翻译章节

状态标签保留英文等级词（IMPLEMENTED / SPECIFIED / PLANNED / PROPOSED），
中文在前、等级词在后，等级不得被提升。

| 英文源（相对 `book/src`） | 中文译文 | 英文状态 | 中文状态 |
| --- | --- | --- | --- |
| `SUMMARY.md` | `src/SUMMARY.md` | —（无状态行） | — |
| `index.md` | `src/index.md` | Historical / Living | 历史 / 持续更新（Historical / Living） |
| `00-introduction/what-is-nudo.md` | `src/00-introduction/what-is-nudo.md` | Specified | 规范已定义（SPECIFIED） |
| `00-introduction/why-exists.md` | `src/00-introduction/why-exists.md` | Specified | 规范已定义（SPECIFIED） |
| `00-introduction/status.md` | `src/00-introduction/status.md` | Implemented / Planned | 已实现 / 计划中（IMPLEMENTED / PLANNED） |
| `01-philosophy/deterministic-first.md` | `src/01-philosophy/deterministic-first.md` | Specified | 规范已定义（SPECIFIED） |
| `01-philosophy/explicit-trust.md` | `src/01-philosophy/explicit-trust.md` | Proposed | 提案中（PROPOSED） |
| `01-philosophy/local-first.md` | `src/01-philosophy/local-first.md` | Specified | 规范已定义（SPECIFIED） |
| `02-language-design/grammar.md` | `src/02-language-design/grammar.md` | Proposed | 提案中（PROPOSED） |
| `02-language-design/expressions.md` | `src/02-language-design/expressions.md` | Open | 待定（Open） |
| `02-language-design/modules.md` | `src/02-language-design/modules.md` | Proposed | 提案中（PROPOSED） |
| `03-compiler/pipeline.md` | `src/03-compiler/pipeline.md` | Specified | 规范已定义（SPECIFIED） |
| `03-compiler/lexer.md` | `src/03-compiler/lexer.md` | Implemented | 已实现（IMPLEMENTED） |
| `03-compiler/parser.md` | `src/03-compiler/parser.md` | Planned | 计划中（PLANNED） |
| `03-compiler/diagnostics.md` | `src/03-compiler/diagnostics.md` | Specified / Planned | 规范已定义 / 计划中（SPECIFIED / PLANNED） |
| `04-type-system/fundamentals.md` | `src/04-type-system/fundamentals.md` | Proposed | 提案中（PROPOSED） |
| `04-type-system/generated-verified.md` | `src/04-type-system/generated-verified.md` | Proposed | 提案中（PROPOSED） |
| `05-agents-and-ai/agents.md` | `src/05-agents-and-ai/agents.md` | Proposed | 提案中（PROPOSED） |
| `05-agents-and-ai/tasks-tools-models.md` | `src/05-agents-and-ai/tasks-tools-models.md` | Proposed | 提案中（PROPOSED） |
| `06-security/capabilities-effects.md` | `src/06-security/capabilities-effects.md` | Proposed | 提案中（PROPOSED） |
| `06-security/threat-model.md` | `src/06-security/threat-model.md` | Specified / Planned | 规范已定义 / 计划中（SPECIFIED / PLANNED） |
| `07-runtime/trace-provenance.md` | `src/07-runtime/trace-provenance.md` | Proposed | 提案中（PROPOSED） |
| `07-runtime/budgets-approvals.md` | `src/07-runtime/budgets-approvals.md` | Proposed | 提案中（PROPOSED） |
| `08-tooling/cli.md` | `src/08-tooling/cli.md` | Implemented / Planned | 已实现 / 计划中（IMPLEMENTED / PLANNED） |
| `09-interoperability/mcp-a2a-wasm.md` | `src/09-interoperability/mcp-a2a-wasm.md` | Planned | 计划中（PLANNED） |
| `10-internals/repository-architecture.md` | `src/10-internals/repository-architecture.md` | Implemented foundation | 已实现的基础（IMPLEMENTED foundation） |
| `11-rationale/why-rust-own-compiler.md` | `src/11-rationale/why-rust-own-compiler.md` | Specified rationale | 规范已定义的论证（SPECIFIED rationale） |
| `11-rationale/why-trust-types.md` | `src/11-rationale/why-trust-types.md` | Proposed rationale | 提案论证（PROPOSED rationale） |
| `12-alternatives/rejected.md` | `src/12-alternatives/rejected.md` | Historical / Design rationale | 历史 / 设计理由（Historical / Design rationale） |
| `13-examples/ordinary-program.md` | `src/13-examples/ordinary-program.md` | Proposed syntax | 提案语法（PROPOSED syntax） |
| `13-examples/trust-flow.md` | `src/13-examples/trust-flow.md` | Proposed syntax | 提案语法（PROPOSED syntax） |
| `14-contributing/spec-nep-adr.md` | `src/14-contributing/spec-nep-adr.md` | Specified process | 规范已定义的流程（SPECIFIED process） |
| `15-history/milestones.md` | `src/15-history/milestones.md` | Historical / Living | 历史 / 持续更新（Historical / Living） |

**合计：33 / 33（`SUMMARY.md`、`index.md` 与 31 章）。**

## 保持原样的内容（不翻译）

- 代码块、行内代码、标识符、类型名（`Generated<T>`、`Verified<T>`、`Article`、`Bool` 等）；
- `NDO` 代码（如 `NDO3xxx`）、命令（`nudo check`）、文件路径（`spec/grammar.md`、`src/main.nudo`）、URL；
- 4 条 `{{#include ../diagrams/*.svg}}` 与 1 条 `{{#include diagrams/status.svg}}`，均逐字节保留；
- 图表（diagram）本体：由构建脚本从规范树同步，只有图注（`*…*`）被翻译。

## 尚未翻译的内容

- 本书 33 个文件之外没有任何内容属于本次范围；这 33 个文件**没有**缺失项。
- 按翻译约定，以下内容本就**不属于**翻译范围：`book/adr/`、`book/quality/`、`book/scripts/`、
  仓库根 `README.md`、`spec/`、`neps/`。
- 译文尚未经人工审校；本次为首次完整翻译（R0）。

## 判断与歧义（Judgement calls）

1. **`02.2` 的状态标签是 `Open`**，不属于四个规范等级词（IMPLEMENTED / SPECIFIED / PLANNED /
   PROPOSED）。译文写作“待定（Open）”，保留原文措辞，未把它归入任何规范等级。
2. **标题**：`The NUDO Technical Book` 译为「NUDO 技术手册」，与 `book.toml` 的
   `title = "NUDO 技术手册"` 一致；`NUDO` 保持原样。
3. **状态行格式**：英文 `> **Status:** X` / `> **Summary:** X` 译为
   `> **状态（Status）：** …（RANK）` / `> **概述（Summary）：** …`，
   行尾两个空格的硬换行保留。等级词以大写英文保留，便于与规范核对。
4. **`index.md` 的 `ROADMAP.md` 链接**：翻译开始时英文写作相对路径 `../../ROADMAP.md`
   （从 `book/i18n/zh/src/` 出发会指向不存在的 `book/i18n/ROADMAP.md`）；在翻译过程中，
   英文被改为绝对 URL。译文现在与英文一致，使用
   `https://github.com/smouj/nudo/blob/main/ROADMAP.md`（URL 逐字节保留）。
5. **英文源在翻译期间被并发修改**：见上方「工作区说明」。已按修改后的 `index.md` 重新核对译文；
   其余 32 个文件的自译内容与戳记哈希一致。
6. **术语策略**：首次出现时给出「中文（English）」，其后只用中文；无通行中文译名的词
   （如 `pre-alpha`、`monorepo`、`crate`、`span`）保留英文写法，必要时在首次出现处加简短中文说明。
   未使用繁体字或地区性表达。

## 复现与校验

```sh
export PATH=/home/smouj/.openclaw/workspace/main/.openclaw/tmp/tools:$PATH
cd /home/smouj/.openclaw/workspace/main/projects/nudo

python3 book/scripts/build_site.py zh          # 构建（含引用解析检查）
python3 book/scripts/stamp_translation.py --lang zh   # 记录所依据的英文哈希
python3 book/scripts/check_translations.py     # 缺失 = 错误；BEHIND = 提示
```

三项均已执行并全部通过，见本次交付报告中的原始输出。
