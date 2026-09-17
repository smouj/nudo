# 12.1 — Alternativas descartadas deliberadamente

> **Status:** Histórico / Justificación de diseño  
> **Summary:** Varios enfoques más sencillos son útiles en las aplicaciones, pero no satisfacen el objetivo de NUDO de una confianza y una autoridad visibles para el lenguaje.

| Alternativa | Por qué es insuficiente como modelo del lenguaje |
| --- | --- |
| Prompts como cadenas ordinarias | El compilador no puede inferir confianza ni autoridad a partir de prosa. |
| Permisos solo en la configuración de despliegue | Las llamadas en el código no pueden comprobarse contra la ruta de concesión. |
| Indicadores `verified: Bool` | La seguridad pasa a ser una convención opcional del runtime. |
| Palabras clave propias de un proveedor | Acopla el significado del lenguaje a las API de un fabricante. |
| Bucle de agente sin límites | Dificulta revisar la autoridad, el presupuesto y la terminación. |
| Procedencia solo en registros | Reconstruye la evidencia a posteriori y puede perder el linaje del valor. |

Estas técnicas pueden seguir apareciendo dentro de los adaptadores del runtime o de
las bibliotecas. Descartarlas aquí significa «no son el fundamento semántico del
lenguaje», no «nunca son útiles».
