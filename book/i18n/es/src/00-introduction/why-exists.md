# 00.2 — Por qué existe NUDO

> **Status:** Especificado  
> **Summary:** El proyecto parte de un problema de visibilidad para el compilador: en la era de la IA, parte del estado crítico de un programa suele quedar oculto en cadenas, paneles y convenciones.

## La brecha de visibilidad

Piénsese en una aplicación de agentes típica. Un prompt es una cadena. El esquema de
una herramienta es JSON. Los permisos viven en la configuración. Los presupuestos de
tokens o de dinero viven en la consola de un proveedor. La aprobación humana ocurre
en un chat. Las trazas se reconstruyen después de la ejecución.

Cada pieza puede funcionar, pero el lenguaje de programación normalmente no tiene
una forma unificada de razonar sobre ellas.

## La tesis de NUDO

Si la confianza, la autoridad, los efectos, los presupuestos y la procedencia se
representan de forma explícita, algunos fallos pueden pasar de las convenciones a
las comprobaciones del compilador y del runtime.

Esto **no** vuelve determinista una salida probabilística. Vuelve más explícito y
más revisable el código determinista que rodea a esa salida.

## Presión de diseño

Por tanto, el lenguaje optimiza para:

- programas ordinarios deterministas, antes que nada;
- ausencia de autoridad ambiental por defecto;
- transiciones de confianza explícitas;
- independencia del proveedor;
- ejecución local-first;
- trazas y procedencia como productos normales de la ejecución.
