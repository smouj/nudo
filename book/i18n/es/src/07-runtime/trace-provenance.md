# 07.1 — Trazas y procedencia

> **Status:** Propuesto  
> **Summary:** Una ejecución autónoma relevante debería dejar evidencia estructurada suficiente para reconstruir lo ocurrido sin necesidad de hacer ingeniería inversa sobre los registros.

Una traza es una narración de la ejecución. La procedencia son los datos de linaje
asociados a los valores. Se solapan, pero no son idénticos.

Un registro de procedencia útil para un valor verificado puede necesitar identificar
el modelo o la herramienta de origen, las entradas relevantes, el paso de
verificación, las decisiones de política y las aprobaciones. El esquema exacto sigue
siendo una tarea de implementación y de especificación; el invariante es que la
confianza no debería desvincularse de la evidencia que la creó.

Las trazas deberían ser deterministas en su estructura siempre que sea posible,
incluso cuando el contenido de modelo dentro de un evento sea probabilístico.
