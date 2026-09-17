# 13.1 — Gewöhnliches deterministisches Programm

> **Status:** Vorgeschlagene Syntax  
> **Summary:** Ein kleines deterministisches Programm zeigt die Entwurfsanforderung, dass gewöhnlicher Code einfach bleibt.

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    let total = add(20, 22);
    print(total);
}
```

Dieses Beispiel stellt die beabsichtigte Richtung über den derzeit implementierten
Lexer hinaus dar. Die genaue `print`-API der Standardbibliothek und die Annahme durch
den Parser hängen von späteren Meilensteinen ab.
