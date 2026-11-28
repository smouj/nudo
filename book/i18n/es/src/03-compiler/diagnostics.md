# 03.4 — Los diagnósticos como diseño de producto

> **Status:** Implementado (IMPLEMENTED) / Planificado (PLANNED)
> **Summary:** Los diagnósticos de NUDO usan códigos estables y spans de origen; las familias implementadas cubren errores léxicos, sintácticos, de resolución de nombres y de comprobación parcial de tipos.

Los diagnósticos forman parte de la experiencia del lenguaje, no son un volcado de depuración. El crate <code>nudo-diagnostics</code> proporciona diagnósticos estructurados con severidad, código, ubicación en el origen, notas y ayuda. El renderer es compartido para que cada fase del compilador informe de hechos sin formatear su propia salida.

## Familias de diagnósticos implementadas

- <code>NDO1xxx</code> cubre errores léxicos y sintácticos. El lexer y el parser se recuperan cuando es posible e informan de spans de origen.
- <code>NDO2001</code>–<code>NDO2003</code> cubren errores de resolución de nombres y los emite <code>nudo check</code>.
- <code>NDO2004</code>–<code>NDO2019</code> cubren las reglas de tipado implementadas. <code>NDO2010</code> señala una magnitud entera que no puede representar un <code>Int</code>; exactamente <code>2^63</code> solo se permite directamente bajo el unario <code>-</code> para escribir <code>Int::MIN</code>. Según NEP-0020, <code>NDO2011</code>–<code>NDO2016</code> cubren patrones de <code>enum</code> y <code>Result</code>, aridad de payloads, exhaustividad, alcanzabilidad y tipos de los brazos. Las pruebas fijan notas estructuradas y ubicaciones de origen.

El comprobador de tipos es parcial y la CLI aún no lo invoca. Por tanto, un <code>nudo check</code> limpio indica que el código pasó el análisis léxico, sintáctico y la resolución de nombres; no significa que haya pasado la comprobación de tipos.

## Familias de diagnósticos planificadas

La familia del sistema de efectos <code>NDO3xxx</code> está planificada para M5. Hoy no hay un comprobador de efectos que emita esos diagnósticos. El ejemplo siguiente es ilustrativo: describe un objetivo y no es una salida de la toolchain actual:

~~~text
error[NDO3xxx]: falta la capability Network

  --> src/main.nudo:14:5
   |
14 |     fetch(url)
   |     ^^^^^^^^^^ requiere Network
   |
   = note: la ruta de llamadas requiere Network
~~~

La especificación de diagnósticos aceptada define su estructura y los requisitos de origen. La sintaxis y las reglas de comprobación de efectos siguen dependiendo de que se acepte una propuesta.

## Calidad de los diagnósticos

Un diagnóstico útil explica qué regla falló, dónde la incumple el código y qué requisito originó el error. Para incompatibilidades de tipos de confianza, debería explicar que <code>Generated&lt;T&gt;</code> y <code>Verified&lt;T&gt;</code> son distintos e identificar el valor que incumple la garantía. No debería sugerir una conversión insegura.
