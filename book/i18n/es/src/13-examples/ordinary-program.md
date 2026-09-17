# 13.1 — Programa determinista ordinario

> **Status:** Sintaxis propuesta (PROPOSED SYNTAX)  
> **Summary:** Un pequeño programa determinista demuestra el requisito de diseño de que el código ordinario siga siendo sencillo.

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    let total = add(20, 22);
    print(total);
}
```

Este ejemplo representa la dirección prevista, más allá del lexer implementado
actualmente. La API exacta de `print` en la biblioteca estándar y su aceptación por
parte del parser dependen de hitos posteriores.
