# 13.1 — 普通确定性程序

> **状态（Status）：** 提案语法（PROPOSED syntax）  
> **概述（Summary）：** 一个小型确定性程序展示了“普通代码必须保持简单”这一设计要求。

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    let total = add(20, 22);
    print(total);
}
```

这个示例代表的是超出当前已实现词法分析器的预期方向。
标准库 `print` API 的确切形态以及解析器的接受范围取决于后续里程碑。
