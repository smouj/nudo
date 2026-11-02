# 03.4 — 作为产品设计的诊断

> **状态（Status）：** 已实现 / 计划中（IMPLEMENTED / PLANNED）  
> **概述（Summary）：** NUDO 诊断使用稳定代码和源码跨度；已实现的诊断族覆盖词法、语法、名称解析和部分类型检查错误。

诊断是语言体验的一部分，而不是调试转储。
<code>nudo-diagnostics</code> crate 提供结构化诊断，包括严重性、代码、源码位置、
附注和帮助信息。渲染器是共享的，因此每个编译器阶段都可以报告事实，而无需自行格式化。

## 已实现的诊断族

- <code>NDO1xxx</code> 覆盖词法和语法错误。词法分析器和解析器会在可能时恢复，并报告源码跨度。
- <code>NDO2001</code>–<code>NDO2003</code> 覆盖名称解析错误，由 <code>nudo check</code> 发出。
- <code>NDO2004</code>–<code>NDO2007</code> 和 <code>NDO2017</code> 覆盖已实现的类型检查规则。
  类型检查器测试会固定结构化的 expected/found 附注与源码位置。

类型检查器尚不完整，也尚未接入 CLI。因此一次干净的
<code>nudo check</code> 只表示词法、语法和名称解析阶段没有报告错误，
不意味着源码通过了类型检查。

## 计划中的诊断族

效果系统诊断族 <code>NDO3xxx</code> 计划在 M5 实现。今天没有效果检查器会发出这些诊断。
下面只是目标形态示例，并非当前工具链的输出：

~~~text
error[NDO3xxx]: missing capability Network

  --> src/main.nudo:14:5
   |
14 |     fetch(url)
   |     ^^^^^^^^^^ requires Network
   |
   = note: the call path requires Network
~~~

已接受的诊断规范定义了诊断结构和源码位置要求。效果语法和检查规则仍需等提案被接受。

## 诊断质量

有用的诊断会说明哪条规则失败、源码在哪处违反规则，以及是哪项要求导致了错误。
对于信任类型不匹配，诊断应说明
<code>Generated&lt;T&gt;</code> 和 <code>Verified&lt;T&gt;</code> 是有意区分的，
并指出违反契约的值，而不应建议不安全的类型转换。
