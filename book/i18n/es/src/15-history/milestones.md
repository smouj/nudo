# 15.1 — Hitos y evidencia

> **Status:** Histórico / Vivo (HISTORICAL / LIVING)  
> **Summary:** NUDO usa hitos basados en evidencia en lugar de promesas de calendario; un hito está completo cuando lo están sus criterios de salida y su evidencia de conformidad.

La hoja de ruta comienza con el fundamento del repositorio y el pipeline léxico, y
después avanza por parser y AST, sistema de tipos, intérprete, efectos, tipos de
confianza, runtime de agentes, capacidades y políticas, interoperabilidad,
herramientas y WebAssembly.

Este orden refleja dependencias, no prioridad de marketing. Un runtime de agentes
construido antes que los fundamentos de parser, tipos y efectos forzaría decisiones
semánticas dentro del código del runtime y dificultaría definir con limpieza las
comprobaciones posteriores del compilador.
