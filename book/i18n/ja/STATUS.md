# 翻訳状況 — 日本語（ja）

**正本:** 英語の <code>book/src</code>。  
**収録範囲:** Markdown ページ 34 / 34。  
**原文の追跡:** <code>translation.json</code> に各英語章の hash を記録します。

日本語版はこの revision の英語各章に合わせて更新されています。図は正本と共有し、include path を維持します。規範上の情報源は英語仕様であり、この翻訳が言語規則を定めることはありません。

名前、identifier、command、NDO code、path、code example は正本の表記を保ちます。status label は[翻訳契約](../README.md)と[編集契約](../../EDITORIAL.md)に従い、英語の status rank を大文字で括弧内に残します。

今回の更新について、母語話者による独立した編集レビューを行ったとは記載しません。hash は原文 revision との対応を記録しますが、それだけで言語品質を保証するものではありません。

## 検証

~~~sh
python3 book/scripts/check_translations.py --strict
python3 book/scripts/build_site.py ja --strict
~~~
