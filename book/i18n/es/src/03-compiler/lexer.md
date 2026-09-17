# 03.2 — Modelo de código fuente y lexer

> **Status:** Implementado (IMPLEMENTED)  
> **Summary:** El lexer es la primera capa real de implementación del lenguaje y se encarga de la tokenización determinista y de los diagnósticos léxicos recuperables.

Un lexer robusto debe preservar las posiciones en bytes, la correspondencia entre
línea y columna y los identificadores de fuente, para que los diagnósticos
posteriores puedan referirse con precisión al código original.

Entre sus propiedades importantes están:

- la carga de código fuente en UTF-8;
- flujos de tokens deterministas;
- reglas de comentarios anidados y de bloque según la especificación léxica;
- la recuperación ante entradas malformadas en lugar de abortar al primer error;
- códigos de diagnóstico estables;
- casos de conformidad que otra implementación pueda reproducir.

La ruta de éxito actual de `nudo check` es solo léxica. Un resultado limpio todavía
no significa que un programa sea sintáctica o semánticamente válido.
