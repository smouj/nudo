# 06.1 — Capacidades, efectos y autoridad

> **Status:** Propuesto (PROPOSED)  
> **Summary:** Los efectos describen qué puede hacer una ejecución; las capacidades describen qué autoridad puede conceder realmente el contexto actual.


{{#include ../diagrams/authority.svg}}

*La autoridad se estrecha a lo largo de una cadena de delegación y no puede fluir de vuelta.*

Estos conceptos deben relacionarse sin confundirse.

- **Efecto**: una descripción estática de que un cómputo puede realizar una operación
  como el acceso a la red.
- **Concesión de capacidad**: autoridad puesta a disposición de un contexto.
- **Política**: reglas que restringen cuándo o cómo puede ejercerse esa autoridad.

El valor por defecto está pensado para ser denegar. El código no debería adquirir
acceso a la red, al sistema de archivos, al shell o a secretos solo porque el proceso
anfitrión lo tenga.

Un invariante crítico es la no amplificación: una herramienta o un agente delegado no
debe poder ampliar su propia autoridad más allá de lo que concedió quien hizo la
llamada.
