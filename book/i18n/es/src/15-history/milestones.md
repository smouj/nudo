# 15.1 — Hitos y evidencia

> **Status:** Histórico (HISTORICAL) / Vivo
> **Summary:** NUDO mide el avance mediante hitos basados en evidencia; M3 sigue en curso, mientras que el intérprete y el sistema de efectos necesarios para llegar a M5 están planificados.

La hoja de ruta mide el progreso mediante criterios de salida y evidencia de conformidad, no por fechas. Esta página resume el recorrido hasta M5; la hoja de ruta del repositorio sigue siendo la referencia autoritativa.

{{#include ../diagrams/status.svg}}

| Hito | Estado | Evidencia de salida |
| --- | --- | --- |
| M0 — Base del repositorio | Completo | Se puede compilar, comprobar y revisar un clon limpio |
| M1 — Modelo de fuentes y lexer | Completo | Tokens deterministas, diagnósticos y recuperación |
| M2 — Parser y árbol sintáctico | Completo | Análisis sin pérdidas, recuperación y casos de conformidad |
| M3.0 — Decisiones semánticas | Completo | Decisiones básicas del sistema de tipos registradas antes de implementarse |
| M3.1 — HIR y resolución de nombres | Completo | HIR resuelto y diagnósticos estables de nombres |
| M3.2 — Sistema de tipos básico | En curso | Reglas aceptadas incorporadas en entregas verticales con evidencia de conformidad |
| M3.3 — Tipos de confianza | Parcial | Existen identidad de tipos y asignación de Verified; verify y la procedencia siguen pendientes |
| M4 — Intérprete | Planificado | El programa de ejemplo calcula e imprime 42 mediante <code>nudo run</code> |
| M5 — Sistema de efectos | Planificado | Se rechaza una llamada con efectos desde una función sin efectos y se informa de su ruta de efectos |

M4 y M5 no son etiquetas consecutivas para trabajo intercambiable. El criterio de salida de M4 requiere una ruta real desde el código fuente hasta la salida. M5 depende de las bases de tipos y confianza, y requiere que se acepte la semántica de efectos. Una propuesta Draft no puede servir de contrato de implementación.

## Qué protegen los hitos

Este orden impide que una demostración pulida sustituya las garantías del compilador. Antes de marcar M5 como completo, deben cumplirse los criterios previos de tipos y confianza, M4 debe ejecutar el ejemplo documentado, debe aceptarse la semántica del sistema de efectos y sus diagnósticos y casos de conformidad deben demostrar el criterio de salida.

La hoja de ruta no promete alcanzar M5 en una fecha concreta. El avance se comunica según lo que se compila y supera las pruebas especificadas.
