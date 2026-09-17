# 01.1 — 确定性软件优先

> **状态（Status）：** 规范已定义（SPECIFIED）  
> **概述（Summary）：** 即使 NUDO 是为 AI 时代的系统设计的，普通的确定性代码也必须保持为简单、可预测的情形。

一门让简单软件变得别扭的智能体语言，是在解决错误的问题。因此 NUDO 把算术、控制流、
函数、数据和模块视为基础。智能体相关的构造位于这个基础之上。

## 概率性周围的确定性

预期的形态是：

```text
validated deterministic input
        ↓
probabilistic operation
        ↓
Generated<T>
        ↓
explicit validation / policy / approval
        ↓
deterministic continuation
```

模型被允许不确定。程序不可以仅仅因为一次响应成功解析，
就假装这种不确定性已经消失。
