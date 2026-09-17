# 09.1 — MCP、A2A 与 WebAssembly

> **状态（Status）：** 计划中（PLANNED）  
> **概述（Summary）：** 互操作性应当让 NUDO 参与现有的工具与智能体生态，同时不允许适配器扩大授权。

MCP 互操作性可以让 NUDO 消费或暴露工具。A2A 风格的委托可以连接独立的智能体（agent）实现。
WASI 可以提供一个可移植的执行目标，具备显式的宿主能力（capability）。

安全规则强于协议兼容性：不能信任适配器（adapter）授予自己 NUDO 调用方并不具备的能力。
