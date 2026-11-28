# 04.1 — 类型系统基础

> **状态（Status）：** 已实现 / 规范已定义 / 计划中（IMPLEMENTED / SPECIFIED / PLANNED）
>
> **概述（Summary）：** NUDO 已有部分类型检查器来执行已接受的规则；完整类型系统仍在开发中，CLI 会运行目前已实现的子集。

类型系统由语言规范章节和已接受的 NEP 定义。Rust crate
<code>nudo-typeck</code> 实现其中逐步扩展的子集，并消费 HIR，而不是源码文本。
它的测试与一致性用例会覆盖已实现规则；名称解析成功后，CLI 会调用该 crate。

## 当前检查器已经实现的规则

| 规则 | 当前行为 |
| --- | --- |
| 基本类型与字面量 | 表示 <code>Int</code>、<code>Float</code>、<code>Bool</code>、<code>Text</code> 和 <code>Unit</code> |
| 整数字面量范围 | 接受 <code>Int::MAX</code>；只有直接位于一元 <code>-</code> 下、用于表示 <code>Int::MIN</code> 时才接受 <code>2^63</code>；更大或其他无效范围使用 <code>NDO2010</code> |
| 类型注解与函数签名 | 将值与声明类型比较；检查函数类型和调用 |
| 函数结果 | 省略结果类型注解时类型为 <code>Unit</code>；返回值的函数必须声明结果类型；函数体不匹配时使用 <code>NDO2004</code> |
| 调用 | 参数个数错误使用 <code>NDO2005</code>，参数类型不兼容使用 <code>NDO2004</code>，不可调用值使用 <code>NDO2006</code> |
| 泛型调用 | 按源码顺序，从实参和已知的期望结果类型推断类型参数；无法推断的泛型类型参数使用 <code>NDO2008</code>，函数签名中未使用的类型参数使用 <code>NDO2009</code> |
| 名义类型 | 使用 <code>NDO2017</code> 检查类型参数个数，并检查名义身份和已支持的字段访问 |
| 运算符 | 算术使用匹配的数值视图；<code>%</code> 仅允许 Int；布尔运算符使用 <code>Bool</code>；相等比较接受匹配的基本类型视图；顺序比较接受匹配的 <code>Int</code> 或 <code>Float</code>；无效类型使用 <code>NDO2018</code> |
| 条件 | <code>if</code> 条件为 <code>Bool</code>；存在期望类型时会检查两个分支，否则分支类型必须完全相同；没有 <code>else</code> 时，结果和 <code>then</code> 分支均为 <code>Unit</code>；无上下文的类型不一致使用 <code>NDO2019</code> |
| 枚举匹配 | 用户定义的枚举和 <code>Result&lt;T, E&gt;</code> 使用显式变体模式、扁平且带类型的 payload 绑定、通配符覆盖、穷尽性检查和严格的分支结果规则；诊断代码为 <code>NDO2011</code>–<code>NDO2016</code> |
| 信任赋值 | 内置 <code>Verified&lt;T&gt;</code> 在已支持的位置可用于要求 <code>T</code> 的地方 |

泛型推断有明确边界。它不会跨整个程序求解类型，不会泛化局部绑定，也不会猜测未解析的类型参数。
泛型参数保持不变，数值类型之间没有隐式转换。[NEP-0015](https://github.com/smouj/nudo/blob/main/neps/0015-uninferable-generic-parameters.md) 明确了这一边界：信息不足的泛型调用报告 <code>NDO2008</code>，函数签名中未出现的类型参数报告 <code>NDO2009</code>。调用函数值时，外层类型参数保持固定。恢复过程只抑制依赖先前错误的诊断，以避免错误级联。

## 尚未完成的规则

序列索引及其运行时边界行为、名义值构造仍未完成。该检查器也尚未实现验证器执行、来源追踪，
以及智能体、任务和工具类型。

这些遗漏是有意的。只有语义和诊断契约已经接受，规则才能实现。
路线图和规范会记录哪些工作已就绪，哪些决定仍然开放。

## 干净的 CLI 检查意味着什么

<code>nudo check</code> 会运行词法、语法、名称解析和部分类型检查阶段。干净结果表示已实现的已接受规则
没有报告错误；尚未支持或受 Draft NEP 限制的类型规则仍未检查。类型检查也通过自身测试和一致性语料覆盖。
