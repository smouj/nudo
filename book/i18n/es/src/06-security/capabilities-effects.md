# 06.1 — Capabilities, efectos y autoridad

> **Status:** Propuesto (PROPOSED) / Planificado (PLANNED)
>
> **Summary:** NUDO propone el seguimiento estático de efectos y autoridad explícita mediante capabilities; la toolchain actual no implementa ni la comprobación de efectos ni la aplicación de capabilities en runtime.

{{#include ../diagrams/authority.svg}}

*Este es el modelo de autoridad propuesto, no un límite de seguridad
implementado por el runtime actual.*

## Tres ideas relacionadas

- **Efecto:** descripción estática de lo que puede hacer un cálculo, como usar la red.
- **Grant de capability:** autoridad disponible para un programa o contexto de ejecución.
- **Policy:** reglas que restringen cuándo o cómo se puede ejercer esa autoridad.

El diseño vincula el efecto requerido por una función con la autoridad disponible en el punto de llamada. Pretende denegar por defecto y evitar que una herramienta o agente delegado amplíe la autoridad de quien lo invocó.

## Estado de la implementación actual

M5 está planificado. El capítulo sobre efectos sigue siendo una propuesta y <code>neps/0004-effects.md</code> está en Draft. Aún no hay inferencia de efectos, comprobador de efectos, implementación de <code>NDO3xxx</code> ni aplicación de capabilities en runtime. El crate <code>nudo-effects</code> marca un límite en la hoja de ruta, no un subsistema terminado.

El ejemplo siguiente utiliza la sintaxis de la propuesta Draft; no es sintaxis aceptada en la que un usuario pueda confiar:

~~~nudo
fn fetch(url: Text) -> Text with Network {
    // Propuesto: llamar a fetch requiere Network.
}
~~~

El criterio de salida de M5 en la hoja de ruta consiste en rechazar una llamada con efectos desde una función sin efectos y mostrar un diagnóstico que nombre tanto el efecto como la ruta de llamadas que lo introdujo. El diseño debe aceptarse antes de implementar ese comprobador. La aplicación de policies en runtime es un hito posterior e independiente.
