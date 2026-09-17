# 13.1 — Ordinary deterministic program

> **Status:** Proposed syntax  
> **Summary:** A small deterministic program demonstrates the design requirement that ordinary code remain simple.

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    let total = add(20, 22);
    print(total);
}
```

This example represents intended direction beyond the currently implemented lexer.
The exact standard-library `print` API and parser acceptance depend on later
milestones.
