# 04.2 — Generated 与 Verified 值

> **状态（Status）：** 已实现 / 规范已定义 / 计划中（IMPLEMENTED / SPECIFIED / PLANNED）
>
> **概述（Summary）：** 检查器会执行生成类型与已验证类型的区分；实际运行验证及记录来源仍属于未来运行时工作。

{{#include ../diagrams/trust-flow.svg}}

*部分静态类型规则已经实现；verify 操作和来源路径是设计预览，今天无法执行。*

## 信任状态的区别

<code>Generated&lt;T&gt;</code> 与 <code>Verified&lt;T&gt;</code> 是不同类型。
检查器不会在二者之间进行隐式转换，它们的泛型参数也保持不变。因此，
生成的结果不能满足显式要求 Verified 类型的参数。

已接受规则也允许内置的 <code>Verified&lt;T&gt;</code> 在已支持的位置用于要求
<code>T</code> 的地方。此规则基于解析到的内置类型身份；拼写相同的用户自定义类型
不会因此获得该规则。类型检查一致性用例覆盖了这些行为，但
<code>nudo check</code> 目前还不运行类型检查。

## 计划中的验证操作

语法 <code>verify value with Verifier</code> 已被解析器接受，并在 HIR 中表示。
验证器契约、执行、失败处理和来源记录尚未实现。验证器设计仍是提案，
因此以下结构只是预期流程示例，不是可运行程序：

~~~nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
~~~

设计要明确两件不同的事实：验证可能失败，成功的值具有不同的信任类型。
这并不会让概率性生成变成确定性；它会让信任边界可以被审查。

## 为什么不用布尔值表示信任？

~~~text
ModelOutput<T> { value: T, verified: Bool }
~~~

调用者可能忘记检查标志位。不同类型使检查器能够在已支持的类型检查边界拒绝错误值，
而无需依赖运行时约定。

## 尚未完成的工作

运行时验证操作、可静态检查的验证器声明、稳定失败类型、来源记录和执行追踪
仍是计划或提案中的工作。<code>Result&lt;T, E&gt;</code> 可以按规定类型参数书写，
但模式匹配和穷尽性检查尚未实现。
