# Estado de la traducción — Español (<code>es</code>)

**Fuente canónica:** <code>book/src</code> en inglés.  
**Cobertura:** 34 de 34 páginas Markdown.  
**Seguimiento de origen:** <code>translation.json</code> registra el hash de cada capítulo inglés.

La edición española está actualizada con los capítulos ingleses de esta revisión. Los diagramas se comparten con la edición canónica; sus rutas de inclusión se mantienen. La especificación en inglés sigue siendo la fuente normativa y esta edición no define reglas del lenguaje.

Los nombres, identificadores, comandos, códigos NDO, rutas y ejemplos de código conservan su forma canónica. Las etiquetas de estado mantienen el rango inglés en mayúsculas entre paréntesis, conforme a [el contrato de traducción](../README.md) y al [contrato editorial](../../EDITORIAL.md).

Esta actualización no afirma una revisión editorial independiente por una persona hablante nativa. Los hashes indican correspondencia con la revisión del original; no certifican por sí solos la calidad lingüística.

## Verificación

~~~sh
python3 book/scripts/check_translations.py --strict
python3 book/scripts/build_site.py es --strict
~~~
