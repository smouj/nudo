# 10.1 — 仓库架构

> **状态（Status）：** 已实现的基础（IMPLEMENTED foundation）  
> **概述（Summary）：** 这个单体仓库（monorepo）把规范、编译器、运行时和工具链的演进放在一份可审计的历史里，同时保留概念上的 crate 边界。

工作区（workspace）分离了编译器阶段、运行时服务、共享基础设施 crate、CLI、工具和后端。
这让归属关系可见，但 pre-alpha 阶段的 crate 边界应当保持可修改。

占位（placeholder）crate 是路线图标记，而不是其公开 API 已就绪的证据。
当经验表明存在更好的边界时，应当允许实现合并或拆分内部单元。
