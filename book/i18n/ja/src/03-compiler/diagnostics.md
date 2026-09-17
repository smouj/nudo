# 03.4 — プロダクト設計としての診断

> **ステータス:** 仕様化済み (Specified) / 計画中 (Planned)  
> **概要:** コンパイラのエラーは、規則、ソース位置、その要件を持ち込んだ連鎖を説明すべきです。

診断はデバッグダンプではなく、言語体験の一部です。

高品質なケイパビリティエラーは、呼び出し地点と理由の両方を特定すべきです。

```text
error[NDO3xxx]: missing capability `Network`

  ┌─ src/main.nudo:14:5
  │
14│     fetch(url)
  │     ^^^^^^^^^^ requires Network
  │
  └─ current task was not granted Network
```

トラスト型については、エラーは `Generated<T>` と `Verified<T>` が意図的に別物であることを説明し、安全でないキャストを勧めるのではなく、明示的な検証の境界を指し示すべきです。
