# 翻译状态 — 简体中文（zh）

**规范来源：** 英文 <code>book/src</code>。  
**覆盖范围：** 34 / 34 个 Markdown 页面。  
**源文追踪：** <code>translation.json</code> 记录每个英文篇章的 hash。

中文版与本 revision 的英文篇章相对应。图表与规范版本共用，保留其 include 路径。英文规范仍是规范性信息的唯一来源，本译文不定义语言规则。

名称、标识符、命令、NDO 代码、路径和代码示例保持规范写法。状态标签按照[翻译约定](../README.md)和[编辑约定](../../EDITORIAL.md)，在译文后保留大写英文状态等级。

本次更新未声称经过母语者的独立编辑审校。hash 记录译文对应的原文 revision，但本身不能证明语言质量。

## 校验

~~~sh
python3 book/scripts/check_translations.py --strict
python3 book/scripts/build_site.py zh --strict
~~~
